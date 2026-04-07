use indexmap::IndexMap;
use serde_json::json;

use data_ingestion_core::{DataClassification, DataContract, LogicalType};

use crate::config::DqConfig;
use crate::expectations::{ExpectationConfig, ExpectationMeta, ExpectationSuite, GeneratedFrom, SuiteMeta};
use crate::contract_analyzer::{contract_abbrev, fmt_contract_test_id};

/// Build a PII/sensitivity expectation suite for a DataContract.
/// Only called when at least one field has pii=true.
pub fn build_pii_suite(contract: &DataContract, config: &DqConfig) -> ExpectationSuite {
    let abbrev = contract_abbrev(&contract.name);
    let suite_name = format!("{}_pii_suite", contract.name);
    let mut expectations = Vec::new();
    let mut counter = 1usize;

    for field in &contract.fields {
        let col = &field.name;
        let col_lower = col.to_lowercase();

        if field.pii {
            // Rule 1: PII masking check (always for pii=true)
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            expectations.push(ExpectationConfig {
                expectation_type: "expect_column_values_to_be_masked_pii".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_contract_test_id(&abbrev, "PII", counter),
                    category: "sensitivity".to_string(),
                    suite: suite_name.clone(),
                    contract_field: Some(col.clone()),
                    contract_name: Some(contract.name.clone()),
                    generated_from: GeneratedFrom::ContractSpecific {
                        reason: "pii_field_masking".to_string(),
                    },
                },
            });
            counter += 1;

            // Rule 2: Email type + pii → not_contain_pii
            if matches!(field.logical_type, LogicalType::Email) {
                let mut kwargs = IndexMap::new();
                kwargs.insert("column".to_string(), json!(col));
                expectations.push(ExpectationConfig {
                    expectation_type: "expect_column_values_to_not_contain_pii".to_string(),
                    kwargs,
                    meta: ExpectationMeta {
                        test_id: fmt_contract_test_id(&abbrev, "PII", counter),
                        category: "sensitivity".to_string(),
                        suite: suite_name.clone(),
                        contract_field: Some(col.clone()),
                        contract_name: Some(contract.name.clone()),
                        generated_from: GeneratedFrom::ContractSpecific {
                            reason: "pii_email_no_raw_pii".to_string(),
                        },
                    },
                });
                counter += 1;
            }

            // Rule 3: SSN field
            if col_lower.contains("ssn") || col_lower.contains("social_security") {
                let mut kwargs = IndexMap::new();
                kwargs.insert("column".to_string(), json!(col));
                expectations.push(ExpectationConfig {
                    expectation_type: "expect_column_values_to_be_valid_ssn".to_string(),
                    kwargs,
                    meta: ExpectationMeta {
                        test_id: fmt_contract_test_id(&abbrev, "PII", counter),
                        category: "sensitivity".to_string(),
                        suite: suite_name.clone(),
                        contract_field: Some(col.clone()),
                        contract_name: Some(contract.name.clone()),
                        generated_from: GeneratedFrom::ContractSpecific {
                            reason: "pii_ssn_format".to_string(),
                        },
                    },
                });
                counter += 1;
            }

            // Rule 4: Phone field
            if col_lower.contains("phone") || col_lower.contains("mobile") || col_lower.contains("tel") {
                let mut kwargs = IndexMap::new();
                kwargs.insert("column".to_string(), json!(col));
                expectations.push(ExpectationConfig {
                    expectation_type: "expect_column_values_to_be_valid_phone_number".to_string(),
                    kwargs,
                    meta: ExpectationMeta {
                        test_id: fmt_contract_test_id(&abbrev, "PII", counter),
                        category: "sensitivity".to_string(),
                        suite: suite_name.clone(),
                        contract_field: Some(col.clone()),
                        contract_name: Some(contract.name.clone()),
                        generated_from: GeneratedFrom::ContractSpecific {
                            reason: "pii_phone_format".to_string(),
                        },
                    },
                });
                counter += 1;
            }

            // Rule 5: Credit card field
            if col_lower.contains("credit_card")
                || col_lower.contains("card_number")
                || col_lower.contains("cc_num")
            {
                let mut kwargs = IndexMap::new();
                kwargs.insert("column".to_string(), json!(col));
                expectations.push(ExpectationConfig {
                    expectation_type: "expect_column_values_to_be_valid_credit_card".to_string(),
                    kwargs,
                    meta: ExpectationMeta {
                        test_id: fmt_contract_test_id(&abbrev, "PII", counter),
                        category: "sensitivity".to_string(),
                        suite: suite_name.clone(),
                        contract_field: Some(col.clone()),
                        contract_name: Some(contract.name.clone()),
                        generated_from: GeneratedFrom::ContractSpecific {
                            reason: "pii_credit_card_format".to_string(),
                        },
                    },
                });
                counter += 1;
            }
        }

        // Rule 6: Restricted classification → encrypted
        if field.classification == DataClassification::Restricted {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            expectations.push(ExpectationConfig {
                expectation_type: "expect_column_values_to_be_encrypted".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_contract_test_id(&abbrev, "PII", counter),
                    category: "sensitivity".to_string(),
                    suite: suite_name.clone(),
                    contract_field: Some(col.clone()),
                    contract_name: Some(contract.name.clone()),
                    generated_from: GeneratedFrom::ContractSpecific {
                        reason: "classification_restricted_encrypted".to_string(),
                    },
                },
            });
            counter += 1;
        }

        // Rule 7: Confidential classification → masked
        if field.classification == DataClassification::Confidential {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            expectations.push(ExpectationConfig {
                expectation_type: "expect_column_values_to_be_masked_pii".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_contract_test_id(&abbrev, "PII", counter),
                    category: "sensitivity".to_string(),
                    suite: suite_name.clone(),
                    contract_field: Some(col.clone()),
                    contract_name: Some(contract.name.clone()),
                    generated_from: GeneratedFrom::ContractSpecific {
                        reason: "classification_confidential_masked".to_string(),
                    },
                },
            });
            counter += 1;
        }
    }

    let _ = config;
    let count = expectations.len();
    ExpectationSuite {
        name: suite_name,
        expectations,
        meta: SuiteMeta {
            great_expectations_version: config.gx_version.clone(),
            suite_id: uuid::Uuid::new_v4().to_string(),
            contract_id: Some(contract.id.clone()),
            generated_at: None,
            test_count: count,
        },
    }
}
