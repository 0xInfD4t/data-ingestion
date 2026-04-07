use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// ── DataContract ──────────────────────────────────────────────────────────────

/// Top-level data contract produced from an `IrDocument`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataContract {
    /// Unique identifier (UUID v4).
    pub id: String,
    /// Human-readable contract name (from source metadata or "unnamed").
    pub name: String,
    /// Semantic version string, default "1.0.0".
    pub version: String,
    pub description: Option<String>,
    pub owner: Option<String>,
    pub domain: Option<String>,
    /// Original source format as a string (e.g. "JsonSchema").
    pub source_format: String,
    pub fields: Vec<ContractField>,
    pub metadata: HashMap<String, String>,
    pub sla: Option<SlaInfo>,
    pub lineage: Option<LineageInfo>,
    pub quality: Option<QualityInfo>,
    /// ISO 8601 creation timestamp (WASM-safe: set by caller or left None).
    pub created_at: Option<String>,
    pub tags: Vec<String>,
}

// ── ContractField ─────────────────────────────────────────────────────────────

/// A single field within a `DataContract`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractField {
    pub name: String,
    pub logical_type: LogicalType,
    /// Original type string from the source format.
    pub physical_type: Option<String>,
    pub nullable: bool,
    pub required: bool,
    pub primary_key: bool,
    pub foreign_key: Option<ForeignKeyRef>,
    pub unique: bool,
    pub description: Option<String>,
    pub constraints: Vec<FieldConstraint>,
    pub example: Option<String>,
    pub default_value: Option<String>,
    /// Whether this field contains Personally Identifiable Information.
    pub pii: bool,
    pub classification: DataClassification,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
    /// Nested fields for object/struct types.
    pub nested_fields: Vec<ContractField>,
}

// ── LogicalType ───────────────────────────────────────────────────────────────

/// Canonical logical type system for data contracts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LogicalType {
    String,
    Integer,
    Long,
    Float,
    Double,
    Boolean,
    Date,
    DateTime,
    Timestamp,
    Time,
    Duration,
    Binary,
    Array {
        item_type: Box<LogicalType>,
    },
    Struct {
        type_name: String,
    },
    Decimal {
        precision: u8,
        scale: u8,
    },
    Uuid,
    Email,
    Uri,
    Json,
    Unknown,
}

impl std::fmt::Display for LogicalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String    => write!(f, "string"),
            Self::Integer   => write!(f, "integer"),
            Self::Long      => write!(f, "long"),
            Self::Float     => write!(f, "float"),
            Self::Double    => write!(f, "double"),
            Self::Boolean   => write!(f, "boolean"),
            Self::Date      => write!(f, "date"),
            Self::DateTime  => write!(f, "date_time"),
            Self::Timestamp => write!(f, "timestamp"),
            Self::Time      => write!(f, "time"),
            Self::Duration  => write!(f, "duration"),
            Self::Binary    => write!(f, "binary"),
            Self::Array { item_type } => write!(f, "array<{}>", item_type),
            Self::Struct { type_name } => write!(f, "struct<{}>", type_name),
            Self::Decimal { precision, scale } => write!(f, "decimal({},{})", precision, scale),
            Self::Uuid      => write!(f, "uuid"),
            Self::Email     => write!(f, "email"),
            Self::Uri       => write!(f, "uri"),
            Self::Json      => write!(f, "json"),
            Self::Unknown   => write!(f, "unknown"),
        }
    }
}

// ── DataClassification ────────────────────────────────────────────────────────

/// Data sensitivity classification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DataClassification {
    Public,
    Internal,
    Confidential,
    Restricted,
}

impl Default for DataClassification {
    fn default() -> Self {
        DataClassification::Internal
    }
}

impl std::fmt::Display for DataClassification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Public       => write!(f, "public"),
            Self::Internal     => write!(f, "internal"),
            Self::Confidential => write!(f, "confidential"),
            Self::Restricted   => write!(f, "restricted"),
        }
    }
}

// ── ForeignKeyRef ─────────────────────────────────────────────────────────────

/// Reference to a foreign key target.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKeyRef {
    pub table: String,
    pub column: String,
}

// ── FieldConstraint ───────────────────────────────────────────────────────────

/// Constraints that can be applied to a `ContractField`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldConstraint {
    MinLength(usize),
    MaxLength(usize),
    Pattern(String),
    Minimum(f64),
    Maximum(f64),
    AllowedValues(Vec<String>),
    NotNull,
    Unique,
}

impl std::fmt::Display for FieldConstraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MinLength(v)      => write!(f, "min_length:{}", v),
            Self::MaxLength(v)      => write!(f, "max_length:{}", v),
            Self::Pattern(p)        => write!(f, "pattern:{}", p),
            Self::Minimum(v)        => write!(f, "minimum:{}", v),
            Self::Maximum(v)        => write!(f, "maximum:{}", v),
            Self::AllowedValues(vs) => write!(f, "allowed_values:{}", vs.join(",")),
            Self::NotNull           => write!(f, "not_null"),
            Self::Unique            => write!(f, "unique"),
        }
    }
}

// ── SlaInfo ───────────────────────────────────────────────────────────────────

/// Service Level Agreement information for a data contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaInfo {
    pub freshness_hours: Option<u32>,
    pub availability_percent: Option<f64>,
    pub max_latency_ms: Option<u64>,
}

// ── LineageInfo ───────────────────────────────────────────────────────────────

/// Data lineage information for a data contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageInfo {
    pub source_system: Option<String>,
    pub source_table: Option<String>,
    pub transformation: Option<String>,
}

// ── QualityInfo ───────────────────────────────────────────────────────────────

/// Data quality thresholds and rules for a data contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityInfo {
    pub completeness_threshold: Option<f64>,
    pub uniqueness_threshold: Option<f64>,
    pub validity_rules: Vec<String>,
}
