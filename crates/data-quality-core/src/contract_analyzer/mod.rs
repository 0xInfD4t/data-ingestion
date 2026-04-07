use data_ingestion_core::DataContract;

use crate::config::DqConfig;
use crate::expectations::ExpectationSuite;

pub mod constraint_analyzer;
pub mod field_analyzer;
pub mod pii_analyzer;
pub mod schema_analyzer;

/// Analyzes a DataContract and generates contract-specific expectation suites.
pub struct ContractAnalyzer<'a> {
    contract: &'a DataContract,
    config: &'a DqConfig,
}

impl<'a> ContractAnalyzer<'a> {
    pub fn new(contract: &'a DataContract, config: &'a DqConfig) -> Self {
        Self { contract, config }
    }

    /// Runs all sub-analyzers and returns the contract-specific suites.
    /// Always produces: schema_suite, field_suite, constraints_suite.
    /// Conditionally produces: pii_suite (only if any field has pii=true).
    pub fn analyze(&self) -> Vec<ExpectationSuite> {
        let mut suites = Vec::new();

        let schema = schema_analyzer::build_schema_suite(self.contract, self.config);
        if !schema.expectations.is_empty() {
            suites.push(schema);
        }

        let field = field_analyzer::build_field_suite(self.contract, self.config);
        if !field.expectations.is_empty() {
            suites.push(field);
        }

        let constraints = constraint_analyzer::build_constraints_suite(self.contract, self.config);
        if !constraints.expectations.is_empty() {
            suites.push(constraints);
        }

        if self.contract.fields.iter().any(|f| f.pii) {
            let pii = pii_analyzer::build_pii_suite(self.contract, self.config);
            if !pii.expectations.is_empty() {
                suites.push(pii);
            }
        }

        suites
    }
}

/// Returns the first N uppercase chars of a contract name as the abbreviation.
pub fn contract_abbrev(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric())
        .take(3)
        .collect::<String>()
        .to_uppercase()
}

/// Format a contract-specific test ID like "ORD-FLD-001"
pub fn fmt_contract_test_id(abbrev: &str, kind: &str, n: usize) -> String {
    if n < 10 {
        format!("{}-{}-00{}", abbrev, kind, n)
    } else if n < 100 {
        format!("{}-{}-0{}", abbrev, kind, n)
    } else {
        format!("{}-{}-{}", abbrev, kind, n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use data_ingestion_core::{
        ContractField, DataClassification, DataContract, FieldConstraint, LogicalType,
    };
    use std::collections::HashMap;

    fn make_test_contract(fields: Vec<ContractField>) -> DataContract {
        DataContract {
            id: "test-id".to_string(),
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            owner: None,
            domain: None,
            source_format: "json".to_string(),
            fields,
            metadata: HashMap::new(),
            sla: None,
            lineage: None,
            quality: None,
            created_at: None,
            tags: vec![],
        }
    }

    fn make_field(name: &str) -> ContractField {
        ContractField {
            name: name.to_string(),
            logical_type: LogicalType::String,
            physical_type: None,
            nullable: true,
            required: false,
            primary_key: false,
            foreign_key: None,
            unique: false,
            description: None,
            constraints: vec![],
            example: None,
            default_value: None,
            pii: false,
            classification: DataClassification::Internal,
            tags: vec![],
            metadata: HashMap::new(),
            nested_fields: vec![],
        }
    }

    #[test]
    fn test_contract_abbrev() {
        assert_eq!(contract_abbrev("order"), "ORD");
        assert_eq!(contract_abbrev("customer"), "CUS");
        assert_eq!(contract_abbrev("patient_record"), "PAT");
        assert_eq!(contract_abbrev("x"), "X");
    }

    #[test]
    fn test_nullable_false_generates_not_null() {
        let field = ContractField {
            nullable: false,
            ..make_field("order_id")
        };
        let contract = make_test_contract(vec![field]);
        let config = DqConfig {
            include_baseline: false,
            ..Default::default()
        };
        let analyzer = ContractAnalyzer::new(&contract, &config);
        let suites = analyzer.analyze();
        let field_suite = suites.iter().find(|s| s.name.ends_with("_field_suite")).unwrap();
        assert!(field_suite.expectations.iter().any(|e| {
            e.expectation_type == "expect_column_values_to_not_be_null"
                && e.kwargs.get("column").and_then(|v| v.as_str()) == Some("order_id")
        }));
    }

    #[test]
    fn test_primary_key_generates_unique_and_not_null() {
        let field = ContractField {
            primary_key: true,
            ..make_field("id")
        };
        let contract = make_test_contract(vec![field]);
        let config = DqConfig {
            include_baseline: false,
            ..Default::default()
        };
        let analyzer = ContractAnalyzer::new(&contract, &config);
        let suites = analyzer.analyze();
        let field_suite = suites.iter().find(|s| s.name.ends_with("_field_suite")).unwrap();
        let has_not_null = field_suite.expectations.iter().any(|e| {
            e.expectation_type == "expect_column_values_to_not_be_null"
                && e.kwargs.get("column").and_then(|v| v.as_str()) == Some("id")
        });
        let has_unique = field_suite.expectations.iter().any(|e| {
            e.expectation_type == "expect_column_values_to_be_unique"
                && e.kwargs.get("column").and_then(|v| v.as_str()) == Some("id")
        });
        assert!(has_not_null && has_unique);
    }

    #[test]
    fn test_pii_field_generates_pii_suite() {
        let field = ContractField {
            pii: true,
            ..make_field("ssn")
        };
        let contract = make_test_contract(vec![field]);
        let config = DqConfig {
            include_baseline: false,
            ..Default::default()
        };
        let analyzer = ContractAnalyzer::new(&contract, &config);
        let suites = analyzer.analyze();
        assert!(suites.iter().any(|s| s.name.ends_with("_pii_suite")));
    }

    #[test]
    fn test_no_pii_fields_no_pii_suite() {
        let field = make_field("name");
        let contract = make_test_contract(vec![field]);
        let config = DqConfig {
            include_baseline: false,
            ..Default::default()
        };
        let analyzer = ContractAnalyzer::new(&contract, &config);
        let suites = analyzer.analyze();
        assert!(!suites.iter().any(|s| s.name.ends_with("_pii_suite")));
    }

    #[test]
    fn test_min_max_constraint_merges_into_single_between() {
        let field = ContractField {
            constraints: vec![
                FieldConstraint::Minimum(0.0),
                FieldConstraint::Maximum(100.0),
            ],
            logical_type: LogicalType::Float,
            ..make_field("score")
        };
        let contract = make_test_contract(vec![field]);
        let config = DqConfig {
            include_baseline: false,
            ..Default::default()
        };
        let analyzer = ContractAnalyzer::new(&contract, &config);
        let suites = analyzer.analyze();
        let con_suite = suites
            .iter()
            .find(|s| s.name.ends_with("_constraints_suite"))
            .unwrap();
        let between_count = con_suite
            .expectations
            .iter()
            .filter(|e| {
                e.expectation_type == "expect_column_values_to_be_between"
                    && e.kwargs.get("column").and_then(|v| v.as_str()) == Some("score")
            })
            .count();
        assert_eq!(
            between_count, 1,
            "Min+Max should merge into one between expectation"
        );
    }

    #[test]
    fn test_allowed_values_generates_in_set() {
        let field = ContractField {
            constraints: vec![FieldConstraint::AllowedValues(vec![
                "A".into(),
                "B".into(),
                "C".into(),
            ])],
            ..make_field("status")
        };
        let contract = make_test_contract(vec![field]);
        let config = DqConfig {
            include_baseline: false,
            ..Default::default()
        };
        let analyzer = ContractAnalyzer::new(&contract, &config);
        let suites = analyzer.analyze();
        let con_suite = suites
            .iter()
            .find(|s| s.name.ends_with("_constraints_suite"))
            .unwrap();
        assert!(con_suite.expectations.iter().any(|e| {
            e.expectation_type == "expect_column_values_to_be_in_set"
                && e.kwargs.get("column").and_then(|v| v.as_str()) == Some("status")
        }));
    }

    #[test]
    fn test_restricted_classification_generates_encrypted() {
        let field = ContractField {
            classification: DataClassification::Restricted,
            pii: true,
            ..make_field("secret")
        };
        let contract = make_test_contract(vec![field]);
        let config = DqConfig {
            include_baseline: false,
            ..Default::default()
        };
        let analyzer = ContractAnalyzer::new(&contract, &config);
        let suites = analyzer.analyze();
        let pii_suite = suites
            .iter()
            .find(|s| s.name.ends_with("_pii_suite"))
            .unwrap();
        assert!(pii_suite
            .expectations
            .iter()
            .any(|e| e.expectation_type == "expect_column_values_to_be_encrypted"));
    }
}
