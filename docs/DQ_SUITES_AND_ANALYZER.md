# Data Quality — Suites & Contract Analyzer

> **Part of:** [`DATA_QUALITY_ARCHITECTURE.md`](DATA_QUALITY_ARCHITECTURE.md)
> **Crate:** `data-quality-core`
> **Module paths:** `src/suites/`, `src/contract_analyzer/`

---

## Part A: Baseline Suites (`src/suites/`)

### A.1 Module Layout

```
src/suites/
├── mod.rs                      <- SuiteGenerator trait, BaselineSuiteSet struct
├── data_validity.rs            <- DV001-DV095   (95 tests)
├── data_completeness.rs        <- DC096-DC165   (70 tests)
├── data_consistency.rs         <- DCN166-DCN255 (90 tests)
├── data_accuracy.rs            <- DA256-DA355   (100 tests)
├── data_profile.rs             <- DP356-DP475   (120 tests)
├── data_integrity.rs           <- DI476-DI535   (60 tests)
├── data_timeliness.rs          <- DT536-DT615   (80 tests)
├── data_sensitivity.rs         <- DS616-DS665   (50 tests)
├── data_uniqueness.rs          <- DU666-DU725   (60 tests)
├── data_business_rules.rs      <- DBR726-DBR805 (80 tests)
├── data_format_consistency.rs  <- DFC806-DFC875 (70 tests)
├── data_volume_anomalies.rs    <- DVA876-DVA965 (90 tests)
├── data_dependency_checks.rs   <- DDC966-DDC1015 (50 tests)
├── cross_system_consistency.rs <- CSC1016-CSC1115 (100 tests)
├── performance_metrics.rs      <- PM1116-PM1175  (60 tests)
└── security_compliance.rs      <- SC1176-SC1328  (153 tests)
```

### A.2 `SuiteGenerator` Trait (`mod.rs`)

```rust
// src/suites/mod.rs

use crate::config::DqConfig;
use crate::expectations::mod::{ExpectationConfig, ExpectationSuite, SuiteMeta};

/// Trait implemented by all 17 baseline suite generator structs.
pub trait SuiteGenerator {
    /// Canonical suite name, e.g. "data_validity_suite"
    fn suite_name(&self) -> &str;
    /// Quality dimension category, e.g. "validity"
    fn category(&self) -> &str;
    /// Test ID prefix, e.g. "DV" for data_validity
    fn test_id_prefix(&self) -> &str;
    /// Starting test ID number, e.g. 1 for DV001
    fn test_id_start(&self) -> usize;
    /// Generates all ExpectationConfig objects for this suite.
    fn generate(&self, config: &DqConfig) -> Vec<ExpectationConfig>;

    /// Builds the complete ExpectationSuite (default implementation).
    fn build_suite(&self, config: &DqConfig) -> ExpectationSuite {
        let expectations = self.generate(config);
        let count = expectations.len();
        ExpectationSuite {
            name: self.suite_name().to_string(),
            expectations,
            meta: SuiteMeta {
                great_expectations_version: config.gx_version.clone(),
                suite_id: uuid::Uuid::new_v4().to_string(),
                contract_id: None,
                generated_at: None,
                test_count: count,
            },
        }
    }
}

/// Generates all 17 baseline suites, respecting config filters.
pub struct BaselineSuiteSet;

impl BaselineSuiteSet {
    /// Returns all 17 suites filtered by config.enabled_suites / config.disabled_suites.
    /// Suite names are matched against SuiteGenerator::suite_name().
    pub fn generate_all(config: &DqConfig) -> Vec<ExpectationSuite> {
        let generators: Vec<Box<dyn SuiteGenerator>> = vec![
            Box::new(DataValiditySuite),
            Box::new(DataCompletenessSuite),
            Box::new(DataConsistencySuite),
            Box::new(DataAccuracySuite),
            Box::new(DataProfileSuite),
            Box::new(DataIntegritySuite),
            Box::new(DataTimelinessSuite),
            Box::new(DataSensitivitySuite),
            Box::new(DataUniquenessSuite),
            Box::new(DataBusinessRulesSuite),
            Box::new(DataFormatConsistencySuite),
            Box::new(DataVolumeAnomaliesSuite),
            Box::new(DataDependencyChecksSuite),
            Box::new(CrossSystemConsistencySuite),
            Box::new(PerformanceMetricsSuite),
            Box::new(SecurityComplianceSuite),
        ];

        generators
            .into_iter()
            .filter(|g| {
                let name = g.suite_name();
                let enabled = config.enabled_suites.as_ref()
                    .map(|list| list.iter().any(|s| s == name))
                    .unwrap_or(true);
                let disabled = config.disabled_suites.iter().any(|s| s == name);
                enabled && !disabled
            })
            .map(|g| g.build_suite(config))
            .collect()
    }
}
```

### A.3 Baseline Suite Inventory

| Module | Suite Name | Test ID Range | Count | Category |
|---|---|---|---|---|
| `data_validity.rs` | `data_validity_suite` | DV001–DV095 | 95 | validity |
| `data_completeness.rs` | `data_completeness_suite` | DC096–DC165 | 70 | completeness |
| `data_consistency.rs` | `data_consistency_suite` | DCN166–DCN255 | 90 | consistency |
| `data_accuracy.rs` | `data_accuracy_suite` | DA256–DA355 | 100 | accuracy |
| `data_profile.rs` | `data_profile_suite` | DP356–DP475 | 120 | profiling |
| `data_integrity.rs` | `data_integrity_suite` | DI476–DI535 | 60 | integrity |
| `data_timeliness.rs` | `data_timeliness_suite` | DT536–DT615 | 80 | timeliness |
| `data_sensitivity.rs` | `data_sensitivity_suite` | DS616–DS665 | 50 | sensitivity |
| `data_uniqueness.rs` | `data_uniqueness_suite` | DU666–DU725 | 60 | uniqueness |
| `data_business_rules.rs` | `data_business_rules_suite` | DBR726–DBR805 | 80 | business_rules |
| `data_format_consistency.rs` | `data_format_consistency_suite` | DFC806–DFC875 | 70 | format_consistency |
| `data_volume_anomalies.rs` | `data_volume_anomalies_suite` | DVA876–DVA965 | 90 | volume_anomalies |
| `data_dependency_checks.rs` | `data_dependency_checks_suite` | DDC966–DDC1015 | 50 | dependency_checks |
| `cross_system_consistency.rs` | `cross_system_consistency_suite` | CSC1016–CSC1115 | 100 | cross_system |
| `performance_metrics.rs` | `performance_metrics_suite` | PM1116–PM1175 | 60 | performance |
| `security_compliance.rs` | `security_compliance_suite` | SC1176–SC1328 | 153 | security |
| **TOTAL** | | | **1328** | |

### A.4 Suite Module Implementation Pattern

Every suite module follows this exact pattern. `data_validity.rs` is the canonical example:

```rust
// src/suites/data_validity.rs

use crate::config::DqConfig;
use crate::expectations::mod::{ExpectationConfig, ExpectationMeta, GeneratedFrom};
use crate::suites::mod::SuiteGenerator;
use indexmap::IndexMap;
use serde_json::json;

pub struct DataValiditySuite;

impl SuiteGenerator for DataValiditySuite {
    fn suite_name(&self) -> &str { "data_validity_suite" }
    fn category(&self) -> &str { "validity" }
    fn test_id_prefix(&self) -> &str { "DV" }
    fn test_id_start(&self) -> usize { 1 }

    fn generate(&self, _config: &DqConfig) -> Vec<ExpectationConfig> {
        let mut expectations = Vec::new();

        // DV001 — column values not null (generic completeness check)
        let mut kwargs = IndexMap::new();
        kwargs.insert("column".to_string(), json!("id"));
        expectations.push(ExpectationConfig {
            expectation_type: "expect_column_values_to_not_be_null".to_string(),
            kwargs,
            meta: ExpectationMeta {
                test_id: "DV001".to_string(),
                category: "completeness".to_string(),
                suite: self.suite_name().to_string(),
                contract_field: None,
                contract_name: None,
                generated_from: GeneratedFrom::Baseline,
            },
        });

        // DV002 through DV095 follow the same pattern ...

        expectations
    }
}
```

**Critical constraint:** Each suite module must define exactly the number of tests shown in the inventory table. The test `tests/baseline_suite_counts.rs` asserts these counts via `assert_eq!` at test time.

---

## Part B: Contract Analyzer (`src/contract_analyzer/`)

### B.1 Module Layout

```
src/contract_analyzer/
├── mod.rs               <- ContractAnalyzer struct, analyze() orchestration
├── field_analyzer.rs    <- per-field test generation
├── schema_analyzer.rs   <- table-level test generation
├── pii_analyzer.rs      <- PII/PHI/PCI sensitivity tests
└── constraint_analyzer.rs <- FieldConstraint -> GX expectations
```

### B.2 `ContractAnalyzer` (`mod.rs`)

```rust
// src/contract_analyzer/mod.rs

use data_ingestion_core::DataContract;
use crate::config::DqConfig;
use crate::expectations::mod::ExpectationSuite;

pub struct ContractAnalyzer<'a> {
    contract: &'a DataContract,
    config: &'a DqConfig,
}

impl<'a> ContractAnalyzer<'a> {
    pub fn new(contract: &'a DataContract, config: &'a DqConfig) -> Self {
        Self { contract, config }
    }

    /// Runs all sub-analyzers and returns the contract-specific suites.
    /// Always produces: schema_suite, field_suite, constraints_suite.
    /// Conditionally produces: pii_suite (only if any field has pii=true).
    pub fn analyze(&self) -> Vec<ExpectationSuite> {
        let mut suites = Vec::new();
        suites.push(schema_analyzer::build_schema_suite(self.contract, self.config));
        suites.push(field_analyzer::build_field_suite(self.contract, self.config));
        suites.push(constraint_analyzer::build_constraints_suite(self.contract, self.config));
        if self.contract.fields.iter().any(|f| f.pii) {
            suites.push(pii_analyzer::build_pii_suite(self.contract, self.config));
        }
        suites
    }
}
```

### B.3 `schema_analyzer.rs` — Table-Level Rules

**Output suite name:** `<contract_name>_schema_suite`
**Test ID format:** `<ABBREV>-SCH-NNN` where `ABBREV` = first 3 uppercase chars of `contract.name`

| Rule | Condition | GX Expectation | kwargs |
|---|---|---|---|
| Column set match | Always | `expect_table_columns_to_match_set` | `column_set`: all field names, `exact_match`: false |
| Column count | Always | `expect_table_column_count_to_equal` | `value`: `fields.len()` |
| Required columns ordered | Any `required=true` fields exist | `expect_table_columns_to_match_ordered_list` | `column_list`: names of required fields in declaration order |
| Completeness threshold | `quality.completeness_threshold` is `Some(t)` | `expect_column_values_to_not_be_null` with `mostly` | `mostly`: `t`, applied to each required field individually |
| Freshness SLA | `sla.freshness_hours` is `Some(h)` | `expect_column_values_to_be_dateutil_parseable` | applied to first `DateTime`/`Timestamp` field found; `meta` includes `freshness_hours: h` |

### B.4 `field_analyzer.rs` — Per-Field Rules

**Output suite name:** `<contract_name>_field_suite`
**Test ID format:** `<ABBREV>-FLD-NNN` (NNN is a zero-padded sequential counter across all fields)

#### Nullability Rules

| Condition | GX Expectation | kwargs | Dedup |
|---|---|---|---|
| `nullable = false` | `expect_column_values_to_not_be_null` | `column` | Yes — one per column |
| `required = true` | `expect_column_values_to_not_be_null` | `column` | Yes — one per column |
| `primary_key = true` | `expect_column_values_to_not_be_null` | `column` | Yes — one per column |

> **Dedup rule:** If multiple conditions trigger `expect_column_values_to_not_be_null` for the same column, emit only one expectation. Same rule applies to `expect_column_values_to_be_unique`.

#### Uniqueness Rules

| Condition | GX Expectation | kwargs |
|---|---|---|
| `unique = true` | `expect_column_values_to_be_unique` | `column` |
| `primary_key = true` | `expect_column_values_to_be_unique` | `column` |

#### Type-Based Rules

| `LogicalType` | GX Expectation | kwargs |
|---|---|---|
| `Email` | `expect_column_values_to_be_valid_email` (custom) | `column` |
| `Uuid` | `expect_column_values_to_be_valid_uuid` (custom) | `column` |
| `Uri` | `expect_column_values_to_be_valid_uri` (custom) | `column` |
| `Date` | `expect_column_values_to_be_dateutil_parseable` | `column` |
| `DateTime` | `expect_column_values_to_be_dateutil_parseable` | `column` |
| `Timestamp` | `expect_column_values_to_be_dateutil_parseable` | `column` |
| `Integer` | `expect_column_values_to_be_of_type` | `column`, `type_`: `"int"` |
| `Long` | `expect_column_values_to_be_of_type` | `column`, `type_`: `"int"` |
| `Float` | `expect_column_values_to_be_of_type` | `column`, `type_`: `"float"` |
| `Double` | `expect_column_values_to_be_of_type` | `column`, `type_`: `"float"` |
| `Decimal { precision, scale }` | `expect_column_values_to_be_valid_decimal_precision` (custom) | `column`, `precision`, `scale` |
| `Boolean` | `expect_column_values_to_be_in_set` | `column`, `value_set`: `[true, false]` |
| `Json` | `expect_column_values_to_be_valid_json` (custom) | `column` |
| `String` | No type expectation (too broad) | — |
| `Binary` | No type expectation | — |
| `Array { .. }` | No type expectation (nested) | — |
| `Struct { .. }` | No type expectation (nested) | — |
| `Unknown` | No type expectation | — |

#### Foreign Key Rules

| Condition | GX Expectation | kwargs | meta |
|---|---|---|---|
| `foreign_key = Some(fk)` | `expect_column_values_to_be_in_set` | `column`, `value_set`: `[]` (empty) | `fk_table`: `fk.table`, `fk_column`: `fk.column` |

> `value_set` is intentionally empty in the generated JSON. The GX runner populates it at runtime from the referenced table. The FK metadata in `meta` provides the reference information for the runner.

### B.5 `pii_analyzer.rs` — PII/Sensitivity Rules

**Output suite name:** `<contract_name>_pii_suite`
**Generated only when:** `contract.fields.iter().any(|f| f.pii)`
**Test ID format:** `<ABBREV>-PII-NNN`

**Field name matching** uses case-insensitive substring matching (e.g., `"phone_number"` matches the pattern `"phone"`).

| Condition | GX Expectation | kwargs |
|---|---|---|
| `pii = true` | `expect_column_values_to_be_masked_pii` (custom) | `column` |
| `pii = true` AND `LogicalType::Email` | `expect_column_values_to_not_contain_pii` (custom) | `column` |
| `pii = true` AND name contains `ssn` or `social_security` | `expect_column_values_to_be_valid_ssn` (custom) | `column` |
| `pii = true` AND name contains `phone`, `mobile`, or `tel` | `expect_column_values_to_be_valid_phone_number` (custom) | `column` |
| `pii = true` AND name contains `credit_card`, `card_number`, or `cc_num` | `expect_column_values_to_be_valid_credit_card` (custom) | `column` |
| `classification = Restricted` | `expect_column_values_to_be_encrypted` (custom) | `column` |
| `classification = Confidential` | `expect_column_values_to_be_masked_pii` (custom) | `column` |

### B.6 `constraint_analyzer.rs` — FieldConstraint Mapping

**Output suite name:** `<contract_name>_constraints_suite`
**Test ID format:** `<ABBREV>-CON-NNN`

| `FieldConstraint` variant | GX Expectation | kwargs |
|---|---|---|
| `MinLength(n)` alone | `expect_column_value_lengths_to_be_between` | `column`, `min_value`: n, `max_value`: null |
| `MaxLength(n)` alone | `expect_column_value_lengths_to_be_between` | `column`, `min_value`: null, `max_value`: n |
| `MinLength(n)` + `MaxLength(m)` on same field | `expect_column_value_lengths_to_be_between` | `column`, `min_value`: n, `max_value`: m (merged into one) |
| `Pattern(p)` | `expect_column_values_to_match_regex` | `column`, `regex`: p |
| `Minimum(n)` alone | `expect_column_values_to_be_between` | `column`, `min_value`: n, `max_value`: null |
| `Maximum(n)` alone | `expect_column_values_to_be_between` | `column`, `min_value`: null, `max_value`: n |
| `Minimum(n)` + `Maximum(m)` on same field | `expect_column_values_to_be_between` | `column`, `min_value`: n, `max_value`: m (merged into one) |
| `AllowedValues(v)` | `expect_column_values_to_be_in_set` | `column`, `value_set`: v |
| `NotNull` | `expect_column_values_to_not_be_null` | `column` |
| `Unique` | `expect_column_values_to_be_unique` | `column` |

**Merging rule:** The constraint analyzer scans all constraints for a field before emitting expectations. When both `Minimum` and `Maximum` (or both `MinLength` and `MaxLength`) are present for the same field, they are merged into a single `between` expectation. This avoids redundant GX expectations and matches GX best practices.

### B.7 Contract-Specific Suite Naming

| Suite | Name Pattern | Example |
|---|---|---|
| Schema suite | `<contract_name>_schema_suite` | `order_schema_suite` |
| Field suite | `<contract_name>_field_suite` | `order_field_suite` |
| PII suite | `<contract_name>_pii_suite` | `order_pii_suite` |
| Constraints suite | `<contract_name>_constraints_suite` | `order_constraints_suite` |

**Test ID abbreviation:** First 3 uppercase characters of `contract.name`. Examples:
- `order` → `ORD`
- `customer` → `CUS`
- `patient_record` → `PAT`
- `x` → `X` (use full name if shorter than 3 chars)
