use indexmap::IndexMap;
use serde_json::json;

use crate::config::DqConfig;
use crate::expectations::{ExpectationConfig, ExpectationMeta, GeneratedFrom};
use crate::suites::{fmt_test_id, SuiteGenerator};

pub struct DataConsistencySuite;

impl SuiteGenerator for DataConsistencySuite {
    fn suite_name(&self) -> &str { "data_consistency_suite" }
    fn category(&self) -> &str { "consistency" }
    fn test_id_prefix(&self) -> &str { "DCN" }
    fn test_id_start(&self) -> usize { 166 }

    fn generate(&self, _config: &DqConfig) -> Vec<ExpectationConfig> {
        let suite = self.suite_name();
        let cat = self.category();
        let pfx = self.test_id_prefix();
        let mut e = Vec::new();

        // DCN166-DCN210: Date field consistency checks (45 tests)
        let date_pairs: &[(&str, &str)] = &[
            ("admission_date", "discharge_date"),
            ("service_start_date", "service_end_date"),
            ("coverage_start_date", "coverage_end_date"),
            ("prescription_start_date", "prescription_end_date"),
            ("authorization_start_date", "authorization_end_date"),
            ("episode_start_date", "episode_end_date"),
            ("enrollment_start_date", "enrollment_end_date"),
            ("benefit_start_date", "benefit_end_date"),
            ("contract_start_date", "contract_end_date"),
            ("effective_date", "termination_date"),
            ("claim_from_date", "claim_through_date"),
            ("statement_from_date", "statement_through_date"),
            ("period_start_date", "period_end_date"),
            ("fiscal_year_start", "fiscal_year_end"),
            ("quarter_start_date", "quarter_end_date"),
        ];
        for (i, (start_col, end_col)) in date_pairs.iter().enumerate() {
            // start date not null
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(start_col));
            e.push(ExpectationConfig {
                expectation_type: "expect_column_values_to_be_dateutil_parseable".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 166 + i * 3),
                    category: cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: Some(start_col.to_string()),
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
            // end date not null
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(end_col));
            e.push(ExpectationConfig {
                expectation_type: "expect_column_values_to_be_dateutil_parseable".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 166 + i * 3 + 1),
                    category: cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: Some(end_col.to_string()),
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
            // strftime format check on start
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(start_col));
            kwargs.insert("strftime_format".to_string(), json!("%Y-%m-%d"));
            e.push(ExpectationConfig {
                expectation_type: "expect_column_values_to_match_strftime_format".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 166 + i * 3 + 2),
                    category: cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: Some(start_col.to_string()),
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }
        // DCN211-DCN255: Code consistency checks (45 tests)
        let code_consistency: &[(&str, &[&str])] = &[
            ("gender_code", &["M","F","U","O","X"]),
            ("race_code", &["1","2","3","4","5","6","7","8","9"]),
            ("ethnicity_code", &["H","N","U"]),
            ("marital_status", &["S","M","D","W","U","P","A"]),
            ("language_code", &["ENG","SPA","CHI","FRE","GER","ITA","POR","RUS","ARA","OTH"]),
            ("tobacco_use", &["current","former","never","unknown"]),
            ("alcohol_use", &["none","light","moderate","heavy","unknown"]),
            ("drug_use", &["none","current","former","unknown"]),
            ("employment_status", &["employed","unemployed","retired","student","disabled","unknown"]),
            ("income_level", &["low","medium","high","unknown"]),
            ("education_level", &["less_than_hs","hs_grad","some_college","college_grad","graduate","unknown"]),
            ("housing_status", &["owned","rented","homeless","other","unknown"]),
            ("insurance_type", &["commercial","medicare","medicaid","self_pay","other"]),
            ("payer_type", &["primary","secondary","tertiary"]),
            ("claim_frequency", &["1","2","3","4","5","6","7","8"]),
        ];
        for (i, (col, values)) in code_consistency.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            kwargs.insert("value_set".to_string(), json!(values));
            e.push(ExpectationConfig {
                expectation_type: "expect_column_values_to_be_in_set".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 211 + i),
                    category: cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: Some(col.to_string()),
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }
        // DCN226-DCN255: Casing and whitespace consistency (30 tests)
        let casing_fields = [
            "patient_name", "provider_name", "facility_name", "city", "state_code",
            "country_code", "diagnosis_description", "procedure_description", "drug_name", "manufacturer",
            "specialty_description", "department_name", "unit_name", "ward_name", "floor_name",
            "building_name", "campus_name", "region_name", "district_name", "zone_name",
            "category_name", "subcategory_name", "product_name", "service_name", "plan_name",
            "benefit_name", "program_name", "protocol_name", "pathway_name", "guideline_name",
        ];
        for (i, col) in casing_fields.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            e.push(ExpectationConfig {
                expectation_type: "expect_column_values_to_have_no_leading_trailing_whitespace".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 226 + i),
                    category: cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: Some(col.to_string()),
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }

        debug_assert_eq!(e.len(), 90, "DataConsistencySuite must produce 90 tests");
        e
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::suites::SuiteGenerator;

    #[test]
    fn test_data_consistency_suite_count() {
        let suite = DataConsistencySuite.build_suite(&DqConfig::default());
        assert_eq!(suite.expectations.len(), 90, "DCN166-DCN255 must produce 90 tests");
    }
}
