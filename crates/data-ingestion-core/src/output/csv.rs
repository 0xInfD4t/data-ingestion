use crate::contract::model::{ContractField, DataContract, FieldConstraint};
use crate::error::OutputError;

// ── CSV column headers ────────────────────────────────────────────────────────

const HEADERS: &[&str] = &[
    "field_name",
    "logical_type",
    "physical_type",
    "nullable",
    "required",
    "primary_key",
    "foreign_key_table",
    "foreign_key_column",
    "unique",
    "pii",
    "classification",
    "description",
    "example",
    "default_value",
    "constraints",
    "tags",
];

// ── Public API ────────────────────────────────────────────────────────────────

/// Serialize a [`DataContract`] to CSV bytes.
///
/// Each row represents one field. Nested fields use dot notation for the name
/// (e.g., `address.street`). Boolean columns use `true`/`false`.
/// The `constraints` column is semicolon-separated (e.g., `min_length:5;max_length:100`).
/// The `tags` column is semicolon-separated.
pub fn to_csv(contract: &DataContract) -> Result<Vec<u8>, OutputError> {
    log::debug!("output::csv: serializing contract '{}'", contract.name);

    let mut wtr = csv::WriterBuilder::new()
        .has_headers(true)
        .from_writer(Vec::new());

    // Write header row
    wtr.write_record(HEADERS).map_err(csv_err)?;

    // Collect all rows (flattening nested fields with dot notation)
    let mut rows: Vec<Vec<String>> = Vec::new();
    for field in &contract.fields {
        collect_field_rows(field, "", &mut rows);
    }

    for row in rows {
        wtr.write_record(&row).map_err(csv_err)?;
    }

    wtr.flush().map_err(|e| OutputError::SerializationFailed {
        format: "csv".to_string(),
        reason: e.to_string(),
    })?;

    let bytes = wtr.into_inner().map_err(|e| OutputError::SerializationFailed {
        format: "csv".to_string(),
        reason: e.to_string(),
    })?;

    Ok(bytes)
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn csv_err(e: csv::Error) -> OutputError {
    OutputError::SerializationFailed {
        format: "csv".to_string(),
        reason: e.to_string(),
    }
}

/// Recursively collect CSV rows for a field and its nested fields.
/// `parent_path` is the dot-notation prefix (empty for top-level fields).
fn collect_field_rows(
    field: &ContractField,
    parent_path: &str,
    rows: &mut Vec<Vec<String>>,
) {
    let qualified_name = if parent_path.is_empty() {
        field.name.clone()
    } else {
        format!("{}.{}", parent_path, field.name)
    };

    let logical_type = field.logical_type.to_string();
    let physical_type = field.physical_type.clone().unwrap_or_default();
    let nullable = field.nullable.to_string();
    let required = field.required.to_string();
    let primary_key = field.primary_key.to_string();

    let (fk_table, fk_column) = match &field.foreign_key {
        Some(fk) => (fk.table.clone(), fk.column.clone()),
        None => (String::new(), String::new()),
    };

    let unique = field.unique.to_string();
    let pii = field.pii.to_string();
    let classification = field.classification.to_string();
    let description = field.description.clone().unwrap_or_default();
    let example = field.example.clone().unwrap_or_default();
    let default_value = field.default_value.clone().unwrap_or_default();

    let constraints = field
        .constraints
        .iter()
        .map(constraint_to_string)
        .collect::<Vec<_>>()
        .join(";");

    let tags = field.tags.join(";");

    rows.push(vec![
        qualified_name.clone(),
        logical_type,
        physical_type,
        nullable,
        required,
        primary_key,
        fk_table,
        fk_column,
        unique,
        pii,
        classification,
        description,
        example,
        default_value,
        constraints,
        tags,
    ]);

    // Recurse into nested fields using the qualified name as the new parent path
    for nested in &field.nested_fields {
        collect_field_rows(nested, &qualified_name, rows);
    }
}

fn constraint_to_string(c: &FieldConstraint) -> String {
    match c {
        FieldConstraint::MinLength(v)      => format!("min_length:{}", v),
        FieldConstraint::MaxLength(v)      => format!("max_length:{}", v),
        FieldConstraint::Pattern(p)        => format!("pattern:{}", p),
        FieldConstraint::Minimum(v)        => format!("minimum:{}", v),
        FieldConstraint::Maximum(v)        => format!("maximum:{}", v),
        FieldConstraint::AllowedValues(vs) => format!("allowed_values:{}", vs.join(",")),
        FieldConstraint::NotNull           => "not_null".to_string(),
        FieldConstraint::Unique            => "unique".to_string(),
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::model::{ContractField, DataClassification, DataContract, LogicalType};
    use std::collections::HashMap;

    fn make_test_contract() -> DataContract {
        DataContract {
            id: "csv-test-id".to_string(),
            name: "CsvTestContract".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            owner: None,
            domain: None,
            source_format: "JsonSchema".to_string(),
            fields: vec![
                ContractField {
                    name: "user_id".to_string(),
                    logical_type: LogicalType::Integer,
                    physical_type: Some("Integer".to_string()),
                    nullable: false,
                    required: true,
                    primary_key: true,
                    foreign_key: None,
                    unique: true,
                    description: Some("User primary key".to_string()),
                    constraints: vec![FieldConstraint::NotNull],
                    example: Some("1".to_string()),
                    default_value: None,
                    pii: false,
                    classification: DataClassification::Internal,
                    tags: vec!["pk".to_string()],
                    metadata: HashMap::new(),
                    nested_fields: Vec::new(),
                },
                ContractField {
                    name: "email".to_string(),
                    logical_type: LogicalType::Email,
                    physical_type: Some("String".to_string()),
                    nullable: false,
                    required: true,
                    primary_key: false,
                    foreign_key: None,
                    unique: true,
                    description: Some("User email address".to_string()),
                    constraints: vec![
                        FieldConstraint::MaxLength(255),
                        FieldConstraint::Pattern(r"^[^@]+@[^@]+$".to_string()),
                    ],
                    example: Some("user@example.com".to_string()),
                    default_value: None,
                    pii: true,
                    classification: DataClassification::Confidential,
                    tags: Vec::new(),
                    metadata: HashMap::new(),
                    nested_fields: Vec::new(),
                },
            ],
            metadata: HashMap::new(),
            sla: None,
            lineage: None,
            quality: None,
            created_at: None,
            tags: Vec::new(),
        }
    }

    #[test]
    fn test_csv_has_correct_header_row() {
        let contract = make_test_contract();
        let bytes = to_csv(&contract).expect("serialization should succeed");
        let s = std::str::from_utf8(&bytes).expect("output should be valid UTF-8");
        let first_line = s.lines().next().expect("should have at least one line");
        assert_eq!(
            first_line,
            "field_name,logical_type,physical_type,nullable,required,primary_key,\
             foreign_key_table,foreign_key_column,unique,pii,classification,description,\
             example,default_value,constraints,tags"
        );
    }

    #[test]
    fn test_csv_has_one_data_row_per_field() {
        let contract = make_test_contract();
        let bytes = to_csv(&contract).expect("serialization should succeed");
        let s = std::str::from_utf8(&bytes).expect("output should be valid UTF-8");
        // header + 2 data rows
        let lines: Vec<&str> = s.lines().collect();
        assert_eq!(lines.len(), 3, "expected header + 2 data rows, got: {:?}", lines);
    }

    #[test]
    fn test_csv_field_names_present() {
        let contract = make_test_contract();
        let bytes = to_csv(&contract).expect("serialization should succeed");
        let s = std::str::from_utf8(&bytes).expect("output should be valid UTF-8");
        assert!(s.contains("user_id"), "CSV should contain 'user_id'");
        assert!(s.contains("email"), "CSV should contain 'email'");
    }

    #[test]
    fn test_csv_boolean_columns() {
        let contract = make_test_contract();
        let bytes = to_csv(&contract).expect("serialization should succeed");
        let s = std::str::from_utf8(&bytes).expect("output should be valid UTF-8");
        // user_id row: primary_key=true, pii=false
        assert!(s.contains("true"), "CSV should contain 'true'");
        assert!(s.contains("false"), "CSV should contain 'false'");
    }

    #[test]
    fn test_csv_constraints_semicolon_separated() {
        let contract = make_test_contract();
        let bytes = to_csv(&contract).expect("serialization should succeed");
        let s = std::str::from_utf8(&bytes).expect("output should be valid UTF-8");
        // email field has max_length and pattern constraints
        assert!(s.contains("max_length:255"), "CSV should contain max_length constraint");
    }

    #[test]
    fn test_csv_nested_fields_dot_notation() {
        let mut contract = make_test_contract();
        // Add a field with a nested field
        let nested = ContractField {
            name: "street".to_string(),
            logical_type: LogicalType::String,
            physical_type: None,
            nullable: true,
            required: false,
            primary_key: false,
            foreign_key: None,
            unique: false,
            description: None,
            constraints: Vec::new(),
            example: None,
            default_value: None,
            pii: false,
            classification: DataClassification::Internal,
            tags: Vec::new(),
            metadata: HashMap::new(),
            nested_fields: Vec::new(),
        };
        let address_field = ContractField {
            name: "address".to_string(),
            logical_type: LogicalType::Struct {
                type_name: "Address".to_string(),
            },
            physical_type: Some("Object".to_string()),
            nullable: true,
            required: false,
            primary_key: false,
            foreign_key: None,
            unique: false,
            description: None,
            constraints: Vec::new(),
            example: None,
            default_value: None,
            pii: false,
            classification: DataClassification::Internal,
            tags: Vec::new(),
            metadata: HashMap::new(),
            nested_fields: vec![nested],
        };
        contract.fields.push(address_field);

        let bytes = to_csv(&contract).expect("serialization should succeed");
        let s = std::str::from_utf8(&bytes).expect("output should be valid UTF-8");
        assert!(
            s.contains("address.street"),
            "CSV should use dot notation for nested fields, got:\n{}",
            s
        );
    }
}
