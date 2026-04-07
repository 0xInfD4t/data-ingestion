use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::contract::model::{ContractField, DataContract, LogicalType};

// ── ValidationResult ──────────────────────────────────────────────────────────

/// Result of validating a [`DataContract`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// `true` if no errors were found (warnings are allowed).
    pub valid: bool,
    /// Non-fatal issues that should be reviewed.
    pub warnings: Vec<String>,
    /// Fatal issues that make the contract invalid.
    pub errors: Vec<String>,
}

impl ValidationResult {
    fn new() -> Self {
        Self {
            valid: true,
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    fn add_error(&mut self, msg: impl Into<String>) {
        self.errors.push(msg.into());
        self.valid = false;
    }

    fn add_warning(&mut self, msg: impl Into<String>) {
        self.warnings.push(msg.into());
    }
}

// ── ContractValidator ─────────────────────────────────────────────────────────

/// Validates a [`DataContract`] for completeness and consistency.
pub struct ContractValidator;

impl ContractValidator {
    /// Validate a [`DataContract`] and return a [`ValidationResult`].
    ///
    /// Validation rules:
    /// - **Error** if `contract.fields` is empty
    /// - **Error** if any field has an empty `name`
    /// - **Error** if two fields share the same name (duplicate detection)
    /// - **Warning** if `contract.name` is `"unnamed"`
    /// - **Warning** if any field has `description = None`
    /// - **Warning** if any field has `logical_type = LogicalType::Unknown`
    pub fn validate(contract: &DataContract) -> ValidationResult {
        let mut result = ValidationResult::new();

        // Rule: contract must have at least one field
        if contract.fields.is_empty() {
            result.add_error("Contract has no fields");
        }

        // Rule: contract name should not be "unnamed"
        if contract.name == "unnamed" {
            result.add_warning("Contract name is 'unnamed'; consider providing a meaningful name");
        }

        // Validate individual fields (top-level only; nested fields validated recursively)
        let mut seen_names: HashSet<String> = HashSet::new();
        for field in &contract.fields {
            Self::validate_field(field, &mut result, &mut seen_names);
        }

        result
    }

    /// Recursively validate a single field and its nested fields.
    fn validate_field(
        field: &ContractField,
        result: &mut ValidationResult,
        seen_names: &mut HashSet<String>,
    ) {
        // Rule: field name must not be empty
        if field.name.is_empty() {
            result.add_error("A field has an empty name");
        }

        // Rule: no duplicate field names
        if !field.name.is_empty() {
            if seen_names.contains(&field.name) {
                result.add_error(format!(
                    "Duplicate field name: '{}'",
                    field.name
                ));
            } else {
                seen_names.insert(field.name.clone());
            }
        }

        // Rule: fields should have descriptions
        if field.description.is_none() {
            result.add_warning(format!(
                "Field '{}' has no description",
                field.name
            ));
        }

        // Rule: fields should not have Unknown logical type
        if field.logical_type == LogicalType::Unknown {
            result.add_warning(format!(
                "Field '{}' has logical_type 'unknown'",
                field.name
            ));
        }

        // Recursively validate nested fields (use a separate seen set per nesting level)
        if !field.nested_fields.is_empty() {
            let mut nested_seen: HashSet<String> = HashSet::new();
            for nested in &field.nested_fields {
                Self::validate_field(nested, result, &mut nested_seen);
            }
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::model::{ContractField, DataClassification, DataContract, LogicalType};
    use std::collections::HashMap;

    fn make_field(name: &str, logical_type: LogicalType) -> ContractField {
        ContractField {
            name: name.to_string(),
            logical_type,
            physical_type: None,
            nullable: false,
            required: true,
            primary_key: false,
            foreign_key: None,
            unique: false,
            description: Some("A test field".to_string()),
            constraints: Vec::new(),
            example: None,
            default_value: None,
            pii: false,
            classification: DataClassification::Internal,
            tags: Vec::new(),
            metadata: HashMap::new(),
            nested_fields: Vec::new(),
        }
    }

    fn make_valid_contract() -> DataContract {
        DataContract {
            id: "test-id".to_string(),
            name: "TestContract".to_string(),
            version: "1.0.0".to_string(),
            description: Some("A test contract".to_string()),
            owner: None,
            domain: None,
            source_format: "JsonSchema".to_string(),
            fields: vec![
                make_field("id", LogicalType::Integer),
                make_field("name", LogicalType::String),
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
    fn test_valid_contract_passes() {
        let contract = make_valid_contract();
        let result = ContractValidator::validate(&contract);
        assert!(result.valid, "valid contract should pass: {:?}", result.errors);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_empty_fields_fails() {
        let mut contract = make_valid_contract();
        contract.fields.clear();
        let result = ContractValidator::validate(&contract);
        assert!(!result.valid);
        assert!(
            result.errors.iter().any(|e| e.contains("no fields")),
            "expected 'no fields' error, got: {:?}",
            result.errors
        );
    }

    #[test]
    fn test_unnamed_contract_warns() {
        let mut contract = make_valid_contract();
        contract.name = "unnamed".to_string();
        let result = ContractValidator::validate(&contract);
        // Should still be valid (warning only)
        assert!(result.valid);
        assert!(
            result.warnings.iter().any(|w| w.contains("unnamed")),
            "expected 'unnamed' warning, got: {:?}",
            result.warnings
        );
    }

    #[test]
    fn test_empty_field_name_fails() {
        let mut contract = make_valid_contract();
        contract.fields.push(make_field("", LogicalType::String));
        let result = ContractValidator::validate(&contract);
        assert!(!result.valid);
        assert!(
            result.errors.iter().any(|e| e.contains("empty name")),
            "expected 'empty name' error, got: {:?}",
            result.errors
        );
    }

    #[test]
    fn test_duplicate_field_name_fails() {
        let mut contract = make_valid_contract();
        contract.fields.push(make_field("id", LogicalType::String));
        let result = ContractValidator::validate(&contract);
        assert!(!result.valid);
        assert!(
            result.errors.iter().any(|e| e.contains("Duplicate")),
            "expected 'Duplicate' error, got: {:?}",
            result.errors
        );
    }

    #[test]
    fn test_missing_description_warns() {
        let mut contract = make_valid_contract();
        let mut field = make_field("no_desc", LogicalType::String);
        field.description = None;
        contract.fields.push(field);
        let result = ContractValidator::validate(&contract);
        assert!(result.valid);
        assert!(
            result.warnings.iter().any(|w| w.contains("no_desc")),
            "expected warning for 'no_desc', got: {:?}",
            result.warnings
        );
    }

    #[test]
    fn test_unknown_type_warns() {
        let mut contract = make_valid_contract();
        contract.fields.push(make_field("mystery", LogicalType::Unknown));
        let result = ContractValidator::validate(&contract);
        assert!(result.valid);
        assert!(
            result.warnings.iter().any(|w| w.contains("mystery")),
            "expected warning for 'mystery', got: {:?}",
            result.warnings
        );
    }
}
