use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod custom;
pub mod serializer;
pub mod types;

// ── ExpectationMeta ───────────────────────────────────────────────────────────

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

// ── ExpectationConfig ─────────────────────────────────────────────────────────

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

// ── SuiteMeta ─────────────────────────────────────────────────────────────────

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

// ── ExpectationSuite ──────────────────────────────────────────────────────────

/// A complete GX expectation suite. Serializes to a GX 1.x suite JSON file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectationSuite {
    /// Suite name, e.g. "data_validity_suite" or "order_field_suite"
    pub name: String,
    pub expectations: Vec<ExpectationConfig>,
    pub meta: SuiteMeta,
}

// ── Expectation trait ─────────────────────────────────────────────────────────

/// Trait implemented by all expectation builder structs.
pub trait Expectation {
    /// Returns the GX expectation_type string.
    fn expectation_type(&self) -> &str;

    /// Builds the kwargs IndexMap for this expectation.
    fn build_kwargs(&self) -> IndexMap<String, Value>;

    /// Builds the ExpectationMeta for this expectation.
    fn build_meta(&self, test_id: &str, category: &str, suite: &str) -> ExpectationMeta;

    /// Produces the final ExpectationConfig (default implementation).
    fn build(&self, test_id: &str, category: &str, suite: &str) -> ExpectationConfig {
        ExpectationConfig {
            expectation_type: self.expectation_type().to_string(),
            kwargs: self.build_kwargs(),
            meta: self.build_meta(test_id, category, suite),
        }
    }
}

// ── Helper: build a baseline ExpectationConfig directly ──────────────────────

/// Convenience function to build a baseline ExpectationConfig without a struct.
pub fn make_expectation(
    expectation_type: &str,
    kwargs: IndexMap<String, Value>,
    test_id: &str,
    category: &str,
    suite: &str,
    contract_field: Option<String>,
) -> ExpectationConfig {
    ExpectationConfig {
        expectation_type: expectation_type.to_string(),
        kwargs,
        meta: ExpectationMeta {
            test_id: test_id.to_string(),
            category: category.to_string(),
            suite: suite.to_string(),
            contract_field,
            contract_name: None,
            generated_from: GeneratedFrom::Baseline,
        },
    }
}
