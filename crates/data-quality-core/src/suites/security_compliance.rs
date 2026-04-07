use indexmap::IndexMap;
use serde_json::json;

use crate::config::DqConfig;
use crate::expectations::{ExpectationConfig, ExpectationMeta, GeneratedFrom};
use crate::suites::{fmt_test_id, SuiteGenerator};

pub struct SecurityComplianceSuite;

impl SuiteGenerator for SecurityComplianceSuite {
    fn suite_name(&self) -> &str { "security_compliance_suite" }
    fn category(&self) -> &str { "security" }
    fn test_id_prefix(&self) -> &str { "SC" }
    fn test_id_start(&self) -> usize { 1176 }

    fn generate(&self, _config: &DqConfig) -> Vec<ExpectationConfig> {
        let suite = self.suite_name();
        let cat = self.category();
        let pfx = self.test_id_prefix();
        let mut e = Vec::new();

        // SC1176-SC1225: PII masking checks (50 tests)
        let pii_mask_fields = [
            "ssn", "social_security_number", "tax_id", "ein", "itin",
            "credit_card_number", "card_number", "cc_num", "bank_account_number", "routing_number",
            "driver_license_number", "passport_number", "national_id", "voter_id", "military_id",
            "patient_name", "member_name", "subscriber_name", "insured_name", "guarantor_name",
            "date_of_birth", "birth_date", "dob", "mother_maiden_name", "biometric_id",
            "fingerprint_hash", "retina_scan_hash", "face_recognition_hash", "voice_print_hash", "dna_hash",
            "home_address", "home_phone", "personal_email", "personal_cell", "emergency_contact",
            "employer_name", "employer_address", "school_name", "student_id", "employee_id",
            "insurance_id", "policy_number", "group_number", "subscriber_id", "dependent_id",
            "medical_record_number", "health_plan_id", "account_number", "certificate_number", "license_plate",
        ];
        for (i, col) in pii_mask_fields.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            e.push(ExpectationConfig {
                expectation_type: "expect_column_values_to_be_masked_pii".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 1176 + i),
                    category: cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: Some(col.to_string()),
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }

        // SC1226-SC1255: Encryption checks (30 tests)
        let encrypted_fields = [
            "ssn_encrypted", "credit_card_encrypted", "bank_account_encrypted",
            "password_hash", "api_key", "secret_key", "private_key", "access_token",
            "refresh_token", "session_token", "auth_token", "encryption_key",
            "signing_key", "certificate_data", "biometric_template",
            "oauth_token", "jwt_token", "saml_assertion", "kerberos_ticket", "ldap_password",
            "database_password", "service_account_key", "ssh_private_key", "ssl_certificate", "tls_key",
            "hmac_secret", "aes_key", "rsa_private_key", "ecdsa_private_key", "pgp_private_key",
        ];
        for (i, col) in encrypted_fields.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            e.push(ExpectationConfig {
                expectation_type: "expect_column_values_to_be_encrypted".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 1226 + i),
                    category: cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: Some(col.to_string()),
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }

        // SC1256-SC1275: No-PII in non-PII fields (20 tests)
        let non_pii_fields = [
            "diagnosis_code", "procedure_code", "drug_ndc", "revenue_code",
            "drg_code", "cpt_code", "hcpcs_code", "icd10_code", "loinc_code", "snomed_code",
            "place_of_service", "type_of_bill", "modifier_code", "taxonomy_code", "specialty_code",
            "claim_type", "encounter_type", "benefit_type", "coverage_type", "program_type",
        ];
        for (i, col) in non_pii_fields.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            e.push(ExpectationConfig {
                expectation_type: "expect_column_values_to_not_contain_pii".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 1256 + i),
                    category: cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: Some(col.to_string()),
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }

        // SC1276-SC1295: HIPAA compliance - PHI field checks (20 tests)
        let phi_fields = [
            "patient_name", "date_of_birth", "admission_date", "discharge_date", "death_date",
            "phone_number", "fax_number", "email_address", "ssn", "medical_record_number",
            "health_plan_id", "account_number", "certificate_number", "vehicle_id", "device_id",
            "web_url", "ip_address", "biometric_id", "full_face_photo", "geographic_data",
        ];
        for (i, col) in phi_fields.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            e.push(ExpectationConfig {
                expectation_type: "expect_column_values_to_be_masked_pii".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 1276 + i),
                    category: "hipaa_phi".to_string(),
                    suite: suite.to_string(),
                    contract_field: Some(col.to_string()),
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }

        // SC1296-SC1315: PCI DSS compliance checks (20 tests)
        let pci_fields = [
            "primary_account_number", "card_holder_name", "expiration_date", "service_code", "cvv",
            "pin_block", "track_data", "chip_data", "magnetic_stripe", "card_verification_value",
            "card_security_code", "card_validation_code", "card_identification_number", "card_unique_code", "card_authentication_value",
            "issuer_identification_number", "bank_identification_number", "card_brand", "card_type", "card_network",
        ];
        for (i, col) in pci_fields.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            e.push(ExpectationConfig {
                expectation_type: "expect_column_values_to_be_encrypted".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 1296 + i),
                    category: "pci_dss".to_string(),
                    suite: suite.to_string(),
                    contract_field: Some(col.to_string()),
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }

        // SC1316-SC1328: Audit and access control checks (13 tests)
        let audit_fields = [
            "created_by", "updated_by", "deleted_by", "approved_by", "reviewed_by",
            "access_level", "permission_set", "role_name", "user_group", "data_classification",
            "retention_policy", "disposal_method", "consent_version",
        ];
        for (i, col) in audit_fields.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            e.push(ExpectationConfig {
                expectation_type: "expect_column_values_to_not_be_null".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 1316 + i),
                    category: "audit_control".to_string(),
                    suite: suite.to_string(),
                    contract_field: Some(col.to_string()),
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }

        debug_assert_eq!(e.len(), 153, "SecurityComplianceSuite must produce 153 tests");
        e
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::suites::SuiteGenerator;

    #[test]
    fn test_security_compliance_suite_count() {
        let suite = SecurityComplianceSuite.build_suite(&DqConfig::default());
        assert_eq!(suite.expectations.len(), 153, "SC1176-SC1328 must produce 153 tests");
    }
}
