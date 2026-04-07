# Data Quality — Output, WASM, Python & CLI

> **Part of:** [`DATA_QUALITY_ARCHITECTURE.md`](DATA_QUALITY_ARCHITECTURE.md)
> **Crates:** `data-quality-core` (output), `data-quality-wasm`, `data-quality-python`, `aytch`

---

## Part A: Output Module (`src/output/`)

### A.1 Module Layout

```
src/output/
├── mod.rs          <- DqOutputFormat enum, serialize_suite_set() orchestrator
├── gx_json.rs      <- ExpectationSuite -> GX 1.x JSON bytes
├── gx_yaml.rs      <- ExpectationSuite -> YAML bytes
├── summary_csv.rs  <- all tests -> single summary.csv
└── manifest.rs     <- all suite files -> manifest.json
```

### A.2 `mod.rs` — Orchestration

```rust
// src/output/mod.rs

/// Serialize a DqSuiteSet to all output files for one contract.
/// Returns:
///   - One JSON or YAML file per suite (baseline + contract-specific)
///   - One summary.csv covering all tests across all suites
///   - One manifest.json listing all suite files with metadata
pub fn serialize_suite_set(
    suite_set: &DqSuiteSet,
    format: DqOutputFormat,
) -> Result<Vec<DqOutputFile>, DqError> {
    let mut files = Vec::new();
    let contract_name = suite_set.contract_name.as_deref().unwrap_or("unnamed");

    // Serialize each baseline suite
    for suite in &suite_set.baseline_suites {
        let filename = format!("{}/baseline/{}.json", contract_name, suite.name);
        let content = match format {
            DqOutputFormat::GxJson  => gx_json::to_gx_json(suite)?,
            DqOutputFormat::GxYaml  => gx_yaml::to_gx_yaml(suite)?,
            _ => gx_json::to_gx_json(suite)?,
        };
        files.push(DqOutputFile { filename, suite_name: suite.name.clone(), content, format: format.clone() });
    }

    // Serialize each contract-specific suite
    for suite in &suite_set.contract_suites {
        let filename = format!("{}/contract_specific/{}.json", contract_name, suite.name);
        let content = match format {
            DqOutputFormat::GxJson  => gx_json::to_gx_json(suite)?,
            DqOutputFormat::GxYaml  => gx_yaml::to_gx_yaml(suite)?,
            _ => gx_json::to_gx_json(suite)?,
        };
        files.push(DqOutputFile { filename, suite_name: suite.name.clone(), content, format: format.clone() });
    }

    // Always generate summary.csv and manifest.json
    files.push(summary_csv::build_summary_csv(suite_set)?);
    files.push(manifest::build_manifest(suite_set, &files)?);

    Ok(files)
}
```

### A.3 `gx_json.rs` — GX JSON Output

```rust
// src/output/gx_json.rs

pub fn to_gx_json(suite: &ExpectationSuite) -> Result<Vec<u8>, DqError> {
    serde_json::to_vec_pretty(suite)
        .map_err(|e| DqError::SerializationError(e.to_string()))
}
```

**Serde annotation required on `ExpectationConfig`:**
```rust
#[derive(Serialize, Deserialize)]
pub struct ExpectationConfig {
    #[serde(rename = "type")]   // GX 1.x uses "type", not "expectation_type"
    pub expectation_type: String,
    pub kwargs: IndexMap<String, Value>,
    pub meta: ExpectationMeta,
}
```

**Output filename pattern:**
- Baseline: `<contract_name>/baseline/<suite_name>.json`
- Contract-specific: `<contract_name>/contract_specific/<suite_name>.json`

### A.4 `gx_yaml.rs` — YAML Output

```rust
// src/output/gx_yaml.rs

pub fn to_gx_yaml(suite: &ExpectationSuite) -> Result<Vec<u8>, DqError> {
    serde_yaml::to_string(suite)
        .map(|s| s.into_bytes())
        .map_err(|e| DqError::SerializationError(e.to_string()))
}
```

Output filename pattern uses `.yaml` extension instead of `.json`.

### A.5 `summary_csv.rs` — CSV Summary

Generates a single `summary.csv` with one row per expectation across all suites.

**CSV columns:**

| Column | Source |
|---|---|
| `test_id` | `meta.test_id` |
| `suite_name` | `ExpectationSuite.name` |
| `expectation_type` | `ExpectationConfig.expectation_type` |
| `category` | `meta.category` |
| `contract_field` | `meta.contract_field` (empty string if None) |
| `contract_name` | `meta.contract_name` (empty string if None) |
| `generated_from` | `"baseline"` or `"contract_specific"` |
| `kwargs_json` | `serde_json::to_string(&kwargs)` |

**Output filename:** `<contract_name>/summary.csv`

### A.6 `manifest.rs` — JSON Manifest

**Output filename:** `<contract_name>/manifest.json`

```json
{
  "contract_id": "550e8400-e29b-41d4-a716-446655440000",
  "contract_name": "order",
  "generated_at": "2026-04-07T16:50:00Z",
  "total_test_count": 1450,
  "suites": [
    {
      "filename": "order/baseline/data_validity_suite.json",
      "suite_name": "data_validity_suite",
      "category": "validity",
      "test_count": 95,
      "suite_type": "baseline"
    },
    {
      "filename": "order/contract_specific/order_schema_suite.json",
      "suite_name": "order_schema_suite",
      "category": "schema",
      "test_count": 5,
      "suite_type": "contract_specific"
    }
  ]
}
```

### A.7 Output File Structure

**Single contract (`order`):**
```
output/
  order/
    baseline/
      data_validity_suite.json           <- 95 tests
      data_completeness_suite.json       <- 70 tests
      data_consistency_suite.json        <- 90 tests
      data_accuracy_suite.json           <- 100 tests
      data_profile_suite.json            <- 120 tests
      data_integrity_suite.json          <- 60 tests
      data_timeliness_suite.json         <- 80 tests
      data_sensitivity_suite.json        <- 50 tests
      data_uniqueness_suite.json         <- 60 tests
      data_business_rules_suite.json     <- 80 tests
      data_format_consistency_suite.json <- 70 tests
      data_volume_anomalies_suite.json   <- 90 tests
      data_dependency_checks_suite.json  <- 50 tests
      cross_system_consistency_suite.json <- 100 tests
      performance_metrics_suite.json     <- 60 tests
      security_compliance_suite.json     <- 153 tests
    contract_specific/
      order_schema_suite.json
      order_field_suite.json
      order_pii_suite.json               <- only if PII fields exist
      order_constraints_suite.json
    manifest.json
    summary.csv
```

**Multi-contract folder:**
```
output/
  order/
    baseline/...
    contract_specific/...
    manifest.json
    summary.csv
  customer/
    baseline/...
    contract_specific/...
    manifest.json
    summary.csv
  manifest.json                          <- top-level manifest of all contracts
```

---

## Part B: WASM Binding (`crates/data-quality-wasm/`)

### B.1 `Cargo.toml`

```toml
[package]
name    = "data-quality-wasm"
version.workspace = true
edition.workspace = true

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
data-quality-core        = { path = "../data-quality-core" }
data-ingestion-core      = { path = "../data-ingestion-core" }
wasm-bindgen             = { workspace = true }
js-sys                   = { workspace = true }
serde_json               = { workspace = true }
console_error_panic_hook = { workspace = true }
```

### B.2 `src/lib.rs` — `DqEngine` WASM Class

```rust
// crates/data-quality-wasm/src/lib.rs

use wasm_bindgen::prelude::*;
use data_quality_core::{DqConfig, generate_all_suites, serialize_suite_set, DqOutputFormat};
use data_ingestion_core::{process, ContractBuilderConfig};

#[wasm_bindgen]
pub struct DqEngine {
    config: DqConfig,
}

#[wasm_bindgen]
impl DqEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> DqEngine {
        console_error_panic_hook::set_once();
        DqEngine { config: DqConfig::default() }
    }

    /// Set whether to include the 17 baseline suites.
    pub fn set_include_baseline(&mut self, include: bool) {
        self.config.include_baseline = include;
    }

    /// Set whether to include contract-specific suites.
    pub fn set_include_contract_specific(&mut self, include: bool) {
        self.config.include_contract_specific = include;
    }

    /// Set the GX version string embedded in suite metadata.
    pub fn set_gx_version(&mut self, version: &str) {
        self.config.gx_version = version.to_string();
    }

    /// Comma-separated suite names to enable. Empty string = all suites.
    pub fn set_enabled_suites(&mut self, suites: &str) {
        self.config.enabled_suites = if suites.is_empty() {
            None
        } else {
            Some(suites.split(',').map(|s| s.trim().to_string()).collect())
        };
    }

    /// Generate suites from a DataContract JSON string.
    /// Returns a DqSuiteSet serialized as a JSON string.
    pub fn generate_from_contract_json(&self, contract_json: &str) -> Result<String, JsValue> {
        let contract: data_ingestion_core::DataContract =
            serde_json::from_str(contract_json)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let suite_set = generate_all_suites(&contract, &self.config);
        serde_json::to_string(&suite_set)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Ingest raw source bytes and generate suites in one step.
    /// format_hint: "json" | "json_schema" | "xml" | "xsd" | "csv" | "yaml"
    /// filename_hint: used for format detection fallback
    /// Returns a DqSuiteSet serialized as a JSON string.
    pub fn generate_from_source(
        &self,
        source: &[u8],
        format_hint: &str,
        filename_hint: &str,
    ) -> Result<String, JsValue> {
        let config = ContractBuilderConfig::default();
        let contract = process(source, Some(format_hint), Some(filename_hint), config)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let suite_set = generate_all_suites(&contract, &self.config);
        serde_json::to_string(&suite_set)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Serialize a DqSuiteSet JSON string to GX suite files.
    /// output_format: "gx_json" | "gx_yaml"
    /// Returns a JSON array of {filename, content_base64, format} objects.
    pub fn serialize_suite_set_json(
        &self,
        suite_set_json: &str,
        output_format: &str,
    ) -> Result<String, JsValue> {
        let suite_set: data_quality_core::DqSuiteSet =
            serde_json::from_str(suite_set_json)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let fmt = match output_format {
            "gx_yaml" => DqOutputFormat::GxYaml,
            _         => DqOutputFormat::GxJson,
        };
        let files = serialize_suite_set(&suite_set, fmt)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        // Encode content as base64 for safe JS string transport
        let result: Vec<serde_json::Value> = files.iter().map(|f| {
            serde_json::json!({
                "filename": f.filename,
                "suite_name": f.suite_name,
                "content_base64": base64_encode(&f.content),
                "format": output_format,
            })
        }).collect();
        serde_json::to_string(&result)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}
```

### B.3 JavaScript/TypeScript Usage

```javascript
import init, { DqEngine } from './data_quality_wasm.js';
await init();

const engine = new DqEngine();
engine.set_include_baseline(true);
engine.set_include_contract_specific(true);

// From a DataContract JSON string
const suiteSetJson = engine.generate_from_contract_json(contractJson);
const suiteSet = JSON.parse(suiteSetJson);
console.log(`Generated ${suiteSet.total_test_count} tests`);

// From raw source bytes (ingest + generate in one step)
const bytes = new Uint8Array(await file.arrayBuffer());
const suiteSetJson2 = engine.generate_from_source(bytes, "json_schema", "schema.json");

// Serialize to GX JSON files
const filesJson = engine.serialize_suite_set_json(suiteSetJson, "gx_json");
const files = JSON.parse(filesJson);
// files: [{filename: "order/baseline/data_validity_suite.json", content_base64: "...", format: "gx_json"}]

// Decode and write files (Node.js example)
for (const f of files) {
    const content = Buffer.from(f.content_base64, 'base64');
    await fs.writeFile(f.filename, content);
}
```

---

## Part C: Python Binding (`crates/data-quality-python/`)

### C.1 `Cargo.toml`

```toml
[package]
name    = "data-quality-python"
version.workspace = true
edition.workspace = true

[lib]
name       = "data_quality"
crate-type = ["cdylib"]

[dependencies]
data-quality-core   = { path = "../data-quality-core" }
data-ingestion-core = { path = "../data-ingestion-core" }
pyo3       = { workspace = true }
serde_json = { workspace = true }
```

### C.2 `src/lib.rs` — `DqEngine` PyO3 Class

```rust
// crates/data-quality-python/src/lib.rs

use pyo3::prelude::*;
use pyo3::types::PyDict;
use data_quality_core::{DqConfig, generate_all_suites, serialize_suite_set, DqOutputFormat};
use data_ingestion_core::{process, ContractBuilderConfig};

#[pyclass]
pub struct DqEngine {
    config: DqConfig,
}

#[pymethods]
impl DqEngine {
    #[new]
    pub fn new() -> Self {
        DqEngine { config: DqConfig::default() }
    }

    pub fn set_include_baseline(&mut self, include: bool) {
        self.config.include_baseline = include;
    }

    pub fn set_include_contract_specific(&mut self, include: bool) {
        self.config.include_contract_specific = include;
    }

    pub fn set_gx_version(&mut self, version: &str) {
        self.config.gx_version = version.to_string();
    }

    /// Generate suites from a DataContract dict.
    /// Converts: Python dict -> JSON string -> DataContract -> DqSuiteSet -> Python dict.
    pub fn generate_from_contract(&self, py: Python, contract_dict: &PyDict) -> PyResult<PyObject> {
        let json_str = pythondict_to_json(contract_dict)?;
        let contract: data_ingestion_core::DataContract =
            serde_json::from_str(&json_str)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        let suite_set = generate_all_suites(&contract, &self.config);
        let result_json = serde_json::to_string(&suite_set)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        json_to_pyobject(py, &result_json)
    }

    /// Generate suites from raw bytes.
    /// format_hint: "json" | "json_schema" | "xml" | "xsd" | "csv" | "yaml"
    pub fn generate_from_source(
        &self,
        py: Python,
        content: &[u8],
        format_hint: Option<&str>,
    ) -> PyResult<PyObject> {
        let config = ContractBuilderConfig::default();
        let contract = process(content, format_hint, None, config)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        let suite_set = generate_all_suites(&contract, &self.config);
        let result_json = serde_json::to_string(&suite_set)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        json_to_pyobject(py, &result_json)
    }

    /// Write all suites in a DqSuiteSet dict to disk.
    pub fn write_suite_set(&self, suite_set: &PyDict, output_dir: &str) -> PyResult<()> {
        let json_str = pythondict_to_json(suite_set)?;
        let suite_set_rs: data_quality_core::DqSuiteSet =
            serde_json::from_str(&json_str)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        let files = serialize_suite_set(&suite_set_rs, DqOutputFormat::GxJson)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        for file in &files {
            let full_path = std::path::Path::new(output_dir).join(&file.filename);
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
            }
            std::fs::write(&full_path, &file.content)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
        }
        Ok(())
    }

    /// Serialize a DqSuiteSet dict to a list of {filename, content, format} dicts.
    pub fn serialize_suite_set(
        &self,
        py: Python,
        suite_set: &PyDict,
        output_format: &str,
    ) -> PyResult<PyObject> {
        let json_str = pythondict_to_json(suite_set)?;
        let suite_set_rs: data_quality_core::DqSuiteSet =
            serde_json::from_str(&json_str)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        let fmt = match output_format {
            "gx_yaml" => DqOutputFormat::GxYaml,
            _         => DqOutputFormat::GxJson,
        };
        let files = serialize_suite_set(&suite_set_rs, fmt)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        let result: Vec<serde_json::Value> = files.iter().map(|f| {
            serde_json::json!({
                "filename": f.filename,
                "suite_name": f.suite_name,
                "content": String::from_utf8_lossy(&f.content).to_string(),
                "format": output_format,
            })
        }).collect();
        let result_json = serde_json::to_string(&result)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        json_to_pyobject(py, &result_json)
    }
}

#[pymodule]
fn data_quality(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<DqEngine>()?;
    Ok(())
}
```

### C.3 `data_quality.pyi` — Python Type Stubs

```python
from typing import Optional

class DqEngine:
    def __init__(self) -> None: ...
    def set_include_baseline(self, include: bool) -> None: ...
    def set_include_contract_specific(self, include: bool) -> None: ...
    def set_gx_version(self, version: str) -> None: ...
    def generate_from_contract(self, contract_dict: dict) -> dict: ...
    def generate_from_source(
        self, content: bytes, format_hint: Optional[str] = None
    ) -> dict: ...
    def write_suite_set(self, suite_set: dict, output_dir: str) -> None: ...
    def serialize_suite_set(
        self, suite_set: dict, output_format: str
    ) -> list[dict]: ...
```

### C.4 Python Usage

```python
import data_quality, json

engine = data_quality.DqEngine()
engine.set_include_baseline(True)
engine.set_include_contract_specific(True)

# From a DataContract dict
with open("order.contract.json") as f:
    contract_dict = json.load(f)

suite_set = engine.generate_from_contract(contract_dict)
print(f"Generated {suite_set['total_test_count']} tests across "
      f"{len(suite_set['baseline_suites'])} baseline + "
      f"{len(suite_set['contract_suites'])} contract-specific suites")

# From raw bytes
with open("schema.json", "rb") as f:
    content = f.read()
suite_set = engine.generate_from_source(content, format_hint="json_schema")

# Write all suites to disk
engine.write_suite_set(suite_set, output_dir="./great_expectations/expectations")

# Or get files as a list of dicts
files = engine.serialize_suite_set(suite_set, output_format="gx_json")
for f in files:
    print(f["filename"], len(f["content"]), "bytes")
```

---

## Part D: CLI Extension (`crates/aytch/`)

### D.1 New CLI Flags

Add to the `Cli` struct in [`crates/aytch/src/main.rs`](../crates/aytch/src/main.rs):

```rust
/// Generate data quality test suites from a source file or folder
#[arg(long = "dataquality", short = 'q')]
dataquality: bool,

/// Skip the 1328 baseline tests; generate only contract-specific suites
#[arg(long = "no-baseline")]
no_baseline: bool,

/// Comma-separated list of suite names to include (default: all)
#[arg(long = "suites", value_name = "SUITES")]
suites: Option<String>,

/// Process all supported files in a folder recursively
#[arg(long = "recursive", short = 'r')]
recursive: bool,
```

The existing `--type` flag gains a new accepted value: `greatexpectations`.

### D.2 Updated `run()` Dispatch

```rust
fn run() -> Result<()> {
    let cli = Cli::parse();
    if cli.ingest {
        run_ingest(&cli)
    } else if cli.dataquality {
        run_dataquality(&cli)
    } else {
        eprintln!("No action specified. Use --ingest or --dataquality.");
        std::process::exit(1);
    }
}
```

### D.3 `run_dataquality()` Logic

```rust
fn run_dataquality(cli: &Cli) -> Result<()> {
    let src = cli.src.as_ref()
        .ok_or_else(|| anyhow::anyhow!("--src is required for --dataquality"))?;
    let output_dir = cli.output.as_ref()
        .ok_or_else(|| anyhow::anyhow!("--output is required for --dataquality"))?;

    if cli.contract_type != "greatexpectations" {
        anyhow::bail!(
            "Unknown --type '{}' for --dataquality. Use: greatexpectations",
            cli.contract_type
        );
    }

    let config = DqConfig {
        include_baseline: !cli.no_baseline,
        enabled_suites: cli.suites.as_ref().map(|s| {
            s.split(',').map(|x| x.trim().to_string()).collect()
        }),
        ..DqConfig::default()
    };

    // Collect source files
    let source_files: Vec<PathBuf> = if src.is_dir() {
        collect_source_files(src, cli.recursive)
    } else {
        vec![src.clone()]
    };

    println!("Processing {} source file(s)...", source_files.len());

    for file_path in &source_files {
        let content = std::fs::read(file_path)
            .with_context(|| format!("Failed to read: {}", file_path.display()))?;
        let path_str = file_path.to_string_lossy();
        let ingest_config = ContractBuilderConfig::default();

        let contract = process(&content, None, Some(&path_str), ingest_config)
            .map_err(|e| anyhow::anyhow!("Ingest failed for {}: {}", file_path.display(), e))?;

        println!("  Contract: \"{}\" ({} fields)", contract.name, contract.fields.len());

        let suite_set = generate_all_suites(&contract, &config);
        let output_files = serialize_suite_set(&suite_set, DqOutputFormat::GxJson)
            .map_err(|e| anyhow::anyhow!("Serialization failed: {}", e))?;

        for out_file in &output_files {
            let full_path = output_dir.join(&out_file.filename);
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("Cannot create dir: {}", parent.display()))?;
            }
            std::fs::write(&full_path, &out_file.content)
                .with_context(|| format!("Failed to write: {}", full_path.display()))?;
        }

        println!(
            "  Written: {} suites, {} total tests -> {}",
            suite_set.baseline_suites.len() + suite_set.contract_suites.len(),
            suite_set.total_test_count,
            output_dir.join(&contract.name).display()
        );
    }
    Ok(())
}

/// Collect supported source files from a directory.
/// Supported extensions: .json, .yaml, .yml, .xml, .xsd, .csv
fn collect_source_files(dir: &Path, recursive: bool) -> Vec<PathBuf> {
    const EXTENSIONS: &[&str] = &["json", "yaml", "yml", "xml", "xsd", "csv"];
    // Walk dir (recursive or top-level) and filter by extension
    // Implementation uses std::fs::read_dir with optional recursion
    todo!()
}
```

### D.4 Updated `aytch` Cargo.toml

```toml
[dependencies]
data-ingestion-core = { path = "../data-ingestion-core" }
data-quality-core   = { path = "../data-quality-core" }   # NEW
clap    = { version = "4", features = ["derive"] }
anyhow  = "1"
```

### D.5 CLI Usage Examples

```bash
# Generate all suites (baseline + contract-specific) from a single file
aytch --dataquality --src schema.json --output ./expectations --type greatexpectations

# Skip baseline; only generate contract-specific suites
aytch --dataquality --src schema.json --output ./expectations \
  --type greatexpectations --no-baseline

# Generate only specific suites
aytch --dataquality --src schema.json --output ./expectations \
  --type greatexpectations --suites data_validity_suite,data_completeness_suite

# Process an entire folder recursively
aytch --dataquality --src ./schemas/ --output ./expectations \
  --type greatexpectations --recursive

# Combined: ingest a file AND generate data quality suites
aytch --ingest --src schema.json --output ./contracts --type datacontract
aytch --dataquality --src schema.json --output ./expectations --type greatexpectations
```

---

## Part E: Test Strategy

### E.1 `tests/baseline_suite_counts.rs`

Verifies that each of the 17 suite generators produces exactly the specified number of tests.

```rust
#[test]
fn test_data_validity_suite_count() {
    let suite = DataValiditySuite.build_suite(&DqConfig::default());
    assert_eq!(suite.expectations.len(), 95, "DV001-DV095 must produce 95 tests");
}

#[test]
fn test_data_completeness_suite_count() {
    let suite = DataCompletenessSuite.build_suite(&DqConfig::default());
    assert_eq!(suite.expectations.len(), 70, "DC096-DC165 must produce 70 tests");
}

// ... one assert_eq! per suite, 17 total ...

#[test]
fn test_total_baseline_count() {
    let suites = BaselineSuiteSet::generate_all(&DqConfig::default());
    let total: usize = suites.iter().map(|s| s.expectations.len()).sum();
    assert_eq!(total, 1328, "Total baseline tests must equal 1328");
}

#[test]
fn test_suite_filtering_by_enabled_suites() {
    let config = DqConfig {
        enabled_suites: Some(vec!["data_validity_suite".to_string()]),
        ..Default::default()
    };
    let suites = BaselineSuiteSet::generate_all(&config);
    assert_eq!(suites.len(), 1);
    assert_eq!(suites[0].name, "data_validity_suite");
}

#[test]
fn test_suite_filtering_by_disabled_suites() {
    let config = DqConfig {
        disabled_suites: vec!["security_compliance_suite".to_string()],
        ..Default::default()
    };
    let suites = BaselineSuiteSet::generate_all(&config);
    assert_eq!(suites.len(), 16);
    assert!(!suites.iter().any(|s| s.name == "security_compliance_suite"));
}
```

### E.2 `tests/contract_analyzer.rs`

Verifies that `ContractAnalyzer` generates the correct expectations for each field type and constraint combination.

```rust
use data_ingestion_core::{DataContract, ContractField, LogicalType, FieldConstraint, DataClassification};
use data_quality_core::{generate_contract_suites, DqConfig};

fn make_test_contract(fields: Vec<ContractField>) -> DataContract {
    DataContract {
        id: "test-id".to_string(),
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        fields,
        ..Default::default()
    }
}

fn make_field(name: &str) -> ContractField {
    ContractField {
        name: name.to_string(),
        logical_type: LogicalType::String,
        nullable: true,
        required: false,
        primary_key: false,
        unique: false,
        pii: false,
        classification: DataClassification::Internal,
        ..Default::default()
    }
}

#[test]
fn test_nullable_false_generates_not_null() {
    let field = ContractField { nullable: false, ..make_field("order_id") };
    let contract = make_test_contract(vec![field]);
    let config = DqConfig { include_baseline: false, ..Default::default() };
    let suites = generate_contract_suites(&contract, &config);
    let field_suite = suites.iter().find(|s| s.name.ends_with("_field_suite")).unwrap();
    assert!(field_suite.expectations.iter().any(|e|
        e.expectation_type == "expect_column_values_to_not_be_null"
        && e.kwargs.get("column").and_then(|v| v.as_str()) == Some("order_id")
    ));
}

#[test]
fn test_primary_key_generates_unique_and_not_null() {
    let field = ContractField { primary_key: true, ..make_field("id") };
    let contract = make_test_contract(vec![field]);
    let config = DqConfig { include_baseline: false, ..Default::default() };
    let suites = generate_contract_suites(&contract, &config);
    let field_suite = suites.iter().find(|s| s.name.ends_with("_field_suite")).unwrap();
    let has_not_null = field_suite.expectations.iter().any(|e|
        e.expectation_type == "expect_column_values_to_not_be_null"
        && e.kwargs.get("column").and_then(|v| v.as_str()) == Some("id")
    );
    let has_unique = field_suite.expectations.iter().any(|e|
        e.expectation_type == "expect_column_values_to_be_unique"
        && e.kwargs.get("column").and_then(|v| v.as_str()) == Some("id")
    );
    assert!(has_not_null && has_unique);
}

#[test]
fn test_email_type_generates_valid_email_expectation() {
    let field = ContractField { logical_type: LogicalType::Email, ..make_field("email") };
    let contract = make_test_contract(vec![field]);
    let config = DqConfig { include_baseline: false, ..Default::default() };
    let suites = generate_contract_suites(&contract, &config);
    let field_suite = suites.iter().find(|s| s.name.ends_with("_field_suite")).unwrap();
    assert!(field_suite.expectations.iter().any(|e|
        e.expectation_type == "expect_column_values_to_be_valid_email"
    ));
}

#[test]
fn test_pii_field_generates_pii_suite() {
    let field = ContractField { pii: true, ..make_field("ssn") };
    let contract = make_test_contract(vec![field]);
    let config = DqConfig { include_baseline: false, ..Default::default() };
    let suites = generate_contract_suites(&contract, &config);
    assert!(suites.iter().any(|s| s.name.ends_with("_pii_suite")));
}

#[test]
fn test_no_pii_fields_no_pii_suite() {
    let field = make_field("name");
    let contract = make_test_contract(vec![field]);
    let config = DqConfig { include_baseline: false, ..Default::default() };
    let suites = generate_contract_suites(&contract, &config);
    assert!(!suites.iter().any(|s| s.name.ends_with("_pii_suite")));
}

#[test]
fn test_min_max_constraint_merges_into_single_between() {
    let field = ContractField {
        constraints: vec![FieldConstraint::Minimum(0.0), FieldConstraint::Maximum(100.0)],
        logical_type: LogicalType::Float,
        ..make_field("score")
    };
    let contract = make_test_contract(vec![field]);
    let config = DqConfig { include_baseline: false, ..Default::default() };
    let suites = generate_contract_suites(&contract, &config);
    let con_suite = suites.iter().find(|s| s.name.ends_with("_constraints_suite")).unwrap();
    let between_count = con_suite.expectations.iter()
        .filter(|e| e.expectation_type == "expect_column_values_to_be_between"
            && e.kwargs.get("column").and_then(|v| v.as_str()) == Some("score"))
        .count();
    assert_eq!(between_count, 1, "Min+Max should merge into one between expectation");
}

#[test]
fn test_allowed_values_generates_in_set() {
    let field = ContractField {
        constraints: vec![FieldConstraint::AllowedValues(vec!["A".into(), "B".into(), "C".into()])],
        ..make_field("status")
    };
    let contract = make_test_contract(vec![field]);
    let config = DqConfig { include_baseline: false, ..Default::default() };
    let suites = generate_contract_suites(&contract, &config);
    let con_suite = suites.iter().find(|s| s.name.ends_with("_constraints_suite")).unwrap();
    assert!(con_suite.expectations.iter().any(|e|
        e.expectation_type == "expect_column_values_to_be_in_set"
        && e.kwargs.get("column").and_then(|v| v.as_str()) == Some("status")
    ));
}

#[test]
fn test_restricted_classification_generates_encrypted() {
    let field = ContractField {
        classification: DataClassification::Restricted,
        pii: true,
        ..make_field("secret")
    };
    let contract = make_test_contract(vec![field]);
    let config = DqConfig { include_baseline: false, ..Default::default() };
    let suites = generate_contract_suites(&contract, &config);
    let pii_suite = suites.iter().find(|s| s.name.ends_with("_pii_suite")).unwrap();
    assert!(pii_suite.expectations.iter().any(|e|
        e.expectation_type == "expect_column_values_to_be_encrypted"
    ));
}
```

### E.3 `tests/integration.rs`

Full pipeline test: `DataContract` → `DqSuiteSet` → JSON files → verify GX format.

```rust
use data_ingestion_core::{process, ContractBuilderConfig};
use data_quality_core::{generate_all_suites, serialize_suite_set, DqConfig, DqOutputFormat};

#[test]
fn test_full_pipeline_json_schema_to_gx_suites() {
    // 1. Ingest a JSON Schema file into a DataContract
    let schema_bytes = include_bytes!("../../../examples/sample_json_schema.json");
    let contract = process(
        schema_bytes, None, Some("sample_json_schema.json"),
        ContractBuilderConfig::default()
    ).expect("Ingestion must succeed");

    // 2. Generate all suites
    let config = DqConfig::default();
    let suite_set = generate_all_suites(&contract, &config);

    // 3. Verify structure
    assert_eq!(suite_set.baseline_suites.len(), 17, "Must have 17 baseline suites");
    let baseline_total: usize = suite_set.baseline_suites.iter()
        .map(|s| s.expectations.len()).sum();
    assert_eq!(baseline_total, 1328, "Baseline must have 1328 tests");
    assert!(suite_set.total_test_count >= 1328);

    // 4. Serialize to GX JSON
    let files = serialize_suite_set(&suite_set, DqOutputFormat::GxJson)
        .expect("Serialization must succeed");
    assert!(!files.is_empty());

    // 5. Verify each suite file is valid GX 1.x JSON
    for file in files.iter().filter(|f| f.filename.ends_with(".json") && !f.filename.ends_with("manifest.json")) {
        let json: serde_json::Value = serde_json::from_slice(&file.content)
            .expect(&format!("File {} must be valid JSON", file.filename));
        assert!(json.get("name").is_some(), "Suite must have 'name'");
        assert!(json.get("expectations").is_some(), "Suite must have 'expectations'");
        let meta = json.get("meta").expect("Suite must have 'meta'");
        assert_eq!(
            meta.get("great_expectations_version").and_then(|v| v.as_str()),
            Some("1.11.3"),
            "GX version must be 1.11.3"
        );
        // Verify each expectation has "type" (not "expectation_type")
        for exp in json["expectations"].as_array().unwrap() {
            assert!(exp.get("type").is_some(), "Expectation must use 'type' key");
            assert!(exp.get("kwargs").is_some(), "Expectation must have 'kwargs'");
        }
    }

    // 6. Verify manifest.json exists and is valid
    let manifest_file = files.iter().find(|f| f.filename.ends_with("manifest.json"))
        .expect("manifest.json must be generated");
    let manifest: serde_json::Value = serde_json::from_slice(&manifest_file.content)
        .expect("manifest.json must be valid JSON");
    assert!(manifest.get("total_test_count").is_some());
    assert!(manifest.get("suites").is_some());

    // 7. Verify summary.csv exists
    assert!(files.iter().any(|f| f.filename.ends_with("summary.csv")));
}
```

---

## Part F: Key Architectural Decisions

| Decision | Choice | Rationale |
|---|---|---|
| Crate separation | `data-quality-core` is pure Rust with no FFI | Mirrors the `data-ingestion-core` pattern; WASM/Python wrappers are thin |
| Expectation model | `ExpectationConfig` with `IndexMap<String, Value>` kwargs | Matches GX JSON structure exactly; `IndexMap` ensures stable key ordering |
| Baseline suite storage | Hardcoded Rust structs (not loaded from JSON files) | Zero runtime I/O; suites always available; compile-time count verification |
| Test ID format | Baseline: `DV001`; Contract-specific: `ORD-FLD-001` | Preserves compatibility with existing data-quality-service test IDs |
| GX version field | Configurable via `DqConfig.gx_version`, default `"1.11.3"` | Allows upgrading GX version without code changes |
| PII suite conditionality | Only generated when `contract.fields.iter().any(|f| f.pii)` | Avoids empty suites; keeps output clean for non-PII contracts |
| Constraint merging | `Minimum`+`Maximum` merged into single `between` expectation | Avoids redundant GX expectations; matches GX best practices |
| FK value_set | Empty `[]` with FK info in `meta` | FK values are runtime data; meta provides reference for GX runner |
| `serde rename` on `expectation_type` | `#[serde(rename = "type")]` | GX 1.x uses `"type"` not `"expectation_type"` as the JSON key |
| `indexmap` dependency | Added to workspace `Cargo.toml` | Needed for stable JSON key ordering in kwargs; not in existing workspace |
| WASM content encoding | Base64-encoded `content_base64` field | Binary-safe transport over JS string boundary |
| Python dict bridge | `PyDict` → `serde_json::to_string` → `DataContract` deserialization | Avoids complex PyO3 type mapping; reuses existing serde derives |