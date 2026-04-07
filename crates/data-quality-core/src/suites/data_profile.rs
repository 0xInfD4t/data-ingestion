use indexmap::IndexMap;
use serde_json::json;

use crate::config::DqConfig;
use crate::expectations::{ExpectationConfig, ExpectationMeta, GeneratedFrom};
use crate::suites::{fmt_test_id, SuiteGenerator};

pub struct DataProfileSuite;

impl SuiteGenerator for DataProfileSuite {
    fn suite_name(&self) -> &str { "data_profile_suite" }
    fn category(&self) -> &str { "profiling" }
    fn test_id_prefix(&self) -> &str { "DP" }
    fn test_id_start(&self) -> usize { 356 }

    fn generate(&self, _config: &DqConfig) -> Vec<ExpectationConfig> {
        let suite = self.suite_name();
        let cat = self.category();
        let pfx = self.test_id_prefix();
        let mut e = Vec::new();

        // DP356-DP415: Column mean profiling (60 tests)
        let mean_profiles: &[(&str, f64, f64)] = &[
            ("age", 30.0, 70.0),
            ("weight_kg", 50.0, 120.0),
            ("height_cm", 150.0, 200.0),
            ("bmi", 18.0, 40.0),
            ("systolic_bp", 100.0, 160.0),
            ("diastolic_bp", 60.0, 100.0),
            ("heart_rate", 55.0, 100.0),
            ("temperature_c", 36.0, 38.0),
            ("oxygen_saturation", 90.0, 100.0),
            ("respiratory_rate", 12.0, 25.0),
            ("glucose_mg_dl", 70.0, 200.0),
            ("hemoglobin_g_dl", 10.0, 18.0),
            ("creatinine_mg_dl", 0.5, 3.0),
            ("cholesterol_mg_dl", 150.0, 300.0),
            ("triglycerides_mg_dl", 50.0, 400.0),
            ("sodium_meq_l", 130.0, 150.0),
            ("potassium_meq_l", 3.0, 6.0),
            ("wbc_k_ul", 3.0, 15.0),
            ("platelet_k_ul", 100.0, 500.0),
            ("inr", 0.8, 4.0),
            ("allowed_amount", 100.0, 20000.0),
            ("paid_amount", 0.0, 20000.0),
            ("billed_amount", 100.0, 50000.0),
            ("copay_amount", 0.0, 200.0),
            ("deductible_amount", 0.0, 5000.0),
            ("coinsurance_amount", 0.0, 2000.0),
            ("premium_amount", 200.0, 1500.0),
            ("length_of_stay", 1.0, 15.0),
            ("days_supply", 15.0, 90.0),
            ("quantity_dispensed", 10.0, 300.0),
            ("units_of_service", 1.0, 50.0),
            ("risk_score", 0.5, 5.0),
            ("hcc_score", 0.5, 5.0),
            ("raf_score", 0.5, 3.0),
            ("quality_score", 50.0, 95.0),
            ("hedis_rate", 0.3, 0.95),
            ("adherence_rate", 0.5, 0.95),
            ("er_visits_count", 0.0, 3.0),
            ("inpatient_admissions", 0.0, 1.0),
            ("outpatient_visits", 1.0, 15.0),
            ("pcp_visits", 1.0, 8.0),
            ("specialist_visits", 0.0, 8.0),
            ("pharmacy_fills", 1.0, 20.0),
            ("lab_tests_count", 0.0, 15.0),
            ("imaging_studies", 0.0, 8.0),
            ("procedures_count", 0.0, 8.0),
            ("diagnoses_count", 1.0, 10.0),
            ("medications_count", 0.0, 10.0),
            ("chronic_conditions_count", 0.0, 4.0),
            ("satisfaction_score", 5.0, 10.0),
            ("engagement_score", 20.0, 80.0),
            ("net_promoter_score", -20.0, 80.0),
            ("star_rating", 2.0, 5.0),
            ("cahps_score", 50.0, 95.0),
            ("icu_days", 0.0, 3.0),
            ("ventilator_days", 0.0, 2.0),
            ("readmission_days", 0.0, 20.0),
            ("telehealth_visits", 0.0, 8.0),
            ("immunizations_count", 0.0, 8.0),
            ("allergies_count", 0.0, 4.0),
        ];
        for (i, (col, min_v, max_v)) in mean_profiles.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            kwargs.insert("min_value".to_string(), json!(min_v));
            kwargs.insert("max_value".to_string(), json!(max_v));
            e.push(ExpectationConfig {
                expectation_type: "expect_column_mean_to_be_between".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 356 + i),
                    category: cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: Some(col.to_string()),
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }

        // DP416-DP445: Median profiling (30 tests)
        let median_profiles: &[(&str, f64, f64)] = &[
            ("age", 35.0, 65.0),
            ("weight_kg", 55.0, 110.0),
            ("bmi", 20.0, 35.0),
            ("systolic_bp", 105.0, 150.0),
            ("diastolic_bp", 65.0, 95.0),
            ("heart_rate", 60.0, 90.0),
            ("glucose_mg_dl", 75.0, 180.0),
            ("hemoglobin_g_dl", 11.0, 17.0),
            ("cholesterol_mg_dl", 160.0, 280.0),
            ("allowed_amount", 50.0, 15000.0),
            ("paid_amount", 0.0, 15000.0),
            ("billed_amount", 50.0, 40000.0),
            ("length_of_stay", 1.0, 10.0),
            ("days_supply", 20.0, 90.0),
            ("risk_score", 0.5, 4.0),
            ("quality_score", 55.0, 95.0),
            ("hedis_rate", 0.35, 0.95),
            ("adherence_rate", 0.55, 0.95),
            ("er_visits_count", 0.0, 2.0),
            ("outpatient_visits", 1.0, 12.0),
            ("pharmacy_fills", 1.0, 15.0),
            ("diagnoses_count", 1.0, 8.0),
            ("medications_count", 0.0, 8.0),
            ("satisfaction_score", 6.0, 10.0),
            ("engagement_score", 25.0, 75.0),
            ("star_rating", 2.5, 5.0),
            ("cahps_score", 55.0, 95.0),
            ("premium_amount", 250.0, 1200.0),
            ("copay_amount", 0.0, 150.0),
            ("deductible_amount", 0.0, 4000.0),
        ];
        for (i, (col, min_v, max_v)) in median_profiles.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            kwargs.insert("min_value".to_string(), json!(min_v));
            kwargs.insert("max_value".to_string(), json!(max_v));
            e.push(ExpectationConfig {
                expectation_type: "expect_column_median_to_be_between".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 416 + i),
                    category: cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: Some(col.to_string()),
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }

        // DP446-DP475: Stdev profiling (30 tests)
        let stdev_profiles: &[(&str, f64, f64)] = &[
            ("age", 5.0, 30.0),
            ("weight_kg", 5.0, 40.0),
            ("bmi", 1.0, 15.0),
            ("systolic_bp", 5.0, 40.0),
            ("diastolic_bp", 3.0, 25.0),
            ("heart_rate", 5.0, 30.0),
            ("glucose_mg_dl", 10.0, 100.0),
            ("hemoglobin_g_dl", 0.5, 5.0),
            ("cholesterol_mg_dl", 20.0, 80.0),
            ("allowed_amount", 50.0, 50000.0),
            ("paid_amount", 0.0, 50000.0),
            ("billed_amount", 50.0, 100000.0),
            ("length_of_stay", 0.5, 20.0),
            ("days_supply", 5.0, 60.0),
            ("risk_score", 0.1, 3.0),
            ("quality_score", 5.0, 30.0),
            ("hedis_rate", 0.05, 0.3),
            ("adherence_rate", 0.05, 0.3),
            ("er_visits_count", 0.0, 3.0),
            ("outpatient_visits", 0.5, 10.0),
            ("pharmacy_fills", 0.5, 15.0),
            ("diagnoses_count", 0.5, 8.0),
            ("medications_count", 0.5, 8.0),
            ("satisfaction_score", 0.5, 3.0),
            ("engagement_score", 5.0, 30.0),
            ("star_rating", 0.1, 1.5),
            ("cahps_score", 5.0, 25.0),
            ("premium_amount", 50.0, 500.0),
            ("copay_amount", 0.0, 100.0),
            ("deductible_amount", 0.0, 3000.0),
        ];
        for (i, (col, min_v, max_v)) in stdev_profiles.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            kwargs.insert("min_value".to_string(), json!(min_v));
            kwargs.insert("max_value".to_string(), json!(max_v));
            e.push(ExpectationConfig {
                expectation_type: "expect_column_stdev_to_be_between".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 446 + i),
                    category: cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: Some(col.to_string()),
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }

        debug_assert_eq!(e.len(), 120, "DataProfileSuite must produce 120 tests");
        e
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::suites::SuiteGenerator;

    #[test]
    fn test_data_profile_suite_count() {
        let suite = DataProfileSuite.build_suite(&DqConfig::default());
        assert_eq!(suite.expectations.len(), 120, "DP356-DP475 must produce 120 tests");
    }
}
