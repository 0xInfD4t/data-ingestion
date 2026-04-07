# Python ABI Strategy

> Part of the [`data-ingestion`](ARCHITECTURE.md) architecture documentation.

This document covers the Python binding strategy, PyO3 class wrappers, module structure, `pyproject.toml` configuration, and usage examples.

---

## Table of Contents

1. [Recommended Approach](#1-recommended-approach)
2. [Crate Structure](#2-crate-structure)
3. [PyO3 Module Registration](#3-pyo3-module-registration)
4. [Ingestion Functions](#4-ingestion-functions)
5. [PyDataContract Class](#5-pydatacontract-class)
6. [PyContractBuilder Class](#6-pycontractbuilder-class)
7. [Output Functions](#7-output-functions)
8. [Python Exception Mapping](#8-python-exception-mapping)
9. [pyproject.toml Configuration](#9-pyprojecttoml-configuration)
10. [Python Package Layout](#10-python-package-layout)
11. [Python Usage Examples](#11-python-usage-examples)
12. [Wheel Distribution](#12-wheel-distribution)

---

## 1. Recommended Approach

### Dual-Target Strategy: PyO3 Native + WASM Fallback

| Environment | Mechanism | Recommended |
|---|---|---|
| CPython 3.8+ (native) | PyO3 + maturin wheel | **Primary** |
| Pyodide / browser Python | WASM + `wasmtime-py` | Fallback only |
| PyPy | Not supported | — |

**Why PyO3 over a pure WASM approach for Python:**

1. **Performance** — PyO3 calls are direct FFI with no serialization overhead; WASM requires JSON round-trips for every call
2. **Python integration** — PyO3 exposes real Python objects (`list`, `dict`, `str`); WASM returns opaque strings
3. **File I/O** — PyO3 can use `std::fs` directly; WASM cannot
4. **Error handling** — PyO3 maps Rust errors to native Python exceptions; WASM returns JSON error strings
5. **Distribution** — `maturin` produces standard `pip`-installable wheels; WASM requires a separate runtime
6. **Ecosystem** — PyO3 wheels work with all Python tooling (pytest, mypy, pandas, etc.) out of the box

**When to use the WASM fallback:**
- Running Python in a browser via Pyodide
- Environments where native compilation is impossible (e.g., locked-down CI)
- See [`WASM_STRATEGY.md § 8`](WASM_STRATEGY.md#8-javascript-usage-examples) for the `wasmtime-py` example

---

## 2. Crate Structure

**Location:** [`crates/data-ingestion-python/src/`](../crates/data-ingestion-python/src/)

```
crates/data-ingestion-python/
├── Cargo.toml
├── pyproject.toml
├── python/
│   └── data_ingestion/
│       ├── __init__.py          # Re-exports from native extension
│       ├── py.typed             # PEP 561 marker
│       └── _types.pyi           # Stub file for IDE support
└── src/
    ├── lib.rs                   # PyO3 module root
    ├── py_contract.rs           # PyDataContract, PyContractField wrappers
    ├── py_ingestion.rs          # ingest_file, ingest_bytes, ingest_string
    └── py_output.rs             # convert_contract, detect_format
```

**`Cargo.toml`:**
```toml
[package]
name = "data-ingestion-python"
version = "0.1.0"
edition = "2021"

[lib]
name = "data_ingestion"
crate-type = ["cdylib"]

[dependencies]
data-ingestion-core = { path = "../data-ingestion-core" }
pyo3 = { version = "0.21", features = ["extension-module"] }
serde_json = "1"
```

---

## 3. PyO3 Module Registration

**File:** [`crates/data-ingestion-python/src/lib.rs`](../crates/data-ingestion-python/src/lib.rs)

```rust
use pyo3::prelude::*;

mod py_contract;
mod py_ingestion;
mod py_output;

use py_contract::{PyContractField, PyDataContract};
use py_ingestion::{detect_format, ingest_bytes, ingest_file, ingest_string};
use py_output::{convert_contract, PyContractBuilder};

/// The `data_ingestion` Python extension module.
///
/// Provides functions and classes for ingesting heterogeneous data format
/// files and producing structured data contracts.
#[pymodule]
fn _data_ingestion(_py: Python, m: &PyModule) -> PyResult<()> {
    // Classes
    m.add_class::<PyDataContract>()?;
    m.add_class::<PyContractField>()?;
    m.add_class::<PyContractBuilder>()?;

    // Ingestion functions
    m.add_function(wrap_pyfunction!(ingest_file, m)?)?;
    m.add_function(wrap_pyfunction!(ingest_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(ingest_string, m)?)?;

    // Utility functions
    m.add_function(wrap_pyfunction!(detect_format, m)?)?;
    m.add_function(wrap_pyfunction!(convert_contract, m)?)?;

    // Version constant
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    Ok(())
}
```

---

## 4. Ingestion Functions

**File:** [`crates/data-ingestion-python/src/py_ingestion.rs`](../crates/data-ingestion-python/src/py_ingestion.rs)

```rust
use pyo3::prelude::*;
use pyo3::types::PyDict;
use data_ingestion_core::{Engine, FormatHint, ContractBuilderConfig};
use crate::py_contract::PyDataContract;

/// Ingest a file from disk and return a DataContract.
///
/// Args:
///     path: Path to the input file
///     config: Optional dict with ContractBuilderConfig fields
///
/// Returns:
///     DataContract object
///
/// Raises:
///     ValueError: If the file format is unsupported or parsing fails
///     IOError: If the file cannot be read
#[pyfunction]
#[pyo3(signature = (path, config=None))]
pub fn ingest_file(
    path: &str,
    config: Option<&PyDict>,
) -> PyResult<PyDataContract>;

/// Ingest raw bytes and return a DataContract.
///
/// Args:
///     data: Raw file bytes
///     filename_hint: Optional filename for format detection
///     config: Optional dict with ContractBuilderConfig fields
///
/// Returns:
///     DataContract object
///
/// Raises:
///     ValueError: If the format is unsupported or parsing fails
#[pyfunction]
#[pyo3(signature = (data, filename_hint=None, config=None))]
pub fn ingest_bytes(
    data: &[u8],
    filename_hint: Option<&str>,
    config: Option<&PyDict>,
) -> PyResult<PyDataContract>;

/// Ingest a string (text content) and return a DataContract.
///
/// Args:
///     text: String content of the schema/data file
///     filename_hint: Optional filename for format detection
///     config: Optional dict with ContractBuilderConfig fields
///
/// Returns:
///     DataContract object
///
/// Raises:
///     ValueError: If the format is unsupported or parsing fails
#[pyfunction]
#[pyo3(signature = (text, filename_hint=None, config=None))]
pub fn ingest_string(
    text: &str,
    filename_hint: Option<&str>,
    config: Option<&PyDict>,
) -> PyResult<PyDataContract>;

/// Detect the format of input bytes without full parsing.
///
/// Args:
///     data: Raw file bytes
///     filename_hint: Optional filename for extension-based detection
///
/// Returns:
///     One of: "JsonDataset", "JsonSchema", "DataDictionary",
///             "DataSchema", "DataStructure", "Xml", "Xsd"
#[pyfunction]
#[pyo3(signature = (data, filename_hint=None))]
pub fn detect_format(
    data: &[u8],
    filename_hint: Option<&str>,
) -> PyResult<String>;
```

### `config` Dict Keys

The optional `config` dict accepts the same fields as `ContractBuilderConfig`:

```python
config = {
    "contract_name": "UserProfile",       # str
    "contract_version": "1.0.0",          # str (semver)
    "owner_team": "data-platform",        # str
    "owner_email": "data@example.com",    # str
    "owner_slack": "#data-platform",      # str
    "domain": "identity",                 # str
    "tags": ["pii", "user-data"],         # list[str]
    "enrich_pii": True,                   # bool
    "default_classification": "Internal", # str
    "primary_key_hints": ["user_id"],     # list[str]
    "sla_freshness": "PT1H",              # str (ISO 8601 duration)
    "sla_availability": 99.9,             # float
    "sla_max_latency_ms": 500,            # int
    "sla_retention_days": 90,             # int
    "lineage_source_system": "postgres",  # str
    "lineage_source_table": "users",      # str
}
```

---

## 5. `PyDataContract` Class

**File:** [`crates/data-ingestion-python/src/py_contract.rs`](../crates/data-ingestion-python/src/py_contract.rs)

```rust
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use data_ingestion_core::contract::model::DataContract;

#[pyclass(name = "DataContract")]
pub struct PyDataContract {
    pub(crate) inner: DataContract,
}

#[pymethods]
impl PyDataContract {
    // --- Identity ---
    #[getter] pub fn id(&self) -> &str;
    #[getter] pub fn name(&self) -> &str;
    #[getter] pub fn version(&self) -> &str;
    #[getter] pub fn description(&self) -> Option<&str>;
    #[getter] pub fn created_at(&self) -> &str;
    #[getter] pub fn updated_at(&self) -> &str;
    #[getter] pub fn domain(&self) -> Option<&str>;
    #[getter] pub fn tags(&self) -> Vec<String>;

    // --- Schema ---
    /// Returns list of PyContractField objects
    #[getter] pub fn fields(&self) -> Vec<PyContractField>;
    #[getter] pub fn primary_keys(&self) -> Vec<String>;

    // --- Ownership ---
    #[getter] pub fn owner_team(&self) -> Option<String>;
    #[getter] pub fn owner_email(&self) -> Option<String>;

    // --- Serialization ---
    /// Serialize to JSON string
    pub fn to_json(&self) -> PyResult<String>;
    /// Serialize to YAML string
    pub fn to_yaml(&self) -> PyResult<String>;
    /// Serialize to XML string
    pub fn to_xml(&self) -> PyResult<String>;
    /// Serialize to CSV string
    pub fn to_csv(&self) -> PyResult<String>;

    /// Convert to a Python dict (deep conversion)
    pub fn to_dict(&self, py: Python) -> PyResult<PyObject>;

    // --- Dunder methods ---
    pub fn __repr__(&self) -> String;
    /// Returns JSON string
    pub fn __str__(&self) -> String;
    /// Enables `len(contract)` — returns number of top-level fields
    pub fn __len__(&self) -> usize;
}

#[pyclass(name = "ContractField")]
pub struct PyContractField {
    pub(crate) inner: ContractField,
}

#[pymethods]
impl PyContractField {
    #[getter] pub fn name(&self) -> &str;
    #[getter] pub fn logical_type(&self) -> String;
    #[getter] pub fn physical_type(&self) -> Option<String>;
    #[getter] pub fn nullable(&self) -> bool;
    #[getter] pub fn required(&self) -> bool;
    #[getter] pub fn pii(&self) -> bool;
    #[getter] pub fn deprecated(&self) -> bool;
    #[getter] pub fn description(&self) -> Option<&str>;
    #[getter] pub fn tags(&self) -> Vec<String>;
    #[getter] pub fn classification(&self) -> Option<String>;
    #[getter] pub fn examples(&self) -> Vec<String>;  // JSON-encoded values
    #[getter] pub fn default_value(&self) -> Option<String>;  // JSON-encoded

    /// Returns list of dicts: [{"type": "MinLength", "value": 3}, ...]
    #[getter] pub fn constraints(&self) -> PyResult<Vec<PyObject>>;

    /// Convert to a Python dict
    pub fn to_dict(&self, py: Python) -> PyResult<PyObject>;

    pub fn __repr__(&self) -> String;
}
```

---

## 6. `PyContractBuilder` Class

**File:** [`crates/data-ingestion-python/src/py_output.rs`](../crates/data-ingestion-python/src/py_output.rs)

A fluent builder that mirrors the `ContractBuilderConfig` API with method chaining:

```rust
#[pyclass(name = "ContractBuilder")]
pub struct PyContractBuilder {
    config: ContractBuilderConfig,
}

#[pymethods]
impl PyContractBuilder {
    #[new]
    pub fn new() -> Self;

    /// Set the contract name. Returns self for chaining.
    pub fn name(mut slf: PyRefMut<Self>, name: &str) -> PyRefMut<Self>;

    /// Set the semantic version. Returns self for chaining.
    pub fn version(mut slf: PyRefMut<Self>, version: &str) -> PyRefMut<Self>;

    /// Set owner info. Returns self for chaining.
    pub fn owner(
        mut slf: PyRefMut<Self>,
        team: &str,
        email: &str,
    ) -> PyRefMut<Self>;

    /// Set the data domain. Returns self for chaining.
    pub fn domain(mut slf: PyRefMut<Self>, domain: &str) -> PyRefMut<Self>;

    /// Set SLA parameters. Returns self for chaining.
    pub fn sla(
        mut slf: PyRefMut<Self>,
        freshness: Option<&str>,
        availability: Option<f64>,
        max_latency_ms: Option<u64>,
        retention_days: Option<u64>,
    ) -> PyRefMut<Self>;

    /// Set lineage info. Returns self for chaining.
    pub fn lineage(
        mut slf: PyRefMut<Self>,
        source_system: Option<&str>,
        source_table: Option<&str>,
        notes: Option<&str>,
    ) -> PyRefMut<Self>;

    /// Set tags. Returns self for chaining.
    pub fn tags(mut slf: PyRefMut<Self>, tags: Vec<String>) -> PyRefMut<Self>;

    /// Enable/disable PII auto-detection. Returns self for chaining.
    pub fn enable_pii_detection(
        mut slf: PyRefMut<Self>,
        enabled: bool,
    ) -> PyRefMut<Self>;

    /// Set default data classification. Returns self for chaining.
    pub fn default_classification(
        mut slf: PyRefMut<Self>,
        classification: &str,
    ) -> PyRefMut<Self>;

    /// Build a DataContract from a file path.
    pub fn build_from_file(&self, path: &str) -> PyResult<PyDataContract>;

    /// Build a DataContract from raw bytes.
    #[pyo3(signature = (data, filename_hint=None))]
    pub fn build_from_bytes(
        &self,
        data: &[u8],
        filename_hint: Option<&str>,
    ) -> PyResult<PyDataContract>;

    /// Build a DataContract from a string.
    #[pyo3(signature = (text, filename_hint=None))]
    pub fn build_from_string(
        &self,
        text: &str,
        filename_hint: Option<&str>,
    ) -> PyResult<PyDataContract>;
}
```

---

## 7. Output Functions

```rust
/// Convert an existing DataContract JSON string to another format.
///
/// Args:
///     contract_json: JSON string of a DataContract
///     output_format: One of "json", "yaml", "xml", "csv"
///
/// Returns:
///     Serialized string in the requested format
///
/// Raises:
///     ValueError: If output_format is invalid or contract_json is malformed
#[pyfunction]
pub fn convert_contract(
    contract_json: &str,
    output_format: &str,
) -> PyResult<String>;
```

---

## 8. Python Exception Mapping

| Rust error | Python exception | When raised |
|---|---|---|
| `IngestionError::UnsupportedFormat` | `ValueError` | Unknown or unsupported input format |
| `IngestionError::ParseError` | `ValueError` | Malformed JSON, XML, XSD, etc. |
| `IngestionError::DetectionFailed` | `ValueError` | Cannot determine format |
| `IngestionError::UnresolvedReference` | `ValueError` | `$ref` or XSD type reference not found |
| `IngestionError::CircularReference` | `ValueError` | Circular `$ref` or XSD type cycle |
| `IngestionError::IoError` | `IOError` | File not found, permission denied |
| `TransformError::InvalidIr` | `RuntimeError` | Internal IR is malformed (bug) |
| `TransformError::TypeResolutionFailed` | `RuntimeError` | Cannot resolve a type (bug) |
| `OutputError::SerializationFailed` | `RuntimeError` | Serialization to output format failed |
| `OutputError::UnsupportedOutputFormat` | `ValueError` | Invalid output format string |

**Implementation pattern:**
```rust
fn map_err(e: IngestionError) -> PyErr {
    match e {
        IngestionError::IoError(io) => PyErr::new::<pyo3::exceptions::PyIOError, _>(
            io.to_string()
        ),
        IngestionError::ParseError { message, .. } => PyErr::new::<pyo3::exceptions::PyValueError, _>(
            message
        ),
        other => PyErr::new::<pyo3::exceptions::PyValueError, _>(
            other.to_string()
        ),
    }
}
```

---

## 9. `pyproject.toml` Configuration

**File:** [`crates/data-ingestion-python/pyproject.toml`](../crates/data-ingestion-python/pyproject.toml)

```toml
[build-system]
requires = ["maturin>=1.4,<2.0"]
build-backend = "maturin"

[project]
name = "data-ingestion"
version = "0.1.0"
description = "Data contract generation from heterogeneous schema formats"
readme = "../../README.md"
license = { text = "MIT" }
requires-python = ">=3.8"
keywords = ["data-contracts", "schema", "data-engineering", "rust"]
classifiers = [
    "Development Status :: 3 - Alpha",
    "Intended Audience :: Developers",
    "License :: OSI Approved :: MIT License",
    "Programming Language :: Rust",
    "Programming Language :: Python :: Implementation :: CPython",
    "Programming Language :: Python :: 3",
    "Programming Language :: Python :: 3.8",
    "Programming Language :: Python :: 3.9",
    "Programming Language :: Python :: 3.10",
    "Programming Language :: Python :: 3.11",
    "Programming Language :: Python :: 3.12",
    "Topic :: Software Development :: Libraries",
]

[tool.maturin]
# The Rust extension module name (must match #[pymodule] fn name)
module-name = "data_ingestion._data_ingestion"
# Python source directory containing the pure-Python wrapper package
python-source = "python"
# Enable the PyO3 extension-module feature
features = ["pyo3/extension-module"]
# Strip debug symbols from release builds
strip = true
```

---

## 10. Python Package Layout

**File:** [`crates/data-ingestion-python/python/data_ingestion/__init__.py`](../crates/data-ingestion-python/python/data_ingestion/__init__.py)

```python
"""
data_ingestion — Data contract generation from heterogeneous schema formats.

The native extension (_data_ingestion) is imported here and its public
symbols are re-exported for a clean top-level API.
"""

from data_ingestion._data_ingestion import (
    DataContract,
    ContractField,
    ContractBuilder,
    ingest_file,
    ingest_bytes,
    ingest_string,
    detect_format,
    convert_contract,
    __version__,
)

__all__ = [
    "DataContract",
    "ContractField",
    "ContractBuilder",
    "ingest_file",
    "ingest_bytes",
    "ingest_string",
    "detect_format",
    "convert_contract",
    "__version__",
]
```

**Stub file:** [`crates/data-ingestion-python/python/data_ingestion/_types.pyi`](../crates/data-ingestion-python/python/data_ingestion/_types.pyi)

```python
from typing import Any, Optional

class ContractField:
    name: str
    logical_type: str
    physical_type: Optional[str]
    nullable: bool
    required: bool
    pii: bool
    deprecated: bool
    description: Optional[str]
    tags: list[str]
    classification: Optional[str]
    examples: list[str]
    default_value: Optional[str]
    def constraints(self) -> list[dict[str, Any]]: ...
    def to_dict(self) -> dict[str, Any]: ...

class DataContract:
    id: str
    name: str
    version: str
    description: Optional[str]
    created_at: str
    updated_at: str
    domain: Optional[str]
    tags: list[str]
    fields: list[ContractField]
    primary_keys: list[str]
    owner_team: Optional[str]
    owner_email: Optional[str]
    def to_json(self) -> str: ...
    def to_yaml(self) -> str: ...
    def to_xml(self) -> str: ...
    def to_csv(self) -> str: ...
    def to_dict(self) -> dict[str, Any]: ...
    def __len__(self) -> int: ...

class ContractBuilder:
    def __init__(self) -> None: ...
    def name(self, name: str) -> "ContractBuilder": ...
    def version(self, version: str) -> "ContractBuilder": ...
    def owner(self, team: str, email: str) -> "ContractBuilder": ...
    def domain(self, domain: str) -> "ContractBuilder": ...
    def sla(
        self,
        freshness: Optional[str] = None,
        availability: Optional[float] = None,
        max_latency_ms: Optional[int] = None,
        retention_days: Optional[int] = None,
    ) -> "ContractBuilder": ...
    def lineage(
        self,
        source_system: Optional[str] = None,
        source_table: Optional[str] = None,
        notes: Optional[str] = None,
    ) -> "ContractBuilder": ...
    def tags(self, tags: list[str]) -> "ContractBuilder": ...
    def enable_pii_detection(self, enabled: bool) -> "ContractBuilder": ...
    def default_classification(self, classification: str) -> "ContractBuilder": ...
    def build_from_file(self, path: str) -> DataContract: ...
    def build_from_bytes(
        self, data: bytes, filename_hint: Optional[str] = None
    ) -> DataContract: ...
    def build_from_string(
        self, text: str, filename_hint: Optional[str] = None
    ) -> DataContract: ...

def ingest_file(path: str, config: Optional[dict] = None) -> DataContract: ...
def ingest_bytes(
    data: bytes,
    filename_hint: Optional[str] = None,
    config: Optional[dict] = None,
) -> DataContract: ...
def ingest_string(
    text: str,
    filename_hint: Optional[str] = None,
    config: Optional[dict] = None,
) -> DataContract: ...
def detect_format(
    data: bytes, filename_hint: Optional[str] = None
) -> str: ...
def convert_contract(contract_json: str, output_format: str) -> str: ...
```

---

## 11. Python Usage Examples

```python
from data_ingestion import (
    ContractBuilder,
    ingest_file,
    ingest_bytes,
    ingest_string,
    detect_format,
    convert_contract,
)

# ── Simple one-liner ──────────────────────────────────────────────────────────
contract = ingest_file("user_schema.json")
print(contract.to_yaml())

# ── Detect format before ingesting ───────────────────────────────────────────
with open("mystery_file.xml", "rb") as f:
    data = f.read()

fmt = detect_format(data, "mystery_file.xml")
print(f"Detected format: {fmt}")  # e.g. "Xsd"

# ── Fluent builder ────────────────────────────────────────────────────────────
contract = (
    ContractBuilder()
    .name("UserProfile")
    .version("2.1.0")
    .owner(team="data-platform", email="data@example.com")
    .domain("identity")
    .sla(freshness="PT1H", availability=99.9, retention_days=90)
    .lineage(source_system="postgres", source_table="users")
    .tags(["pii", "user-data", "identity"])
    .enable_pii_detection(True)
    .default_classification("Internal")
    .build_from_file("user_profile.xsd")
)

# ── Inspect fields ────────────────────────────────────────────────────────────
print(f"Contract: {contract.name} v{contract.version}")
print(f"Fields: {len(contract)}")

for field in contract.fields:
    pii_marker = " [PII]" if field.pii else ""
    print(f"  {field.name}: {field.logical_type}"
          f" (nullable={field.nullable}){pii_marker}")

# ── Export to multiple formats ────────────────────────────────────────────────
with open("contract.json", "w") as f:
    f.write(contract.to_json())

with open("contract.yaml", "w") as f:
    f.write(contract.to_yaml())

with open("contract.xml", "w") as f:
    f.write(contract.to_xml())

with open("contract.csv", "w") as f:
    f.write(contract.to_csv())

# ── Convert between formats ───────────────────────────────────────────────────
json_str = contract.to_json()
yaml_str = convert_contract(json_str, "yaml")
csv_str  = convert_contract(json_str, "csv")

# ── Use with pandas ───────────────────────────────────────────────────────────
import pandas as pd

fields_data = [f.to_dict() for f in contract.fields]
df = pd.DataFrame(fields_data)
print(df[["name", "logical_type", "nullable", "pii"]].to_string())

# ── Ingest from string (e.g. fetched from API) ────────────────────────────────
import urllib.request

url = "https://example.com/schemas/order.json"
with urllib.request.urlopen(url) as resp:
    schema_text = resp.read().decode("utf-8")

contract = ingest_string(schema_text, filename_hint="order.json")

# ── Error handling ────────────────────────────────────────────────────────────
try:
    contract = ingest_file("broken_schema.json")
except ValueError as e:
    print(f"Parse error: {e}")
except IOError as e:
    print(f"File error: {e}")
```

---

## 12. Wheel Distribution

### Development Install

```bash
cd crates/data-ingestion-python
pip install maturin
maturin develop --release
```

### Production Wheel

```bash
cd crates/data-ingestion-python
maturin build --release --strip
# Output: ../../target/wheels/data_ingestion-0.1.0-cp311-cp311-linux_x86_64.whl
pip install ../../target/wheels/data_ingestion-*.whl
```

### Cross-Platform Builds (CI)

Use `maturin` with `zig` for cross-compilation, or use the official `maturin-action` GitHub Action:

```yaml
# .github/workflows/python-release.yml
- uses: PyO3/maturin-action@v1
  with:
    command: build
    args: --release --strip --out dist
    manylinux: auto   # builds manylinux2014 wheels automatically
```

**Supported wheel platforms:**

| Platform | Tag |
|---|---|
| Linux x86_64 | `manylinux2014_x86_64` |
| Linux aarch64 | `manylinux2014_aarch64` |
| macOS x86_64 | `macosx_10_12_x86_64` |
| macOS arm64 | `macosx_11_0_arm64` |
| Windows x86_64 | `win_amd64` |
