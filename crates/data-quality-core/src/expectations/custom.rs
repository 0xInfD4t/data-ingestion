use indexmap::IndexMap;
use serde_json::{json, Value};

use super::{Expectation, ExpectationMeta, GeneratedFrom};

// ── 19 Custom Expectations ────────────────────────────────────────────────────

pub struct ExpectColumnValuesToBeValidEmail {
    pub column: String,
}
impl Expectation for ExpectColumnValuesToBeValidEmail {
    fn expectation_type(&self) -> &str { "expect_column_values_to_be_valid_email" }
    fn build_kwargs(&self) -> IndexMap<String, Value> {
        let mut m = IndexMap::new(); m.insert("column".to_string(), json!(self.column)); m
    }
    fn build_meta(&self, test_id: &str, category: &str, suite: &str) -> ExpectationMeta {
        ExpectationMeta { test_id: test_id.to_string(), category: category.to_string(),
            suite: suite.to_string(), contract_field: Some(self.column.clone()),
            contract_name: None, generated_from: GeneratedFrom::Baseline }
    }
}

pub struct ExpectColumnValuesToBeValidUuid {
    pub column: String,
}
impl Expectation for ExpectColumnValuesToBeValidUuid {
    fn expectation_type(&self) -> &str { "expect_column_values_to_be_valid_uuid" }
    fn build_kwargs(&self) -> IndexMap<String, Value> {
        let mut m = IndexMap::new(); m.insert("column".to_string(), json!(self.column)); m
    }
    fn build_meta(&self, test_id: &str, category: &str, suite: &str) -> ExpectationMeta {
        ExpectationMeta { test_id: test_id.to_string(), category: category.to_string(),
            suite: suite.to_string(), contract_field: Some(self.column.clone()),
            contract_name: None, generated_from: GeneratedFrom::Baseline }
    }
}

pub struct ExpectColumnValuesToBeValidUri {
    pub column: String,
}
impl Expectation for ExpectColumnValuesToBeValidUri {
    fn expectation_type(&self) -> &str { "expect_column_values_to_be_valid_uri" }
    fn build_kwargs(&self) -> IndexMap<String, Value> {
        let mut m = IndexMap::new(); m.insert("column".to_string(), json!(self.column)); m
    }
    fn build_meta(&self, test_id: &str, category: &str, suite: &str) -> ExpectationMeta {
        ExpectationMeta { test_id: test_id.to_string(), category: category.to_string(),
            suite: suite.to_string(), contract_field: Some(self.column.clone()),
            contract_name: None, generated_from: GeneratedFrom::Baseline }
    }
}

pub struct ExpectColumnValuesToBeValidIso8601 {
    pub column: String,
}
impl Expectation for ExpectColumnValuesToBeValidIso8601 {
    fn expectation_type(&self) -> &str { "expect_column_values_to_be_valid_iso8601" }
    fn build_kwargs(&self) -> IndexMap<String, Value> {
        let mut m = IndexMap::new(); m.insert("column".to_string(), json!(self.column)); m
    }
    fn build_meta(&self, test_id: &str, category: &str, suite: &str) -> ExpectationMeta {
        ExpectationMeta { test_id: test_id.to_string(), category: category.to_string(),
            suite: suite.to_string(), contract_field: Some(self.column.clone()),
            contract_name: None, generated_from: GeneratedFrom::Baseline }
    }
}

pub struct ExpectColumnValuesToBeValidJson {
    pub column: String,
}
impl Expectation for ExpectColumnValuesToBeValidJson {
    fn expectation_type(&self) -> &str { "expect_column_values_to_be_valid_json" }
    fn build_kwargs(&self) -> IndexMap<String, Value> {
        let mut m = IndexMap::new(); m.insert("column".to_string(), json!(self.column)); m
    }
    fn build_meta(&self, test_id: &str, category: &str, suite: &str) -> ExpectationMeta {
        ExpectationMeta { test_id: test_id.to_string(), category: category.to_string(),
            suite: suite.to_string(), contract_field: Some(self.column.clone()),
            contract_name: None, generated_from: GeneratedFrom::Baseline }
    }
}

pub struct ExpectColumnValuesToBeMaskedPii {
    pub column: String,
}
impl Expectation for ExpectColumnValuesToBeMaskedPii {
    fn expectation_type(&self) -> &str { "expect_column_values_to_be_masked_pii" }
    fn build_kwargs(&self) -> IndexMap<String, Value> {
        let mut m = IndexMap::new(); m.insert("column".to_string(), json!(self.column)); m
    }
    fn build_meta(&self, test_id: &str, category: &str, suite: &str) -> ExpectationMeta {
        ExpectationMeta { test_id: test_id.to_string(), category: category.to_string(),
            suite: suite.to_string(), contract_field: Some(self.column.clone()),
            contract_name: None, generated_from: GeneratedFrom::Baseline }
    }
}

pub struct ExpectColumnValuesToBeEncrypted {
    pub column: String,
}
impl Expectation for ExpectColumnValuesToBeEncrypted {
    fn expectation_type(&self) -> &str { "expect_column_values_to_be_encrypted" }
    fn build_kwargs(&self) -> IndexMap<String, Value> {
        let mut m = IndexMap::new(); m.insert("column".to_string(), json!(self.column)); m
    }
    fn build_meta(&self, test_id: &str, category: &str, suite: &str) -> ExpectationMeta {
        ExpectationMeta { test_id: test_id.to_string(), category: category.to_string(),
            suite: suite.to_string(), contract_field: Some(self.column.clone()),
            contract_name: None, generated_from: GeneratedFrom::Baseline }
    }
}

pub struct ExpectColumnValuesToNotContainPii {
    pub column: String,
}
impl Expectation for ExpectColumnValuesToNotContainPii {
    fn expectation_type(&self) -> &str { "expect_column_values_to_not_contain_pii" }
    fn build_kwargs(&self) -> IndexMap<String, Value> {
        let mut m = IndexMap::new(); m.insert("column".to_string(), json!(self.column)); m
    }
    fn build_meta(&self, test_id: &str, category: &str, suite: &str) -> ExpectationMeta {
        ExpectationMeta { test_id: test_id.to_string(), category: category.to_string(),
            suite: suite.to_string(), contract_field: Some(self.column.clone()),
            contract_name: None, generated_from: GeneratedFrom::Baseline }
    }
}

pub struct ExpectColumnValuesToBeValidPhoneNumber {
    pub column: String,
}
impl Expectation for ExpectColumnValuesToBeValidPhoneNumber {
    fn expectation_type(&self) -> &str { "expect_column_values_to_be_valid_phone_number" }
    fn build_kwargs(&self) -> IndexMap<String, Value> {
        let mut m = IndexMap::new(); m.insert("column".to_string(), json!(self.column)); m
    }
    fn build_meta(&self, test_id: &str, category: &str, suite: &str) -> ExpectationMeta {
        ExpectationMeta { test_id: test_id.to_string(), category: category.to_string(),
            suite: suite.to_string(), contract_field: Some(self.column.clone()),
            contract_name: None, generated_from: GeneratedFrom::Baseline }
    }
}

pub struct ExpectColumnValuesToBeValidPostalCode {
    pub column: String,
}
impl Expectation for ExpectColumnValuesToBeValidPostalCode {
    fn expectation_type(&self) -> &str { "expect_column_values_to_be_valid_postal_code" }
    fn build_kwargs(&self) -> IndexMap<String, Value> {
        let mut m = IndexMap::new(); m.insert("column".to_string(), json!(self.column)); m
    }
    fn build_meta(&self, test_id: &str, category: &str, suite: &str) -> ExpectationMeta {
        ExpectationMeta { test_id: test_id.to_string(), category: category.to_string(),
            suite: suite.to_string(), contract_field: Some(self.column.clone()),
            contract_name: None, generated_from: GeneratedFrom::Baseline }
    }
}

pub struct ExpectColumnValuesToBeValidCreditCard {
    pub column: String,
}
impl Expectation for ExpectColumnValuesToBeValidCreditCard {
    fn expectation_type(&self) -> &str { "expect_column_values_to_be_valid_credit_card" }
    fn build_kwargs(&self) -> IndexMap<String, Value> {
        let mut m = IndexMap::new(); m.insert("column".to_string(), json!(self.column)); m
    }
    fn build_meta(&self, test_id: &str, category: &str, suite: &str) -> ExpectationMeta {
        ExpectationMeta { test_id: test_id.to_string(), category: category.to_string(),
            suite: suite.to_string(), contract_field: Some(self.column.clone()),
            contract_name: None, generated_from: GeneratedFrom::Baseline }
    }
}

pub struct ExpectColumnValuesToBeValidSsn {
    pub column: String,
}
impl Expectation for ExpectColumnValuesToBeValidSsn {
    fn expectation_type(&self) -> &str { "expect_column_values_to_be_valid_ssn" }
    fn build_kwargs(&self) -> IndexMap<String, Value> {
        let mut m = IndexMap::new(); m.insert("column".to_string(), json!(self.column)); m
    }
    fn build_meta(&self, test_id: &str, category: &str, suite: &str) -> ExpectationMeta {
        ExpectationMeta { test_id: test_id.to_string(), category: category.to_string(),
            suite: suite.to_string(), contract_field: Some(self.column.clone()),
            contract_name: None, generated_from: GeneratedFrom::Baseline }
    }
}

pub struct ExpectColumnValuesToBeWithinZScore {
    pub column: String,
    pub z_score_max: f64,
}
impl Expectation for ExpectColumnValuesToBeWithinZScore {
    fn expectation_type(&self) -> &str { "expect_column_values_to_be_within_z_score" }
    fn build_kwargs(&self) -> IndexMap<String, Value> {
        let mut m = IndexMap::new();
        m.insert("column".to_string(), json!(self.column));
        m.insert("z_score_max".to_string(), json!(self.z_score_max));
        m
    }
    fn build_meta(&self, test_id: &str, category: &str, suite: &str) -> ExpectationMeta {
        ExpectationMeta { test_id: test_id.to_string(), category: category.to_string(),
            suite: suite.to_string(), contract_field: Some(self.column.clone()),
            contract_name: None, generated_from: GeneratedFrom::Baseline }
    }
}

pub struct ExpectColumnValuesToBeWithinIqr {
    pub column: String,
}
impl Expectation for ExpectColumnValuesToBeWithinIqr {
    fn expectation_type(&self) -> &str { "expect_column_values_to_be_within_iqr" }
    fn build_kwargs(&self) -> IndexMap<String, Value> {
        let mut m = IndexMap::new(); m.insert("column".to_string(), json!(self.column)); m
    }
    fn build_meta(&self, test_id: &str, category: &str, suite: &str) -> ExpectationMeta {
        ExpectationMeta { test_id: test_id.to_string(), category: category.to_string(),
            suite: suite.to_string(), contract_field: Some(self.column.clone()),
            contract_name: None, generated_from: GeneratedFrom::Baseline }
    }
}

pub struct ExpectColumnValuesToHaveConsistentCasing {
    pub column: String,
}
impl Expectation for ExpectColumnValuesToHaveConsistentCasing {
    fn expectation_type(&self) -> &str { "expect_column_values_to_have_consistent_casing" }
    fn build_kwargs(&self) -> IndexMap<String, Value> {
        let mut m = IndexMap::new(); m.insert("column".to_string(), json!(self.column)); m
    }
    fn build_meta(&self, test_id: &str, category: &str, suite: &str) -> ExpectationMeta {
        ExpectationMeta { test_id: test_id.to_string(), category: category.to_string(),
            suite: suite.to_string(), contract_field: Some(self.column.clone()),
            contract_name: None, generated_from: GeneratedFrom::Baseline }
    }
}

pub struct ExpectColumnValuesToHaveNoLeadingTrailingWhitespace {
    pub column: String,
}
impl Expectation for ExpectColumnValuesToHaveNoLeadingTrailingWhitespace {
    fn expectation_type(&self) -> &str { "expect_column_values_to_have_no_leading_trailing_whitespace" }
    fn build_kwargs(&self) -> IndexMap<String, Value> {
        let mut m = IndexMap::new(); m.insert("column".to_string(), json!(self.column)); m
    }
    fn build_meta(&self, test_id: &str, category: &str, suite: &str) -> ExpectationMeta {
        ExpectationMeta { test_id: test_id.to_string(), category: category.to_string(),
            suite: suite.to_string(), contract_field: Some(self.column.clone()),
            contract_name: None, generated_from: GeneratedFrom::Baseline }
    }
}

pub struct ExpectColumnValuesToBeValidDecimalPrecision {
    pub column: String,
    pub precision: u8,
    pub scale: u8,
}
impl Expectation for ExpectColumnValuesToBeValidDecimalPrecision {
    fn expectation_type(&self) -> &str { "expect_column_values_to_be_valid_decimal_precision" }
    fn build_kwargs(&self) -> IndexMap<String, Value> {
        let mut m = IndexMap::new();
        m.insert("column".to_string(), json!(self.column));
        m.insert("precision".to_string(), json!(self.precision));
        m.insert("scale".to_string(), json!(self.scale));
        m
    }
    fn build_meta(&self, test_id: &str, category: &str, suite: &str) -> ExpectationMeta {
        ExpectationMeta { test_id: test_id.to_string(), category: category.to_string(),
            suite: suite.to_string(), contract_field: Some(self.column.clone()),
            contract_name: None, generated_from: GeneratedFrom::Baseline }
    }
}

pub struct ExpectTableToHaveNoOrphanedForeignKeys {
    pub column: String,
    pub ref_table: String,
    pub ref_column: String,
}
impl Expectation for ExpectTableToHaveNoOrphanedForeignKeys {
    fn expectation_type(&self) -> &str { "expect_table_to_have_no_orphaned_foreign_keys" }
    fn build_kwargs(&self) -> IndexMap<String, Value> {
        let mut m = IndexMap::new();
        m.insert("column".to_string(), json!(self.column));
        m.insert("ref_table".to_string(), json!(self.ref_table));
        m.insert("ref_column".to_string(), json!(self.ref_column));
        m
    }
    fn build_meta(&self, test_id: &str, category: &str, suite: &str) -> ExpectationMeta {
        ExpectationMeta { test_id: test_id.to_string(), category: category.to_string(),
            suite: suite.to_string(), contract_field: Some(self.column.clone()),
            contract_name: None, generated_from: GeneratedFrom::Baseline }
    }
}

pub struct ExpectColumnValuesToBeWithinHistoricalRange {
    pub column: String,
    pub historical_mean: f64,
    pub historical_std: f64,
    pub z_score_max: f64,
}
impl Expectation for ExpectColumnValuesToBeWithinHistoricalRange {
    fn expectation_type(&self) -> &str { "expect_column_values_to_be_within_historical_range" }
    fn build_kwargs(&self) -> IndexMap<String, Value> {
        let mut m = IndexMap::new();
        m.insert("column".to_string(), json!(self.column));
        m.insert("historical_mean".to_string(), json!(self.historical_mean));
        m.insert("historical_std".to_string(), json!(self.historical_std));
        m.insert("z_score_max".to_string(), json!(self.z_score_max));
        m
    }
    fn build_meta(&self, test_id: &str, category: &str, suite: &str) -> ExpectationMeta {
        ExpectationMeta { test_id: test_id.to_string(), category: category.to_string(),
            suite: suite.to_string(), contract_field: Some(self.column.clone()),
            contract_name: None, generated_from: GeneratedFrom::Baseline }
    }
}
