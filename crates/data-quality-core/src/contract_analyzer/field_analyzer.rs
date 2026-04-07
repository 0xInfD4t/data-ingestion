use indexmap::IndexMap;
use serde_json::json;
use std::collections::HashSet;

use data_ingestion_core::{DataContract, LogicalType};

use crate::config::DqConfig;
use crate::expectations::{ExpectationConfig, ExpectationMeta, ExpectationSuite, GeneratedFrom, SuiteMeta};
use crate::contract_analyzer::{contract_abbrev, fmt_contract_test_id};

/// Build a per-field expectation suite for a DataContract.
pub fn build_field_suite(contract: &DataContract, config: &DqConfig) -> ExpectationSuite {
    let abbrev = contract_abbrev(&contract.name);
    let suite_name = format!("{}_field_suite", contract.name);
    let mut expectations = Vec::new();
    let mut counter = 1usize;

    // Track which columns already have not_null / unique expectations to avoid duplicates
    let mut not_null_cols: HashSet<String> = HashSet::new();
    let mut unique_cols: HashSet<String> = HashSet::new();

    for field in &contract.fields {
        let col = &field.name;

        // ── Nullability rules ──────────────────────────────────────────────
        let needs_not_null = !field.nullable || field.required || field.primary_key;
        if needs_not_null && not_null_cols.insert(col.clone()) {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            expectations.push(ExpectationConfig {
                expectation_type: "expect_column_values_to_not_be_null".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_contract_test_id(&abbrev, "FLD", counter),
                    category: "completeness".to_string(),
                    suite: suite_name.clone(),
                    contract_field: Some(col.clone()),
                    contract_name: Some(contract.name.clone()),
                    generated_from: GeneratedFrom::ContractSpecific {
                        reason: "nullable_false_or_required_or_pk".to_string(),
                    },
                },
            });
            counter += 1;
        }

        // ── Uniqueness rules ───────────────────────────────────────────────
        let needs_unique = field.unique || field.primary_key;
        if needs_unique && unique_cols.insert(col.clone()) {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            expectations.push(ExpectationConfig {
                expectation_type: "expect_column_values_to_be_unique".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_contract_test_id(&abbrev, "FLD", counter),
                    category: "uniqueness".to_string(),
                    suite: suite_name.clone(),
                    contract_field: Some(col.clone()),
                    contract_name: Some(contract.name.clone()),
                    generated_from: GeneratedFrom::ContractSpecific {
                        reason: "unique_or_primary_key".to_string(),
                    },
                },
            });
            counter += 1;
        }

        // ── Type-based rules ───────────────────────────────────────────────
        match &field.logical_type {
            LogicalType::Email => {
                let mut kwargs = IndexMap::new();
                kwargs.insert("column".to_string(), json!(col));
                expectations.push(ExpectationConfig {
                    expectation_type: "expect_column_values_to_be_valid_email".to_string(),
                    kwargs,
                    meta: ExpectationMeta {
                        test_id: fmt_contract_test_id(&abbrev, "FLD", counter),
                        category: "validity".to_string(),
                        suite: suite_name.clone(),
                        contract_field: Some(col.clone()),
                        contract_name: Some(contract.name.clone()),
                        generated_from: GeneratedFrom::ContractSpecific {
                            reason: "logical_type_email".to_string(),
                        },
                    },
                });
                counter += 1;
            }
            LogicalType::Uuid => {
                let mut kwargs = IndexMap::new();
                kwargs.insert("column".to_string(), json!(col));
                expectations.push(ExpectationConfig {
                    expectation_type: "expect_column_values_to_be_valid_uuid".to_string(),
                    kwargs,
                    meta: ExpectationMeta {
                        test_id: fmt_contract_test_id(&abbrev, "FLD", counter),
                        category: "validity".to_string(),
                        suite: suite_name.clone(),
                        contract_field: Some(col.clone()),
                        contract_name: Some(contract.name.clone()),
                        generated_from: GeneratedFrom::ContractSpecific {
                            reason: "logical_type_uuid".to_string(),
                        },
                    },
                });
                counter += 1;
            }
            LogicalType::Uri => {
                let mut kwargs = IndexMap::new();
                kwargs.insert("column".to_string(), json!(col));
                expectations.push(ExpectationConfig {
                    expectation_type: "expect_column_values_to_be_valid_uri".to_string(),
                    kwargs,
                    meta: ExpectationMeta {
                        test_id: fmt_contract_test_id(&abbrev, "FLD", counter),
                        category: "validity".to_string(),
                        suite: suite_name.clone(),
                        contract_field: Some(col.clone()),
                        contract_name: Some(contract.name.clone()),
                        generated_from: GeneratedFrom::ContractSpecific {
                            reason: "logical_type_uri".to_string(),
                        },
                    },
                });
                counter += 1;
            }
            LogicalType::Date | LogicalType::DateTime | LogicalType::Timestamp => {
                let mut kwargs = IndexMap::new();
                kwargs.insert("column".to_string(), json!(col));
                expectations.push(ExpectationConfig {
                    expectation_type: "expect_column_values_to_be_dateutil_parseable".to_string(),
                    kwargs,
                    meta: ExpectationMeta {
                        test_id: fmt_contract_test_id(&abbrev, "FLD", counter),
                        category: "validity".to_string(),
                        suite: suite_name.clone(),
                        contract_field: Some(col.clone()),
                        contract_name: Some(contract.name.clone()),
                        generated_from: GeneratedFrom::ContractSpecific {
                            reason: "logical_type_date_or_datetime".to_string(),
                        },
                    },
                });
                counter += 1;
            }
            LogicalType::Integer | LogicalType::Long => {
                let mut kwargs = IndexMap::new();
                kwargs.insert("column".to_string(), json!(col));
                kwargs.insert("type_".to_string(), json!("int"));
                expectations.push(ExpectationConfig {
                    expectation_type: "expect_column_values_to_be_of_type".to_string(),
                    kwargs,
                    meta: ExpectationMeta {
                        test_id: fmt_contract_test_id(&abbrev, "FLD", counter),
                        category: "validity".to_string(),
                        suite: suite_name.clone(),
                        contract_field: Some(col.clone()),
                        contract_name: Some(contract.name.clone()),
                        generated_from: GeneratedFrom::ContractSpecific {
                            reason: "logical_type_integer".to_string(),
                        },
                    },
                });
                counter += 1;
            }
            LogicalType::Float | LogicalType::Double => {
                let mut kwargs = IndexMap::new();
                kwargs.insert("column".to_string(), json!(col));
                kwargs.insert("type_".to_string(), json!("float"));
                expectations.push(ExpectationConfig {
                    expectation_type: "expect_column_values_to_be_of_type".to_string(),
                    kwargs,
                    meta: ExpectationMeta {
                        test_id: fmt_contract_test_id(&abbrev, "FLD", counter),
                        category: "validity".to_string(),
                        suite: suite_name.clone(),
                        contract_field: Some(col.clone()),
                        contract_name: Some(contract.name.clone()),
                        generated_from: GeneratedFrom::ContractSpecific {
                            reason: "logical_type_float".to_string(),
                        },
                    },
                });
                counter += 1;
            }
            LogicalType::Decimal { precision, scale } => {
                let mut kwargs = IndexMap::new();
                kwargs.insert("column".to_string(), json!(col));
                kwargs.insert("precision".to_string(), json!(precision));
                kwargs.insert("scale".to_string(), json!(scale));
                expectations.push(ExpectationConfig {
                    expectation_type: "expect_column_values_to_be_valid_decimal_precision".to_string(),
                    kwargs,
                    meta: ExpectationMeta {
                        test_id: fmt_contract_test_id(&abbrev, "FLD", counter),
                        category: "validity".to_string(),
                        suite: suite_name.clone(),
                        contract_field: Some(col.clone()),
                        contract_name: Some(contract.name.clone()),
                        generated_from: GeneratedFrom::ContractSpecific {
                            reason: format!("logical_type_decimal_{}_{}", precision, scale),
                        },
                    },
                });
                counter += 1;
            }
            LogicalType::Boolean => {
                let mut kwargs = IndexMap::new();
                kwargs.insert("column".to_string(), json!(col));
                kwargs.insert("value_set".to_string(), json!([true, false]));
                expectations.push(ExpectationConfig {
                    expectation_type: "expect_column_values_to_be_in_set".to_string(),
                    kwargs,
                    meta: ExpectationMeta {
                        test_id: fmt_contract_test_id(&abbrev, "FLD", counter),
                        category: "validity".to_string(),
                        suite: suite_name.clone(),
                        contract_field: Some(col.clone()),
                        contract_name: Some(contract.name.clone()),
                        generated_from: GeneratedFrom::ContractSpecific {
                            reason: "logical_type_boolean".to_string(),
                        },
                    },
                });
                counter += 1;
            }
            LogicalType::Json => {
                let mut kwargs = IndexMap::new();
                kwargs.insert("column".to_string(), json!(col));
                expectations.push(ExpectationConfig {
                    expectation_type: "expect_column_values_to_be_valid_json".to_string(),
                    kwargs,
                    meta: ExpectationMeta {
                        test_id: fmt_contract_test_id(&abbrev, "FLD", counter),
                        category: "validity".to_string(),
                        suite: suite_name.clone(),
                        contract_field: Some(col.clone()),
                        contract_name: Some(contract.name.clone()),
                        generated_from: GeneratedFrom::ContractSpecific {
                            reason: "logical_type_json".to_string(),
                        },
                    },
                });
                counter += 1;
            }
            // String, Binary, Array, Struct, Unknown, Time, Duration — no type expectation
            _ => {}
        }

        // ── Foreign key rules ──────────────────────────────────────────────
        if let Some(fk) = &field.foreign_key {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            kwargs.insert("value_set".to_string(), json!([]));
            expectations.push(ExpectationConfig {
                expectation_type: "expect_column_values_to_be_in_set".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_contract_test_id(&abbrev, "FLD", counter),
                    category: "integrity".to_string(),
                    suite: suite_name.clone(),
                    contract_field: Some(col.clone()),
                    contract_name: Some(format!("fk:{}:{}", fk.table, fk.column)),
                    generated_from: GeneratedFrom::ContractSpecific {
                        reason: format!("foreign_key_{}_{}", fk.table, fk.column),
                    },
                },
            });
            counter += 1;
        }
    }

    let _ = config; // config used for future extensions
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
