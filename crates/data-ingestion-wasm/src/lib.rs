//! # data-ingestion-wasm
//!
//! WebAssembly bindings for `data-ingestion-core`, exposing a [`ContractEngine`]
//! class to JavaScript via `wasm-bindgen`.
//!
//! ## JavaScript Usage
//!
//! ```javascript
//! import init, { ContractEngine } from "./data_ingestion_wasm.js";
//!
//! await init();
//!
//! const engine = new ContractEngine();
//! engine.set_owner("data-team");
//! engine.set_domain("finance");
//! engine.set_enrich_pii(true);
//!
//! // Process a Uint8Array of bytes
//! const contractJson = engine.process(uint8Array, "json_schema", "schema.json");
//! const csvOutput    = engine.process_to_format(uint8Array, "json_schema", "schema.json", "csv");
//! const validation   = engine.validate_contract_json(contractJson);
//! ```

use wasm_bindgen::prelude::*;

use data_ingestion_core::{
    contract::{
        builder::ContractBuilderConfig,
        model::DataContract,
        validator::ContractValidator,
    },
    output::OutputFormat,
    process, to_format,
};

// ── Panic hook ────────────────────────────────────────────────────────────────

/// Initialise the `console_error_panic_hook` so that Rust panics are printed
/// to the browser console as readable messages instead of "unreachable executed".
#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

// ── ContractEngine ────────────────────────────────────────────────────────────

/// JavaScript-facing engine for generating and validating data contracts.
///
/// Create one instance, configure it with the `set_*` methods, then call
/// [`process`](ContractEngine::process) or
/// [`process_to_format`](ContractEngine::process_to_format) for each document.
#[wasm_bindgen]
pub struct ContractEngine {
    owner: Option<String>,
    domain: Option<String>,
    version: String,
    enrich_pii: bool,
    include_nested: bool,
}

#[wasm_bindgen]
impl ContractEngine {
    /// Create a new `ContractEngine` with default settings.
    ///
    /// Defaults: version `"1.0.0"`, PII enrichment enabled, nested fields included.
    #[wasm_bindgen(constructor)]
    pub fn new() -> ContractEngine {
        ContractEngine {
            owner: None,
            domain: None,
            version: "1.0.0".to_string(),
            enrich_pii: true,
            include_nested: true,
        }
    }

    /// Set the owner identifier for generated contracts.
    pub fn set_owner(&mut self, owner: &str) {
        self.owner = Some(owner.to_string());
    }

    /// Set the domain label for generated contracts.
    pub fn set_domain(&mut self, domain: &str) {
        self.domain = Some(domain.to_string());
    }

    /// Set the semantic version string for generated contracts (default: `"1.0.0"`).
    pub fn set_version(&mut self, version: &str) {
        self.version = version.to_string();
    }

    /// Enable or disable automatic PII field detection (default: `true`).
    pub fn set_enrich_pii(&mut self, enrich: bool) {
        self.enrich_pii = enrich;
    }

    /// Enable or disable preservation of nested object fields (default: `true`).
    ///
    /// When `false`, nested objects are flattened into the parent field list.
    pub fn set_include_nested(&mut self, include: bool) {
        self.include_nested = include;
    }

    // ── Core methods ──────────────────────────────────────────────────────────

    /// Process raw bytes into a [`DataContract`], returned as a JSON string.
    ///
    /// # Arguments
    /// * `content`      – Raw file bytes (e.g. a `Uint8Array` from JavaScript).
    /// * `format_hint`  – Optional format string: `"json_schema"`, `"xsd"`,
    ///                    `"xml"`, `"csv"`, `"yaml"`, `"json"`.
    ///                    Pass `null` / `undefined` to rely on extension detection.
    /// * `source_path`  – Optional filename used for extension-based detection
    ///                    and lineage metadata (e.g. `"schema.json"`).
    ///
    /// # Returns
    /// A JSON string representation of the [`DataContract`] on success.
    ///
    /// # Throws
    /// A JavaScript `Error` string if any stage of the pipeline fails.
    pub fn process(
        &self,
        content: &[u8],
        format_hint: Option<String>,
        source_path: Option<String>,
    ) -> Result<String, JsValue> {
        let config = self.build_config();

        let contract = process(
            content,
            format_hint.as_deref(),
            source_path.as_deref(),
            config,
        )
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

        serde_json::to_string(&contract)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Process raw bytes and serialize the resulting [`DataContract`] to the
    /// specified output format.
    ///
    /// # Arguments
    /// * `content`       – Raw file bytes.
    /// * `format_hint`   – Optional source format hint (see [`process`]).
    /// * `source_path`   – Optional filename for detection / lineage.
    /// * `output_format` – Target format: `"json"`, `"yaml"`, `"xml"`, or `"csv"`.
    ///
    /// # Returns
    /// A UTF-8 string in the requested format.
    ///
    /// # Throws
    /// A JavaScript `Error` string on failure.
    pub fn process_to_format(
        &self,
        content: &[u8],
        format_hint: Option<String>,
        source_path: Option<String>,
        output_format: &str,
    ) -> Result<String, JsValue> {
        let config = self.build_config();

        let contract = process(
            content,
            format_hint.as_deref(),
            source_path.as_deref(),
            config,
        )
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let fmt: OutputFormat = output_format
            .parse()
            .map_err(|e: data_ingestion_core::OutputError| JsValue::from_str(&e.to_string()))?;

        let bytes = to_format(&contract, fmt)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        String::from_utf8(bytes)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Validate a [`DataContract`] provided as a JSON string.
    ///
    /// # Arguments
    /// * `contract_json` – A JSON string previously returned by [`process`].
    ///
    /// # Returns
    /// A JSON object string: `{ "valid": bool, "warnings": string[], "errors": string[] }`.
    ///
    /// # Throws
    /// A JavaScript `Error` string if the input cannot be deserialized.
    pub fn validate_contract_json(&self, contract_json: &str) -> Result<String, JsValue> {
        let contract: DataContract = serde_json::from_str(contract_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse contract JSON: {}", e)))?;

        let result = ContractValidator::validate(&contract);

        serde_json::to_string(&result)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    fn build_config(&self) -> ContractBuilderConfig {
        ContractBuilderConfig {
            version: self.version.clone(),
            owner: self.owner.clone(),
            domain: self.domain.clone(),
            enrich_pii: self.enrich_pii,
            include_nested: self.include_nested,
            ..ContractBuilderConfig::default()
        }
    }
}
