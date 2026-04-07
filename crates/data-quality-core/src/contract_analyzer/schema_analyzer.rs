use indexmap::IndexMap;
use serde_json::json;

use data_ingestion_core::{DataContract, LogicalType};

use crate::config::DqConfig;
use crate::expectations::{ExpectationConfig, ExpectationMeta, ExpectationSuite, GeneratedFrom, SuiteMeta};
use crate::contract_analyzer::{contract_abbrev, fmt_contract_test_id};

/// Build a schema-level expectation suite for a DataContract.
pub fn build_schema_suite(contract: &DataContract, config: &DqConfig) -> ExpectationSuite {
    let abbrev = contract_abbrev(&contract.name);
    let suite_name = format!("{}_schema_suite", contract.name);
    let mut expectations = Vec::new();
    let mut counter = 1usize;

    // Rule 1: Column set match (always)
    let col_names: Vec<&str> = contract.fields.iter().map(|f| f.name.as_str()).collect();
    let mut kwargs = IndexMap::new();
    kwargs.insert("column_set".to_string(), json!(col_names));
    kwargs.insert("exact_match".to_string(), json!(false));
    expectations.push(ExpectationConfig {
        expectation_type: "expect_table_columns_to_match_set".to_string(),
        kwargs,
        meta: ExpectationMeta {
            test_id: fmt_contract_test_id(&abbrev, "SCH", counter),
            category: "schema".to_string(),
            suite: suite_name.clone(),
            contract_field: None,
            contract_name: Some(contract.name.clone()),
            generated_from: GeneratedFrom::ContractSpecific {
                reason: "column_set_from_contract_fields".to_string(),
            },
        },
    });
    counter += 1;

    // Rule 2: Column count (always)
    let mut kwargs = IndexMap::new();
    kwargs.insert("value".to_string(), json!(contract.fields.len()));
    expectations.push(ExpectationConfig {
        expectation_type: "expect_table_column_count_to_equal".to_string(),
        kwargs,
        meta: ExpectationMeta {
            test_id: fmt_contract_test_id(&abbrev, "SCH", counter),
            category: "schema".to_string(),
            suite: suite_name.clone(),
            contract_field: None,
            contract_name: Some(contract.name.clone()),
            generated_from: GeneratedFrom::ContractSpecific {
                reason: "column_count_from_contract_fields".to_string(),
            },
        },
    });
    counter += 1;

    // Rule 3: Required columns ordered list (if any required fields)
    let required_cols: Vec<&str> = contract
        .fields
        .iter()
        .filter(|f| f.required)
        .map(|f| f.name.as_str())
        .collect();
    if !required_cols.is_empty() {
        let mut kwargs = IndexMap::new();
        kwargs.insert("column_list".to_string(), json!(required_cols));
        expectations.push(ExpectationConfig {
            expectation_type: "expect_table_columns_to_match_ordered_list".to_string(),
            kwargs,
            meta: ExpectationMeta {
                test_id: fmt_contract_test_id(&abbrev, "SCH", counter),
                category: "schema".to_string(),
                suite: suite_name.clone(),
                contract_field: None,
                contract_name: Some(contract.name.clone()),
                generated_from: GeneratedFrom::ContractSpecific {
                    reason: "required_columns_ordered".to_string(),
                },
            },
        });
        counter += 1;
    }

    // Rule 4: Completeness threshold (if quality.completeness_threshold is Some)
    if let Some(quality) = &contract.quality {
        if let Some(threshold) = quality.completeness_threshold {
            for field in contract.fields.iter().filter(|f| f.required) {
                let mut kwargs = IndexMap::new();
                kwargs.insert("column".to_string(), json!(field.name));
                kwargs.insert("mostly".to_string(), json!(threshold));
                expectations.push(ExpectationConfig {
                    expectation_type: "expect_column_values_to_not_be_null".to_string(),
                    kwargs,
                    meta: ExpectationMeta {
                        test_id: fmt_contract_test_id(&abbrev, "SCH", counter),
                        category: "completeness".to_string(),
                        suite: suite_name.clone(),
                        contract_field: Some(field.name.clone()),
                        contract_name: Some(contract.name.clone()),
                        generated_from: GeneratedFrom::ContractSpecific {
                            reason: format!("completeness_threshold_{}", threshold),
                        },
                    },
                });
                counter += 1;
            }
        }
    }

    // Rule 5: Freshness SLA (if sla.freshness_hours is Some)
    if let Some(sla) = &contract.sla {
        if let Some(freshness_hours) = sla.freshness_hours {
            // Find first DateTime/Timestamp field
            if let Some(dt_field) = contract.fields.iter().find(|f| {
                matches!(
                    f.logical_type,
                    LogicalType::DateTime | LogicalType::Timestamp | LogicalType::Date
                )
            }) {
                let mut kwargs = IndexMap::new();
                kwargs.insert("column".to_string(), json!(dt_field.name));
                expectations.push(ExpectationConfig {
                    expectation_type: "expect_column_values_to_be_dateutil_parseable".to_string(),
                    kwargs,
                    meta: ExpectationMeta {
                        test_id: fmt_contract_test_id(&abbrev, "SCH", counter),
                        category: "timeliness".to_string(),
                        suite: suite_name.clone(),
                        contract_field: Some(dt_field.name.clone()),
                        contract_name: Some(contract.name.clone()),
                        generated_from: GeneratedFrom::ContractSpecific {
                            reason: format!("freshness_sla_{}h", freshness_hours),
                        },
                    },
                });
                // counter += 1; // suppress unused warning
            }
        }
    }

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
