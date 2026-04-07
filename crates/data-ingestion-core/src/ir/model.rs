use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// ── Source Format ─────────────────────────────────────────────────────────────

/// Identifies which format was read to produce an IrDocument.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceFormat {
    JsonDataset,
    JsonSchema,
    /// CSV or YAML data dictionary
    DataDictionary,
    /// Generic schema descriptor (Avro-like, custom)
    DataSchema,
    /// Arbitrary nested JSON/YAML structure
    DataStructure,
    Xml,
    Xsd,
    Unknown,
}

impl std::fmt::Display for SourceFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::JsonDataset   => "JsonDataset",
            Self::JsonSchema    => "JsonSchema",
            Self::DataDictionary=> "DataDictionary",
            Self::DataSchema    => "DataSchema",
            Self::DataStructure => "DataStructure",
            Self::Xml           => "Xml",
            Self::Xsd           => "Xsd",
            Self::Unknown       => "Unknown",
        };
        write!(f, "{}", s)
    }
}

// ── Top-Level Document ────────────────────────────────────────────────────────

/// Top-level IR document produced by any reader.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrDocument {
    /// Which format was read to produce this document.
    pub source_format: SourceFormat,
    /// Original filename or URI hint (for error messages and metadata).
    pub source_hint: Option<String>,
    /// The root node of the IR tree.
    pub root: IrNode,
}

impl IrDocument {
    pub fn new(source_format: SourceFormat, source_hint: Option<String>, root: IrNode) -> Self {
        Self { source_format, source_hint, root }
    }
}

// ── IR Node Tree ──────────────────────────────────────────────────────────────

/// Recursive IR node — represents any schema element.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum IrNode {
    /// A named object with typed fields (maps to a struct/record/complexType).
    Object(IrObject),
    /// A homogeneous array of items.
    Array(IrArray),
    /// A single typed field (leaf node).
    Field(IrField),
    /// An enumeration of allowed values.
    Enum(IrEnum),
    /// A union of multiple possible types (oneOf / anyOf / XSD choice).
    Union(Vec<IrNode>),
    /// A named reference to another type (resolved during normalization).
    Reference(String),
}

impl IrNode {
    /// Returns the name of this node if it has one.
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Object(o) => o.name.as_deref(),
            Self::Array(a)  => a.name.as_deref(),
            Self::Field(f)  => Some(&f.name),
            Self::Enum(e)   => e.name.as_deref(),
            Self::Union(_)  => None,
            Self::Reference(r) => Some(r.as_str()),
        }
    }
}

// ── IrObject ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrObject {
    pub name: Option<String>,
    pub fields: Vec<IrField>,
    pub description: Option<String>,
    /// Arbitrary extra metadata from the source format.
    pub metadata: HashMap<String, serde_json::Value>,
}

impl IrObject {
    pub fn new(name: Option<String>) -> Self {
        Self {
            name,
            fields: Vec::new(),
            description: None,
            metadata: HashMap::new(),
        }
    }
}

// ── IrArray ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrArray {
    pub name: Option<String>,
    pub item_type: Box<IrNode>,
    pub min_items: Option<u64>,
    pub max_items: Option<u64>,
}

// ── IrField ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrField {
    pub name: String,
    pub ir_type: IrType,
    pub nullable: bool,
    pub required: bool,
    pub description: Option<String>,
    pub constraints: Vec<IrConstraint>,
    /// Arbitrary extra metadata from the source format.
    pub metadata: HashMap<String, serde_json::Value>,
    pub examples: Vec<serde_json::Value>,
    pub default_value: Option<serde_json::Value>,
    pub deprecated: bool,
    pub tags: Vec<String>,
}

impl IrField {
    pub fn new(name: impl Into<String>, ir_type: IrType) -> Self {
        Self {
            name: name.into(),
            ir_type,
            nullable: false,
            required: false,
            description: None,
            constraints: Vec::new(),
            metadata: HashMap::new(),
            examples: Vec::new(),
            default_value: None,
            deprecated: false,
            tags: Vec::new(),
        }
    }
}

// ── IrEnum ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrEnum {
    pub name: Option<String>,
    pub values: Vec<serde_json::Value>,
    pub base_type: Box<IrType>,
}

// ── IrType ────────────────────────────────────────────────────────────────────

/// The type system for the Intermediate Representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IrType {
    String,
    Integer,
    Float,
    Boolean,
    Date,
    DateTime,
    Time,
    Duration,
    Binary,
    Uuid,
    Uri,
    Email,
    Object(Box<IrObject>),
    Array(Box<IrArray>),
    Enum(IrEnum),
    /// Represents oneOf / anyOf / XSD union.
    Union(Vec<IrType>),
    /// Type could not be determined from source.
    Unknown,
}

impl IrType {
    /// Returns true if this type is Unknown.
    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }

    /// Returns true if this type is a scalar (non-composite).
    pub fn is_scalar(&self) -> bool {
        matches!(
            self,
            Self::String
                | Self::Integer
                | Self::Float
                | Self::Boolean
                | Self::Date
                | Self::DateTime
                | Self::Time
                | Self::Duration
                | Self::Binary
                | Self::Uuid
                | Self::Uri
                | Self::Email
                | Self::Unknown
        )
    }
}

impl std::fmt::Display for IrType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::String   => "String",
            Self::Integer  => "Integer",
            Self::Float    => "Float",
            Self::Boolean  => "Boolean",
            Self::Date     => "Date",
            Self::DateTime => "DateTime",
            Self::Time     => "Time",
            Self::Duration => "Duration",
            Self::Binary   => "Binary",
            Self::Uuid     => "Uuid",
            Self::Uri      => "Uri",
            Self::Email    => "Email",
            Self::Object(_)=> "Object",
            Self::Array(_) => "Array",
            Self::Enum(_)  => "Enum",
            Self::Union(_) => "Union",
            Self::Unknown  => "Unknown",
        };
        write!(f, "{}", s)
    }
}

// ── IrConstraint ──────────────────────────────────────────────────────────────

/// Constraints that can be applied to IR fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IrConstraint {
    MinLength(u64),
    MaxLength(u64),
    Pattern(String),
    Minimum(f64),
    Maximum(f64),
    ExclusiveMinimum(f64),
    ExclusiveMaximum(f64),
    MultipleOf(f64),
    MinItems(u64),
    MaxItems(u64),
    UniqueItems,
    Const(serde_json::Value),
    AllOf(Vec<IrConstraint>),
    AnyOf(Vec<IrConstraint>),
    /// Escape hatch for format-specific constraints not covered above.
    /// Stored as (constraint_name, constraint_value).
    Custom(String, serde_json::Value),
}
