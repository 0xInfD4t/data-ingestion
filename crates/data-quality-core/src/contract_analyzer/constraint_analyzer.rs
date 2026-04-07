use indexmap::IndexMap;
use serde_json::json;

use data_ingestion_core::{DataContract, FieldConstraint};

use crate::config::DqConfig;
use crate::expectations::{ExpectationConfig, ExpectationMeta, ExpectationSuite, GeneratedFrom, SuiteMeta};
use crate::contract_analyzer::{contract_abbrev, fmt_contract_test_id};

/// Build a constraints expectation suite for a DataContract.
pub fn build_constraints_suite(contract: &DataContract, config: &DqConfig) -> ExpectationSuite {
    let abbrev = contract_abbrev(&contract.name);
    let suite_name = format!("{}_constraints_suite", contract.name);
    let mut expectations = Vec::new();
    let mut counter = 1usize;

    for field in &contract.fields {
        if field.constraints.is_empty() {
            continue;
        }

        let col = &field.name;
        let constraints = &field.constraints;

        // Collect min/max length and min/max value for merging
        let mut min_length: Option<usize> = None;
        let mut max_length: Option<usize> = None;
        let mut min_value: Option<f64> = None;
        let mut max_value: Option<f64> = None;

        for c in constraints {
            match c {
                FieldConstraint::MinLength(n) => {
                    min_length = Some(*n);
                }
                FieldConstraint::MaxLength(n) => {
                    max_length = Some(*n);
                }
                FieldConstraint::Minimum(v) => {
                    min_value = Some(*v);
                }
                FieldConstraint::Maximum(v) => {
                    max_value = Some(*v);
                }
                _ => {}
            }
        }

        // Emit merged length expectation if any length constraint exists
        if min_length.is_some() || max_length.is_some() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            kwargs.insert("min_value".to_string(), json!(min_length));
            kwargs.insert("max_value".to_string(), json!(max_length));
            expectations.push(ExpectationConfig {
                expectation_type: "expect_column_value_lengths_to_be_between".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_contract_test_id(&abbrev, "CON", counter),
                    category: "validity".to_string(),
                    suite: suite_name.clone(),
                    contract_field: Some(col.clone()),
                    contract_name: Some(contract.name.clone()),
                    generated_from: GeneratedFrom::ContractSpecific {
                        reason: format!(
                            "length_constraint_min_{:?}_max_{:?}",
                            min_length, max_length
                        ),
                    },
                },
            });
            counter += 1;
        }

        // Emit merged value between expectation if any value constraint exists
        if min_value.is_some() || max_value.is_some() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            kwargs.insert("min_value".to_string(), json!(min_value));
            kwargs.insert("max_value".to_string(), json!(max_value));
            expectations.push(ExpectationConfig {
                expectation_type: "expect_column_values_to_be_between".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_contract_test_id(&abbrev, "CON", counter),
                    category: "validity".to_string(),
                    suite: suite_name.clone(),
                    contract_field: Some(col.clone()),
                    contract_name: Some(contract.name.clone()),
                    generated_from: GeneratedFrom::ContractSpecific {
                        reason: format!(
                            "value_constraint_min_{:?}_max_{:?}",
                            min_value, max_value
                        ),
                    },
                },
            });
            counter += 1;
        }

        // Emit individual non-mergeable constraints
        for c in constraints {
            match c {
                FieldConstraint::Pattern(p) => {
                    let mut kwargs = IndexMap::new();
                    kwargs.insert("column".to_string(), json!(col));
                    kwargs.insert("regex".to_string(), json!(p));
                    expectations.push(ExpectationConfig {
                        expectation_type: "expect_column_values_to_match_regex".to_string(),
                        kwargs,
                        meta: ExpectationMeta {
                            test_id: fmt_contract_test_id(&abbrev, "CON", counter),
                            category: "validity".to_string(),
                            suite: suite_name.clone(),
                            contract_field: Some(col.clone()),
                            contract_name: Some(contract.name.clone()),
                            generated_from: GeneratedFrom::ContractSpecific {
                                reason: format!("pattern_constraint_{}", p),
                            },
                        },
                    });
                    counter += 1;
                }
                FieldConstraint::AllowedValues(values) => {
                    let mut kwargs = IndexMap::new();
                    kwargs.insert("column".to_string(), json!(col));
                    kwargs.insert("value_set".to_string(), json!(values));
                    expectations.push(ExpectationConfig {
                        expectation_type: "expect_column_values_to_be_in_set".to_string(),
                        kwargs,
                        meta: ExpectationMeta {
                            test_id: fmt_contract_test_id(&abbrev, "CON", counter),
                            category: "validity".to_string(),
                            suite: suite_name.clone(),
                            contract_field: Some(col.clone()),
                            contract_name: Some(contract.name.clone()),
                            generated_from: GeneratedFrom::ContractSpecific {
                                reason: "allowed_values_constraint".to_string(),
                            },
                        },
                    });
                    counter += 1;
                }
                FieldConstraint::NotNull => {
                    let mut kwargs = IndexMap::new();
                    kwargs.insert("column".to_string(), json!(col));
                    expectations.push(ExpectationConfig {
                        expectation_type: "expect_column_values_to_not_be_null".to_string(),
                        kwargs,
                        meta: ExpectationMeta {
                            test_id: fmt_contract_test_id(&abbrev, "CON", counter),
                            category: "completeness".to_string(),
                            suite: suite_name.clone(),
                            contract_field: Some(col.clone()),
                            contract_name: Some(contract.name.clone()),
                            generated_from: GeneratedFrom::ContractSpecific {
                                reason: "not_null_constraint".to_string(),
                            },
                        },
                    });
                    counter += 1;
                }
                FieldConstraint::Unique => {
                    let mut kwargs = IndexMap::new();
                    kwargs.insert("column".to_string(), json!(col));
                    expectations.push(ExpectationConfig {
                        expectation_type: "expect_column_values_to_be_unique".to_string(),
                        kwargs,
                        meta: ExpectationMeta {
                            test_id: fmt_contract_test_id(&abbrev, "CON", counter),
                            category: "uniqueness".to_string(),
                            suite: suite_name.clone(),
                            contract_field: Some(col.clone()),
                            contract_name: Some(contract.name.clone()),
                            generated_from: GeneratedFrom::ContractSpecific {
                                reason: "unique_constraint".to_string(),
                            },
                        },
                    });
                    counter += 1;
                }
                // MinLength, MaxLength, Minimum, Maximum already handled above (merged)
                FieldConstraint::MinLength(_)
                | FieldConstraint::MaxLength(_)
                | FieldConstraint::Minimum(_)
                | FieldConstraint::Maximum(_) => {}
            }
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
