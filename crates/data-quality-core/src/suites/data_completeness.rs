use indexmap::IndexMap;
use serde_json::json;

use crate::config::DqConfig;
use crate::expectations::{ExpectationConfig, ExpectationMeta, GeneratedFrom};
use crate::suites::{fmt_test_id, SuiteGenerator};

pub struct DataCompletenessSuite;

impl SuiteGenerator for DataCompletenessSuite {
    fn suite_name(&self) -> &str { "data_completeness_suite" }
    fn category(&self) -> &str { "completeness" }
    fn test_id_prefix(&self) -> &str { "DC" }
    fn test_id_start(&self) -> usize { 96 }

    fn generate(&self, config: &DqConfig) -> Vec<ExpectationConfig> {
        let suite = self.suite_name();
        let cat = self.category();
        let pfx = self.test_id_prefix();
        let mostly = 1.0 - config.null_ratio_max;
        let mut e = Vec::new();

        // DC096-DC130: Not-null checks for critical healthcare fields (35 tests)
        let critical_fields = [
            "patient_id", "encounter_id", "claim_id", "member_id", "provider_npi",
            "service_date", "diagnosis_code", "procedure_code", "facility_id", "plan_id",
            "beneficiary_id", "recipient_id", "admission_date", "discharge_date", "claim_type",
            "claim_status", "allowed_amount", "paid_amount", "place_of_service", "type_of_bill",
            "revenue_code", "drg_code", "attending_provider_npi", "billing_provider_npi", "rendering_provider_npi",
            "referring_provider_npi", "ordering_provider_npi", "supervising_provider_npi", "primary_diagnosis", "principal_procedure",
            "admission_source", "discharge_disposition", "patient_status", "coverage_start_date", "coverage_end_date",
        ];
        for (i, col) in critical_fields.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            kwargs.insert("mostly".to_string(), json!(mostly));
            e.push(ExpectationConfig {
                expectation_type: "expect_column_values_to_not_be_null".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 96 + i),
                    category: cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: Some(col.to_string()),
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }

        // DC131-DC155: Column proportion of unique values (25 tests)
        let uniqueness_fields = [
            "patient_id", "encounter_id", "claim_id", "member_id", "record_id",
            "transaction_id", "authorization_number", "referral_number", "prior_auth_number", "batch_id",
            "submission_id", "tracking_number", "control_number", "document_id", "case_id",
            "episode_id", "visit_id", "order_id", "prescription_id", "lab_order_id",
            "imaging_order_id", "referral_id", "appeal_id", "grievance_id", "audit_id",
        ];
        for (i, col) in uniqueness_fields.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            kwargs.insert("min_value".to_string(), json!(config.uniqueness_min));
            kwargs.insert("max_value".to_string(), json!(1.0));
            e.push(ExpectationConfig {
                expectation_type: "expect_column_proportion_of_unique_values_to_be_between".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 131 + i),
                    category: cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: Some(col.to_string()),
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }

        // DC156-DC165: Row count and column count checks (10 tests)
        let table_checks: &[(&str, usize, usize)] = &[
            ("claims_table", 1000, 10_000_000),
            ("members_table", 100, 5_000_000),
            ("providers_table", 10, 500_000),
            ("encounters_table", 500, 20_000_000),
            ("eligibility_table", 100, 10_000_000),
            ("pharmacy_table", 100, 5_000_000),
            ("lab_results_table", 50, 10_000_000),
            ("diagnoses_table", 1000, 50_000_000),
            ("procedures_table", 500, 20_000_000),
            ("authorizations_table", 10, 1_000_000),
        ];
        for (i, (table, min_rows, max_rows)) in table_checks.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("min_value".to_string(), json!(min_rows));
            kwargs.insert("max_value".to_string(), json!(max_rows));
            e.push(ExpectationConfig {
                expectation_type: "expect_table_row_count_to_be_between".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 156 + i),
                    category: cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: None,
                    contract_name: Some(table.to_string()),
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }

        debug_assert_eq!(e.len(), 70, "DataCompletenessSuite must produce 70 tests");
        e
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::suites::SuiteGenerator;

    #[test]
    fn test_data_completeness_suite_count() {
        let suite = DataCompletenessSuite.build_suite(&DqConfig::default());
        assert_eq!(suite.expectations.len(), 70, "DC096-DC165 must produce 70 tests");
    }
}
