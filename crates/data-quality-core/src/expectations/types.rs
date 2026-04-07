use indexmap::IndexMap;
use serde_json::{json, Value};

use super::{Expectation, ExpectationMeta, GeneratedFrom};

// ── Column-Level Expectations ─────────────────────────────────────────────────

pub struct ExpectColumnValuesToNotBeNull {
    pub column: String,
    pub mostly: Option<f64>,
}

impl Expectation for ExpectColumnValuesToNotBeNull {
    fn expectation_type(&self) -> &str {
        "expect_column_values_to_not_be_null"
    }
    fn build_kwargs(&self) -> IndexMap<String, Value> {
        let mut m = IndexMap::new();
        m.insert("column".to_string(), json!(self.column));
        if let Some(mostly) = self.mostly {
            m.insert("mostly".to_string(), json!(mostly));
        }
        m
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

pub struct ExpectColumnValuesToBeUnique {
    pub column: String,
}

impl Expectation for ExpectColumnValuesToBeUnique {
    fn expectation_type(&self) -> &str {
        "expect_column_values_to_be_unique"
    }
    fn build_kwargs(&self) -> IndexMap<String, Value> {
        let mut m = IndexMap::new();
        m.insert("column".to_string(), json!(self.column));
        m
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

pub struct ExpectColumnValuesToBeOfType {
    pub column: String,
    pub type_: String,
}

impl Expectation for ExpectColumnValuesToBeOfType {
    fn expectation_type(&self) -> &str {
        "expect_column_values_to_be_of_type"
    }
    fn build_kwargs(&self) -> IndexMap<String, Value> {
        let mut m = IndexMap::new();
        m.insert("column".to_string(), json!(self.column));
        m.insert("type_".to_string(), json!(self.type_));
        m
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

pub struct ExpectColumnValuesToBeInSet {
    pub column: String,
    pub value_set: Vec<Value>,
}

impl Expectation for ExpectColumnValuesToBeInSet {
    fn expectation_type(&self) -> &str {
        "expect_column_values_to_be_in_set"
    }
    fn build_kwargs(&self) -> IndexMap<String, Value> {
        let mut m = IndexMap::new();
        m.insert("column".to_string(), json!(self.column));
        m.insert("value_set".to_string(), json!(self.value_set));
        m
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

pub struct ExpectColumnValuesToMatchRegex {
    pub column: String,
    pub regex: String,
}

impl Expectation for ExpectColumnValuesToMatchRegex {
    fn expectation_type(&self) -> &str {
        "expect_column_values_to_match_regex"
    }
    fn build_kwargs(&self) -> IndexMap<String, Value> {
        let mut m = IndexMap::new();
        m.insert("column".to_string(), json!(self.column));
        m.insert("regex".to_string(), json!(self.regex));
        m
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

pub struct ExpectColumnValuesToNotMatchRegex {
    pub column: String,
    pub regex: String,
}

impl Expectation for ExpectColumnValuesToNotMatchRegex {
    fn expectation_type(&self) -> &str {
        "expect_column_values_to_not_match_regex"
    }
    fn build_kwargs(&self) -> IndexMap<String, Value> {
        let mut m = IndexMap::new();
        m.insert("column".to_string(), json!(self.column));
        m.insert("regex".to_string(), json!(self.regex));
        m
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

pub struct ExpectColumnValueLengthsToBeBetween {
    pub column: String,
    pub min_value: Option<usize>,
    pub max_value: Option<usize>,
}

impl Expectation for ExpectColumnValueLengthsToBeBetween {
    fn expectation_type(&self) -> &str {
        "expect_column_value_lengths_to_be_between"
    }
    fn build_kwargs(&self) -> IndexMap<String, Value> {
        let mut m = IndexMap::new();
        m.insert("column".to_string(), json!(self.column));
        m.insert("min_value".to_string(), json!(self.min_value));
        m.insert("max_value".to_string(), json!(self.max_value));
        m
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

pub struct ExpectColumnValuesToBeBetween {
    pub column: String,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
}

impl Expectation for ExpectColumnValuesToBeBetween {
    fn expectation_type(&self) -> &str {
        "expect_column_values_to_be_between"
    }
    fn build_kwargs(&self) -> IndexMap<String, Value> {
        let mut m = IndexMap::new();
        m.insert("column".to_string(), json!(self.column));
        m.insert("min_value".to_string(), json!(self.min_value));
        m.insert("max_value".to_string(), json!(self.max_value));
        m
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

pub struct ExpectColumnValuesToBeDateutilParseable {
    pub column: String,
}

impl Expectation for ExpectColumnValuesToBeDateutilParseable {
    fn expectation_type(&self) -> &str {
        "expect_column_values_to_be_dateutil_parseable"
    }
    fn build_kwargs(&self) -> IndexMap<String, Value> {
        let mut m = IndexMap::new();
        m.insert("column".to_string(), json!(self.column));
        m
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

pub struct ExpectColumnValuesToMatchStrftimeFormat {
    pub column: String,
    pub strftime_format: String,
}

impl Expectation for ExpectColumnValuesToMatchStrftimeFormat {
    fn expectation_type(&self) -> &str {
        "expect_column_values_to_match_strftime_format"
    }
    fn build_kwargs(&self) -> IndexMap<String, Value> {
        let mut m = IndexMap::new();
        m.insert("column".to_string(), json!(self.column));
        m.insert("strftime_format".to_string(), json!(self.strftime_format));
        m
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

pub struct ExpectColumnMeanToBeBetween {
    pub column: String,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
}

impl Expectation for ExpectColumnMeanToBeBetween {
    fn expectation_type(&self) -> &str {
        "expect_column_mean_to_be_between"
    }
    fn build_kwargs(&self) -> IndexMap<String, Value> {
        let mut m = IndexMap::new();
        m.insert("column".to_string(), json!(self.column));
        m.insert("min_value".to_string(), json!(self.min_value));
        m.insert("max_value".to_string(), json!(self.max_value));
        m
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

pub struct ExpectColumnMedianToBeBetween {
    pub column: String,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
}

impl Expectation for ExpectColumnMedianToBeBetween {
    fn expectation_type(&self) -> &str {
        "expect_column_median_to_be_between"
    }
    fn build_kwargs(&self) -> IndexMap<String, Value> {
        let mut m = IndexMap::new();
        m.insert("column".to_string(), json!(self.column));
        m.insert("min_value".to_string(), json!(self.min_value));
        m.insert("max_value".to_string(), json!(self.max_value));
        m
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

pub struct ExpectColumnStdevToBeBetween {
    pub column: String,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
}

impl Expectation for ExpectColumnStdevToBeBetween {
    fn expectation_type(&self) -> &str {
        "expect_column_stdev_to_be_between"
    }
    fn build_kwargs(&self) -> IndexMap<String, Value> {
        let mut m = IndexMap::new();
        m.insert("column".to_string(), json!(self.column));
        m.insert("min_value".to_string(), json!(self.min_value));
        m.insert("max_value".to_string(), json!(self.max_value));
        m
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

pub struct ExpectColumnProportionOfUniqueValuesToBeBetween {
    pub column: String,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
}

impl Expectation for ExpectColumnProportionOfUniqueValuesToBeBetween {
    fn expectation_type(&self) -> &str {
        "expect_column_proportion_of_unique_values_to_be_between"
    }
    fn build_kwargs(&self) -> IndexMap<String, Value> {
        let mut m = IndexMap::new();
        m.insert("column".to_string(), json!(self.column));
        m.insert("min_value".to_string(), json!(self.min_value));
        m.insert("max_value".to_string(), json!(self.max_value));
        m
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

// ── Table-Level Expectations ──────────────────────────────────────────────────

pub struct ExpectTableColumnsToMatchSet {
    pub column_set: Vec<String>,
    pub exact_match: bool,
}

impl Expectation for ExpectTableColumnsToMatchSet {
    fn expectation_type(&self) -> &str {
        "expect_table_columns_to_match_set"
    }
    fn build_kwargs(&self) -> IndexMap<String, Value> {
        let mut m = IndexMap::new();
        m.insert("column_set".to_string(), json!(self.column_set));
        m.insert("exact_match".to_string(), json!(self.exact_match));
        m
    }
    fn build_meta(&self, test_id: &str, category: &str, suite: &str) -> ExpectationMeta {
        ExpectationMeta {
            test_id: test_id.to_string(),
            category: category.to_string(),
            suite: suite.to_string(),
            contract_field: None,
            contract_name: None,
            generated_from: GeneratedFrom::Baseline,
        }
    }
}

pub struct ExpectTableColumnsToMatchOrderedList {
    pub column_list: Vec<String>,
}

impl Expectation for ExpectTableColumnsToMatchOrderedList {
    fn expectation_type(&self) -> &str {
        "expect_table_columns_to_match_ordered_list"
    }
    fn build_kwargs(&self) -> IndexMap<String, Value> {
        let mut m = IndexMap::new();
        m.insert("column_list".to_string(), json!(self.column_list));
        m
    }
    fn build_meta(&self, test_id: &str, category: &str, suite: &str) -> ExpectationMeta {
        ExpectationMeta {
            test_id: test_id.to_string(),
            category: category.to_string(),
            suite: suite.to_string(),
            contract_field: None,
            contract_name: None,
            generated_from: GeneratedFrom::Baseline,
        }
    }
}

pub struct ExpectTableColumnCountToEqual {
    pub value: usize,
}

impl Expectation for ExpectTableColumnCountToEqual {
    fn expectation_type(&self) -> &str {
        "expect_table_column_count_to_equal"
    }
    fn build_kwargs(&self) -> IndexMap<String, Value> {
        let mut m = IndexMap::new();
        m.insert("value".to_string(), json!(self.value));
        m
    }
    fn build_meta(&self, test_id: &str, category: &str, suite: &str) -> ExpectationMeta {
        ExpectationMeta {
            test_id: test_id.to_string(),
            category: category.to_string(),
            suite: suite.to_string(),
            contract_field: None,
            contract_name: None,
            generated_from: GeneratedFrom::Baseline,
        }
    }
}

pub struct ExpectTableRowCountToBeBetween {
    pub min_value: Option<usize>,
    pub max_value: Option<usize>,
}

impl Expectation for ExpectTableRowCountToBeBetween {
    fn expectation_type(&self) -> &str {
        "expect_table_row_count_to_be_between"
    }
    fn build_kwargs(&self) -> IndexMap<String, Value> {
        let mut m = IndexMap::new();
        m.insert("min_value".to_string(), json!(self.min_value));
        m.insert("max_value".to_string(), json!(self.max_value));
        m
    }
    fn build_meta(&self, test_id: &str, category: &str, suite: &str) -> ExpectationMeta {
        ExpectationMeta {
            test_id: test_id.to_string(),
            category: category.to_string(),
            suite: suite.to_string(),
            contract_field: None,
            contract_name: None,
            generated_from: GeneratedFrom::Baseline,
        }
    }
}

pub struct ExpectTableRowCountToEqual {
    pub value: usize,
}

impl Expectation for ExpectTableRowCountToEqual {
    fn expectation_type(&self) -> &str {
        "expect_table_row_count_to_equal"
    }
    fn build_kwargs(&self) -> IndexMap<String, Value> {
        let mut m = IndexMap::new();
        m.insert("value".to_string(), json!(self.value));
        m
    }
    fn build_meta(&self, test_id: &str, category: &str, suite: &str) -> ExpectationMeta {
        ExpectationMeta {
            test_id: test_id.to_string(),
            category: category.to_string(),
            suite: suite.to_string(),
            contract_field: None,
            contract_name: None,
            generated_from: GeneratedFrom::Baseline,
        }
    }
}
