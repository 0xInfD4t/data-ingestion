use crate::error::DqError;
use crate::expectations::ExpectationSuite;

pub mod gx_json;
pub mod gx_yaml;
pub mod manifest;
pub mod summary_csv;

// ── DqOutputFormat ────────────────────────────────────────────────────────────

/// Output format selector.
#[derive(Debug, Clone, PartialEq)]
pub enum DqOutputFormat {
    /// GX 1.x-compatible JSON suite files
    GxJson,
    /// YAML suite files (mirrors JSON structure)
    GxYaml,
    /// CSV summary of all tests across all suites
    SummaryCsv,
    /// JSON manifest listing all suite files
    Manifest,
}

// ── DqOutputFile ──────────────────────────────────────────────────────────────

/// A single serialized output file ready to write to disk or return to a caller.
#[derive(Debug, Clone)]
pub struct DqOutputFile {
    /// Relative path, e.g. "order/baseline/data_validity_suite.json"
    pub filename: String,
    /// Suite name this file represents
    pub suite_name: String,
    /// Raw bytes of the serialized content
    pub content: Vec<u8>,
    /// Format of the content
    pub format: DqOutputFormat,
}

// ── serialize_suite_set ───────────────────────────────────────────────────────

/// Serialize a DqSuiteSet to all output files for one contract.
/// Returns:
///   - One JSON or YAML file per suite (baseline + contract-specific)
///   - One summary.csv covering all tests across all suites
///   - One manifest.json listing all suite files with metadata
pub fn serialize_suite_set(
    suite_set: &crate::DqSuiteSet,
    format: DqOutputFormat,
) -> Result<Vec<DqOutputFile>, DqError> {
    let mut files = Vec::new();
    let contract_name = suite_set.contract_name.as_deref().unwrap_or("unnamed");

    // Serialize each baseline suite
    for suite in &suite_set.baseline_suites {
        let filename = format!("{}/baseline/{}.json", contract_name, suite.name);
        let content = serialize_suite(suite, &format)?;
        files.push(DqOutputFile {
            filename,
            suite_name: suite.name.clone(),
            content,
            format: format.clone(),
        });
    }

    // Serialize each contract-specific suite
    for suite in &suite_set.contract_suites {
        let filename = format!(
            "{}/contract_specific/{}.json",
            contract_name, suite.name
        );
        let content = serialize_suite(suite, &format)?;
        files.push(DqOutputFile {
            filename,
            suite_name: suite.name.clone(),
            content,
            format: format.clone(),
        });
    }

    // Always generate summary.csv
    files.push(summary_csv::build_summary_csv(suite_set)?);

    // Always generate manifest.json (pass current files before manifest is added)
    let manifest = manifest::build_manifest(suite_set, &files)?;
    files.push(manifest);

    Ok(files)
}

fn serialize_suite(suite: &ExpectationSuite, format: &DqOutputFormat) -> Result<Vec<u8>, DqError> {
    match format {
        DqOutputFormat::GxYaml => gx_yaml::to_gx_yaml(suite),
        _ => gx_json::to_gx_json(suite),
    }
}
