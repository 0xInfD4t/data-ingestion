use crate::error::IngestionError;
use crate::ingestion::traits::{FormatHint, FormatReader};
use crate::ingestion::type_inference::{infer_type_from_json, infer_type_from_type_string};
use crate::ir::model::{IrDocument, IrField, IrNode, IrObject, IrType, SourceFormat};

/// Reads YAML files — data dictionaries, JSON Schema-like structures, or raw data.
pub struct YamlReader;

impl FormatReader for YamlReader {
    fn can_read(&self, hint: &FormatHint) -> bool {
        if let Some(ext) = hint.extension() {
            if ext.eq_ignore_ascii_case("yaml") || ext.eq_ignore_ascii_case("yml") {
                return true;
            }
        }
        if let Some(explicit) = &hint.explicit_format {
            return matches!(
                explicit,
                SourceFormat::DataDictionary | SourceFormat::DataStructure | SourceFormat::DataSchema
            );
        }
        true
    }

    fn read(&self, input: &[u8], hint: &FormatHint) -> Result<IrDocument, IngestionError> {
        let text = std::str::from_utf8(input).map_err(|e| {
            IngestionError::parse("Yaml", "$", format!("Invalid UTF-8 in YAML: {}", e))
        })?;

        let value: serde_yaml::Value = serde_yaml::from_str(text).map_err(|e| {
            IngestionError::parse("Yaml", "$", format!("Failed to parse YAML: {}", e))
        })?;

        // Convert serde_yaml::Value to serde_json::Value for uniform processing
        let json_value = yaml_to_json(value).map_err(|e| {
            IngestionError::parse("Yaml", "$", format!("Failed to convert YAML to JSON: {}", e))
        })?;

        let (root, source_format) = parse_yaml_value(&json_value)?;

        Ok(IrDocument::new(
            source_format,
            hint.filename.clone(),
            root,
        ))
    }
}

// ── YAML value parsing ────────────────────────────────────────────────────────

fn parse_yaml_value(value: &serde_json::Value) -> Result<(IrNode, SourceFormat), IngestionError> {
    match value {
        serde_json::Value::Array(arr) => {
            // Check if it's a data dictionary (list of field descriptor objects)
            if is_data_dictionary_array(arr) {
                let root = read_as_data_dictionary(arr)?;
                return Ok((root, SourceFormat::DataDictionary));
            }
            // Otherwise treat as raw data
            let root = read_as_raw_data_array(arr)?;
            Ok((root, SourceFormat::DataStructure))
        }
        serde_json::Value::Object(map) => {
            // Check for JSON Schema-like structure
            if map.contains_key("$schema")
                || map.contains_key("properties")
                || map.contains_key("$defs")
                || map.contains_key("definitions")
            {
                // Delegate to JSON Schema reader
                let json_bytes = serde_json::to_vec(value).map_err(|e| {
                    IngestionError::parse("Yaml", "$", format!("Failed to serialize YAML as JSON: {}", e))
                })?;
                use crate::ingestion::reader_json_schema::JsonSchemaReader;
                use crate::ingestion::traits::FormatReader;
                let hint = crate::ingestion::traits::FormatHint::default();
                let doc = JsonSchemaReader.read(&json_bytes, &hint)?;
                return Ok((doc.root, SourceFormat::DataSchema));
            }

            // Check if it's a data dictionary (map with "fields" key)
            if let Some(serde_json::Value::Array(fields)) = map.get("fields") {
                if is_data_dictionary_array(fields) {
                    let root = read_as_data_dictionary(fields)?;
                    return Ok((root, SourceFormat::DataDictionary));
                }
            }

            // Generic object structure
            let root = read_as_object(map)?;
            Ok((root, SourceFormat::DataStructure))
        }
        _ => {
            Err(IngestionError::parse(
                "Yaml",
                "$",
                "YAML root must be an array or object",
            ))
        }
    }
}

// ── Data dictionary detection ─────────────────────────────────────────────────

fn is_data_dictionary_array(arr: &[serde_json::Value]) -> bool {
    if arr.is_empty() {
        return false;
    }

    // Check first few items for data dictionary structure
    let sample = &arr[..arr.len().min(3)];
    let dict_keys = ["name", "field_name", "column_name", "type", "data_type", "description"];

    sample.iter().any(|item| {
        if let serde_json::Value::Object(map) = item {
            let matching = dict_keys.iter().filter(|k| map.contains_key(**k)).count();
            matching >= 2
        } else {
            false
        }
    })
}

// ── Data dictionary mode ──────────────────────────────────────────────────────

fn read_as_data_dictionary(arr: &[serde_json::Value]) -> Result<IrNode, IngestionError> {
    let mut obj = IrObject::new(Some("data_dictionary".to_string()));

    for (idx, item) in arr.iter().enumerate() {
        let map = match item {
            serde_json::Value::Object(m) => m,
            _ => {
                log::warn!("Skipping non-object item at index {} in YAML data dictionary", idx);
                continue;
            }
        };

        // Extract field name
        let field_name = get_str(map, &["name", "field_name", "column_name", "field", "attribute"])
            .unwrap_or_else(|| format!("field_{}", idx));

        // Extract type
        let type_str = get_str(map, &["type", "data_type", "dtype", "logical_type"]);
        let ir_type = type_str
            .as_deref()
            .map(infer_type_from_type_string)
            .unwrap_or(IrType::Unknown);

        let mut field = IrField::new(field_name, ir_type);

        // Description
        if let Some(desc) = get_str(map, &["description", "desc", "comment", "notes"]) {
            field.description = Some(desc);
        }

        // Nullable
        if let Some(nullable_val) = map.get("nullable").or_else(|| map.get("optional")).or_else(|| map.get("is_nullable")) {
            field.nullable = coerce_bool(nullable_val).unwrap_or(true);
        }

        // Required
        if let Some(required_val) = map.get("required").or_else(|| map.get("is_required")) {
            field.required = coerce_bool(required_val).unwrap_or(false);
        }

        // Default value
        if let Some(default) = map.get("default").or_else(|| map.get("default_value")) {
            field.default_value = Some(default.clone());
        }

        // Examples
        if let Some(examples) = map.get("examples").or_else(|| map.get("example")) {
            match examples {
                serde_json::Value::Array(arr) => field.examples = arr.clone(),
                other => field.examples = vec![other.clone()],
            }
        }

        // Tags
        if let Some(tags_val) = map.get("tags").or_else(|| map.get("labels")) {
            match tags_val {
                serde_json::Value::Array(arr) => {
                    field.tags = arr
                        .iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect();
                }
                serde_json::Value::String(s) => {
                    field.tags = s.split(',').map(|t| t.trim().to_string()).collect();
                }
                _ => {}
            }
        }

        // PII
        if let Some(pii_val) = map.get("pii").or_else(|| map.get("is_pii")).or_else(|| map.get("sensitive")) {
            if let Some(pii) = coerce_bool(pii_val) {
                field.metadata.insert("pii".to_string(), serde_json::Value::Bool(pii));
            }
        }

        // Classification
        if let Some(class_val) = map.get("classification").or_else(|| map.get("data_class")) {
            field.metadata.insert("classification".to_string(), class_val.clone());
        }

        // Deprecated
        if let Some(dep_val) = map.get("deprecated") {
            field.deprecated = coerce_bool(dep_val).unwrap_or(false);
        }

        obj.fields.push(field);
    }

    Ok(IrNode::Object(obj))
}

// ── Raw data array mode ───────────────────────────────────────────────────────

fn read_as_raw_data_array(arr: &[serde_json::Value]) -> Result<IrNode, IngestionError> {
    if arr.is_empty() {
        return Ok(IrNode::Object(IrObject::new(Some("root".to_string()))));
    }

    // Collect all keys
    let mut all_keys: Vec<String> = Vec::new();
    for item in arr.iter().take(100) {
        if let serde_json::Value::Object(map) = item {
            for key in map.keys() {
                if !all_keys.contains(key) {
                    all_keys.push(key.clone());
                }
            }
        }
    }

    let mut obj = IrObject::new(Some("root".to_string()));

    for key in &all_keys {
        let mut type_obs = Vec::new();
        let mut examples = Vec::new();

        for item in arr.iter().take(100) {
            if let serde_json::Value::Object(map) = item {
                let val = map.get(key).unwrap_or(&serde_json::Value::Null);
                type_obs.push(infer_type_from_json(val));
                if examples.len() < 3 && !val.is_null() {
                    examples.push(val.clone());
                }
            }
        }

        let (unified, nullable) = crate::ingestion::type_inference::unify_types(type_obs);
        let mut field = IrField::new(key.clone(), unified);
        field.nullable = nullable;
        field.examples = examples;
        obj.fields.push(field);
    }

    Ok(IrNode::Object(obj))
}

// ── Generic object mode ───────────────────────────────────────────────────────

fn read_as_object(map: &serde_json::Map<String, serde_json::Value>) -> Result<IrNode, IngestionError> {
    let mut obj = IrObject::new(Some("root".to_string()));

    for (key, val) in map {
        let ir_type = infer_type_from_json(val);
        let mut field = IrField::new(key.clone(), ir_type);
        field.nullable = val.is_null();
        if !val.is_null() {
            field.examples = vec![val.clone()];
        }
        obj.fields.push(field);
    }

    Ok(IrNode::Object(obj))
}

// ── YAML → JSON conversion ────────────────────────────────────────────────────

fn yaml_to_json(value: serde_yaml::Value) -> Result<serde_json::Value, String> {
    match value {
        serde_yaml::Value::Null => Ok(serde_json::Value::Null),
        serde_yaml::Value::Bool(b) => Ok(serde_json::Value::Bool(b)),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(serde_json::Value::Number(serde_json::Number::from(i)))
            } else if let Some(f) = n.as_f64() {
                serde_json::Number::from_f64(f)
                    .map(serde_json::Value::Number)
                    .ok_or_else(|| format!("Invalid float: {}", f))
            } else {
                Ok(serde_json::Value::Null)
            }
        }
        serde_yaml::Value::String(s) => Ok(serde_json::Value::String(s)),
        serde_yaml::Value::Sequence(seq) => {
            let arr: Result<Vec<_>, _> = seq.into_iter().map(yaml_to_json).collect();
            Ok(serde_json::Value::Array(arr?))
        }
        serde_yaml::Value::Mapping(map) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in map {
                let key = match k {
                    serde_yaml::Value::String(s) => s,
                    other => format!("{:?}", other),
                };
                obj.insert(key, yaml_to_json(v)?);
            }
            Ok(serde_json::Value::Object(obj))
        }
        serde_yaml::Value::Tagged(tagged) => yaml_to_json(tagged.value),
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn get_str(map: &serde_json::Map<String, serde_json::Value>, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(serde_json::Value::String(s)) = map.get(*key) {
            return Some(s.clone());
        }
    }
    None
}

fn coerce_bool(val: &serde_json::Value) -> Option<bool> {
    match val {
        serde_json::Value::Bool(b) => Some(*b),
        serde_json::Value::String(s) => match s.to_lowercase().as_str() {
            "true" | "yes" | "1" | "y" => Some(true),
            "false" | "no" | "0" | "n" => Some(false),
            _ => None,
        },
        serde_json::Value::Number(n) => n.as_i64().map(|i| i != 0),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_yaml_data_dictionary() {
        let input = b"- name: id\n  type: integer\n  description: Primary key\n  nullable: false\n  required: true\n- name: email\n  type: string\n  description: User email\n  nullable: false\n";

        let reader = YamlReader;
        let hint = FormatHint::from_filename("dict.yaml");
        let doc = reader.read(input, &hint).unwrap();

        assert_eq!(doc.source_format, SourceFormat::DataDictionary);
        if let IrNode::Object(obj) = &doc.root {
            assert_eq!(obj.fields.len(), 2);
            let id_field = obj.fields.iter().find(|f| f.name == "id").unwrap();
            assert!(matches!(id_field.ir_type, IrType::Integer));
            assert!(!id_field.nullable);
            assert!(id_field.required);
        } else {
            panic!("Expected IrNode::Object");
        }
    }

    #[test]
    fn test_read_yaml_schema_like() {
        let input = b"type: object\nproperties:\n  id:\n    type: string\n    format: uuid\n  name:\n    type: string\n";

        let reader = YamlReader;
        let hint = FormatHint::from_filename("schema.yaml");
        let doc = reader.read(input, &hint).unwrap();

        // Should be parsed as DataSchema (JSON Schema-like)
        assert_eq!(doc.source_format, SourceFormat::DataSchema);
    }
}
