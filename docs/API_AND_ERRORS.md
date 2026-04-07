# Public API Surface & Error Handling

> Part of the [`data-ingestion`](ARCHITECTURE.md) architecture documentation.

This document defines the complete public API surface for all three targets (native Rust, WASM/JavaScript, Python) and the full error handling strategy.

---

## Table of Contents

1. [Native Rust API](#1-native-rust-api)
2. [WASM / JavaScript API Summary](#2-wasm--javascript-api-summary)
3. [Python API Summary](#3-python-api-summary)
4. [Error Type Hierarchy](#4-error-type-hierarchy)
5. [Error Codes Reference](#5-error-codes-reference)
6. [Error Propagation by Target](#6-error-propagation-by-target)
7. [Design Principles](#7-design-principles)

---

## 1. Native Rust API

**File:** [`crates/data-ingestion-core/src/lib.rs`](../crates/data-ingestion-core/src/lib.rs)

The public Rust API is organized around the `Engine` struct as the primary entry point, with lower-level types re-exported for advanced use.

### 1.1 Re-exports from `lib.rs`

```rust
// Top-level re-exports — these form the public API surface
pub use contract::model::{
    ContractField,
    ContractSchema,
    DataClassification,
    DataContract,
    FieldConstraint,
    FieldLineage,
    ForeignKey,
    IndexDef,
    LineageInfo,
    LogicalType,
    OwnerInfo,
    QualityInfo,
    QualityRule,
    RuleSeverity,
    SlaInfo,
};

pub use error::{IngestionError, OutputError, TransformError};

pub use ingestion::{FormatHint, SourceFormat};

pub use transform::contract_builder::{ContractBuilder, ContractBuilderConfig};

pub use output::OutputFormat;

pub use ir::model::IrDocument;  // exposed for advanced/custom pipelines
```

### 1.2 `Engine` — High-Level Entry Point

```rust
/// High-level engine that combines all pipeline stages.
///
/// For most use cases, construct an Engine and call `ingest()` or
/// `ingest_to_string()`. Use `ContractBuilder` directly for advanced
/// configuration.
pub struct Engine {
    builder_config: ContractBuilderConfig,
}

impl Engine {
    /// Create an Engine with default configuration.
    pub fn new() -> Self;

    /// Create an Engine with custom configuration.
    pub fn with_config(config: ContractBuilderConfig) -> Self;

    /// Ingest raw bytes and produce a DataContract.
    ///
    /// # Arguments
    /// * `input` - Raw file bytes
    /// * `hint`  - Format hint (filename, MIME type, or explicit format)
    ///
    /// # Errors
    /// Returns `IngestionError` if format detection, parsing, or
    /// transformation fails.
    pub fn ingest(
        &self,
        input: &[u8],
        hint: FormatHint,
    ) -> Result<DataContract, IngestionError>;

    /// Ingest raw bytes and serialize directly to a string in the
    /// requested output format.
    ///
    /// Equivalent to `ingest()` followed by `serialize()`.
    pub fn ingest_to_string(
        &self,
        input: &[u8],
        hint: FormatHint,
        output_format: OutputFormat,
    ) -> Result<String, IngestionError>;

    /// Serialize an existing DataContract to a string.
    pub fn serialize(
        &self,
        contract: &DataContract,
        output_format: OutputFormat,
    ) -> Result<String, OutputError>;

    /// Detect the format of input bytes without full parsing.
    pub fn detect_format(input: &[u8], hint: &FormatHint) -> SourceFormat;
}

impl Default for Engine {
    fn default() -> Self { Self::new() }
}
```

### 1.3 `OutputFormat` Enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Json,
    Yaml,
    Xml,
    Csv,
}

impl OutputFormat {
    /// Parse from a string slice ("json", "yaml", "xml", "csv")
    pub fn from_str(s: &str) -> Result<Self, OutputError>;

    /// Returns the MIME content type
    pub fn content_type(&self) -> &'static str;

    /// Returns the file extension (without dot)
    pub fn file_extension(&self) -> &'static str;
}
```

### 1.4 `FormatHint` Struct

```rust
#[derive(Debug, Default, Clone)]
pub struct FormatHint {
    /// Original filename — used for extension-based detection
    pub filename: Option<String>,
    /// MIME type if known (e.g. "application/json")
    pub mime_type: Option<String>,
    /// Explicit format override — skips auto-detection entirely
    pub explicit_format: Option<SourceFormat>,
}

impl FormatHint {
    pub fn from_filename(filename: impl Into<String>) -> Self;
    pub fn from_format(format: SourceFormat) -> Self;
}
```

### 1.5 `ContractBuilderConfig` Struct

```rust
#[derive(Debug, Clone, Default)]
pub struct ContractBuilderConfig {
    /// Override for the contract name.
    /// Defaults to the filename stem from FormatHint.
    pub contract_name: Option<String>,

    /// Semantic version string. Default: "1.0.0"
    pub contract_version: String,

    pub owner: Option<OwnerInfo>,
    pub domain: Option<String>,
    pub sla: Option<SlaInfo>,
    pub lineage: Option<LineageInfo>,
    pub tags: Vec<String>,

    /// When true, auto-detect PII fields by name pattern matching.
    /// Default: false
    pub enrich_pii: bool,

    /// Default classification for fields without an explicit one.
    /// Default: DataClassification::Public
    pub default_classification: DataClassification,

    /// Additional field names to treat as primary keys.
    /// Auto-detection always runs; these are additive.
    pub primary_key_hints: Vec<String>,
}
```

### 1.6 File I/O Helpers (native only)

```rust
// Available only on non-WASM targets
#[cfg(not(target_arch = "wasm32"))]
impl Engine {
    /// Ingest a file from disk.
    pub fn ingest_file(
        &self,
        path: &std::path::Path,
    ) -> Result<DataContract, IngestionError>;

    /// Ingest a file and serialize to string.
    pub fn ingest_file_to_string(
        &self,
        path: &std::path::Path,
        output_format: OutputFormat,
    ) -> Result<String, IngestionError>;
}
```

### 1.7 Low-Level Pipeline Access

For advanced use cases (custom readers, custom transformers):

```rust
// Ingestion layer
pub use ingestion::{FormatDetector, FormatReader};

// IR layer
pub use ir::{IrDocument, IrNormalizer};
pub use ir::model::{
    IrArray, IrConstraint, IrEnum, IrField, IrNode, IrObject, IrType,
};

// Transform layer
pub use transform::{
    ContractBuilder, ConstraintExtractor, MetadataEnricher, TypeResolver,
};

// Output layer
pub use output::ContractSerializer;
```

### 1.8 Usage Examples (Rust)

```rust
use data_ingestion_core::{Engine, FormatHint, OutputFormat, ContractBuilderConfig,
                           OwnerInfo, SlaInfo, DataClassification};

// ── Simple usage ──────────────────────────────────────────────────────────────
let engine = Engine::new();
let bytes = std::fs::read("schema.json")?;
let hint = FormatHint::from_filename("schema.json");
let contract = engine.ingest(&bytes, hint)?;
println!("{}", engine.serialize(&contract, OutputFormat::Yaml)?);

// ── With configuration ────────────────────────────────────────────────────────
let config = ContractBuilderConfig {
    contract_name: Some("UserProfile".into()),
    contract_version: "2.0.0".into(),
    owner: Some(OwnerInfo {
        team: Some("data-platform".into()),
        email: Some("data@example.com".into()),
        slack_channel: Some("#data-platform".into()),
    }),
    domain: Some("identity".into()),
    enrich_pii: true,
    default_classification: DataClassification::Internal,
    tags: vec!["pii".into(), "user-data".into()],
    ..Default::default()
};

let engine = Engine::with_config(config);
let yaml = engine.ingest_to_string(&bytes, hint, OutputFormat::Yaml)?;

// ── Low-level pipeline ────────────────────────────────────────────────────────
use data_ingestion_core::{FormatDetector, IrNormalizer, ContractBuilder};

let format = FormatDetector::detect(&bytes, &hint);
let reader = format.reader();
let ir_doc = reader.read(&bytes, &hint)?;
let normalized = IrNormalizer::normalize(ir_doc)?;
let builder = ContractBuilder::new();
let contract = builder.build(normalized)?;
```

---

## 2. WASM / JavaScript API Summary

Full details in [`WASM_STRATEGY.md`](WASM_STRATEGY.md).

### Exported Functions

| Function | Input | Output | Description |
|---|---|---|---|
| `ingest_to_contract_json` | `Uint8Array`, `string?`, `string?` | `string` | Ingest → JSON contract |
| `ingest_to_contract_yaml` | `Uint8Array`, `string?`, `string?` | `string` | Ingest → YAML contract |
| `ingest_to_contract_xml` | `Uint8Array`, `string?`, `string?` | `string` | Ingest → XML contract |
| `ingest_to_contract_csv` | `Uint8Array`, `string?`, `string?` | `string` | Ingest → CSV contract |
| `detect_format` | `Uint8Array`, `string?` | `string` | Detect format only |
| `convert_contract` | `string`, `string` | `string` | Re-serialize contract |
| `validate_contract_json` | `string` | `boolean` | Validate contract JSON |

### Exported Class

| Class | Description |
|---|---|
| `WasmContractBuilder` | Stateful builder; call `free()` when done |

### Error Shape (thrown as `string`)

```json
{
  "code": "PARSE_ERROR",
  "message": "human-readable message",
  "details": { "source_format": "JsonSchema", "field_path": "$.properties.id" }
}
```

---

## 3. Python API Summary

Full details in [`PYTHON_ABI.md`](PYTHON_ABI.md).

### Module-Level Functions

| Function | Signature | Description |
|---|---|---|
| `ingest_file` | `(path: str, config?: dict) -> DataContract` | Ingest from disk |
| `ingest_bytes` | `(data: bytes, filename_hint?: str, config?: dict) -> DataContract` | Ingest from bytes |
| `ingest_string` | `(text: str, filename_hint?: str, config?: dict) -> DataContract` | Ingest from string |
| `detect_format` | `(data: bytes, filename_hint?: str) -> str` | Detect format only |
| `convert_contract` | `(contract_json: str, output_format: str) -> str` | Re-serialize contract |

### Classes

| Class | Key Methods |
|---|---|
| `DataContract` | `.to_json()`, `.to_yaml()`, `.to_xml()`, `.to_csv()`, `.to_dict()`, `.fields`, `.primary_keys` |
| `ContractField` | `.name`, `.logical_type`, `.nullable`, `.pii`, `.constraints`, `.to_dict()` |
| `ContractBuilder` | Fluent builder: `.name()`, `.version()`, `.owner()`, `.domain()`, `.sla()`, `.enable_pii_detection()`, `.build_from_file()` |

---

## 4. Error Type Hierarchy

**File:** [`crates/data-ingestion-core/src/error.rs`](../crates/data-ingestion-core/src/error.rs)

```rust
use thiserror::Error;

/// Top-level error type for the ingestion pipeline.
/// Wraps TransformError and OutputError for unified handling.
#[derive(Debug, Error)]
pub enum IngestionError {
    #[error("Unsupported format: {format}")]
    UnsupportedFormat { format: String },

    #[error("Parse error in {source_format} at '{field_path}': {message}")]
    ParseError {
        source_format: String,
        field_path: String,
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Format detection failed: {reason}")]
    DetectionFailed { reason: String },

    #[error("Unresolved reference: '{reference}'")]
    UnresolvedReference { reference: String },

    #[error("Circular reference detected at path: {path}")]
    CircularReference { path: String },

    /// Only available on non-WASM targets
    #[cfg(not(target_arch = "wasm32"))]
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Transform failed: {0}")]
    Transform(#[from] TransformError),

    #[error("Output failed: {0}")]
    Output(#[from] OutputError),
}

/// Errors from the IR → DataContract transformation stage.
#[derive(Debug, Error)]
pub enum TransformError {
    #[error("Invalid IR: {reason}")]
    InvalidIr { reason: String },

    #[error("Type resolution failed for field '{field_name}': {reason}")]
    TypeResolutionFailed { field_name: String, reason: String },

    #[error("Constraint extraction failed: {reason}")]
    ConstraintExtractionFailed { reason: String },
}

/// Errors from the DataContract → output serialization stage.
#[derive(Debug, Error)]
pub enum OutputError {
    #[error("Serialization to '{format}' failed: {reason}")]
    SerializationFailed { format: String, reason: String },

    #[error("Unsupported output format: '{format}'. Expected one of: json, yaml, xml, csv")]
    UnsupportedOutputFormat { format: String },

    #[error("Contract deserialization failed: {reason}")]
    DeserializationFailed { reason: String },
}
```

### Error Code Method

Each error variant exposes a stable string code for programmatic handling:

```rust
impl IngestionError {
    /// Returns a stable, uppercase error code string.
    pub fn code(&self) -> &'static str {
        match self {
            Self::UnsupportedFormat { .. }  => "UNSUPPORTED_FORMAT",
            Self::ParseError { .. }         => "PARSE_ERROR",
            Self::DetectionFailed { .. }    => "DETECTION_FAILED",
            Self::UnresolvedReference { .. }=> "UNRESOLVED_REFERENCE",
            Self::CircularReference { .. }  => "CIRCULAR_REFERENCE",
            #[cfg(not(target_arch = "wasm32"))]
            Self::IoError(_)                => "IO_ERROR",
            Self::Transform(e)              => e.code(),
            Self::Output(e)                 => e.code(),
        }
    }
}

impl TransformError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidIr { .. }               => "INVALID_IR",
            Self::TypeResolutionFailed { .. }    => "TYPE_RESOLUTION_FAILED",
            Self::ConstraintExtractionFailed { .. } => "CONSTRAINT_EXTRACTION_FAILED",
        }
    }
}

impl OutputError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::SerializationFailed { .. }    => "SERIALIZATION_FAILED",
            Self::UnsupportedOutputFormat { .. }=> "UNSUPPORTED_OUTPUT_FORMAT",
            Self::DeserializationFailed { .. }  => "DESERIALIZATION_FAILED",
        }
    }
}
```

---

## 5. Error Codes Reference

| Code | Type | When raised | Recovery |
|---|---|---|---|
| `UNSUPPORTED_FORMAT` | `IngestionError` | Input format not recognized or not implemented | Check supported formats; use `explicit_format` hint |
| `PARSE_ERROR` | `IngestionError` | Malformed JSON, XML, XSD, YAML, or CSV | Fix the input file; check `field_path` for location |
| `DETECTION_FAILED` | `IngestionError` | Cannot determine format from content or hint | Provide `filename_hint` or `explicit_format` |
| `UNRESOLVED_REFERENCE` | `IngestionError` | `$ref` or XSD type reference not found | Ensure all referenced types are defined in the same document |
| `CIRCULAR_REFERENCE` | `IngestionError` | Circular `$ref` or XSD type cycle detected | Break the cycle in the source schema |
| `IO_ERROR` | `IngestionError` | File not found, permission denied (native only) | Check file path and permissions |
| `INVALID_IR` | `TransformError` | Internal IR is structurally invalid (indicates a bug) | File a bug report with the input file |
| `TYPE_RESOLUTION_FAILED` | `TransformError` | Cannot map an IR type to a LogicalType (indicates a bug) | File a bug report |
| `CONSTRAINT_EXTRACTION_FAILED` | `TransformError` | Cannot convert an IR constraint (indicates a bug) | File a bug report |
| `SERIALIZATION_FAILED` | `OutputError` | Serialization to the output format failed | Check output format; may indicate a bug |
| `UNSUPPORTED_OUTPUT_FORMAT` | `OutputError` | Invalid output format string | Use one of: `json`, `yaml`, `xml`, `csv` |
| `DESERIALIZATION_FAILED` | `OutputError` | `convert_contract` received invalid contract JSON | Ensure input is a valid DataContract JSON |

---

## 6. Error Propagation by Target

### 6.1 Native Rust

Errors propagate as `Result<T, IngestionError>`. Callers use standard `?` operator or `match`:

```rust
match engine.ingest(&bytes, hint) {
    Ok(contract) => { /* use contract */ }
    Err(IngestionError::ParseError { field_path, message, .. }) => {
        eprintln!("Parse failed at {field_path}: {message}");
    }
    Err(IngestionError::UnsupportedFormat { format }) => {
        eprintln!("Format not supported: {format}");
    }
    Err(e) => {
        eprintln!("Error [{}]: {}", e.code(), e);
    }
}
```

### 6.2 WASM / JavaScript

Errors are thrown as JSON strings. Callers use `try/catch`:

```javascript
try {
    const contract = ingest_to_contract_json(bytes, 'schema.json');
} catch (errJson) {
    const err = JSON.parse(errJson);
    // err.code: "PARSE_ERROR"
    // err.message: "Failed to parse JSON Schema: ..."
    // err.details.field_path: "$.properties.id"
    console.error(`[${err.code}] ${err.message}`);
    if (err.details?.field_path) {
        console.error(`  at: ${err.details.field_path}`);
    }
}
```

**Conversion in `utils.rs`:**

```rust
pub fn ingestion_error_to_js(err: IngestionError) -> JsValue {
    let details = match &err {
        IngestionError::ParseError { source_format, field_path, .. } => {
            serde_json::json!({
                "source_format": source_format,
                "field_path": field_path,
            })
        }
        IngestionError::UnresolvedReference { reference } => {
            serde_json::json!({ "reference": reference })
        }
        IngestionError::CircularReference { path } => {
            serde_json::json!({ "path": path })
        }
        _ => serde_json::Value::Null,
    };

    let payload = serde_json::json!({
        "code": err.code(),
        "message": err.to_string(),
        "details": details,
    });

    JsValue::from_str(&payload.to_string())
}
```

### 6.3 Python

Errors are mapped to native Python exceptions. Callers use standard `try/except`:

```python
from data_ingestion import ingest_file

try:
    contract = ingest_file("schema.json")
except ValueError as e:
    # Covers: UnsupportedFormat, ParseError, DetectionFailed,
    #         UnresolvedReference, CircularReference
    print(f"Ingestion error: {e}")
except IOError as e:
    # Covers: IoError (file not found, permission denied)
    print(f"File error: {e}")
except RuntimeError as e:
    # Covers: InvalidIr, TypeResolutionFailed (internal bugs)
    print(f"Internal error: {e}")
```

**Conversion in `py_ingestion.rs`:**

```rust
fn map_ingestion_error(e: IngestionError) -> PyErr {
    use pyo3::exceptions::{PyIOError, PyRuntimeError, PyValueError};
    match e {
        #[cfg(not(target_arch = "wasm32"))]
        IngestionError::IoError(io) =>
            PyErr::new::<PyIOError, _>(io.to_string()),
        IngestionError::Transform(TransformError::InvalidIr { .. }) |
        IngestionError::Transform(TransformError::TypeResolutionFailed { .. }) =>
            PyErr::new::<PyRuntimeError, _>(e.to_string()),
        _ =>
            PyErr::new::<PyValueError, _>(e.to_string()),
    }
}
```

---

## 7. Design Principles

### 7.1 No Panics in Library Code

All error conditions are represented as `Result<T, E>`. The library never calls `unwrap()`, `expect()`, or `panic!()` in production code paths. The only exception is the WASM panic hook (`console_error_panic_hook`) which catches unexpected panics and routes them to `console.error` for debugging.

### 7.2 Structured Errors with Context

Every error variant carries enough context for the caller to:
1. Identify **what** went wrong (the error code)
2. Identify **where** it went wrong (field path, reference name, etc.)
3. Understand **why** it went wrong (human-readable message)

### 7.3 Stable Error Codes

The `code()` method returns a stable `&'static str` that will not change between versions. This allows downstream consumers to match on error codes programmatically without depending on error message strings.

### 7.4 `#[source]` Chaining

`ParseError` carries an optional `#[source]` field for the underlying parse error (e.g., `serde_json::Error`, `quick_xml::Error`). This preserves the full error chain for debugging while keeping the top-level error type clean.

### 7.5 WASM-Safe Error Types

All error types implement `Send + Sync` and do not contain non-WASM-safe types (e.g., no `std::io::Error` on WASM targets — it is gated behind `#[cfg(not(target_arch = "wasm32"))]`).

### 7.6 No `anyhow` in Public API

`anyhow` is used only in examples, tests, and internal helpers. The public API always returns typed errors (`IngestionError`, `TransformError`, `OutputError`) so callers can match on specific variants.
