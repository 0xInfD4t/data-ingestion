# Data Quality — Expectations Module

> **Part of:** [`DATA_QUALITY_ARCHITECTURE.md`](DATA_QUALITY_ARCHITECTURE.md)
> **Crate:** `data-quality-core`
> **Module path:** `src/expectations/`

---

## 1. Module Layout

```
src/expectations/
├── mod.rs          <- Expectation trait, core structs (see DQ_DATA_MODELS.md)
├── types.rs        <- 19+ built-in GX expectation type structs
├── custom.rs       <- 19 custom expectations from data-quality-service
└── serializer.rs   <- ExpectationSuite -> GX-compatible JSON/YAML bytes
```

---

## 2. `Expectation` Trait (`mod.rs`)

All expectation builder structs implement this trait. The `build()` method has a default implementation that composes the other three methods.

```rust
// src/expectations/mod.rs

use indexmap::IndexMap;
use serde_json::Value;

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
```

---

## 3. Built-in GX Expectation Types (`types.rs`)

Each struct holds the typed parameters needed to build the `kwargs` map and implements the `Expectation` trait.

### 3.1 Column-Level Expectations

| Struct | `expectation_type` | Fields |
|---|---|---|
| `ExpectColumnValuesToNotBeNull` | `expect_column_values_to_not_be_null` | `column: String`, `mostly: Option<f64>` |
| `ExpectColumnValuesToBeUnique` | `expect_column_values_to_be_unique` | `column: String` |
| `ExpectColumnValuesToBeOfType` | `expect_column_values_to_be_of_type` | `column: String`, `type_: String` |
| `ExpectColumnValuesToBeInSet` | `expect_column_values_to_be_in_set` | `column: String`, `value_set: Vec<Value>` |
| `ExpectColumnValuesToMatchRegex` | `expect_column_values_to_match_regex` | `column: String`, `regex: String` |
| `ExpectColumnValuesToNotMatchRegex` | `expect_column_values_to_not_match_regex` | `column: String`, `regex: String` |
| `ExpectColumnValueLengthsToBeBetween` | `expect_column_value_lengths_to_be_between` | `column: String`, `min_value: Option<usize>`, `max_value: Option<usize>` |
| `ExpectColumnValuesToBeBetween` | `expect_column_values_to_be_between` | `column: String`, `min_value: Option<f64>`, `max_value: Option<f64>` |
| `ExpectColumnValuesToBeDateutilParseable` | `expect_column_values_to_be_dateutil_parseable` | `column: String` |
| `ExpectColumnValuesToMatchStrftimeFormat` | `expect_column_values_to_match_strftime_format` | `column: String`, `strftime_format: String` |
| `ExpectColumnMeanToBeBetween` | `expect_column_mean_to_be_between` | `column: String`, `min_value: Option<f64>`, `max_value: Option<f64>` |
| `ExpectColumnMedianToBeBetween` | `expect_column_median_to_be_between` | `column: String`, `min_value: Option<f64>`, `max_value: Option<f64>` |
| `ExpectColumnStdevToBeBetween` | `expect_column_stdev_to_be_between` | `column: String`, `min_value: Option<f64>`, `max_value: Option<f64>` |
| `ExpectColumnProportionOfUniqueValuesToBeBetween` | `expect_column_proportion_of_unique_values_to_be_between` | `column: String`, `min_value: Option<f64>`, `max_value: Option<f64>` |

### 3.2 Table-Level Expectations

| Struct | `expectation_type` | Fields |
|---|---|---|
| `ExpectTableColumnsToMatchSet` | `expect_table_columns_to_match_set` | `column_set: Vec<String>`, `exact_match: bool` |
| `ExpectTableColumnsToMatchOrderedList` | `expect_table_columns_to_match_ordered_list` | `column_list: Vec<String>` |
| `ExpectTableColumnCountToEqual` | `expect_table_column_count_to_equal` | `value: usize` |
| `ExpectTableRowCountToBeBetween` | `expect_table_row_count_to_be_between` | `min_value: Option<usize>`, `max_value: Option<usize>` |
| `ExpectTableRowCountToEqual` | `expect_table_row_count_to_equal` | `value: usize` |

### 3.3 Example Implementation

```rust
// src/expectations/types.rs

pub struct ExpectColumnValuesToNotBeNull {
    pub column: String,
    pub mostly: Option<f64>,
}

impl Expectation for ExpectColumnValuesToNotBeNull {
    fn expectation_type(&self) -> &str {
        "expect_column_values_to_not_be_null"
    }

    fn build_kwargs(&self) -> IndexMap<String, Value> {
        let mut kwargs = IndexMap::new();
        kwargs.insert("column".to_string(), json!(self.column));
        if let Some(mostly) = self.mostly {
            kwargs.insert("mostly".to_string(), json!(mostly));
        }
        kwargs
    }

    fn build_meta(&self, test_id: &str, category: &str, suite: &str) -> ExpectationMeta {
        ExpectationMeta {
            test_id: test_id.to_string(),
            category: category.to_string(),
            suite: suite.to_string(),
            contract_field: Some(self.column.clone()),
            contract_name: None,
            generated_from: GeneratedFrom::Baseline,
        }
    }
}
```

---

## 4. Custom Expectations (`custom.rs`)

The 19 custom expectations from the `data-quality-service` are represented as structs. Each maps to a GX custom expectation registered in the data-quality-service plugin registry.

| Struct | `expectation_type` | Fields | Purpose |
|---|---|---|---|
| `ExpectColumnValuesToBeValidEmail` | `expect_column_values_to_be_valid_email` | `column: String` | RFC 5322 email validation |
| `ExpectColumnValuesToBeValidUuid` | `expect_column_values_to_be_valid_uuid` | `column: String` | UUID v4 format |
| `ExpectColumnValuesToBeValidUri` | `expect_column_values_to_be_valid_uri` | `column: String` | URI/URL format |
| `ExpectColumnValuesToBeValidIso8601` | `expect_column_values_to_be_valid_iso8601` | `column: String` | ISO 8601 date/datetime |
| `ExpectColumnValuesToBeValidJson` | `expect_column_values_to_be_valid_json` | `column: String` | Valid JSON string |
| `ExpectColumnValuesToBeMaskedPii` | `expect_column_values_to_be_masked_pii` | `column: String` | PII masking/tokenization check |
| `ExpectColumnValuesToBeEncrypted` | `expect_column_values_to_be_encrypted` | `column: String` | Encrypted field check |
| `ExpectColumnValuesToNotContainPii` | `expect_column_values_to_not_contain_pii` | `column: String` | No raw PII in non-PII fields |
| `ExpectColumnValuesToBeValidPhoneNumber` | `expect_column_values_to_be_valid_phone_number` | `column: String` | E.164 phone format |
| `ExpectColumnValuesToBeValidPostalCode` | `expect_column_values_to_be_valid_postal_code` | `column: String` | Postal code format |
| `ExpectColumnValuesToBeValidCreditCard` | `expect_column_values_to_be_valid_credit_card` | `column: String` | Luhn-valid card number |
| `ExpectColumnValuesToBeValidSsn` | `expect_column_values_to_be_valid_ssn` | `column: String` | SSN format (masked) |
| `ExpectColumnValuesToBeWithinZScore` | `expect_column_values_to_be_within_z_score` | `column: String`, `z_score_max: f64` | Statistical outlier check |
| `ExpectColumnValuesToBeWithinIqr` | `expect_column_values_to_be_within_iqr` | `column: String` | IQR outlier check |
| `ExpectColumnValuesToHaveConsistentCasing` | `expect_column_values_to_have_consistent_casing` | `column: String` | Casing consistency |
| `ExpectColumnValuesToHaveNoLeadingTrailingWhitespace` | `expect_column_values_to_have_no_leading_trailing_whitespace` | `column: String` | Whitespace check |
| `ExpectColumnValuesToBeValidDecimalPrecision` | `expect_column_values_to_be_valid_decimal_precision` | `column: String`, `precision: u8`, `scale: u8` | Decimal precision/scale |
| `ExpectTableToHaveNoOrphanedForeignKeys` | `expect_table_to_have_no_orphaned_foreign_keys` | `column: String`, `ref_table: String`, `ref_column: String` | FK referential integrity |
| `ExpectColumnValuesToBeWithinHistoricalRange` | `expect_column_values_to_be_within_historical_range` | `column: String`, `historical_mean: f64`, `historical_std: f64`, `z_score_max: f64` | Historical bounds check |

---

## 5. Serializer (`serializer.rs`)

Converts an `ExpectationSuite` to GX-compatible bytes.

```rust
// src/expectations/serializer.rs

/// Serialize an ExpectationSuite to GX 1.x-compatible JSON bytes.
pub fn to_gx_json(suite: &ExpectationSuite) -> Result<Vec<u8>, DqError> {
    serde_json::to_vec_pretty(suite)
        .map_err(|e| DqError::SerializationError(e.to_string()))
}

/// Serialize an ExpectationSuite to YAML bytes.
pub fn to_gx_yaml(suite: &ExpectationSuite) -> Result<Vec<u8>, DqError> {
    serde_yaml::to_string(suite)
        .map(|s| s.into_bytes())
        .map_err(|e| DqError::SerializationError(e.to_string()))
}
```

### Serialization Rules

| Rule | Detail |
|---|---|
| `expectation_type` key | Serialized as `"type"` via `#[serde(rename = "type")]` — GX 1.x requirement |
| `kwargs` ordering | Uses `IndexMap<String, Value>` — keys appear in insertion order in JSON |
| `Option<String>` fields | Serialize as `null` (not omitted) for GX compatibility |
| `GeneratedFrom::Baseline` | Serializes as the string `"baseline"` |
| `GeneratedFrom::ContractSpecific` | Serializes as `{"contract_specific": {"reason": "..."}}` |
| Pretty-printing | `serde_json::to_vec_pretty` — human-readable output matching GX convention |
