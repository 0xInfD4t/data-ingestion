use indexmap::IndexMap;
use serde_json::json;

use crate::config::DqConfig;
use crate::expectations::{ExpectationConfig, ExpectationMeta, GeneratedFrom};
use crate::suites::{fmt_test_id, SuiteGenerator};

pub struct DataSensitivitySuite;

impl SuiteGenerator for DataSensitivitySuite {
    fn suite_name(&self) -> &str { "data_sensitivity_suite" }
    fn category(&self) -> &str { "sensitivity" }
    fn test_id_prefix(&self) -> &str { "DS" }
    fn test_id_start(&self) -> usize { 616 }

    fn generate(&self, _config: &DqConfig) -> Vec<ExpectationConfig> {
        let suite = self.suite_name();
        let cat = self.category();
        let pfx = self.test_id_prefix();
        let mut e = Vec::new();

        // DS616-DS640: PII masking checks (25 tests)
        let pii_fields = [
            "ssn", "social_security_number", "tax_id", "ein", "itin",
            "credit_card_number", "card_number", "cc_num", "bank_account_number", "routing_number",
            "driver_license_number", "passport_number", "national_id", "voter_id", "military_id",
            "patient_name", "member_name", "subscriber_name", "insured_name", "guarantor_name",
            "date_of_birth", "birth_date", "dob", "mother_maiden_name", "biometric_id",
        ];
        for (i, col) in pii_fields.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            e.push(ExpectationConfig {
                expectation_type: "expect_column_values_to_be_masked_pii".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 616 + i),
                    category: cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: Some(col.to_string()),
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }

        // DS641-DS655: Encryption checks for restricted fields (15 tests)
        let encrypted_fields = [
            "ssn_encrypted", "credit_card_encrypted", "bank_account_encrypted",
            "password_hash", "api_key", "secret_key", "private_key", "access_token",
            "refresh_token", "session_token", "auth_token", "encryption_key",
            "signing_key", "certificate_data", "biometric_template",
        ];
        for (i, col) in encrypted_fields.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            e.push(ExpectationConfig {
                expectation_type: "expect_column_values_to_be_encrypted".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 641 + i),
                    category: cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: Some(col.to_string()),
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }

        // DS656-DS665: No-PII checks for non-PII fields (10 tests)
        let non_pii_fields = [
            "diagnosis_code", "procedure_code", "drug_ndc", "revenue_code",
            "drg_code", "cpt_code", "hcpcs_code", "icd10_code", "loinc_code", "snomed_code",
        ];
        for (i, col) in non_pii_fields.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            e.push(ExpectationConfig {
                expectation_type: "expect_column_values_to_not_contain_pii".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 656 + i),
                    category: cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: Some(col.to_string()),
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }

        debug_assert_eq!(e.len(), 50, "DataSensitivitySuite must produce 50 tests");
        e
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::suites::SuiteGenerator;

    #[test]
    fn test_data_sensitivity_suite_count() {
        let suite = DataSensitivitySuite.build_suite(&DqConfig::default());
        assert_eq!(suite.expectations.len(), 50, "DS616-DS665 must produce 50 tests");
    }
}
