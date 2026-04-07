# Data Quality — Data Models & Public API

> **Part of:** [`DATA_QUALITY_ARCHITECTURE.md`](DATA_QUALITY_ARCHITECTURE.md)
> **Crate:** `data-quality-core`

---

## 1. `DqConfig` — Runtime Configuration

```rust
// crates/data-quality-core/src/config.rs

use serde::{Deserialize, Serialize};

/// Runtime configuration for the DQ generation pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DqConfig {
    /// GX version string embedded in suite metadata. Default: "1.11.3"
    pub gx_version: String,
    /// Include the 17 baseline suites (1328 tests). Default: true
    pub include_baseline: bool,
    /// Include contract-specific suites. Default: true
    pub include_contract_specific: bool,
    /// If Some, only generate these named suites; None = all suites
    pub enabled_suites: Option<Vec<String>>,
    /// Suite names to explicitly skip
    pub disabled_suites: Vec<String>,
    /// Max acceptable null ratio for completeness expectations. Default: 0.05
    pub null_ratio_max: f64,
    /// Min acceptable uniqueness ratio. Default: 0.95
    pub uniqueness_min: f64,
    /// Z-score threshold for anomaly detection expectations. Default: 3.0
    pub z_score_max: f64,
    /// Historical mean for volume anomaly expectations (optional)
    pub historical_mean: Option<f64>,
    /// Historical std dev for volume anomaly expectations (optional)
    pub historical_std: Option<f64>,
}

impl Default for DqConfig {
    fn default() -> Self {
        Self {
            gx_version: "1.11.3".to_string(),
            include_baseline: true,
            include_contract_specific: true,
            enabled_suites: None,
            disabled_suites: vec![],
            null_ratio_max: 0.05,
            uniqueness_min: 0.95,
            z_score_max: 3.0,
            historical_mean: None,
            historical_std: None,
        }
    }
}
```

---

## 2. `ExpectationMeta` — Expectation Metadata

```rust
// crates/data-quality-core/src/expectations/mod.rs

use serde::{Deserialize, Serialize};

/// Metadata attached to every generated expectation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectationMeta {
    /// Unique test ID.
    /// Baseline format:  "DV001", "DC096", "SC1328"
    /// Contract format:  "ORD-FLD-001", "ORD-SCH-002", "ORD-PII-003", "ORD-CON-004"
    pub test_id: String,
    /// Quality dimension category, e.g. "completeness", "validity", "sensitivity"
    pub category: String,
    /// Name of the suite this expectation belongs to
    pub suite: String,
    /// Field name if this is a column-level expectation; None for table-level
    pub contract_field: Option<String>,
    /// DataContract.name if generated from a contract; None for pure baseline
    pub contract_name: Option<String>,
    /// Origin of this expectation
    pub generated_from: GeneratedFrom,
}

/// Distinguishes baseline tests from contract-derived tests.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeneratedFrom {
    /// One of the 1328 pre-defined baseline tests
    Baseline,
    /// Derived from analysis of a specific DataContract
    ContractSpecific { reason: String },
}
```

---

## 3. `ExpectationConfig` — Single GX Expectation

Maps 1:1 to a GX `ExpectationConfiguration` object.

```rust
// crates/data-quality-core/src/expectations/mod.rs

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A single GX expectation. Serializes to the GX 1.x wire format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectationConfig {
    /// GX expectation type string.
    /// Serialized as "type" (not "expectation_type") per GX 1.x spec.
    #[serde(rename = "type")]
    pub expectation_type: String,
    /// GX kwargs dict. Uses IndexMap for stable JSON key ordering.
    pub kwargs: IndexMap<String, Value>,
    /// Metadata for this expectation
    pub meta: ExpectationMeta,
}
```

**Serialization note:** The `#[serde(rename = "type")]` annotation is critical — GX 1.x uses `"type"` as the JSON key, not `"expectation_type"`.

---

## 4. `SuiteMeta` — Suite-Level Metadata

```rust
// crates/data-quality-core/src/expectations/mod.rs

/// Metadata for an ExpectationSuite. Serializes to the GX suite "meta" object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiteMeta {
    /// GX version string for compatibility. Default: "1.11.3"
    pub great_expectations_version: String,
    /// UUID v4 for this suite instance (generated at build time)
    pub suite_id: String,
    /// DataContract.id if this suite was generated from a contract
    pub contract_id: Option<String>,
    /// ISO 8601 generation timestamp
    pub generated_at: Option<String>,
    /// Cached count of expectations in this suite
    pub test_count: usize,
}
```

---

## 5. `ExpectationSuite` — A GX Suite

Maps to a single GX suite JSON file.

```rust
// crates/data-quality-core/src/expectations/mod.rs

/// A complete GX expectation suite. Serializes to a GX 1.x suite JSON file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectationSuite {
    /// Suite name, e.g. "data_validity_suite" or "order_field_suite"
    pub name: String,
    pub expectations: Vec<ExpectationConfig>,
    pub meta: SuiteMeta,
}
```

**GX 1.x JSON wire format:**
```json
{
  "name": "data_validity_suite",
  "expectations": [
    {
      "type": "expect_column_values_to_not_be_null",
      "kwargs": { "column": "patient_id" },
      "meta": {
        "test_id": "DV001",
        "category": "completeness",
        "suite": "data_validity_suite",
        "contract_field": null,
        "contract_name": null,
        "generated_from": "baseline"
      }
    }
  ],
  "meta": {
    "great_expectations_version": "1.11.3",
    "suite_id": "550e8400-e29b-41d4-a716-446655440000",
    "contract_id": null,
    "generated_at": "2026-04-07T16:50:00Z",
    "test_count": 95
  }
}
```

---

## 6. `DqSuiteSet` — Top-Level Output

The complete output of the generation pipeline for one contract.

```rust
// crates/data-quality-core/src/lib.rs

/// The complete output of the DQ generation pipeline for one DataContract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DqSuiteSet {
    /// DataContract.id if generated from a contract; None for pure baseline
    pub contract_id: Option<String>,
    /// DataContract.name if generated from a contract
    pub contract_name: Option<String>,
    /// The 17 baseline suites (1328 tests total, subject to config filtering)
    pub baseline_suites: Vec<ExpectationSuite>,
    /// Contract-specific suites: schema, field, pii (conditional), constraints
    pub contract_suites: Vec<ExpectationSuite>,
    /// Sum of all expectations across all suites in this set
    pub total_test_count: usize,
}
```

---

## 7. `DqOutputFile` — Serialized Output File

```rust
// crates/data-quality-core/src/output/mod.rs

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
```

---

## 8. `DqError` — Error Type

```rust
// crates/data-quality-core/src/error.rs

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DqError {
    #[error("Contract parse error: {0}")]
    ContractParseError(String),

    #[error("Suite generation error in '{suite}': {reason}")]
    SuiteGenerationError { suite: String, reason: String },

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("I/O error: {0}")]
    IoError(String),
}
```

---

## 9. Public API (`src/lib.rs`)

```rust
// crates/data-quality-core/src/lib.rs

use data_ingestion_core::DataContract;

pub use config::DqConfig;
pub use error::DqError;
pub use expectations::mod::{ExpectationConfig, ExpectationSuite, ExpectationMeta, GeneratedFrom, SuiteMeta};
pub use output::mod::{DqOutputFile, DqOutputFormat};

/// Generate only the 17 baseline suites (1328 tests), not contract-specific.
/// Respects config.enabled_suites and config.disabled_suites.
pub fn generate_baseline_suites(config: &DqConfig) -> Vec<ExpectationSuite>;

/// Generate only contract-specific suites from a DataContract.
/// Produces: schema_suite, field_suite, constraints_suite, and optionally pii_suite.
pub fn generate_contract_suites(
    contract: &DataContract,
    config: &DqConfig,
) -> Vec<ExpectationSuite>;

/// Generate both baseline and contract-specific suites.
/// Returns a DqSuiteSet with total_test_count pre-computed.
pub fn generate_all_suites(
    contract: &DataContract,
    config: &DqConfig,
) -> DqSuiteSet;

/// Serialize a DqSuiteSet to output files.
/// Returns one DqOutputFile per suite (JSON or YAML) plus manifest.json and summary.csv.
pub fn serialize_suite_set(
    suite_set: &DqSuiteSet,
    format: DqOutputFormat,
) -> Result<Vec<DqOutputFile>, DqError>;
```

---

## 10. `data-quality-core` Cargo.toml

```toml
[package]
name    = "data-quality-core"
version.workspace = true
edition.workspace = true

[dependencies]
data-ingestion-core = { path = "../data-ingestion-core" }
serde       = { workspace = true }
serde_json  = { workspace = true }
serde_yaml  = { workspace = true }
csv         = { workspace = true }
uuid        = { workspace = true }
thiserror   = { workspace = true }
regex       = { workspace = true }
once_cell   = { workspace = true }
log         = { workspace = true }
indexmap    = { workspace = true }

[dev-dependencies]
pretty_assertions = { workspace = true }
```

**New workspace dependency to add to root `Cargo.toml`:**
```toml
indexmap = { version = "2" }
```
