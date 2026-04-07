//! # data-quality-wasm
//!
//! WebAssembly bindings for `data-quality-core`, exposing a [`DqEngine`]
//! class to JavaScript via `wasm-bindgen`.
//!
//! ## JavaScript Usage
//!
//! ```javascript
//! import init, { DqEngine } from "./data_quality_wasm.js";
//!
//! await init();
//!
//! const engine = new DqEngine();
//! engine.set_include_baseline(true);
//! engine.set_include_contract_specific(true);
//! engine.set_gx_version("1.11.3");
//!
//! // From a DataContract JSON string → returns DqSuiteSet as JSON string
//! const suiteSetJson = engine.generate_from_contract_json(contractJson);
//!
//! // From raw source bytes (ingest + generate in one step)
//! const suiteSetJson = engine.generate_from_source(uint8Array, "json_schema", "schema.json");
//!
//! // Serialize a suite set JSON to GX output files
//! const filesJson = engine.serialize_suite_set(suiteSetJson, "gx_json");
//!
//! // Get just the baseline suites as JSON
//! const baselineJson = engine.generate_baseline();
//! ```

use wasm_bindgen::prelude::*;

use data_ingestion_core::{process, ContractBuilderConfig};
use data_quality_core::{
    generate_all_suites, generate_baseline_suites, serialize_suite_set, DqConfig, DqOutputFormat,
    DqSuiteSet,
};

// ── Panic hook ────────────────────────────────────────────────────────────────

/// Initialise the `console_error_panic_hook` so that Rust panics are printed
/// to the browser console as readable messages instead of "unreachable executed".
#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

// ── DqEngine ──────────────────────────────────────────────────────────────────

/// JavaScript-facing engine for generating Great Expectations data quality suites.
///
/// Create one instance, configure it with the `set_*` methods, then call
/// the generation methods for each data contract or source file.
#[wasm_bindgen]
pub struct DqEngine {
    include_baseline: bool,
    include_contract_specific: bool,
    gx_version: String,
}

#[wasm_bindgen]
impl DqEngine {
    /// Create a new `DqEngine` with default settings.
    ///
    /// Defaults: baseline included, contract-specific included, GX version "1.11.3".
    #[wasm_bindgen(constructor)]
    pub fn new() -> DqEngine {
        DqEngine {
            include_baseline: true,
            include_contract_specific: true,
            gx_version: "1.11.3".to_string(),
        }
    }

    /// Enable or disable the 1328 baseline test suites (default: `true`).
    pub fn set_include_baseline(&mut self, v: bool) {
        self.include_baseline = v;
    }

    /// Enable or disable contract-specific test suites (default: `true`).
    pub fn set_include_contract_specific(&mut self, v: bool) {
        self.include_contract_specific = v;
    }

    /// Set the GX version string embedded in suite metadata (default: `"1.11.3"`).
    pub fn set_gx_version(&mut self, v: &str) {
        self.gx_version = v.to_string();
    }

    /// Generate all suites from a DataContract JSON string.
    ///
    /// # Arguments
    /// * `contract_json` - A JSON string representing a `DataContract`
    ///
    /// # Returns
    /// A JSON string representing a `DqSuiteSet`
    pub fn generate_from_contract_json(&self, contract_json: &str) -> Result<String, JsValue> {
        let contract: data_ingestion_core::DataContract =
            serde_json::from_str(contract_json).map_err(|e| {
                JsValue::from_str(&format!("Failed to parse DataContract JSON: {}", e))
            })?;

        let config = self.build_config();
        let suite_set = generate_all_suites(&contract, &config);

        serde_json::to_string(&suite_set)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize DqSuiteSet: {}", e)))
    }

    /// Ingest raw source bytes and generate all suites in one step.
    ///
    /// # Arguments
    /// * `content`     - Raw file bytes (Uint8Array from JavaScript)
    /// * `format_hint` - Optional format hint string (e.g. `"json_schema"`, `"csv"`)
    /// * `source_path` - Optional source file path (used for format detection and lineage)
    ///
    /// # Returns
    /// A JSON string representing a `DqSuiteSet`
    pub fn generate_from_source(
        &self,
        content: &[u8],
        format_hint: Option<String>,
        source_path: Option<String>,
    ) -> Result<String, JsValue> {
        let ingest_config = ContractBuilderConfig::default();

        let contract = process(
            content,
            format_hint.as_deref(),
            source_path.as_deref(),
            ingest_config,
        )
        .map_err(|e| JsValue::from_str(&format!("Failed to ingest source: {}", e)))?;

        let config = self.build_config();
        let suite_set = generate_all_suites(&contract, &config);

        serde_json::to_string(&suite_set)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize DqSuiteSet: {}", e)))
    }

    /// Serialize a `DqSuiteSet` JSON string to output files.
    ///
    /// # Arguments
    /// * `suite_set_json` - A JSON string representing a `DqSuiteSet`
    /// * `format`         - Output format: `"gx_json"`, `"gx_yaml"`, `"summary_csv"`, or `"manifest"`
    ///
    /// # Returns
    /// A JSON array of file objects: `[{"filename": "...", "content": "...", "format": "gx_json"}, ...]`
    pub fn serialize_suite_set(
        &self,
        suite_set_json: &str,
        format: &str,
    ) -> Result<String, JsValue> {
        let suite_set: DqSuiteSet = serde_json::from_str(suite_set_json).map_err(|e| {
            JsValue::from_str(&format!("Failed to parse DqSuiteSet JSON: {}", e))
        })?;

        let output_format = parse_output_format(format)?;

        let files = serialize_suite_set(&suite_set, output_format)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        // Build JSON array of file objects with UTF-8 content strings
        let file_objects: Vec<serde_json::Value> = files
            .iter()
            .map(|f| {
                let content_str = String::from_utf8_lossy(&f.content).into_owned();
                let format_str = match f.format {
                    DqOutputFormat::GxJson => "gx_json",
                    DqOutputFormat::GxYaml => "gx_yaml",
                    DqOutputFormat::SummaryCsv => "summary_csv",
                    DqOutputFormat::Manifest => "manifest",
                };
                serde_json::json!({
                    "filename": f.filename,
                    "content": content_str,
                    "format": format_str,
                })
            })
            .collect();

        serde_json::to_string(&file_objects)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize file list: {}", e)))
    }

    /// Generate only the baseline suites (1328 tests) as a JSON array of `ExpectationSuite` objects.
    ///
    /// # Returns
    /// A JSON array of `ExpectationSuite` objects
    pub fn generate_baseline(&self) -> Result<String, JsValue> {
        let config = self.build_config();
        let suites = generate_baseline_suites(&config);

        serde_json::to_string(&suites)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize baseline suites: {}", e)))
    }
}

impl DqEngine {
    fn build_config(&self) -> DqConfig {
        DqConfig {
            gx_version: self.gx_version.clone(),
            include_baseline: self.include_baseline,
            include_contract_specific: self.include_contract_specific,
            ..DqConfig::default()
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn parse_output_format(format: &str) -> Result<DqOutputFormat, JsValue> {
    match format.to_lowercase().as_str() {
        "gx_json" | "json" => Ok(DqOutputFormat::GxJson),
        "gx_yaml" | "yaml" => Ok(DqOutputFormat::GxYaml),
        "summary_csv" | "csv" => Ok(DqOutputFormat::SummaryCsv),
        "manifest" => Ok(DqOutputFormat::Manifest),
        other => Err(JsValue::from_str(&format!(
            "Unknown output format '{}'. Use: gx_json, gx_yaml, summary_csv, manifest",
            other
        ))),
    }
}
