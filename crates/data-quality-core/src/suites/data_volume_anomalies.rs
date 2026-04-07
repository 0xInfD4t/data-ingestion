use indexmap::IndexMap;
use serde_json::json;

use crate::config::DqConfig;
use crate::expectations::{ExpectationConfig, ExpectationMeta, GeneratedFrom};
use crate::suites::{fmt_test_id, SuiteGenerator};

pub struct DataVolumeAnomaliesSuite;

impl SuiteGenerator for DataVolumeAnomaliesSuite {
    fn suite_name(&self) -> &str { "data_volume_anomalies_suite" }
    fn category(&self) -> &str { "volume_anomalies" }
    fn test_id_prefix(&self) -> &str { "DVA" }
    fn test_id_start(&self) -> usize { 876 }

    fn generate(&self, config: &DqConfig) -> Vec<ExpectationConfig> {
        let suite = self.suite_name();
        let cat = self.category();
        let pfx = self.test_id_prefix();
        let mut e = Vec::new();

        // DVA876-DVA920: Row count range checks for various tables (45 tests)
        let table_row_counts: &[(&str, u64, u64)] = &[
            ("claims", 10_000, 100_000_000),
            ("members", 1_000, 10_000_000),
            ("providers", 100, 1_000_000),
            ("encounters", 5_000, 200_000_000),
            ("eligibility", 1_000, 50_000_000),
            ("pharmacy_claims", 5_000, 100_000_000),
            ("lab_results", 1_000, 500_000_000),
            ("diagnoses", 10_000, 500_000_000),
            ("procedures", 5_000, 200_000_000),
            ("authorizations", 100, 10_000_000),
            ("referrals", 100, 10_000_000),
            ("appeals", 10, 1_000_000),
            ("grievances", 10, 500_000),
            ("care_plans", 100, 5_000_000),
            ("care_gaps", 100, 10_000_000),
            ("risk_scores", 1_000, 10_000_000),
            ("quality_measures", 100, 5_000_000),
            ("hedis_measures", 100, 5_000_000),
            ("star_ratings", 10, 100_000),
            ("cahps_surveys", 10, 1_000_000),
            ("claims_adjustments", 100, 10_000_000),
            ("claims_voids", 10, 1_000_000),
            ("remittances", 100, 10_000_000),
            ("payments", 100, 10_000_000),
            ("capitation_payments", 10, 1_000_000),
            ("withholds", 10, 500_000),
            ("risk_corridors", 1, 10_000),
            ("mlr_reports", 1, 1_000),
            ("audit_logs", 1_000, 1_000_000_000),
            ("error_logs", 0, 100_000_000),
            ("batch_jobs", 1, 1_000_000),
            ("data_loads", 1, 100_000),
            ("data_exports", 1, 100_000),
            ("api_calls", 100, 1_000_000_000),
            ("user_sessions", 10, 100_000_000),
            ("notifications", 100, 100_000_000),
            ("alerts", 10, 10_000_000),
            ("tasks", 100, 10_000_000),
            ("workflows", 10, 1_000_000),
            ("documents", 100, 100_000_000),
            ("attachments", 10, 100_000_000),
            ("messages", 100, 1_000_000_000),
            ("events", 1_000, 10_000_000_000),
            ("metrics", 1_000, 10_000_000_000),
            ("snapshots", 1, 10_000_000),
        ];
        for (i, (table, min_rows, max_rows)) in table_row_counts.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("min_value".to_string(), json!(min_rows));
            kwargs.insert("max_value".to_string(), json!(max_rows));
            e.push(ExpectationConfig {
                expectation_type: "expect_table_row_count_to_be_between".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 876 + i),
                    category: cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: None,
                    contract_name: Some(table.to_string()),
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }

        // DVA921-DVA965: Z-score anomaly checks for numeric columns (45 tests)
        let zscore_fields = [
            "allowed_amount", "paid_amount", "billed_amount", "copay_amount", "deductible_amount",
            "coinsurance_amount", "premium_amount", "capitation_amount", "withhold_amount", "out_of_pocket_amount",
            "length_of_stay", "days_supply", "quantity_dispensed", "units_of_service", "refill_number",
            "age", "weight_kg", "height_cm", "bmi", "systolic_bp",
            "diastolic_bp", "heart_rate", "temperature_c", "oxygen_saturation", "respiratory_rate",
            "glucose_mg_dl", "hemoglobin_g_dl", "creatinine_mg_dl", "cholesterol_mg_dl", "triglycerides_mg_dl",
            "risk_score", "hcc_score", "raf_score", "quality_score", "hedis_rate",
            "er_visits_count", "inpatient_admissions", "outpatient_visits", "pharmacy_fills", "lab_tests_count",
            "diagnoses_count", "medications_count", "procedures_count", "chronic_conditions_count", "days_in_hospital",
        ];
        for (i, col) in zscore_fields.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            kwargs.insert("z_score_max".to_string(), json!(config.z_score_max));
            e.push(ExpectationConfig {
                expectation_type: "expect_column_values_to_be_within_z_score".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 921 + i),
                    category: cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: Some(col.to_string()),
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }

        debug_assert_eq!(e.len(), 90, "DataVolumeAnomaliesSuite must produce 90 tests");
        e
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::suites::SuiteGenerator;

    #[test]
    fn test_data_volume_anomalies_suite_count() {
        let suite = DataVolumeAnomaliesSuite.build_suite(&DqConfig::default());
        assert_eq!(suite.expectations.len(), 90, "DVA876-DVA965 must produce 90 tests");
    }
}
