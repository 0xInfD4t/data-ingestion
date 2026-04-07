# Module Breakdown

> Part of the [`data-ingestion`](ARCHITECTURE.md) architecture documentation.

This document describes the internal module structure of `data-ingestion-core`, covering all four pipeline layers: ingestion, parsing (IR), transformation, and serialization.

---

## Table of Contents

1. [Ingestion Layer](#1-ingestion-layer)
2. [Parsing Layer — IR Normalization](#2-parsing-layer--ir-normalization)
3. [Transformation Layer — IR → DataContract](#3-transformation-layer--ir--datacontract)
4. [Serialization Layer — DataContract → Output](#4-serialization-layer--datacontract--output)

---

## 1. Ingestion Layer

**Location:** [`crates/data-ingestion-core/src/ingestion/`](../crates/data-ingestion-core/src/ingestion/)

The ingestion layer reads raw input bytes and produces an `IrDocument`. All readers implement the `FormatReader` trait. The `FormatDetector` selects the appropriate reader automatically.

### 1.1 `FormatReader` Trait

```rust
/// All format readers implement this trait
pub trait FormatReader {
    /// Returns true if this reader can handle the given hint
    fn can_read(&self, hint: &FormatHint) -> bool;

    /// Parse input bytes into an IrDocument
    fn read(&self, input: &[u8], hint: &FormatHint) -> Result<IrDocument, IngestionError>;
}

pub struct FormatHint {
    /// Original filename (used for extension-based detection)
    pub filename: Option<String>,
    /// MIME type if known
    pub mime_type: Option<String>,
    /// Explicit override — skips auto-detection
    pub explicit_format: Option<SourceFormat>,
}
```

### 1.2 `detector.rs` — Format Auto-Detection

`FormatDetector` applies a three-pass heuristic:

1. **Explicit override** — if `FormatHint.explicit_format` is set, use it directly
2. **File extension** — `.json` → `JsonDataset` or `JsonSchema`, `.xml` → `Xml`, `.xsd` → `Xsd`, `.yaml`/`.yml` → `DataStructure`, `.csv` → `DataDictionary`
3. **Content sniffing** (first 512 bytes):
   - Starts with `<?xml` or `<xs:schema` → `Xsd`
   - Starts with `<` → `Xml`
   - Contains `"$schema"` key → `JsonSchema`
   - Starts with `[` or `{` → `JsonDataset`
   - Has CSV header with `field`/`name`/`type` columns → `DataDictionary`
   - Otherwise → `DataStructure`

```rust
pub struct FormatDetector;

impl FormatDetector {
    pub fn detect(input: &[u8], hint: &FormatHint) -> SourceFormat;
}
```

### 1.3 `reader_json.rs` — JSON Dataset Reader

Handles raw JSON arrays (tabular) and objects (nested/hierarchical).

**Algorithm:**
- Parse with `serde_json::from_slice::<serde_json::Value>`
- If root is `Array`: sample up to 1,000 rows; for each key, collect all observed value types; unify types (see type inference rules below); produce `IrNode::Object` with inferred fields
- If root is `Object`: recursively walk the object tree; produce `IrNode::Object`

**Type inference rules (JSON value → IrType):**

| JSON value | IrType | Detection method |
|---|---|---|
| `null` | `Unknown` | Sets `nullable = true` |
| `true` / `false` | `Boolean` | |
| integer number | `Integer` | No decimal point in JSON |
| float number | `Float` | Has decimal point |
| string `"2024-01-15"` | `Date` | Regex: `^\d{4}-\d{2}-\d{2}$` |
| string `"2024-01-15T10:30:00Z"` | `DateTime` | Regex: ISO 8601 datetime |
| string matching UUID | `Uuid` | Regex: RFC 4122 |
| string matching email | `Email` | Regex: simplified RFC 5322 |
| string matching URI | `Uri` | Regex: starts with scheme |
| other string | `String` | |
| object | `Object` | Recursive |
| array | `Array` | Recursive |

**Multi-row type unification:**
- If a field is `Integer` in 90% of rows and `null` in 10% → `IrType::Integer`, `nullable = true`
- If a field is `Integer` in 50% and `String` in 50% → `IrType::Union([Integer, String])`
- Threshold for `Unknown` promotion: if >50% of values are non-null, the non-null type wins

### 1.4 `reader_json_schema.rs` — JSON Schema Reader

Supports JSON Schema drafts: **Draft 4**, **Draft 7**, **Draft 2019-09**, **Draft 2020-12**.

**Algorithm:**
1. Parse with `serde_json::from_slice::<serde_json::Value>`
2. Detect draft version from `$schema` URI
3. Collect `$defs` / `definitions` into a reference map
4. Recursively walk `properties`, resolving `$ref` inline
5. Handle `allOf` by merging field lists
6. Handle `anyOf` / `oneOf` as `IrType::Union`

**JSON Schema keyword → IR mapping:**

| JSON Schema keyword | IR field |
|---|---|
| `type` | `IrField.ir_type` |
| `description` | `IrField.description` |
| `title` | `IrField.metadata["title"]` |
| `required` array | `IrField.required = true` |
| `nullable` / `type: ["T","null"]` | `IrField.nullable = true` |
| `minimum` / `maximum` | `IrConstraint::Minimum/Maximum` |
| `exclusiveMinimum` / `exclusiveMaximum` | `IrConstraint::ExclusiveMinimum/Maximum` |
| `minLength` / `maxLength` | `IrConstraint::MinLength/MaxLength` |
| `pattern` | `IrConstraint::Pattern` |
| `multipleOf` | `IrConstraint::MultipleOf` |
| `minItems` / `maxItems` | `IrConstraint::MinItems/MaxItems` |
| `uniqueItems` | `IrConstraint::UniqueItems` |
| `enum` | `IrType::Enum` |
| `const` | `IrConstraint::Const` |
| `examples` | `IrField.examples` |
| `default` | `IrField.default_value` |
| `deprecated` | `IrField.deprecated = true` |
| `format: "date"` | `IrType::Date` |
| `format: "date-time"` | `IrType::DateTime` |
| `format: "time"` | `IrType::Time` |
| `format: "duration"` | `IrType::Duration` |
| `format: "uuid"` | `IrType::Uuid` |
| `format: "email"` | `IrType::Email` |
| `format: "uri"` | `IrType::Uri` |
| `$ref` | `IrNode::Reference` (resolved) |
| `allOf` | Fields merged into parent object |
| `anyOf` / `oneOf` | `IrType::Union` |

### 1.5 `reader_data_dict.rs` — Data Dictionary Reader

Supports two sub-formats:

**Sub-format A: CSV data dictionary**
- Parsed with the `csv` crate
- Header row detection is case-insensitive with alias support:

| Canonical column | Accepted aliases |
|---|---|
| `field_name` | `name`, `column`, `field`, `attribute` |
| `type` | `data_type`, `dtype`, `logical_type` |
| `nullable` | `optional`, `null`, `is_nullable` |
| `description` | `desc`, `comment`, `notes` |
| `constraints` | `rules`, `validation` |
| `tags` | `labels`, `categories` |
| `pii` | `is_pii`, `sensitive` |
| `classification` | `data_class`, `sensitivity` |

**Sub-format B: YAML/JSON data dictionary**
- Parsed with `serde_yaml` or `serde_json`
- Expected structure: a list of field descriptor objects
- Each object maps directly to an `IrField`

### 1.6 `reader_xml.rs` — XML Data Reader

Parses raw XML documents using `quick-xml`.

**Algorithm:**
- SAX-style event parsing; builds an `IrNode::Object` tree
- XML elements → `IrField` entries with `IrType::Object` for nested elements
- XML attributes → `IrField` entries with `metadata["xml_attribute"] = true`
- Text content of elements → `IrField` with name `"_text"`, type inferred from content
- Type inference uses the same rules as the JSON dataset reader

**Example XML → IR mapping:**
```xml
<User id="123" active="true">
  <Name>Alice</Name>
  <Email>alice@example.com</Email>
</User>
```
Produces:
```
IrObject "User"
  IrField "id"     → IrType::Integer, metadata["xml_attribute"]=true
  IrField "active" → IrType::Boolean, metadata["xml_attribute"]=true
  IrField "Name"   → IrType::String
  IrField "Email"  → IrType::Email
```

### 1.7 `reader_xsd.rs` — XSD Reader

Parses XML Schema Definition files using `quick-xml`.

**Supported XSD constructs:**

| XSD element | Handling |
|---|---|
| `xs:element` | Produces `IrField` |
| `xs:complexType` | Produces `IrObject` |
| `xs:simpleType` | Produces `IrField` with constraints |
| `xs:sequence` | Fields added in order to parent object |
| `xs:all` | Fields added (unordered) to parent object |
| `xs:choice` | Produces `IrType::Union` |
| `xs:attribute` | Produces `IrField` with `metadata["xml_attribute"]=true` |
| `xs:restriction` | Produces `IrConstraint` list |
| `xs:extension` | Merges base type fields with extension fields |
| `xs:complexContent` | Handled via extension/restriction |
| `xs:simpleContent` | Produces `IrField` with `_text` convention |

**XSD facets → IrConstraint:**

| XSD facet | IrConstraint |
|---|---|
| `xs:minLength` | `MinLength` |
| `xs:maxLength` | `MaxLength` |
| `xs:pattern` | `Pattern` |
| `xs:minInclusive` | `Minimum` |
| `xs:maxInclusive` | `Maximum` |
| `xs:minExclusive` | `ExclusiveMinimum` |
| `xs:maxExclusive` | `ExclusiveMaximum` |
| `xs:enumeration` | Collected into `IrType::Enum` |
| `xs:totalDigits` | `Custom("totalDigits", n)` |
| `xs:fractionDigits` | `Custom("fractionDigits", n)` |
| `xs:whiteSpace` | `Custom("whiteSpace", value)` |

**XSD built-in type → IrType:**

| XSD type | IrType |
|---|---|
| `xs:string`, `xs:token`, `xs:normalizedString` | `String` |
| `xs:integer`, `xs:int`, `xs:long`, `xs:short`, `xs:byte` | `Integer` |
| `xs:decimal`, `xs:float`, `xs:double` | `Float` |
| `xs:boolean` | `Boolean` |
| `xs:date` | `Date` |
| `xs:dateTime` | `DateTime` |
| `xs:time` | `Time` |
| `xs:duration` | `Duration` |
| `xs:base64Binary`, `xs:hexBinary` | `Binary` |
| `xs:anyURI` | `Uri` |
| `xs:ID`, `xs:IDREF` | `String` with `metadata["xsd_type"]` |

**Reference resolution:**
- `xs:complexType` definitions are collected into a `HashMap<String, IrNode>` during a first pass
- `xs:element type="..."` references are resolved in a second pass
- Circular references are detected via a `HashSet<String>` visited set and produce `IngestionError::CircularReference`

---

## 2. Parsing Layer — IR Normalization

**Location:** [`crates/data-ingestion-core/src/ir/normalizer.rs`](../crates/data-ingestion-core/src/ir/normalizer.rs)

The `IrNormalizer` post-processes the raw `IrDocument` produced by readers. It is always applied before transformation.

```rust
pub struct IrNormalizer;

impl IrNormalizer {
    pub fn normalize(doc: IrDocument) -> Result<IrDocument, IngestionError>;
}
```

### Normalization Steps (applied in order)

**Step 1 — Reference Resolution**
- Walks all `IrNode::Reference(name)` nodes
- Looks up `name` in a collected reference map
- Replaces the `Reference` node with the resolved `IrNode`
- Detects cycles via a `HashSet<String>` visited set
- Unresolved references produce `IngestionError::UnresolvedReference`

**Step 2 — Type Unification (JSON datasets only)**
- For `SourceFormat::JsonDataset`, merges type observations across sampled rows
- If a field observed as `Integer` in 90% of rows and `null` in 10%: `IrType::Integer`, `nullable = true`
- If a field observed as two different non-null types: `IrType::Union([t1, t2])`

**Step 3 — Name Normalization**
- Converts field names to `snake_case`
- Preserves original name in `IrField.metadata["original_name"]`
- Handles common separators: spaces, hyphens, dots, camelCase

**Step 4 — AllOf Flattening**
- For `IrConstraint::AllOf(constraints)`, flattens into the parent field's constraint list
- Removes duplicate constraints (by structural equality)

**Step 5 — Nullable Inference**
- If `IrType::Union` contains `Unknown` as one of its variants, removes `Unknown` from the union and sets `nullable = true`
- If `IrType::Union` has only one non-null variant after this, collapses to that single type

---

## 3. Transformation Layer — IR → DataContract

**Location:** [`crates/data-ingestion-core/src/transform/`](../crates/data-ingestion-core/src/transform/)

The transformation layer converts a normalized `IrDocument` into a `DataContract`. It is composed of four sub-modules orchestrated by `ContractBuilder`.

### 3.1 `contract_builder.rs` — Orchestrator

```rust
pub struct ContractBuilder {
    pub config: ContractBuilderConfig,
}

pub struct ContractBuilderConfig {
    /// Override for the contract name (defaults to IrDocument.source_hint filename stem)
    pub contract_name: Option<String>,
    /// Semantic version string (default: "1.0.0")
    pub contract_version: String,
    pub owner: Option<OwnerInfo>,
    pub domain: Option<String>,
    pub sla: Option<SlaInfo>,
    pub lineage: Option<LineageInfo>,
    pub tags: Vec<String>,
    /// When true, auto-detect PII fields by name pattern matching
    pub enrich_pii: bool,
    pub default_classification: DataClassification,
    /// Field names to treat as primary keys (in addition to auto-detected ones)
    pub primary_key_hints: Vec<String>,
}

impl ContractBuilder {
    pub fn new() -> Self;
    pub fn with_config(config: ContractBuilderConfig) -> Self;
    pub fn build(&self, doc: IrDocument) -> Result<DataContract, TransformError>;
}
```

**Build algorithm:**
1. Generate UUID v4 `id`
2. Set `created_at` / `updated_at` to current UTC timestamp (or accept from config)
3. Determine `name` from `config.contract_name` or `IrDocument.source_hint` filename stem
4. Walk `IrDocument.root` recursively, converting each `IrField` via `TypeResolver` and `ConstraintExtractor`
5. Apply `MetadataEnricher` to each `ContractField`
6. Infer `primary_keys` from fields named `id`, `*_id`, `uuid`, `key`, `pk` (case-insensitive) plus `config.primary_key_hints`
7. Assemble `ContractSchema`, `OwnerInfo`, `SlaInfo`, `LineageInfo` from config

### 3.2 `type_resolver.rs` — IrType → LogicalType

```rust
pub struct TypeResolver;

impl TypeResolver {
    /// Convert an IrType to a LogicalType
    pub fn resolve(ir_type: &IrType) -> LogicalType;
}
```

See the full mapping table in [`DATA_MODELS.md § 3.4`](DATA_MODELS.md#34-irtype--logicaltype-transformation).

**Design notes:**
- `Integer` maps to `Long` (64-bit) as the safe default to avoid overflow
- `Float` maps to `Double` (64-bit) as the safe default
- Nested `Object` → `Struct` is recursive; the same `ContractBuilder` logic applies to nested fields
- `Union` types are preserved as-is; the consumer decides how to handle them

### 3.3 `constraint_extractor.rs` — IrConstraint → FieldConstraint

```rust
pub struct ConstraintExtractor;

impl ConstraintExtractor {
    pub fn extract(constraints: &[IrConstraint]) -> Vec<FieldConstraint>;
}
```

**Mapping:**

| IrConstraint | FieldConstraint.constraint_type | FieldConstraint.value |
|---|---|---|
| `MinLength(n)` | `MinLength` | `n` |
| `MaxLength(n)` | `MaxLength` | `n` |
| `Pattern(s)` | `Pattern` | `s` |
| `Minimum(f)` | `Minimum` | `f` |
| `Maximum(f)` | `Maximum` | `f` |
| `ExclusiveMinimum(f)` | `ExclusiveMinimum` | `f` |
| `ExclusiveMaximum(f)` | `ExclusiveMaximum` | `f` |
| `MultipleOf(f)` | `MultipleOf` | `f` |
| `MinItems(n)` | `MinItems` | `n` |
| `MaxItems(n)` | `MaxItems` | `n` |
| `UniqueItems` | `UniqueItems` | `null` |
| `Const(v)` | `Const` | `v` |
| `AllOf(cs)` | Flattened recursively | |
| `AnyOf(cs)` | Flattened recursively | |
| `Custom(k, v)` | `Custom(k)` | `v` |

Additionally, if `IrField.nullable = false`, a `FieldConstraint { constraint_type: NotNull, .. }` is prepended.

### 3.4 `metadata_enricher.rs` — PII Detection & Classification

```rust
pub struct MetadataEnricher {
    pub enrich_pii: bool,
    pub default_classification: DataClassification,
}

impl MetadataEnricher {
    pub fn enrich(&self, field: &mut ContractField);
}
```

**PII detection** (when `enrich_pii = true`):

Scans `ContractField.name` (lowercased) for substring matches against a built-in pattern list. Matching fields get `pii = true` and `classification = Confidential` (unless already set to a more restrictive value).

**Built-in PII patterns:**

```
email, phone, mobile, cell, ssn, social_security, passport,
dob, birth_date, date_of_birth, birthdate, first_name, last_name,
full_name, given_name, surname, address, street, zip, postal,
ip_address, ip_addr, credit_card, card_number, card_num, cvv,
iban, account_number, account_num, routing_number, tax_id,
national_id, drivers_license, license_number, gender, race,
ethnicity, religion, biometric, fingerprint, face_id
```

**Classification logic:**
- If `pii = true` and no classification set → `Confidential`
- If field name contains `internal`, `private` → `Internal`
- Otherwise → `config.default_classification` (default: `Public`)

---

## 4. Serialization Layer — DataContract → Output

**Location:** [`crates/data-ingestion-core/src/output/`](../crates/data-ingestion-core/src/output/)

### 4.1 `ContractSerializer` Trait

```rust
pub trait ContractSerializer {
    fn serialize(&self, contract: &DataContract) -> Result<Vec<u8>, OutputError>;
    fn content_type(&self) -> &'static str;
    fn file_extension(&self) -> &'static str;
}

pub enum OutputFormat {
    Json,
    Yaml,
    Xml,
    Csv,
}

impl OutputFormat {
    pub fn serializer(&self) -> Box<dyn ContractSerializer>;
}
```

### 4.2 `serializer_json.rs`

- Uses `serde_json::to_string_pretty` with 2-space indentation
- `DataContract` derives `serde::Serialize`
- `LogicalType` uses `#[serde(tag = "type")]` for tagged union serialization
- Output: UTF-8 JSON

### 4.3 `serializer_yaml.rs`

- Uses `serde_yaml::to_string`
- `DataContract` derives `serde::Serialize`
- Output: YAML with block style for nested structures
- Timestamps serialized as quoted strings to prevent YAML date auto-parsing

### 4.4 `serializer_xml.rs`

- Uses `quick-xml` writer API (manual serialization, not serde)
- Manual serialization produces clean, readable XML

**XML output structure:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<DataContract id="..." version="..." name="...">
  <Description>...</Description>
  <Owner team="..." email="..." slackChannel="..."/>
  <SLA freshnessInterval="PT1H" availabilityPercent="99.9"
       maxLatencyMs="500" retentionDays="90"/>
  <Lineage sourceSystem="..." sourceTable="..."/>
  <Tags>
    <Tag>finance</Tag>
    <Tag>pii</Tag>
  </Tags>
  <Schema>
    <PrimaryKeys>
      <Key>id</Key>
    </PrimaryKeys>
    <Fields>
      <Field name="id" logicalType="Uuid" nullable="false"
             required="true" pii="false" classification="Public">
        <Description>Unique identifier</Description>
        <Constraints>
          <Constraint type="NotNull"/>
          <Constraint type="Unique"/>
        </Constraints>
        <Tags/>
      </Field>
      <Field name="email" logicalType="Email" nullable="false"
             required="true" pii="true" classification="Confidential">
        <Description>User email address</Description>
        <Constraints>
          <Constraint type="NotNull"/>
          <Constraint type="MaxLength" value="255"/>
        </Constraints>
      </Field>
    </Fields>
  </Schema>
</DataContract>
```

### 4.5 `serializer_csv.rs`

- Uses the `csv` crate
- Flattens nested `ContractField` structures to one row per field
- Nested `Struct` fields use dot-notation names (e.g., `address.street`)
- Constraints serialized as semicolon-delimited `type:value` pairs

**CSV output columns (in order):**

| Column | Type | Example |
|---|---|---|
| `name` | string | `address.street` |
| `logical_type` | string | `String` |
| `physical_type` | string | `VARCHAR(255)` |
| `nullable` | bool | `false` |
| `required` | bool | `true` |
| `pii` | bool | `false` |
| `classification` | string | `Public` |
| `description` | string | `Street address line 1` |
| `constraints` | string | `minLength:1;maxLength:255` |
| `tags` | string | `address,location` |
| `default_value` | string | `""` (JSON-encoded) |
| `examples` | string | `["123 Main St"]` (JSON array) |
| `deprecated` | bool | `false` |

**Flattening algorithm:**
```
flatten(field, prefix=""):
  if field.logical_type is Struct:
    for each nested_field in field.logical_type.fields:
      flatten(nested_field, prefix=prefix + field.name + ".")
  else:
    emit row with name = prefix + field.name
```
