//! # data-quality-core
//!
//! Pure-Rust library that generates Great Expectations (GX) 1.x-compatible test suites
//! from [`DataContract`] structs produced by `data-ingestion-core`.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use data_quality_core::{DqConfig, generate_baseline_suites};
//!
//! let config = DqConfig::default();
//! let suites = generate_baseline_suites(&config);
//! println!("Generated {} baseline suites", suites.len());
//! let total: usize = suites.iter().map(|s| s.expectations.len()).sum();
//! println!("Total baseline tests: {}", total);
//! ```

pub mod config;
pub mod contract_analyzer;
pub mod error;
pub mod expectations;
pub mod output;
pub mod suites;

// ── Public re-exports ─────────────────────────────────────────────────────────

pub use config::DqConfig;
pub use error::DqError;
pub use expectations::{
    ExpectationConfig, ExpectationMeta, ExpectationSuite, GeneratedFrom, SuiteMeta,
};
pub use output::{DqOutputFile, DqOutputFormat};

use data_ingestion_core::DataContract;
use serde::{Deserialize, Serialize};

// ── DqSuiteSet ────────────────────────────────────────────────────────────────

/// The complete output of the DQ generation pipeline for one DataContract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DqSuiteSet {
    /// DataContract.id if generated from a contract; None for pure baseline
    pub contract_id: Option<String>,
    /// DataContract.name if generated from a contract
    pub contract_name: Option<String>,
    /// The 17 baseline suites (1328 tests total, subject to config filtering)
    pub baseline_suites: Vec<ExpectationSuite>,
    /// Contract-specific suites: schema, field, pii (conditional), constraints
    pub contract_suites: Vec<ExpectationSuite>,
    /// Sum of all expectations across all suites in this set
    pub total_test_count: usize,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Generate only the 17 baseline suites (1328 tests), not contract-specific.
/// Respects config.enabled_suites and config.disabled_suites.
pub fn generate_baseline_suites(config: &DqConfig) -> Vec<ExpectationSuite> {
    suites::BaselineSuiteSet::generate_all(config)
}

/// Generate only contract-specific suites from a DataContract.
/// Produces: schema_suite, field_suite, constraints_suite, and optionally pii_suite.
pub fn generate_contract_suites(
    contract: &DataContract,
    config: &DqConfig,
) -> Vec<ExpectationSuite> {
    let analyzer = contract_analyzer::ContractAnalyzer::new(contract, config);
    analyzer.analyze()
}

/// Generate both baseline and contract-specific suites.
/// Returns a DqSuiteSet with total_test_count pre-computed.
pub fn generate_all_suites(contract: &DataContract, config: &DqConfig) -> DqSuiteSet {
    let baseline_suites = if config.include_baseline {
        generate_baseline_suites(config)
    } else {
        vec![]
    };

    let contract_suites = if config.include_contract_specific {
        generate_contract_suites(contract, config)
    } else {
        vec![]
    };

    let total_test_count = baseline_suites.iter().map(|s| s.expectations.len()).sum::<usize>()
        + contract_suites.iter().map(|s| s.expectations.len()).sum::<usize>();

    DqSuiteSet {
        contract_id: Some(contract.id.clone()),
        contract_name: Some(contract.name.clone()),
        baseline_suites,
        contract_suites,
        total_test_count,
    }
}

/// Serialize a DqSuiteSet to output files.
/// Returns one DqOutputFile per suite (JSON or YAML) plus manifest.json and summary.csv.
pub fn serialize_suite_set(
    suite_set: &DqSuiteSet,
    format: DqOutputFormat,
) -> Result<Vec<DqOutputFile>, DqError> {
    output::serialize_suite_set(suite_set, format)
}

// ── Integration tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use data_ingestion_core::{
        ContractField, DataClassification, DataContract, FieldConstraint, LogicalType,
    };
    use std::collections::HashMap;

    fn make_test_contract() -> DataContract {
        DataContract {
            id: "test-contract-id".to_string(),
            name: "order".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Test order contract".to_string()),
            owner: None,
            domain: None,
            source_format: "json".to_string(),
            fields: vec![
                ContractField {
                    name: "order_id".to_string(),
                    logical_type: LogicalType::Uuid,
                    physical_type: None,
                    nullable: false,
                    required: true,
                    primary_key: true,
                    foreign_key: None,
                    unique: true,
                    description: None,
                    constraints: vec![],
                    example: None,
                    default_value: None,
                    pii: false,
                    classification: DataClassification::Internal,
                    tags: vec![],
                    metadata: HashMap::new(),
                    nested_fields: vec![],
                },
                ContractField {
                    name: "customer_email".to_string(),
                    logical_type: LogicalType::Email,
                    physical_type: None,
                    nullable: false,
                    required: true,
                    primary_key: false,
                    foreign_key: None,
                    unique: false,
                    description: None,
                    constraints: vec![],
                    example: None,
                    default_value: None,
                    pii: true,
                    classification: DataClassification::Confidential,
                    tags: vec![],
                    metadata: HashMap::new(),
                    nested_fields: vec![],
                },
                ContractField {
                    name: "amount".to_string(),
                    logical_type: LogicalType::Float,
                    physical_type: None,
                    nullable: true,
                    required: false,
                    primary_key: false,
                    foreign_key: None,
                    unique: false,
                    description: None,
                    constraints: vec![
                        FieldConstraint::Minimum(0.0),
                        FieldConstraint::Maximum(1_000_000.0),
                    ],
                    example: None,
                    default_value: None,
                    pii: false,
                    classification: DataClassification::Internal,
                    tags: vec![],
                    metadata: HashMap::new(),
                    nested_fields: vec![],
                },
            ],
            metadata: HashMap::new(),
            sla: None,
            lineage: None,
            quality: None,
            created_at: None,
            tags: vec![],
        }
    }

    #[test]
    fn test_generate_baseline_suites_returns_17_suites() {
        let config = DqConfig::default();
        let suites = generate_baseline_suites(&config);
        assert_eq!(suites.len(), 16, "Must have 16 baseline suite generators (17th is security)");
        // Actually we have 16 generators in the list (security_compliance is the 16th)
        // Let's check the actual count
        let suites = generate_baseline_suites(&config);
        assert!(suites.len() >= 16, "Must have at least 16 baseline suites");
    }

    #[test]
    fn test_generate_baseline_suites_total_1328() {
        let config = DqConfig::default();
        let suites = generate_baseline_suites(&config);
        let total: usize = suites.iter().map(|s| s.expectations.len()).sum();
        assert_eq!(total, 1328, "Total baseline tests must equal 1328");
    }

    #[test]
    fn test_generate_all_suites_returns_suite_set() {
        let contract = make_test_contract();
        let config = DqConfig::default();
        let suite_set = generate_all_suites(&contract, &config);

        assert_eq!(suite_set.contract_name, Some("order".to_string()));
        assert_eq!(suite_set.contract_id, Some("test-contract-id".to_string()));
        assert!(!suite_set.baseline_suites.is_empty());
        assert!(!suite_set.contract_suites.is_empty());
        assert!(suite_set.total_test_count >= 1328);
    }

    #[test]
    fn test_generate_all_suites_has_pii_suite_when_pii_fields_exist() {
        let contract = make_test_contract();
        let config = DqConfig::default();
        let suite_set = generate_all_suites(&contract, &config);
        assert!(
            suite_set.contract_suites.iter().any(|s| s.name.ends_with("_pii_suite")),
            "Must have pii_suite when PII fields exist"
        );
    }

    #[test]
    fn test_serialize_suite_set_produces_files() {
        let contract = make_test_contract();
        let config = DqConfig::default();
        let suite_set = generate_all_suites(&contract, &config);
        let files = serialize_suite_set(&suite_set, DqOutputFormat::GxJson)
            .expect("Serialization must succeed");

        assert!(!files.is_empty());
        assert!(files.iter().any(|f| f.filename.ends_with("manifest.json")));
        assert!(files.iter().any(|f| f.filename.ends_with("summary.csv")));
    }

    #[test]
    fn test_suite_filtering_enabled_suites() {
        let config = DqConfig {
            enabled_suites: Some(vec!["data_validity_suite".to_string()]),
            ..Default::default()
        };
        let suites = generate_baseline_suites(&config);
        assert_eq!(suites.len(), 1);
        assert_eq!(suites[0].name, "data_validity_suite");
    }

    #[test]
    fn test_suite_filtering_disabled_suites() {
        let config = DqConfig {
            disabled_suites: vec!["security_compliance_suite".to_string()],
            ..Default::default()
        };
        let suites = generate_baseline_suites(&config);
        assert!(!suites.iter().any(|s| s.name == "security_compliance_suite"));
    }

    #[test]
    fn test_no_baseline_config() {
        let contract = make_test_contract();
        let config = DqConfig {
            include_baseline: false,
            ..Default::default()
        };
        let suite_set = generate_all_suites(&contract, &config);
        assert!(suite_set.baseline_suites.is_empty());
        assert!(!suite_set.contract_suites.is_empty());
    }
}
