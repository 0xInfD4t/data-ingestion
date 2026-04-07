use indexmap::IndexMap;
use serde_json::json;

use crate::config::DqConfig;
use crate::expectations::{ExpectationConfig, ExpectationMeta, GeneratedFrom};
use crate::suites::{fmt_test_id, SuiteGenerator};

pub struct DataDependencyChecksSuite;

impl SuiteGenerator for DataDependencyChecksSuite {
    fn suite_name(&self) -> &str { "data_dependency_checks_suite" }
    fn category(&self) -> &str { "dependency_checks" }
    fn test_id_prefix(&self) -> &str { "DDC" }
    fn test_id_start(&self) -> usize { 966 }

    fn generate(&self, _config: &DqConfig) -> Vec<ExpectationConfig> {
        let suite = self.suite_name();
        let cat = self.category();
        let pfx = self.test_id_prefix();
        let mut e = Vec::new();

        // DDC966-DDC1000: Column set dependency checks (35 tests)
        let schema_checks: &[(&str, &[&str])] = &[
            ("claims_header", &["claim_id","patient_id","provider_npi","service_date","claim_type","claim_status","allowed_amount","paid_amount"]),
            ("claims_detail", &["claim_id","line_number","procedure_code","diagnosis_code","units_of_service","allowed_amount","paid_amount"]),
            ("member_eligibility", &["member_id","plan_id","coverage_start_date","coverage_end_date","coverage_type","benefit_type"]),
            ("provider_roster", &["npi","provider_name","specialty","taxonomy_code","network_indicator","effective_date","termination_date"]),
            ("pharmacy_claims", &["claim_id","member_id","ndc_code","days_supply","quantity_dispensed","paid_amount","pharmacy_id"]),
            ("lab_results", &["order_id","patient_id","loinc_code","result_value","result_unit","reference_range","result_date"]),
            ("diagnoses", &["encounter_id","patient_id","icd10_code","diagnosis_type","diagnosis_date","provider_npi"]),
            ("procedures", &["encounter_id","patient_id","cpt_code","procedure_date","provider_npi","facility_id"]),
            ("authorizations", &["auth_id","member_id","provider_npi","service_type","auth_start_date","auth_end_date","auth_status"]),
            ("referrals", &["referral_id","member_id","referring_npi","referred_to_npi","referral_date","specialty","referral_status"]),
            ("care_plans", &["plan_id","patient_id","care_manager_id","plan_start_date","plan_end_date","goals","interventions"]),
            ("risk_scores", &["member_id","score_date","risk_model","risk_score","risk_tier","hcc_codes"]),
            ("quality_measures", &["member_id","measure_id","measurement_year","numerator","denominator","exclusion","rate"]),
            ("appeals", &["appeal_id","claim_id","member_id","appeal_date","appeal_reason","appeal_status","resolution_date"]),
            ("grievances", &["grievance_id","member_id","grievance_date","grievance_type","grievance_status","resolution_date"]),
            ("capitation_payments", &["payment_id","member_id","plan_id","payment_month","pmpm_amount","total_amount"]),
            ("remittances", &["remittance_id","payer_id","payment_date","payment_amount","claim_count","check_number"]),
            ("audit_logs", &["log_id","user_id","action","entity_type","entity_id","timestamp","ip_address"]),
            ("batch_jobs", &["job_id","job_name","job_type","start_time","end_time","status","records_processed"]),
            ("data_loads", &["load_id","source_system","load_date","file_name","record_count","error_count","status"]),
            ("notifications", &["notification_id","recipient_id","notification_type","channel","sent_at","status"]),
            ("tasks", &["task_id","assignee_id","task_type","priority","due_date","status","created_at"]),
            ("documents", &["document_id","owner_id","document_type","file_name","file_size","created_at","status"]),
            ("messages", &["message_id","sender_id","recipient_id","message_type","sent_at","read_at","status"]),
            ("events", &["event_id","event_type","entity_id","entity_type","occurred_at","source_system","payload"]),
            ("metrics", &["metric_id","metric_name","metric_value","metric_unit","measured_at","source"]),
            ("snapshots", &["snapshot_id","entity_type","entity_id","snapshot_date","data_hash","record_count"]),
            ("workflows", &["workflow_id","workflow_type","initiator_id","start_date","end_date","status","steps_completed"]),
            ("protocols", &["protocol_id","protocol_name","version","effective_date","expiration_date","status"]),
            ("pathways", &["pathway_id","pathway_name","condition","steps","evidence_level","last_updated"]),
            ("programs", &["program_id","program_name","program_type","start_date","end_date","eligibility_criteria","status"]),
            ("care_teams", &["team_id","patient_id","primary_provider_id","care_manager_id","team_type","effective_date"]),
            ("episodes", &["episode_id","patient_id","episode_type","start_date","end_date","primary_diagnosis","status"]),
            ("cases", &["case_id","patient_id","case_type","opened_date","closed_date","case_manager_id","status"]),
            ("enrollments", &["enrollment_id","member_id","plan_id","enrollment_date","disenrollment_date","enrollment_type","status"]),
        ];
        for (i, (table, cols)) in schema_checks.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column_set".to_string(), json!(cols));
            kwargs.insert("exact_match".to_string(), json!(false));
            e.push(ExpectationConfig {
                expectation_type: "expect_table_columns_to_match_set".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 966 + i),
                    category: cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: None,
                    contract_name: Some(table.to_string()),
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }

        // DDC1001-DDC1015: Column count checks (15 tests)
        let col_counts: &[(&str, usize)] = &[
            ("claims_header", 8),
            ("claims_detail", 7),
            ("member_eligibility", 6),
            ("provider_roster", 7),
            ("pharmacy_claims", 7),
            ("lab_results", 7),
            ("diagnoses", 6),
            ("procedures", 6),
            ("authorizations", 7),
            ("referrals", 7),
            ("care_plans", 7),
            ("risk_scores", 6),
            ("quality_measures", 7),
            ("appeals", 7),
            ("grievances", 6),
        ];
        for (i, (table, count)) in col_counts.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("value".to_string(), json!(count));
            e.push(ExpectationConfig {
                expectation_type: "expect_table_column_count_to_equal".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 1001 + i),
                    category: cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: None,
                    contract_name: Some(table.to_string()),
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }

        debug_assert_eq!(e.len(), 50, "DataDependencyChecksSuite must produce 50 tests");
        e
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::suites::SuiteGenerator;

    #[test]
    fn test_data_dependency_checks_suite_count() {
        let suite = DataDependencyChecksSuite.build_suite(&DqConfig::default());
        assert_eq!(suite.expectations.len(), 50, "DDC966-DDC1015 must produce 50 tests");
    }
}
