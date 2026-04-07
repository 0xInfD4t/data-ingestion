use indexmap::IndexMap;
use serde_json::json;

use crate::config::DqConfig;
use crate::expectations::{ExpectationConfig, ExpectationMeta, GeneratedFrom};
use crate::suites::{fmt_test_id, SuiteGenerator};

pub struct DataAccuracySuite;

impl SuiteGenerator for DataAccuracySuite {
    fn suite_name(&self) -> &str { "data_accuracy_suite" }
    fn category(&self) -> &str { "accuracy" }
    fn test_id_prefix(&self) -> &str { "DA" }
    fn test_id_start(&self) -> usize { 256 }

    fn generate(&self, _config: &DqConfig) -> Vec<ExpectationConfig> {
        let suite = self.suite_name();
        let cat = self.category();
        let pfx = self.test_id_prefix();
        let mut e = Vec::new();

        // DA256-DA305: Numeric range accuracy checks (50 tests)
        let numeric_ranges: &[(&str, f64, f64)] = &[
            ("age_at_service", 0.0, 130.0),
            ("age_at_admission", 0.0, 130.0),
            ("age_at_discharge", 0.0, 130.0),
            ("length_of_stay", 0.0, 365.0),
            ("days_since_last_visit", 0.0, 3650.0),
            ("allowed_amount", 0.0, 10_000_000.0),
            ("paid_amount", 0.0, 10_000_000.0),
            ("billed_amount", 0.0, 10_000_000.0),
            ("copay_amount", 0.0, 100_000.0),
            ("deductible_amount", 0.0, 100_000.0),
            ("coinsurance_amount", 0.0, 100_000.0),
            ("out_of_pocket_amount", 0.0, 100_000.0),
            ("premium_amount", 0.0, 100_000.0),
            ("capitation_amount", 0.0, 100_000.0),
            ("withhold_amount", 0.0, 100_000.0),
            ("units_of_service", 0.0, 10_000.0),
            ("days_supply", 1.0, 365.0),
            ("quantity_dispensed", 0.0, 10_000.0),
            ("refill_number", 0.0, 99.0),
            ("days_in_hospital", 0.0, 365.0),
            ("icu_days", 0.0, 365.0),
            ("ventilator_days", 0.0, 365.0),
            ("readmission_days", 0.0, 365.0),
            ("er_visits_count", 0.0, 1000.0),
            ("inpatient_admissions", 0.0, 100.0),
            ("outpatient_visits", 0.0, 1000.0),
            ("specialist_visits", 0.0, 500.0),
            ("pcp_visits", 0.0, 500.0),
            ("telehealth_visits", 0.0, 500.0),
            ("pharmacy_fills", 0.0, 1000.0),
            ("lab_tests_count", 0.0, 1000.0),
            ("imaging_studies", 0.0, 500.0),
            ("procedures_count", 0.0, 500.0),
            ("diagnoses_count", 0.0, 100.0),
            ("medications_count", 0.0, 100.0),
            ("allergies_count", 0.0, 100.0),
            ("immunizations_count", 0.0, 100.0),
            ("chronic_conditions_count", 0.0, 50.0),
            ("risk_score", 0.0, 100.0),
            ("hcc_score", 0.0, 50.0),
            ("raf_score", 0.0, 10.0),
            ("quality_score", 0.0, 100.0),
            ("star_rating", 1.0, 5.0),
            ("hedis_rate", 0.0, 1.0),
            ("cahps_score", 0.0, 100.0),
            ("net_promoter_score", -100.0, 100.0),
            ("satisfaction_score", 0.0, 10.0),
            ("engagement_score", 0.0, 100.0),
            ("adherence_rate", 0.0, 1.0),
            ("compliance_rate", 0.0, 1.0),
        ];
        for (i, (col, min_v, max_v)) in numeric_ranges.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            kwargs.insert("min_value".to_string(), json!(min_v));
            kwargs.insert("max_value".to_string(), json!(max_v));
            e.push(ExpectationConfig {
                expectation_type: "expect_column_values_to_be_between".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 256 + i),
                    category: cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: Some(col.to_string()),
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }

        // DA306-DA355: Mean/median accuracy checks (50 tests)
        let stat_checks: &[(&str, f64, f64)] = &[
            ("allowed_amount", 50.0, 50_000.0),
            ("paid_amount", 0.0, 50_000.0),
            ("billed_amount", 50.0, 100_000.0),
            ("length_of_stay", 1.0, 30.0),
            ("age_at_service", 18.0, 80.0),
            ("days_supply", 15.0, 90.0),
            ("quantity_dispensed", 10.0, 500.0),
            ("units_of_service", 1.0, 100.0),
            ("risk_score", 0.5, 5.0),
            ("hcc_score", 0.5, 5.0),
            ("raf_score", 0.5, 3.0),
            ("quality_score", 50.0, 95.0),
            ("hedis_rate", 0.3, 0.95),
            ("adherence_rate", 0.5, 0.95),
            ("compliance_rate", 0.5, 0.95),
            ("er_visits_count", 0.0, 5.0),
            ("inpatient_admissions", 0.0, 2.0),
            ("outpatient_visits", 1.0, 20.0),
            ("pcp_visits", 1.0, 10.0),
            ("specialist_visits", 0.0, 10.0),
            ("pharmacy_fills", 1.0, 30.0),
            ("lab_tests_count", 0.0, 20.0),
            ("imaging_studies", 0.0, 10.0),
            ("procedures_count", 0.0, 10.0),
            ("diagnoses_count", 1.0, 15.0),
            ("medications_count", 0.0, 15.0),
            ("chronic_conditions_count", 0.0, 5.0),
            ("copay_amount", 0.0, 100.0),
            ("deductible_amount", 0.0, 5000.0),
            ("premium_amount", 100.0, 2000.0),
            ("satisfaction_score", 5.0, 10.0),
            ("engagement_score", 20.0, 80.0),
            ("net_promoter_score", -20.0, 80.0),
            ("star_rating", 2.0, 5.0),
            ("cahps_score", 50.0, 95.0),
            ("icu_days", 0.0, 5.0),
            ("ventilator_days", 0.0, 3.0),
            ("readmission_days", 0.0, 30.0),
            ("telehealth_visits", 0.0, 10.0),
            ("immunizations_count", 0.0, 10.0),
            ("allergies_count", 0.0, 5.0),
            ("out_of_pocket_amount", 0.0, 10000.0),
            ("coinsurance_amount", 0.0, 5000.0),
            ("capitation_amount", 0.0, 1000.0),
            ("withhold_amount", 0.0, 500.0),
            ("days_since_last_visit", 0.0, 365.0),
            ("refill_number", 0.0, 5.0),
            ("days_in_hospital", 0.0, 10.0),
            ("age_at_admission", 18.0, 80.0),
            ("age_at_discharge", 18.0, 80.0),
        ];
        for (i, (col, min_v, max_v)) in stat_checks.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            kwargs.insert("min_value".to_string(), json!(min_v));
            kwargs.insert("max_value".to_string(), json!(max_v));
            e.push(ExpectationConfig {
                expectation_type: "expect_column_mean_to_be_between".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 306 + i),
                    category: cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: Some(col.to_string()),
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }

        debug_assert_eq!(e.len(), 100, "DataAccuracySuite must produce 100 tests");
        e
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::suites::SuiteGenerator;

    #[test]
    fn test_data_accuracy_suite_count() {
        let suite = DataAccuracySuite.build_suite(&DqConfig::default());
        assert_eq!(suite.expectations.len(), 100, "DA256-DA355 must produce 100 tests");
    }
}
