use serde_json::json;

use crate::error::DqError;
use crate::output::{DqOutputFile, DqOutputFormat};

/// Build a manifest.json file listing all suite files.
pub fn build_manifest(
    suite_set: &crate::DqSuiteSet,
    files: &[DqOutputFile],
) -> Result<DqOutputFile, DqError> {
    let contract_name = suite_set.contract_name.as_deref().unwrap_or("unnamed");
    let filename = format!("{}/manifest.json", contract_name);

    // Build suite entries from the files list (exclude summary.csv and manifest.json itself)
    let suite_entries: Vec<serde_json::Value> = files
        .iter()
        .filter(|f| {
            !f.filename.ends_with("summary.csv") && !f.filename.ends_with("manifest.json")
        })
        .map(|f| {
            let suite_type = if f.filename.contains("/baseline/") {
                "baseline"
            } else {
                "contract_specific"
            };
            // Find the suite in the set to get category and test count
            let (category, test_count) = find_suite_info(suite_set, &f.suite_name);
            json!({
                "filename": f.filename,
                "suite_name": f.suite_name,
                "category": category,
                "test_count": test_count,
                "suite_type": suite_type,
            })
        })
        .collect();

    let manifest = json!({
        "contract_id": suite_set.contract_id,
        "contract_name": suite_set.contract_name,
        "generated_at": null,
        "total_test_count": suite_set.total_test_count,
        "suites": suite_entries,
    });

    let content = serde_json::to_vec_pretty(&manifest)
        .map_err(|e| DqError::SerializationError(e.to_string()))?;

    Ok(DqOutputFile {
        filename,
        suite_name: "manifest".to_string(),
        content,
        format: DqOutputFormat::Manifest,
    })
}

fn find_suite_info(suite_set: &crate::DqSuiteSet, suite_name: &str) -> (String, usize) {
    // Search baseline suites
    for suite in &suite_set.baseline_suites {
        if suite.name == suite_name {
            // Derive category from suite name
            let category = derive_category(&suite.name);
            return (category, suite.expectations.len());
        }
    }
    // Search contract suites
    for suite in &suite_set.contract_suites {
        if suite.name == suite_name {
            let category = derive_category(&suite.name);
            return (category, suite.expectations.len());
        }
    }
    ("unknown".to_string(), 0)
}

fn derive_category(suite_name: &str) -> String {
    if suite_name.contains("validity") {
        "validity".to_string()
    } else if suite_name.contains("completeness") {
        "completeness".to_string()
    } else if suite_name.contains("consistency") {
        "consistency".to_string()
    } else if suite_name.contains("accuracy") {
        "accuracy".to_string()
    } else if suite_name.contains("profile") {
        "profiling".to_string()
    } else if suite_name.contains("integrity") {
        "integrity".to_string()
    } else if suite_name.contains("timeliness") {
        "timeliness".to_string()
    } else if suite_name.contains("sensitivity") {
        "sensitivity".to_string()
    } else if suite_name.contains("uniqueness") {
        "uniqueness".to_string()
    } else if suite_name.contains("business_rules") {
        "business_rules".to_string()
    } else if suite_name.contains("format_consistency") {
        "format_consistency".to_string()
    } else if suite_name.contains("volume_anomalies") {
        "volume_anomalies".to_string()
    } else if suite_name.contains("dependency_checks") {
        "dependency_checks".to_string()
    } else if suite_name.contains("cross_system") {
        "cross_system".to_string()
    } else if suite_name.contains("performance") {
        "performance".to_string()
    } else if suite_name.contains("security") {
        "security".to_string()
    } else if suite_name.ends_with("_schema_suite") {
        "schema".to_string()
    } else if suite_name.ends_with("_field_suite") {
        "field".to_string()
    } else if suite_name.ends_with("_pii_suite") {
        "pii".to_string()
    } else if suite_name.ends_with("_constraints_suite") {
        "constraints".to_string()
    } else {
        "general".to_string()
    }
}
