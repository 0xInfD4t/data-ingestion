use indexmap::IndexMap;
use serde_json::json;

use crate::config::DqConfig;
use crate::expectations::{ExpectationConfig, ExpectationMeta, GeneratedFrom};
use crate::suites::{fmt_test_id, SuiteGenerator};

pub struct DataIntegritySuite;

impl SuiteGenerator for DataIntegritySuite {
    fn suite_name(&self) -> &str { "data_integrity_suite" }
    fn category(&self) -> &str { "integrity" }
    fn test_id_prefix(&self) -> &str { "DI" }
    fn test_id_start(&self) -> usize { 476 }

    fn generate(&self, _config: &DqConfig) -> Vec<ExpectationConfig> {
        let suite = self.suite_name();
        let cat = self.category();
        let pfx = self.test_id_prefix();
        let mut e = Vec::new();

        // DI476-DI510: Foreign key integrity checks (35 tests)
        let fk_checks: &[(&str, &str, &str)] = &[
            ("patient_id", "patients", "id"),
            ("encounter_id", "encounters", "id"),
            ("claim_id", "claims", "id"),
            ("member_id", "members", "id"),
            ("provider_npi", "providers", "npi"),
            ("facility_id", "facilities", "id"),
            ("plan_id", "plans", "id"),
            ("diagnosis_code", "icd10_codes", "code"),
            ("procedure_code", "cpt_codes", "code"),
            ("drug_ndc", "ndc_codes", "ndc"),
            ("attending_provider_npi", "providers", "npi"),
            ("billing_provider_npi", "providers", "npi"),
            ("rendering_provider_npi", "providers", "npi"),
            ("referring_provider_npi", "providers", "npi"),
            ("ordering_provider_npi", "providers", "npi"),
            ("payer_id", "payers", "id"),
            ("group_id", "groups", "id"),
            ("benefit_id", "benefits", "id"),
            ("authorization_id", "authorizations", "id"),
            ("referral_id", "referrals", "id"),
            ("lab_order_id", "lab_orders", "id"),
            ("imaging_order_id", "imaging_orders", "id"),
            ("prescription_id", "prescriptions", "id"),
            ("episode_id", "episodes", "id"),
            ("case_id", "cases", "id"),
            ("program_id", "programs", "id"),
            ("protocol_id", "protocols", "id"),
            ("pathway_id", "pathways", "id"),
            ("care_team_id", "care_teams", "id"),
            ("care_manager_id", "care_managers", "id"),
            ("primary_care_provider_id", "providers", "id"),
            ("specialist_id", "providers", "id"),
            ("pharmacy_id", "pharmacies", "id"),
            ("lab_id", "labs", "id"),
            ("imaging_center_id", "imaging_centers", "id"),
        ];
        for (i, (col, ref_table, ref_col)) in fk_checks.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            kwargs.insert("value_set".to_string(), json!([]));
            e.push(ExpectationConfig {
                expectation_type: "expect_column_values_to_be_in_set".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 476 + i),
                    category: cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: Some(col.to_string()),
                    contract_name: Some(format!("fk:{}:{}", ref_table, ref_col)),
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }

        // DI511-DI535: Uniqueness integrity checks (25 tests)
        let unique_fields = [
            "patient_id", "encounter_id", "claim_id", "member_id", "record_id",
            "transaction_id", "authorization_number", "referral_number", "prior_auth_number", "batch_id",
            "submission_id", "tracking_number", "control_number", "document_id", "case_id",
            "episode_id", "visit_id", "order_id", "prescription_id", "lab_order_id",
            "imaging_order_id", "referral_id", "appeal_id", "grievance_id", "audit_id",
        ];
        for (i, col) in unique_fields.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            e.push(ExpectationConfig {
                expectation_type: "expect_column_values_to_be_unique".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 511 + i),
                    category: cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: Some(col.to_string()),
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }

        debug_assert_eq!(e.len(), 60, "DataIntegritySuite must produce 60 tests");
        e
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::suites::SuiteGenerator;

    #[test]
    fn test_data_integrity_suite_count() {
        let suite = DataIntegritySuite.build_suite(&DqConfig::default());
        assert_eq!(suite.expectations.len(), 60, "DI476-DI535 must produce 60 tests");
    }
}
