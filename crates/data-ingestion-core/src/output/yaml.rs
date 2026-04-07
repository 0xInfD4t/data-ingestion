use crate::contract::model::DataContract;
use crate::error::OutputError;

/// Serialize a [`DataContract`] to YAML bytes.
pub fn to_yaml(contract: &DataContract) -> Result<Vec<u8>, OutputError> {
    log::debug!("output::yaml: serializing contract '{}'", contract.name);

    let yaml = serde_yaml::to_string(contract).map_err(|e| OutputError::SerializationFailed {
        format: "yaml".to_string(),
        reason: e.to_string(),
    })?;

    Ok(yaml.into_bytes())
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
            name: "YamlTestContract".to_string(),
            version: "1.0.0".to_string(),
            description: Some("A YAML test contract".to_string()),
            owner: Some("team-data".to_string()),
            domain: None,
            source_format: "JsonSchema".to_string(),
            fields: vec![ContractField {
                name: "record_id".to_string(),
                logical_type: LogicalType::String,
                physical_type: Some("String".to_string()),
                nullable: false,
                required: true,
                primary_key: false,
                foreign_key: None,
                unique: false,
                description: Some("Record identifier".to_string()),
                constraints: Vec::new(),
                example: None,
                default_value: None,
                pii: false,
                classification: DataClassification::Internal,
                tags: Vec::new(),
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
    fn test_yaml_output_is_valid_yaml() {
        let contract = make_test_contract();
        let bytes = to_yaml(&contract).expect("serialization should succeed");
        let s = std::str::from_utf8(&bytes).expect("output should be valid UTF-8");
        // Must parse as valid YAML
        let parsed: serde_yaml::Value =
            serde_yaml::from_str(s).expect("output should be valid YAML");
        assert!(parsed.is_mapping());
    }

    #[test]
    fn test_yaml_output_contains_contract_name() {
        let contract = make_test_contract();
        let bytes = to_yaml(&contract).expect("serialization should succeed");
        let s = std::str::from_utf8(&bytes).expect("output should be valid UTF-8");
        assert!(
            s.contains("YamlTestContract"),
            "YAML should contain contract name"
        );
    }

    #[test]
    fn test_yaml_output_contains_field_name() {
        let contract = make_test_contract();
        let bytes = to_yaml(&contract).expect("serialization should succeed");
        let s = std::str::from_utf8(&bytes).expect("output should be valid UTF-8");
        assert!(
            s.contains("record_id"),
            "YAML should contain field name 'record_id'"
        );
    }
}
