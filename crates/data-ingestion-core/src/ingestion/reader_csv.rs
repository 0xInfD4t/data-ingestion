use crate::error::IngestionError;
use crate::ingestion::traits::{FormatHint, FormatReader};
use crate::ingestion::type_inference::{infer_type_from_string, infer_type_from_type_string, unify_types};
use crate::ir::model::{IrDocument, IrField, IrNode, IrObject, IrType, SourceFormat};

const MAX_SAMPLE_ROWS: usize = 100;

/// Reads CSV files — either as data dictionaries or raw data (schema inference).
pub struct CsvReader;

impl FormatReader for CsvReader {
    fn can_read(&self, hint: &FormatHint) -> bool {
        if let Some(ext) = hint.extension() {
            if ext.eq_ignore_ascii_case("csv") {
                return true;
            }
        }
        if let Some(explicit) = &hint.explicit_format {
            return matches!(explicit, SourceFormat::DataDictionary);
        }
        true
    }

    fn read(&self, input: &[u8], hint: &FormatHint) -> Result<IrDocument, IngestionError> {
        let mut rdr = csv::ReaderBuilder::new()
            .flexible(true)
            .trim(csv::Trim::All)
            .from_reader(input);

        // Read headers
        let headers: Vec<String> = rdr
            .headers()
            .map_err(|e| {
                IngestionError::parse_with_source(
                    "Csv",
                    "$",
                    format!("Failed to read CSV headers: {}", e),
                    Box::new(e),
                )
            })?
            .iter()
            .map(String::from)
            .collect();

        if headers.is_empty() {
            return Err(IngestionError::parse("Csv", "$", "CSV file has no headers"));
        }

        // Detect if this is a data dictionary
        let col_map = detect_data_dictionary_columns(&headers);

        // Read all rows (up to sample limit for raw data)
        let mut rows: Vec<Vec<String>> = Vec::new();
        for result in rdr.records() {
            let record = result.map_err(|e| {
                IngestionError::parse_with_source(
                    "Csv",
                    "$",
                    format!("Failed to read CSV record: {}", e),
                    Box::new(e),
                )
            })?;
            rows.push(record.iter().map(String::from).collect());
            if rows.len() >= MAX_SAMPLE_ROWS && col_map.is_none() {
                break; // Only limit for raw data inference
            }
        }

        let root = if let Some(col_indices) = col_map {
            // Data dictionary mode
            read_as_data_dictionary(&headers, &rows, &col_indices)?
        } else {
            // Raw data mode — infer schema from headers and sample rows
            read_as_raw_data(&headers, &rows)?
        };

        Ok(IrDocument::new(
            SourceFormat::DataDictionary,
            hint.filename.clone(),
            root,
        ))
    }
}

// ── Column detection ──────────────────────────────────────────────────────────

/// Canonical column names and their accepted aliases.
struct DataDictColumns {
    field_name: Option<usize>,
    data_type: Option<usize>,
    description: Option<usize>,
    nullable: Option<usize>,
    required: Option<usize>,
    constraints: Option<usize>,
    tags: Option<usize>,
    pii: Option<usize>,
    classification: Option<usize>,
    default_value: Option<usize>,
    example: Option<usize>,
}

fn detect_data_dictionary_columns(headers: &[String]) -> Option<DataDictColumns> {
    let mut col = DataDictColumns {
        field_name: None,
        data_type: None,
        description: None,
        nullable: None,
        required: None,
        constraints: None,
        tags: None,
        pii: None,
        classification: None,
        default_value: None,
        example: None,
    };

    for (i, header) in headers.iter().enumerate() {
        let h = header.to_lowercase();
        let h = h.trim();

        match h {
            // Only unambiguous field-name column headers qualify (not "name" alone — too generic)
            "field_name" | "column_name" | "field" | "attribute" | "column" => {
                col.field_name = Some(i);
            }
            "type" | "data_type" | "dtype" | "logical_type" | "datatype" => {
                col.data_type = Some(i);
            }
            "description" | "desc" | "comment" | "notes" | "note" => {
                col.description = Some(i);
            }
            "nullable" | "is_nullable" | "null" | "optional" => {
                col.nullable = Some(i);
            }
            "required" | "is_required" | "mandatory" => {
                col.required = Some(i);
            }
            "constraints" | "rules" | "validation" | "constraint" => {
                col.constraints = Some(i);
            }
            "tags" | "labels" | "categories" | "tag" => {
                col.tags = Some(i);
            }
            "pii" | "is_pii" | "sensitive" => {
                col.pii = Some(i);
            }
            "classification" | "data_class" | "sensitivity" => {
                col.classification = Some(i);
            }
            "default" | "default_value" | "default_val" => {
                col.default_value = Some(i);
            }
            "example" | "examples" | "sample" | "sample_value" => {
                col.example = Some(i);
            }
            _ => {}
        }
    }

    // Must have at least field_name to be a data dictionary
    if col.field_name.is_some() {
        Some(col)
    } else {
        None
    }
}

// ── Data dictionary mode ──────────────────────────────────────────────────────

fn read_as_data_dictionary(
    headers: &[String],
    rows: &[Vec<String>],
    col: &DataDictColumns,
) -> Result<IrNode, IngestionError> {
    let mut obj = IrObject::new(Some("data_dictionary".to_string()));

    for (row_idx, row) in rows.iter().enumerate() {
        let get = |idx: Option<usize>| -> &str {
            idx.and_then(|i| row.get(i)).map(|s| s.as_str()).unwrap_or("")
        };

        let field_name = get(col.field_name).trim().to_string();
        if field_name.is_empty() {
            log::debug!("Skipping empty field name at row {}", row_idx + 2);
            continue;
        }

        let type_str = get(col.data_type);
        let ir_type = if type_str.is_empty() {
            IrType::Unknown
        } else {
            infer_type_from_type_string(type_str)
        };

        let mut field = IrField::new(field_name, ir_type);

        // Description
        let desc = get(col.description).trim().to_string();
        if !desc.is_empty() {
            field.description = Some(desc);
        }

        // Nullable
        let nullable_str = get(col.nullable).trim().to_lowercase();
        if !nullable_str.is_empty() {
            field.nullable = parse_bool_str(&nullable_str).unwrap_or(true);
        }

        // Required
        let required_str = get(col.required).trim().to_lowercase();
        if !required_str.is_empty() {
            field.required = parse_bool_str(&required_str).unwrap_or(false);
        }

        // Tags
        let tags_str = get(col.tags).trim().to_string();
        if !tags_str.is_empty() {
            field.tags = tags_str
                .split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect();
        }

        // PII
        let pii_str = get(col.pii).trim().to_lowercase();
        if !pii_str.is_empty() {
            if let Some(pii) = parse_bool_str(&pii_str) {
                field.metadata.insert(
                    "pii".to_string(),
                    serde_json::Value::Bool(pii),
                );
            }
        }

        // Classification
        let class_str = get(col.classification).trim().to_string();
        if !class_str.is_empty() {
            field.metadata.insert(
                "classification".to_string(),
                serde_json::Value::String(class_str),
            );
        }

        // Default value
        let default_str = get(col.default_value).trim().to_string();
        if !default_str.is_empty() {
            field.default_value = Some(serde_json::Value::String(default_str));
        }

        // Example
        let example_str = get(col.example).trim().to_string();
        if !example_str.is_empty() {
            field.examples = vec![serde_json::Value::String(example_str)];
        }

        // Constraints (semicolon-delimited "type:value" pairs)
        let constraints_str = get(col.constraints).trim().to_string();
        if !constraints_str.is_empty() {
            field.constraints = parse_constraint_string(&constraints_str);
        }

        obj.fields.push(field);
    }

    Ok(IrNode::Object(obj))
}

// ── Raw data mode ─────────────────────────────────────────────────────────────

fn read_as_raw_data(headers: &[String], rows: &[Vec<String>]) -> Result<IrNode, IngestionError> {
    let mut obj = IrObject::new(Some("root".to_string()));

    // For each column, collect type observations from sample rows
    for (col_idx, header) in headers.iter().enumerate() {
        let mut type_observations = Vec::new();
        let mut examples = Vec::new();

        for row in rows {
            let val = row.get(col_idx).map(|s| s.as_str()).unwrap_or("");
            if val.is_empty() {
                type_observations.push(IrType::Unknown); // treat empty as null
            } else {
                let t = infer_type_from_string(val);
                type_observations.push(t);
                if examples.len() < 3 {
                    examples.push(serde_json::Value::String(val.to_string()));
                }
            }
        }

        let (unified_type, nullable) = unify_types(type_observations);
        let mut field = IrField::new(header.clone(), unified_type);
        field.nullable = nullable;
        field.examples = examples;
        obj.fields.push(field);
    }

    Ok(IrNode::Object(obj))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn parse_bool_str(s: &str) -> Option<bool> {
    match s {
        "true" | "yes" | "1" | "y" | "t" => Some(true),
        "false" | "no" | "0" | "n" | "f" => Some(false),
        _ => None,
    }
}

fn parse_constraint_string(s: &str) -> Vec<crate::ir::model::IrConstraint> {
    use crate::ir::model::IrConstraint;
    let mut constraints = Vec::new();

    for part in s.split(';') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        let (key, value) = if let Some(pos) = part.find(':') {
            (&part[..pos], &part[pos + 1..])
        } else {
            (part, "")
        };

        match key.trim().to_lowercase().as_str() {
            "minlength" | "min_length" => {
                if let Ok(n) = value.trim().parse::<u64>() {
                    constraints.push(IrConstraint::MinLength(n));
                }
            }
            "maxlength" | "max_length" => {
                if let Ok(n) = value.trim().parse::<u64>() {
                    constraints.push(IrConstraint::MaxLength(n));
                }
            }
            "pattern" | "regex" => {
                constraints.push(IrConstraint::Pattern(value.trim().to_string()));
            }
            "minimum" | "min" => {
                if let Ok(f) = value.trim().parse::<f64>() {
                    constraints.push(IrConstraint::Minimum(f));
                }
            }
            "maximum" | "max" => {
                if let Ok(f) = value.trim().parse::<f64>() {
                    constraints.push(IrConstraint::Maximum(f));
                }
            }
            "unique" | "uniqueitems" => {
                constraints.push(IrConstraint::UniqueItems);
            }
            other => {
                constraints.push(IrConstraint::Custom(
                    other.to_string(),
                    serde_json::Value::String(value.trim().to_string()),
                ));
            }
        }
    }

    constraints
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_data_dictionary_csv() {
        let input = b"field_name,type,description,nullable,required\n\
                      id,integer,Primary key,false,true\n\
                      email,string,User email,false,true\n\
                      age,integer,User age,true,false\n";

        let reader = CsvReader;
        let hint = FormatHint::from_filename("dict.csv");
        let doc = reader.read(input, &hint).unwrap();

        assert_eq!(doc.source_format, SourceFormat::DataDictionary);
        if let IrNode::Object(obj) = &doc.root {
            assert_eq!(obj.fields.len(), 3);

            let id_field = obj.fields.iter().find(|f| f.name == "id").unwrap();
            assert!(matches!(id_field.ir_type, IrType::Integer));
            assert!(!id_field.nullable);
            assert!(id_field.required);

            let age_field = obj.fields.iter().find(|f| f.name == "age").unwrap();
            assert!(age_field.nullable);
        } else {
            panic!("Expected IrNode::Object");
        }
    }

    #[test]
    fn test_read_raw_csv() {
        let input = b"id,name,score,active\n\
                      1,Alice,9.5,true\n\
                      2,Bob,8.0,false\n\
                      3,Carol,7.5,true\n";

        let reader = CsvReader;
        let hint = FormatHint::from_filename("data.csv");
        let doc = reader.read(input, &hint).unwrap();

        // No field_name column → raw data mode
        if let IrNode::Object(obj) = &doc.root {
            assert_eq!(obj.fields.len(), 4);
            let id_field = obj.fields.iter().find(|f| f.name == "id").unwrap();
            assert!(matches!(id_field.ir_type, IrType::Integer));
        } else {
            panic!("Expected IrNode::Object");
        }
    }
}
