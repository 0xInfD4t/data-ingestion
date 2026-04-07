use std::collections::HashMap;

use serde_json::Value;

use crate::error::IngestionError;
use crate::ingestion::traits::{FormatHint, FormatReader};
use crate::ingestion::type_inference::{infer_type_from_json, unify_types};
use crate::ir::model::{
    IrArray, IrDocument, IrField, IrNode, IrObject, IrType, SourceFormat,
};

const MAX_SAMPLE_ROWS: usize = 1000;

/// Reads raw JSON datasets (arrays of objects or single objects).
pub struct JsonDatasetReader;

impl FormatReader for JsonDatasetReader {
    fn can_read(&self, hint: &FormatHint) -> bool {
        if let Some(ext) = hint.extension() {
            if ext.eq_ignore_ascii_case("json") {
                return true;
            }
        }
        if let Some(explicit) = &hint.explicit_format {
            return matches!(explicit, SourceFormat::JsonDataset);
        }
        true // fallback: try to read
    }

    fn read(&self, input: &[u8], hint: &FormatHint) -> Result<IrDocument, IngestionError> {
        let value: Value = serde_json::from_slice(input).map_err(|e| {
            IngestionError::parse_with_source(
                "JsonDataset",
                "$",
                format!("Failed to parse JSON: {}", e),
                Box::new(e),
            )
        })?;

        let root = match &value {
            Value::Array(arr) => read_json_array(arr)?,
            Value::Object(_) => read_json_object(&value, None)?,
            _ => {
                return Err(IngestionError::parse(
                    "JsonDataset",
                    "$",
                    "Root JSON value must be an array or object",
                ));
            }
        };

        Ok(IrDocument::new(
            SourceFormat::JsonDataset,
            hint.filename.clone(),
            root,
        ))
    }
}

// ── Array of objects (tabular) ────────────────────────────────────────────────

fn read_json_array(arr: &[Value]) -> Result<IrNode, IngestionError> {
    if arr.is_empty() {
        let obj = IrObject::new(Some("root".to_string()));
        return Ok(IrNode::Object(obj));
    }

    // Sample up to MAX_SAMPLE_ROWS rows
    let sample = &arr[..arr.len().min(MAX_SAMPLE_ROWS)];

    // Collect all keys across all rows
    let mut all_keys: Vec<String> = Vec::new();
    for row in sample {
        if let Value::Object(map) = row {
            for key in map.keys() {
                if !all_keys.contains(key) {
                    all_keys.push(key.clone());
                }
            }
        }
    }

    // For each key, collect all observed types
    let mut field_type_observations: HashMap<String, Vec<IrType>> = HashMap::new();
    let mut field_values: HashMap<String, Vec<&Value>> = HashMap::new();

    for row in sample {
        if let Value::Object(map) = row {
            for key in &all_keys {
                let val = map.get(key).unwrap_or(&Value::Null);
                field_type_observations
                    .entry(key.clone())
                    .or_default()
                    .push(infer_type_from_json(val));
                field_values.entry(key.clone()).or_default().push(val);
            }
        }
    }

    // Build IrFields from observations
    let mut fields = Vec::new();
    for key in &all_keys {
        let observations = field_type_observations.remove(key).unwrap_or_default();
        let values = field_values.get(key).cloned().unwrap_or_default();

        let (unified_type, nullable) = unify_types(observations);

        // For object/array types, recurse into the first non-null value
        let final_type = match unified_type {
            IrType::Object(_) => {
                // Find first non-null object value and recurse
                let first_obj = values.iter().find(|v| v.is_object());
                if let Some(obj_val) = first_obj {
                    match read_json_object(obj_val, Some(key.clone()))? {
                        IrNode::Object(obj) => IrType::Object(Box::new(obj)),
                        _ => IrType::Object(Box::new(IrObject::new(Some(key.clone())))),
                    }
                } else {
                    IrType::Object(Box::new(IrObject::new(Some(key.clone()))))
                }
            }
            IrType::Array(_) => {
                // Find first non-null array value and infer item type
                let first_arr = values.iter().find(|v| v.is_array());
                if let Some(Value::Array(arr_val)) = first_arr {
                    let item_type = infer_array_item_type(arr_val, key)?;
                    IrType::Array(Box::new(IrArray {
                        name: Some(key.clone()),
                        item_type: Box::new(item_type),
                        min_items: None,
                        max_items: None,
                    }))
                } else {
                    IrType::Array(Box::new(IrArray {
                        name: Some(key.clone()),
                        item_type: Box::new(IrNode::Field(IrField::new("item", IrType::Unknown))),
                        min_items: None,
                        max_items: None,
                    }))
                }
            }
            other => other,
        };

        let mut field = IrField::new(key.clone(), final_type);
        field.nullable = nullable;

        // Collect examples (up to 3 non-null values)
        let examples: Vec<serde_json::Value> = values
            .iter()
            .filter(|v| !v.is_null())
            .take(3)
            .map(|v| (*v).clone())
            .collect();
        field.examples = examples;

        fields.push(field);
    }

    let mut obj = IrObject::new(Some("root".to_string()));
    obj.fields = fields;
    Ok(IrNode::Object(obj))
}

// ── Single object (hierarchical) ──────────────────────────────────────────────

fn read_json_object(value: &Value, name: Option<String>) -> Result<IrNode, IngestionError> {
    let map = match value {
        Value::Object(m) => m,
        _ => {
            return Err(IngestionError::parse(
                "JsonDataset",
                "$",
                "Expected JSON object",
            ));
        }
    };

    let mut obj = IrObject::new(name.or(Some("root".to_string())));

    for (key, val) in map {
        let ir_type = match val {
            Value::Object(_) => {
                match read_json_object(val, Some(key.clone()))? {
                    IrNode::Object(child_obj) => IrType::Object(Box::new(child_obj)),
                    _ => IrType::Unknown,
                }
            }
            Value::Array(arr) => {
                let item_node = infer_array_item_type(arr, key)?;
                IrType::Array(Box::new(IrArray {
                    name: Some(key.clone()),
                    item_type: Box::new(item_node),
                    min_items: None,
                    max_items: None,
                }))
            }
            other => infer_type_from_json(other),
        };

        let mut field = IrField::new(key.clone(), ir_type);
        field.nullable = val.is_null();
        if !val.is_null() {
            field.examples = vec![val.clone()];
        }
        obj.fields.push(field);
    }

    Ok(IrNode::Object(obj))
}

// ── Array item type inference ─────────────────────────────────────────────────

fn infer_array_item_type(arr: &[Value], field_name: &str) -> Result<IrNode, IngestionError> {
    if arr.is_empty() {
        return Ok(IrNode::Field(IrField::new("item", IrType::Unknown)));
    }

    // Check if all items are objects → produce a child IrObject
    let all_objects = arr.iter().all(|v| v.is_object());
    if all_objects {
        return read_json_array(arr);
    }

    // Otherwise infer scalar type from items
    let types: Vec<IrType> = arr.iter().map(infer_type_from_json).collect();
    let (unified, _nullable) = unify_types(types);
    Ok(IrNode::Field(IrField::new(
        format!("{}_item", field_name),
        unified,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_json_array() {
        let input = br#"[
            {"id": 1, "name": "Alice", "email": "alice@example.com"},
            {"id": 2, "name": "Bob",   "email": null}
        ]"#;
        let reader = JsonDatasetReader;
        let hint = FormatHint::from_filename("data.json");
        let doc = reader.read(input, &hint).unwrap();
        assert_eq!(doc.source_format, SourceFormat::JsonDataset);
        if let IrNode::Object(obj) = &doc.root {
            assert_eq!(obj.fields.len(), 3);
            let email_field = obj.fields.iter().find(|f| f.name == "email").unwrap();
            assert!(email_field.nullable);
        } else {
            panic!("Expected IrNode::Object");
        }
    }

    #[test]
    fn test_read_json_object() {
        let input = br#"{"id": "550e8400-e29b-41d4-a716-446655440000", "score": 3.14}"#;
        let reader = JsonDatasetReader;
        let hint = FormatHint::default();
        let doc = reader.read(input, &hint).unwrap();
        if let IrNode::Object(obj) = &doc.root {
            let id_field = obj.fields.iter().find(|f| f.name == "id").unwrap();
            assert!(matches!(id_field.ir_type, IrType::Uuid));
            let score_field = obj.fields.iter().find(|f| f.name == "score").unwrap();
            assert!(matches!(score_field.ir_type, IrType::Float));
        } else {
            panic!("Expected IrNode::Object");
        }
    }
}
