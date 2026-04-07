//! # data-ingestion-python
//!
//! PyO3 Python bindings for `data-ingestion-core`, exposing a [`ContractEngine`]
//! class and a `data_ingestion` module.
//!
//! ## Python Usage
//!
//! ```python
//! import data_ingestion
//!
//! engine = data_ingestion.ContractEngine()
//! engine.set_owner("data-team")
//! engine.set_domain("finance")
//! engine.set_enrich_pii(True)
//!
//! with open("schema.json", "rb") as f:
//!     content = f.read()
//!
//! # Returns a Python dict
//! contract = engine.process_bytes(content, format_hint="json_schema", source_path="schema.json")
//!
//! # Returns a string in the requested format
//! csv_str  = engine.process_to_format(content, format_hint="json_schema", output_format="csv")
//! yaml_str = engine.process_to_format(content, output_format="yaml")
//!
//! # Validate a contract dict
//! result = engine.validate_contract(contract)
//! # {"valid": True, "warnings": [...], "errors": [...]}
//!
//! # Process a file by path (native only)
//! contract = engine.process_file("schema.json")
//! ```

use pyo3::prelude::*;
use pyo3::exceptions::{PyIOError, PyRuntimeError, PyValueError};
use pyo3::types::PyDict;

use data_ingestion_core::{
    contract::{
        builder::ContractBuilderConfig,
        model::DataContract,
        validator::ContractValidator,
    },
    output::OutputFormat,
    process, to_format,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Serialize a [`DataContract`] to a Python object (dict) by round-tripping
/// through JSON using Python's `json.loads`.
fn contract_to_pyobject(py: Python<'_>, contract: &DataContract) -> PyResult<PyObject> {
    let json_str = serde_json::to_string(contract)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    json_str_to_pyobject(py, &json_str)
}

/// Parse a JSON string into a Python object using Python's `json.loads`.
fn json_str_to_pyobject(py: Python<'_>, json_str: &str) -> PyResult<PyObject> {
    let json_mod = py.import_bound("json")?;
    let py_obj: PyObject = json_mod
        .call_method1("loads", (json_str,))?
        .into();
    Ok(py_obj)
}

/// Dump a Python dict to a JSON string using Python's `json.dumps`.
fn pydict_to_json_str(py: Python<'_>, dict: &Bound<'_, PyDict>) -> PyResult<String> {
    let json_mod = py.import_bound("json")?;
    let json_str: String = json_mod
        .call_method1("dumps", (dict,))?
        .extract()?;
    Ok(json_str)
}

// ── ContractEngine ────────────────────────────────────────────────────────────

/// Python-facing engine for generating and validating data contracts.
///
/// Create one instance, configure it with the `set_*` methods, then call
/// `process_bytes`, `process_to_format`, `validate_contract`, or `process_file`.
#[pyclass]
pub struct ContractEngine {
    owner: Option<String>,
    domain: Option<String>,
    version: String,
    enrich_pii: bool,
    include_nested: bool,
}

impl ContractEngine {
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

#[pymethods]
impl ContractEngine {
    /// Create a new `ContractEngine` with default settings.
    ///
    /// Defaults: version `"1.0.0"`, PII enrichment enabled, nested fields included.
    #[new]
    pub fn new() -> Self {
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

    /// Enable or disable automatic PII field detection (default: `True`).
    pub fn set_enrich_pii(&mut self, enrich: bool) {
        self.enrich_pii = enrich;
    }

    /// Enable or disable preservation of nested object fields (default: `True`).
    ///
    /// When `False`, nested objects are flattened into the parent field list.
    pub fn set_include_nested(&mut self, include: bool) {
        self.include_nested = include;
    }

    // ── Core methods ──────────────────────────────────────────────────────────

    /// Process raw bytes into a `DataContract`, returned as a Python `dict`.
    ///
    /// Parameters
    /// ----------
    /// content : bytes
    ///     Raw file bytes.
    /// format_hint : str, optional
    ///     Source format hint: ``"json_schema"``, ``"xsd"``, ``"xml"``,
    ///     ``"csv"``, ``"yaml"``, ``"json"``.
    /// source_path : str, optional
    ///     Filename for extension-based detection and lineage metadata.
    #[pyo3(signature = (content, format_hint=None, source_path=None))]
    pub fn process_bytes(
        &self,
        py: Python<'_>,
        content: &[u8],
        format_hint: Option<&str>,
        source_path: Option<&str>,
    ) -> PyResult<PyObject> {
        let config = self.build_config();
        let contract = process(content, format_hint, source_path, config)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        contract_to_pyobject(py, &contract)
    }

    /// Process raw bytes and serialize the resulting `DataContract` to the
    /// specified output format string.
    ///
    /// `output_format`: `"json"`, `"yaml"`, `"xml"`, or `"csv"`.
    #[pyo3(signature = (content, format_hint=None, source_path=None, output_format="json"))]
    pub fn process_to_format(
        &self,
        content: &[u8],
        format_hint: Option<&str>,
        source_path: Option<&str>,
        output_format: &str,
    ) -> PyResult<String> {
        let config = self.build_config();
        let contract = process(content, format_hint, source_path, config)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;

        let fmt: OutputFormat = output_format
            .parse()
            .map_err(|e: data_ingestion_core::OutputError| PyValueError::new_err(e.to_string()))?;

        let bytes = to_format(&contract, fmt)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        String::from_utf8(bytes)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Validate a `DataContract` provided as a Python `dict`.
    ///
    /// Returns `{"valid": bool, "warnings": list[str], "errors": list[str]}`.
    pub fn validate_contract(
        &self,
        py: Python<'_>,
        contract: &Bound<'_, PyDict>,
    ) -> PyResult<PyObject> {
        // Python dict → JSON string → DataContract
        let json_str = pydict_to_json_str(py, contract)?;

        let data_contract: DataContract = serde_json::from_str(&json_str)
            .map_err(|e| PyValueError::new_err(format!("Failed to parse contract dict: {}", e)))?;

        let result = ContractValidator::validate(&data_contract);

        // ValidationResult → JSON string → Python dict
        let result_json = serde_json::to_string(&result)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        json_str_to_pyobject(py, &result_json)
    }

    /// Process a file by path and return the `DataContract` as a Python `dict`.
    ///
    /// When `output_format` is provided, returns a `str` in that format instead.
    #[pyo3(signature = (path, format_hint=None, output_format=None))]
    pub fn process_file(
        &self,
        py: Python<'_>,
        path: &str,
        format_hint: Option<&str>,
        output_format: Option<&str>,
    ) -> PyResult<PyObject> {
        let content = std::fs::read(path)
            .map_err(|e| PyIOError::new_err(format!("Cannot read '{}': {}", path, e)))?;

        let config = self.build_config();
        let contract = process(&content, format_hint, Some(path), config)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;

        match output_format {
            Some(fmt_str) => {
                let fmt: OutputFormat = fmt_str
                    .parse()
                    .map_err(|e: data_ingestion_core::OutputError| {
                        PyValueError::new_err(e.to_string())
                    })?;

                let bytes = to_format(&contract, fmt)
                    .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

                let s = String::from_utf8(bytes)
                    .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

                Ok(s.into_py(py))
            }
            None => contract_to_pyobject(py, &contract),
        }
    }
}

// ── Module definition ─────────────────────────────────────────────────────────

/// Python module `data_ingestion`.
///
/// Exposes:
/// - `ContractEngine` — the main engine class
/// - `__version__`    — crate version string
#[pymodule]
fn data_ingestion(m: &Bound<'_, pyo3::types::PyModule>) -> PyResult<()> {
    m.add_class::<ContractEngine>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
