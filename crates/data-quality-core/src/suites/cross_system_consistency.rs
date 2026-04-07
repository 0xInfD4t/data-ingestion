use indexmap::IndexMap;
use serde_json::json;

use crate::config::DqConfig;
use crate::expectations::{ExpectationConfig, ExpectationMeta, GeneratedFrom};
use crate::suites::{fmt_test_id, SuiteGenerator};

pub struct CrossSystemConsistencySuite;

impl SuiteGenerator for CrossSystemConsistencySuite {
    fn suite_name(&self) -> &str { "cross_system_consistency_suite" }
    fn category(&self) -> &str { "cross_system" }
    fn test_id_prefix(&self) -> &str { "CSC" }
    fn test_id_start(&self) -> usize { 1016 }

    fn generate(&self, _config: &DqConfig) -> Vec<ExpectationConfig> {
        let suite = self.suite_name();
        let cat = self.category();
        let pfx = self.test_id_prefix();
        let mut e = Vec::new();

        // CSC1016-CSC1065: Cross-system not-null checks (50 tests)
        let cross_system_fields = [
            // EHR fields
            "ehr_patient_id", "ehr_encounter_id", "ehr_provider_id", "ehr_facility_id", "ehr_diagnosis_code",
            "ehr_procedure_code", "ehr_medication_id", "ehr_lab_order_id", "ehr_imaging_order_id", "ehr_note_id",
            // Claims fields
            "claims_patient_id", "claims_claim_id", "claims_provider_npi", "claims_service_date", "claims_diagnosis_code",
            "claims_procedure_code", "claims_allowed_amount", "claims_paid_amount", "claims_plan_id", "claims_member_id",
            // Pharmacy fields
            "rx_patient_id", "rx_prescription_id", "rx_ndc_code", "rx_fill_date", "rx_days_supply",
            "rx_quantity", "rx_pharmacy_id", "rx_prescriber_npi", "rx_paid_amount", "rx_plan_id",
            // Lab fields
            "lab_patient_id", "lab_order_id", "lab_loinc_code", "lab_result_value", "lab_result_date",
            "lab_reference_range", "lab_abnormal_flag", "lab_performing_lab_id", "lab_ordering_provider_npi", "lab_specimen_type",
            // Registry fields
            "registry_patient_id", "registry_condition_code", "registry_enrollment_date", "registry_status", "registry_program_id",
            "registry_care_manager_id", "registry_risk_tier", "registry_last_contact_date", "registry_next_review_date", "registry_source",
        ];
        for (i, col) in cross_system_fields.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            e.push(ExpectationConfig {
                expectation_type: "expect_column_values_to_not_be_null".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 1016 + i),
                    category: cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: Some(col.to_string()),
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }

        // CSC1066-CSC1115: Cross-system type and format checks (50 tests)
        let cross_system_formats: &[(&str, &str, &str)] = &[
            ("ehr_patient_id", "expect_column_values_to_match_regex", r"^EHR-\d{8}$"),
            ("ehr_encounter_id", "expect_column_values_to_match_regex", r"^ENC-\d{10}$"),
            ("ehr_provider_id", "expect_column_values_to_match_regex", r"^\d{10}$"),
            ("claims_patient_id", "expect_column_values_to_match_regex", r"^[A-Z]{2}\d{9}$"),
            ("claims_claim_id", "expect_column_values_to_match_regex", r"^CLM-\d{12}$"),
            ("claims_provider_npi", "expect_column_values_to_match_regex", r"^\d{10}$"),
            ("rx_patient_id", "expect_column_values_to_match_regex", r"^[A-Z]{2}\d{9}$"),
            ("rx_prescription_id", "expect_column_values_to_match_regex", r"^RX-\d{10}$"),
            ("rx_ndc_code", "expect_column_values_to_match_regex", r"^\d{5}-\d{4}-\d{2}$"),
            ("lab_patient_id", "expect_column_values_to_match_regex", r"^[A-Z]{2}\d{9}$"),
            ("lab_order_id", "expect_column_values_to_match_regex", r"^LAB-\d{10}$"),
            ("lab_loinc_code", "expect_column_values_to_match_regex", r"^\d{1,5}-\d$"),
            ("registry_patient_id", "expect_column_values_to_match_regex", r"^[A-Z]{2}\d{9}$"),
            ("registry_condition_code", "expect_column_values_to_match_regex", r"^[A-Z]\d{2}(\.\d{1,4})?$"),
            ("ehr_diagnosis_code", "expect_column_values_to_match_regex", r"^[A-Z]\d{2}(\.\d{1,4})?$"),
            ("ehr_procedure_code", "expect_column_values_to_match_regex", r"^\d{5}[A-Z0-9]?$"),
            ("claims_diagnosis_code", "expect_column_values_to_match_regex", r"^[A-Z]\d{2}(\.\d{1,4})?$"),
            ("claims_procedure_code", "expect_column_values_to_match_regex", r"^\d{5}[A-Z0-9]?$"),
            ("ehr_encounter_id", "expect_column_values_to_be_unique", ""),
            ("claims_claim_id", "expect_column_values_to_be_unique", ""),
            ("rx_prescription_id", "expect_column_values_to_be_unique", ""),
            ("lab_order_id", "expect_column_values_to_be_unique", ""),
            ("ehr_patient_id", "expect_column_values_to_be_unique", ""),
            ("claims_allowed_amount", "expect_column_values_to_be_between", "0,10000000"),
            ("claims_paid_amount", "expect_column_values_to_be_between", "0,10000000"),
            ("rx_paid_amount", "expect_column_values_to_be_between", "0,100000"),
            ("rx_days_supply", "expect_column_values_to_be_between", "1,365"),
            ("rx_quantity", "expect_column_values_to_be_between", "0,10000"),
            ("lab_result_value", "expect_column_values_to_not_be_null", ""),
            ("lab_result_date", "expect_column_values_to_be_dateutil_parseable", ""),
            ("ehr_note_id", "expect_column_values_to_not_be_null", ""),
            ("registry_enrollment_date", "expect_column_values_to_be_dateutil_parseable", ""),
            ("registry_last_contact_date", "expect_column_values_to_be_dateutil_parseable", ""),
            ("registry_next_review_date", "expect_column_values_to_be_dateutil_parseable", ""),
            ("registry_status", "expect_column_values_to_be_in_set", "active,inactive,pending,closed"),
            ("registry_risk_tier", "expect_column_values_to_be_in_set", "low,medium,high,very_high,critical"),
            ("lab_abnormal_flag", "expect_column_values_to_be_in_set", "N,L,H,LL,HH,A,AA"),
            ("lab_specimen_type", "expect_column_values_to_be_in_set", "blood,urine,serum,plasma,csf,tissue,swab,other"),
            ("claims_service_date", "expect_column_values_to_be_dateutil_parseable", ""),
            ("rx_fill_date", "expect_column_values_to_be_dateutil_parseable", ""),
            ("ehr_facility_id", "expect_column_values_to_not_be_null", ""),
            ("claims_plan_id", "expect_column_values_to_not_be_null", ""),
            ("rx_plan_id", "expect_column_values_to_not_be_null", ""),
            ("registry_program_id", "expect_column_values_to_not_be_null", ""),
            ("registry_care_manager_id", "expect_column_values_to_not_be_null", ""),
            ("ehr_medication_id", "expect_column_values_to_not_be_null", ""),
            ("ehr_imaging_order_id", "expect_column_values_to_not_be_null", ""),
            ("lab_performing_lab_id", "expect_column_values_to_not_be_null", ""),
            ("lab_ordering_provider_npi", "expect_column_values_to_match_regex", r"^\d{10}$"),
            ("rx_prescriber_npi", "expect_column_values_to_match_regex", r"^\d{10}$"),
        ];
        for (i, (col, exp_type, extra)) in cross_system_formats.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            match *exp_type {
                "expect_column_values_to_match_regex" => {
                    kwargs.insert("regex".to_string(), json!(extra));
                }
                "expect_column_values_to_be_between" => {
                    let parts: Vec<&str> = extra.split(',').collect();
                    if parts.len() == 2 {
                        let min: f64 = parts[0].parse().unwrap_or(0.0);
                        let max: f64 = parts[1].parse().unwrap_or(1_000_000.0);
                        kwargs.insert("min_value".to_string(), json!(min));
                        kwargs.insert("max_value".to_string(), json!(max));
                    }
                }
                "expect_column_values_to_be_in_set" => {
                    let values: Vec<&str> = extra.split(',').collect();
                    kwargs.insert("value_set".to_string(), json!(values));
                }
                _ => {}
            }
            e.push(ExpectationConfig {
                expectation_type: exp_type.to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 1066 + i),
                    category: cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: Some(col.to_string()),
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }

        debug_assert_eq!(e.len(), 100, "CrossSystemConsistencySuite must produce 100 tests");
        e
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::suites::SuiteGenerator;

    #[test]
    fn test_cross_system_consistency_suite_count() {
        let suite = CrossSystemConsistencySuite.build_suite(&DqConfig::default());
        assert_eq!(suite.expectations.len(), 100, "CSC1016-CSC1115 must produce 100 tests");
    }
}
