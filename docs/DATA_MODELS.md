# Data Models

> Part of the [`data-ingestion`](ARCHITECTURE.md) architecture documentation.

This document defines the two core data models:
1. **Intermediate Representation (IR)** — the unified internal model produced by all format readers
2. **DataContract** — the final output model representing a complete data contract

---

## Table of Contents

1. [Intermediate Representation (IR)](#1-intermediate-representation-ir)
2. [DataContract Model](#2-datacontract-model)
3. [Type Mapping Tables](#3-type-mapping-tables)

---

## 1. Intermediate Representation (IR)

**File:** [`crates/data-ingestion-core/src/ir/model.rs`](../crates/data-ingestion-core/src/ir/model.rs)

The IR is the unified internal model that all format readers produce. Every reader — regardless of whether it reads JSON, XSD, or a CSV data dictionary — must produce an `IrDocument`. This decouples the ingestion layer from the transformation layer.

### 1.1 Top-Level Document

```rust
/// Top-level IR document produced by any reader
pub struct IrDocument {
    /// Which format was read to produce this document
    pub source_format: SourceFormat,
    /// Original filename or URI hint (for error messages and metadata)
    pub source_hint: Option<String>,
    /// The root node of the IR tree
    pub root: IrNode,
}

pub enum SourceFormat {
    JsonDataset,
    JsonSchema,
    DataDictionary,
    DataSchema,
    DataStructure,
    Xml,
    Xsd,
}
```

### 1.2 IR Node Tree

```rust
/// Recursive IR node — represents any schema element
pub enum IrNode {
    /// A named object with typed fields (maps to a struct/record/complexType)
    Object(IrObject),
    /// A homogeneous array of items
    Array(IrArray),
    /// A single typed field (leaf node)
    Field(IrField),
    /// An enumeration of allowed values
    Enum(IrEnum),
    /// A union of multiple possible types (oneOf / anyOf / XSD choice)
    Union(Vec<IrNode>),
    /// A named reference to another type (resolved during normalization)
    Reference(String),
}

pub struct IrObject {
    pub name: Option<String>,
    pub fields: Vec<IrField>,
    pub description: Option<String>,
    /// Arbitrary extra metadata from the source format
    pub metadata: HashMap<String, serde_json::Value>,
}

pub struct IrArray {
    pub name: Option<String>,
    pub item_type: Box<IrNode>,
    pub min_items: Option<u64>,
    pub max_items: Option<u64>,
}

pub struct IrField {
    pub name: String,
    pub ir_type: IrType,
    pub nullable: bool,
    pub required: bool,
    pub description: Option<String>,
    pub constraints: Vec<IrConstraint>,
    /// Arbitrary extra metadata from the source format
    pub metadata: HashMap<String, serde_json::Value>,
    pub examples: Vec<serde_json::Value>,
    pub default_value: Option<serde_json::Value>,
    pub deprecated: bool,
    pub tags: Vec<String>,
}

pub struct IrEnum {
    pub name: Option<String>,
    pub values: Vec<serde_json::Value>,
    pub base_type: Box<IrType>,
}
```

### 1.3 IR Types

```rust
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
    /// Represents oneOf / anyOf / XSD union
    Union(Vec<IrType>),
    /// Type could not be determined from source
    Unknown,
}
```

### 1.4 IR Constraints

```rust
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
    /// Escape hatch for format-specific constraints not covered above
    Custom(String, serde_json::Value),
}
```

---

## 2. DataContract Model

**File:** [`crates/data-ingestion-core/src/contract/model.rs`](../crates/data-ingestion-core/src/contract/model.rs)

The `DataContract` is the primary output model. It represents a complete data contract in the data engineering sense, including schema, ownership, SLAs, lineage, and quality rules.

All structs derive `serde::Serialize` and `serde::Deserialize` to support all four output formats.

### 2.1 Root Contract

```rust
/// Root data contract document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataContract {
    /// UUID v4 — generated at build time
    pub id: String,
    pub name: String,
    /// Semantic version string, e.g. "1.0.0"
    pub version: String,
    pub description: Option<String>,
    /// ISO 8601 UTC timestamp
    pub created_at: String,
    /// ISO 8601 UTC timestamp
    pub updated_at: String,
    pub owner: Option<OwnerInfo>,
    pub domain: Option<String>,
    pub tags: Vec<String>,
    pub sla: Option<SlaInfo>,
    pub lineage: Option<LineageInfo>,
    pub schema: ContractSchema,
    pub quality: Option<QualityInfo>,
    /// Arbitrary extra metadata (pass-through from source)
    pub metadata: HashMap<String, serde_json::Value>,
}
```

### 2.2 Ownership & SLA

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerInfo {
    pub team: Option<String>,
    pub email: Option<String>,
    pub slack_channel: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaInfo {
    /// ISO 8601 duration, e.g. "PT1H" = 1 hour freshness
    pub freshness_interval: Option<String>,
    /// e.g. 99.9 for 99.9% uptime
    pub availability_percent: Option<f64>,
    pub max_latency_ms: Option<u64>,
    pub retention_days: Option<u64>,
}
```

### 2.3 Lineage

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageInfo {
    pub source_system: Option<String>,
    pub source_table: Option<String>,
    pub transformation_notes: Option<String>,
    /// IDs of upstream DataContracts this contract depends on
    pub upstream_contracts: Vec<String>,
}
```

### 2.4 Quality Rules

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityInfo {
    /// Minimum fraction of non-null values required (0.0–1.0)
    pub completeness_threshold: Option<f64>,
    /// Fields that must be unique across all rows
    pub uniqueness_fields: Vec<String>,
    pub custom_rules: Vec<QualityRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRule {
    pub name: String,
    /// SQL-like or CEL expression, e.g. "amount > 0"
    pub expression: String,
    pub severity: RuleSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleSeverity {
    Error,
    Warning,
    Info,
}
```

### 2.5 Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractSchema {
    pub fields: Vec<ContractField>,
    /// Field names that form the primary key
    pub primary_keys: Vec<String>,
    pub foreign_keys: Vec<ForeignKey>,
    pub indexes: Vec<IndexDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKey {
    pub fields: Vec<String>,
    /// ID of the referenced DataContract
    pub references_contract: String,
    pub references_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexDef {
    pub name: String,
    pub fields: Vec<String>,
    pub unique: bool,
}
```

### 2.6 ContractField — The Core Field Descriptor

```rust
/// A single field in the data contract schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractField {
    /// Field name (snake_case; original preserved in metadata["original_name"])
    pub name: String,
    /// Logical type — portable across systems
    pub logical_type: LogicalType,
    /// Physical type hint — system-specific, e.g. "VARCHAR(255)", "BIGINT"
    pub physical_type: Option<String>,
    pub nullable: bool,
    pub required: bool,
    pub description: Option<String>,
    pub constraints: Vec<FieldConstraint>,
    pub examples: Vec<serde_json::Value>,
    pub default_value: Option<serde_json::Value>,
    pub deprecated: bool,
    /// Personally Identifiable Information flag
    pub pii: bool,
    pub classification: Option<DataClassification>,
    pub tags: Vec<String>,
    pub lineage: Option<FieldLineage>,
    /// Pass-through metadata from source format
    pub metadata: HashMap<String, serde_json::Value>,
}
```

### 2.7 Logical Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "PascalCase")]
pub enum LogicalType {
    String,
    Integer,
    Long,
    Float,
    Double,
    Boolean,
    Date,
    Timestamp,
    Time,
    Duration,
    Binary,
    Uuid,
    Uri,
    Email,
    Decimal { precision: u8, scale: u8 },
    Enum { values: Vec<String> },
    Array { item_type: Box<LogicalType> },
    Map {
        key_type: Box<LogicalType>,
        value_type: Box<LogicalType>,
    },
    Struct { fields: Vec<ContractField> },
    Union { types: Vec<LogicalType> },
    Unknown,
}
```

### 2.8 Data Classification

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum DataClassification {
    Public,
    Internal,
    Confidential,
    Restricted,
    Custom(String),
}
```

### 2.9 Field Constraints

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldConstraint {
    pub constraint_type: ConstraintType,
    /// The constraint value (e.g. the minimum number, the regex pattern)
    pub value: Option<serde_json::Value>,
    /// Human-readable validation failure message
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ConstraintType {
    NotNull,
    Unique,
    MinLength,
    MaxLength,
    Pattern,
    Minimum,
    Maximum,
    ExclusiveMinimum,
    ExclusiveMaximum,
    MultipleOf,
    MinItems,
    MaxItems,
    UniqueItems,
    Enum,
    Const,
    Custom(String),
}
```

### 2.10 Field Lineage

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldLineage {
    pub source_field: Option<String>,
    pub source_table: Option<String>,
    /// Free-text description of any transformation applied
    pub transformation: Option<String>,
}
```

---

## 3. Type Mapping Tables

### 3.1 JSON Dataset → IrType

| JSON value | IrType | Notes |
|---|---|---|
| `null` | `Unknown` | Also sets `nullable = true` on the field |
| `true` / `false` | `Boolean` | |
| integer number | `Integer` | No decimal point |
| float number | `Float` | Has decimal point |
| string matching ISO 8601 date | `Date` | e.g. `"2024-01-15"` |
| string matching ISO 8601 datetime | `DateTime` | e.g. `"2024-01-15T10:30:00Z"` |
| string matching UUID pattern | `Uuid` | RFC 4122 format |
| string matching email pattern | `Email` | RFC 5322 simplified |
| string matching URI pattern | `Uri` | Starts with scheme |
| other string | `String` | |
| object | `Object` | Recursive |
| array | `Array` | Recursive |

### 3.2 JSON Schema → IrType

| JSON Schema keyword/value | IrType |
|---|---|
| `"type": "string"` | `String` |
| `"type": "integer"` | `Integer` |
| `"type": "number"` | `Float` |
| `"type": "boolean"` | `Boolean` |
| `"type": "object"` | `Object` |
| `"type": "array"` | `Array` |
| `"type": ["T", "null"]` | `T` with `nullable = true` |
| `"format": "date"` | `Date` |
| `"format": "date-time"` | `DateTime` |
| `"format": "time"` | `Time` |
| `"format": "duration"` | `Duration` |
| `"format": "uuid"` | `Uuid` |
| `"format": "email"` | `Email` |
| `"format": "uri"` | `Uri` |
| `"enum": [...]` | `Enum` |
| `"oneOf"` / `"anyOf"` | `Union` |

### 3.3 XSD → IrType

| XSD built-in type | IrType |
|---|---|
| `xs:string`, `xs:token`, `xs:normalizedString` | `String` |
| `xs:integer`, `xs:int`, `xs:long`, `xs:short` | `Integer` |
| `xs:decimal`, `xs:float`, `xs:double` | `Float` |
| `xs:boolean` | `Boolean` |
| `xs:date` | `Date` |
| `xs:dateTime` | `DateTime` |
| `xs:time` | `Time` |
| `xs:duration` | `Duration` |
| `xs:base64Binary`, `xs:hexBinary` | `Binary` |
| `xs:anyURI` | `Uri` |
| `xs:complexType` | `Object` |
| `xs:sequence`, `xs:all` | `Object` (fields merged) |
| `xs:choice` | `Union` |

### 3.4 IrType → LogicalType (Transformation)

| IrType | LogicalType | Notes |
|---|---|---|
| `String` | `String` | |
| `Integer` | `Long` | Safe default (64-bit) |
| `Float` | `Double` | Safe default (64-bit) |
| `Boolean` | `Boolean` | |
| `Date` | `Date` | |
| `DateTime` | `Timestamp` | |
| `Time` | `Time` | |
| `Duration` | `Duration` | |
| `Binary` | `Binary` | |
| `Uuid` | `Uuid` | |
| `Uri` | `Uri` | |
| `Email` | `Email` | |
| `Enum(e)` | `Enum { values }` | Values cast to strings |
| `Array(a)` | `Array { item_type }` | Recursive |
| `Object(o)` | `Struct { fields }` | Recursive |
| `Union(types)` | `Union { types }` | Recursive |
| `Unknown` | `Unknown` | |
