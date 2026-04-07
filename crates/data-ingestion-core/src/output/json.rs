use crate::contract::model::DataContract;
use crate::error::OutputError;

/// Serialize a [`DataContract`] to pretty-printed JSON bytes.
pub fn to_json(contract: &DataContract) -> Result<Vec<u8>, OutputError> {
    log::debug!("output::json: serializing contract '{}'", contract.name);

    let json = serde_json::to_string_pretty(contract).map_err(|e| {
        OutputError::SerializationFailed {
            format: "json".to_string(),
            reason: e.to_string(),
        }
    })?;

    Ok(json.into_bytes())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::model::{ContractField, DataClassification, DataContract, LogicalType};
    use std::collections::HashMap;

    fn make_test_contract() -> DataContract {
        DataContract {
            id: "abc-123".to_string(),
            name: "TestContract".to_string(),
            version: "1.0.0".to_string(),
            description: Some("A test contract".to_string()),
            owner: None,
            domain: None,
            source_format: "JsonSchema".to_string(),
            fields: vec![ContractField {
                name: "user_id".to_string(),
                logical_type: LogicalType::Integer,
                physical_type: Some("Integer".to_string()),
                nullable: false,
                required: true,
                primary_key: true,
                foreign_key: None,
                unique: true,
                description: Some("Primary key".to_string()),
                constraints: Vec::new(),
                example: Some("42".to_string()),
                default_value: None,
                pii: false,
                classification: DataClassification::Internal,
                tags: vec!["pk".to_string()],
                metadata: HashMap::new(),
                nested_fields: Vec::new(),
            }],
            metadata: HashMap::new(),
            sla: None,
            lineage: None,
            quality: None,
            created_at: None,
            tags: Vec::new(),
        }
    }

    #[test]
    fn test_json_output_is_valid_json() {
        let contract = make_test_contract();
        let bytes = to_json(&contract).expect("serialization should succeed");
        let s = std::str::from_utf8(&bytes).expect("output should be valid UTF-8");
        // Must parse as valid JSON
        let parsed: serde_json::Value =
            serde_json::from_str(s).expect("output should be valid JSON");
        assert!(parsed.is_object());
    }

    #[test]
    fn test_json_output_contains_field_names() {
        let contract = make_test_contract();
        let bytes = to_json(&contract).expect("serialization should succeed");
        let s = std::str::from_utf8(&bytes).expect("output should be valid UTF-8");
        assert!(s.contains("user_id"), "JSON should contain field name 'user_id'");
        assert!(s.contains("TestContract"), "JSON should contain contract name");
    }

    #[test]
    fn test_json_output_contains_logical_type() {
        let contract = make_test_contract();
        let bytes = to_json(&contract).expect("serialization should succeed");
        let s = std::str::from_utf8(&bytes).expect("output should be valid UTF-8");
        assert!(s.contains("integer"), "JSON should contain logical type 'integer'");
    }
}
