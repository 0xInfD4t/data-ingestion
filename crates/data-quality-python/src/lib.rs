//! # data-quality-python
//!
//! PyO3 Python bindings for `data-quality-core`, exposing a [`DqEngine`]
//! class and a `data_quality` module.
//!
//! ## Python Usage
//!
//! ```python
//! import data_quality
//!
//! engine = data_quality.DqEngine()
//! engine.set_include_baseline(True)
//! engine.set_include_contract_specific(True)
//!
//! # From a DataContract dict → returns dict
//! suite_set = engine.generate_from_contract(contract_dict)
//!
//! # From raw bytes → returns dict
//! suite_set = engine.generate_from_source(content, format_hint="json_schema")
//!
//! # Write all suite files to disk
//! engine.write_suite_set(suite_set, output_dir="./expectations")
//!
//! # Get baseline suites as list of dicts
//! baseline = engine.generate_baseline()
//!
//! # Serialize to specific format → list of {"filename": str, "content": str} dicts
//! files = engine.serialize_suite_set(suite_set, output_format="gx_json")
//! ```

use pyo3::exceptions::{PyIOError, PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;

use data_ingestion_core::{process, ContractBuilderConfig, DataContract};
use data_quality_core::{
    generate_all_suites, generate_baseline_suites, serialize_suite_set, DqConfig, DqOutputFormat,
    DqSuiteSet,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Serialize any serde-serializable value to a Python object by round-tripping
/// through JSON using Python's `json.loads`.
fn to_pyobject<T: serde::Serialize>(py: Python<'_>, value: &T) -> PyResult<PyObject> {
    let json_str = serde_json::to_string(value)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    json_str_to_pyobject(py, &json_str)
}

/// Parse a JSON string into a Python object using Python's `json.loads`.
fn json_str_to_pyobject(py: Python<'_>, json_str: &str) -> PyResult<PyObject> {
    let json_mod = py.import_bound("json")?;
    let py_obj: PyObject = json_mod.call_method1("loads", (json_str,))?.into();
    Ok(py_obj)
}

/// Dump a Python dict to a JSON string using Python's `json.dumps`.
fn pydict_to_json_str(py: Python<'_>, dict: &Bound<'_, PyDict>) -> PyResult<String> {
    let json_mod = py.import_bound("json")?;
    let json_str: String = json_mod.call_method1("dumps", (dict,))?.extract()?;
    Ok(json_str)
}

/// Parse a Python dict into a `DataContract` via JSON round-trip.
fn pydict_to_contract(py: Python<'_>, dict: &Bound<'_, PyDict>) -> PyResult<DataContract> {
    let json_str = pydict_to_json_str(py, dict)?;
    serde_json::from_str::<DataContract>(&json_str)
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to parse DataContract: {}", e)))
}

/// Parse a Python dict into a `DqSuiteSet` via JSON round-trip.
fn pydict_to_suite_set(py: Python<'_>, dict: &Bound<'_, PyDict>) -> PyResult<DqSuiteSet> {
    let json_str = pydict_to_json_str(py, dict)?;
    serde_json::from_str::<DqSuiteSet>(&json_str)
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to parse DqSuiteSet: {}", e)))
}

/// Parse an output format string into a `DqOutputFormat`.
fn parse_output_format(format: &str) -> PyResult<DqOutputFormat> {
    match format.to_lowercase().as_str() {
        "gx_json" | "json" => Ok(DqOutputFormat::GxJson),
        "gx_yaml" | "yaml" => Ok(DqOutputFormat::GxYaml),
        "summary_csv" | "csv" => Ok(DqOutputFormat::SummaryCsv),
        "manifest" => Ok(DqOutputFormat::Manifest),
        other => Err(PyValueError::new_err(format!(
            "Unknown output format '{}'. Use: gx_json, gx_yaml, summary_csv, manifest",
            other
        ))),
    }
}

// ── DqEngine ──────────────────────────────────────────────────────────────────

/// Python-facing engine for generating Great Expectations data quality suites.
///
/// Create one instance, configure it with the `set_*` methods, then call
/// the generation methods for each data contract or source file.
#[pyclass]
pub struct DqEngine {
    include_baseline: bool,
    include_contract_specific: bool,
    gx_version: String,
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

#[pymethods]
impl DqEngine {
    /// Create a new `DqEngine` with default settings.
    ///
    /// Defaults: baseline included, contract-specific included, GX version "1.11.3".
    #[new]
    pub fn new() -> Self {
        DqEngine {
            include_baseline: true,
            include_contract_specific: true,
            gx_version: "1.11.3".to_string(),
        }
    }

    /// Enable or disable the 1328 baseline test suites (default: `True`).
    pub fn set_include_baseline(&mut self, v: bool) {
        self.include_baseline = v;
    }

    /// Enable or disable contract-specific test suites (default: `True`).
    pub fn set_include_contract_specific(&mut self, v: bool) {
        self.include_contract_specific = v;
    }

    /// Set the GX version string embedded in suite metadata (default: `"1.11.3"`).
    pub fn set_gx_version(&mut self, v: &str) {
        self.gx_version = v.to_string();
    }

    /// Generate all suites from a DataContract dict.
    ///
    /// # Arguments
    /// * `contract` - A Python dict representing a `DataContract`
    ///
    /// # Returns
    /// A Python dict representing a `DqSuiteSet`
    #[pyo3(signature = (contract))]
    pub fn generate_from_contract(
        &self,
        py: Python<'_>,
        contract: &Bound<'_, PyDict>,
    ) -> PyResult<PyObject> {
        let data_contract = pydict_to_contract(py, contract)?;
        let config = self.build_config();
        let suite_set = generate_all_suites(&data_contract, &config);
        to_pyobject(py, &suite_set)
    }

    /// Ingest raw bytes and generate all suites in one step.
    ///
    /// # Arguments
    /// * `content`     - Raw file bytes
    /// * `format_hint` - Optional format hint string (e.g. `"json_schema"`, `"csv"`)
    /// * `source_path` - Optional source file path (used for format detection and lineage)
    ///
    /// # Returns
    /// A Python dict representing a `DqSuiteSet`
    #[pyo3(signature = (content, format_hint=None, source_path=None))]
    pub fn generate_from_source(
        &self,
        py: Python<'_>,
        content: &[u8],
        format_hint: Option<&str>,
        source_path: Option<&str>,
    ) -> PyResult<PyObject> {
        let ingest_config = ContractBuilderConfig::default();

        let contract = process(content, format_hint, source_path, ingest_config)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to ingest source: {}", e)))?;

        let config = self.build_config();
        let suite_set = generate_all_suites(&contract, &config);
        to_pyobject(py, &suite_set)
    }

    /// Write all suite files to disk.
    ///
    /// # Arguments
    /// * `suite_set`     - A Python dict representing a `DqSuiteSet`
    /// * `output_dir`    - Directory path to write files into
    /// * `output_format` - Output format: `"gx_json"` (default), `"gx_yaml"`, etc.
    #[pyo3(signature = (suite_set, output_dir, output_format="gx_json"))]
    pub fn write_suite_set(
        &self,
        py: Python<'_>,
        suite_set: &Bound<'_, PyDict>,
        output_dir: &str,
        output_format: &str,
    ) -> PyResult<()> {
        let dq_suite_set = pydict_to_suite_set(py, suite_set)?;
        let fmt = parse_output_format(output_format)?;

        let files = serialize_suite_set(&dq_suite_set, fmt)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        std::fs::create_dir_all(output_dir)
            .map_err(|e| PyIOError::new_err(format!("Cannot create output dir '{}': {}", output_dir, e)))?;

        for file in &files {
            // Build the full path, creating subdirectories as needed
            let out_path = std::path::Path::new(output_dir).join(&file.filename);
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    PyIOError::new_err(format!(
                        "Cannot create directory '{}': {}",
                        parent.display(),
                        e
                    ))
                })?;
            }
            std::fs::write(&out_path, &file.content).map_err(|e| {
                PyIOError::new_err(format!("Failed to write '{}': {}", out_path.display(), e))
            })?;
        }

        Ok(())
    }

    /// Generate only the baseline suites (1328 tests).
    ///
    /// # Returns
    /// A Python list of `ExpectationSuite` dicts
    pub fn generate_baseline(&self, py: Python<'_>) -> PyResult<PyObject> {
        let config = self.build_config();
        let suites = generate_baseline_suites(&config);
        to_pyobject(py, &suites)
    }

    /// Serialize a suite set to a list of file dicts.
    ///
    /// # Arguments
    /// * `suite_set`     - A Python dict representing a `DqSuiteSet`
    /// * `output_format` - Output format: `"gx_json"` (default), `"gx_yaml"`, etc.
    ///
    /// # Returns
    /// A Python list of `{"filename": str, "content": str}` dicts
    #[pyo3(signature = (suite_set, output_format="gx_json"))]
    pub fn serialize_suite_set(
        &self,
        py: Python<'_>,
        suite_set: &Bound<'_, PyDict>,
        output_format: &str,
    ) -> PyResult<PyObject> {
        let dq_suite_set = pydict_to_suite_set(py, suite_set)?;
        let fmt = parse_output_format(output_format)?;

        let files = serialize_suite_set(&dq_suite_set, fmt)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        // Build list of {"filename": str, "content": str} dicts
        let file_list: Vec<serde_json::Value> = files
            .iter()
            .map(|f| {
                let content_str = String::from_utf8_lossy(&f.content).into_owned();
                serde_json::json!({
                    "filename": f.filename,
                    "content": content_str,
                })
            })
            .collect();

        let json_str = serde_json::to_string(&file_list)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        json_str_to_pyobject(py, &json_str)
    }
}

// ── Module ────────────────────────────────────────────────────────────────────

#[pymodule]
fn data_quality(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<DqEngine>()?;
    m.add("__version__", "0.1.0")?;
    Ok(())
}
