use indexmap::IndexMap;
use serde_json::json;

use crate::config::DqConfig;
use crate::expectations::{ExpectationConfig, ExpectationMeta, GeneratedFrom};
use crate::suites::{fmt_test_id, SuiteGenerator};

pub struct DataUniquenessSuite;

impl SuiteGenerator for DataUniquenessSuite {
    fn suite_name(&self) -> &str { "data_uniqueness_suite" }
    fn category(&self) -> &str { "uniqueness" }
    fn test_id_prefix(&self) -> &str { "DU" }
    fn test_id_start(&self) -> usize { 666 }

    fn generate(&self, config: &DqConfig) -> Vec<ExpectationConfig> {
        let suite = self.suite_name();
        let cat = self.category();
        let pfx = self.test_id_prefix();
        let mut e = Vec::new();

        // DU666-DU695: Unique value checks (30 tests)
        let unique_fields = [
            "patient_id", "encounter_id", "claim_id", "member_id", "record_id",
            "transaction_id", "authorization_number", "referral_number", "prior_auth_number", "batch_id",
            "submission_id", "tracking_number", "control_number", "document_id", "case_id",
            "episode_id", "visit_id", "order_id", "prescription_id", "lab_order_id",
            "imaging_order_id", "referral_id", "appeal_id", "grievance_id", "audit_id",
            "npi", "tin", "dea_number", "upin", "license_number",
        ];
        for (i, col) in unique_fields.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            e.push(ExpectationConfig {
                expectation_type: "expect_column_values_to_be_unique".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 666 + i),
                    category: cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: Some(col.to_string()),
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }

        // DU696-DU725: Proportion of unique values checks (30 tests)
        let proportion_fields = [
            "patient_id", "encounter_id", "claim_id", "member_id", "record_id",
            "transaction_id", "authorization_number", "referral_number", "prior_auth_number", "batch_id",
            "submission_id", "tracking_number", "control_number", "document_id", "case_id",
            "episode_id", "visit_id", "order_id", "prescription_id", "lab_order_id",
            "imaging_order_id", "referral_id", "appeal_id", "grievance_id", "audit_id",
            "npi", "tin", "dea_number", "upin", "license_number",
        ];
        for (i, col) in proportion_fields.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            kwargs.insert("min_value".to_string(), json!(config.uniqueness_min));
            kwargs.insert("max_value".to_string(), json!(1.0));
            e.push(ExpectationConfig {
                expectation_type: "expect_column_proportion_of_unique_values_to_be_between".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 696 + i),
                    category: cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: Some(col.to_string()),
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }

        debug_assert_eq!(e.len(), 60, "DataUniquenessSuite must produce 60 tests");
        e
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::suites::SuiteGenerator;

    #[test]
    fn test_data_uniqueness_suite_count() {
        let suite = DataUniquenessSuite.build_suite(&DqConfig::default());
        assert_eq!(suite.expectations.len(), 60, "DU666-DU725 must produce 60 tests");
    }
}
