use indexmap::IndexMap;
use serde_json::json;

use crate::config::DqConfig;
use crate::expectations::{ExpectationConfig, ExpectationMeta, GeneratedFrom};
use crate::suites::{fmt_test_id, SuiteGenerator};

pub struct DataFormatConsistencySuite;

impl SuiteGenerator for DataFormatConsistencySuite {
    fn suite_name(&self) -> &str { "data_format_consistency_suite" }
    fn category(&self) -> &str { "format_consistency" }
    fn test_id_prefix(&self) -> &str { "DFC" }
    fn test_id_start(&self) -> usize { 806 }

    fn generate(&self, _config: &DqConfig) -> Vec<ExpectationConfig> {
        let suite = self.suite_name();
        let cat = self.category();
        let pfx = self.test_id_prefix();
        let mut e = Vec::new();

        // DFC806-DFC845: Regex format checks (40 tests)
        let regex_formats: &[(&str, &str)] = &[
            ("npi", r"^\d{10}$"),
            ("tin", r"^\d{2}-\d{7}$"),
            ("dea_number", r"^[A-Z]{2}\d{7}$"),
            ("upin", r"^[A-Z]\d{5}$"),
            ("ssn", r"^\d{3}-\d{2}-\d{4}$"),
            ("icd10_code", r"^[A-Z]\d{2}(\.\d{1,4})?$"),
            ("icd9_code", r"^\d{3}(\.\d{1,2})?$"),
            ("cpt_code", r"^\d{5}[A-Z0-9]?$"),
            ("hcpcs_code", r"^[A-Z]\d{4}$"),
            ("ndc_code", r"^\d{5}-\d{4}-\d{2}$"),
            ("rxnorm_code", r"^\d{1,7}$"),
            ("snomed_code", r"^\d{6,18}$"),
            ("loinc_code", r"^\d{1,5}-\d$"),
            ("drg_code", r"^\d{3}$"),
            ("revenue_code", r"^\d{4}$"),
            ("zip_code", r"^\d{5}(-\d{4})?$"),
            ("phone_number", r"^\+?1?\s?\(?\d{3}\)?[\s.-]?\d{3}[\s.-]?\d{4}$"),
            ("email_address", r"^[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}$"),
            ("url", r"^https?://[^\s/$.?#].[^\s]*$"),
            ("ip_address", r"^(\d{1,3}\.){3}\d{1,3}$"),
            ("mac_address", r"^([0-9A-Fa-f]{2}[:-]){5}([0-9A-Fa-f]{2})$"),
            ("uuid", r"^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$"),
            ("hl7_message_id", r"^[A-Z]{3}_[A-Z]\d{2}$"),
            ("x12_transaction_set", r"^\d{3}$"),
            ("fhir_resource_id", r"^[A-Za-z0-9\-\.]{1,64}$"),
            ("taxonomy_code", r"^\d{10}[A-Z]$"),
            ("place_of_service", r"^\d{2}$"),
            ("type_of_bill", r"^\d{3,4}$"),
            ("modifier_code", r"^[A-Z0-9]{2}$"),
            ("occurrence_code", r"^\d{2}$"),
            ("condition_code", r"^\d{2}$"),
            ("value_code", r"^\d{2}$"),
            ("adjustment_reason_code", r"^\d{1,3}$"),
            ("remark_code", r"^[A-Z]{1,2}\d{1,4}$"),
            ("group_number", r"^[A-Z0-9]{1,20}$"),
            ("member_number", r"^[A-Z0-9]{1,20}$"),
            ("policy_number", r"^[A-Z0-9\-]{1,30}$"),
            ("authorization_number", r"^[A-Z0-9]{6,20}$"),
            ("referral_number", r"^[A-Z0-9]{6,20}$"),
            ("batch_id", r"^[A-Z0-9\-]{8,36}$"),
        ];
        for (i, (col, regex)) in regex_formats.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            kwargs.insert("regex".to_string(), json!(regex));
            e.push(ExpectationConfig {
                expectation_type: "expect_column_values_to_match_regex".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 806 + i),
                    category: cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: Some(col.to_string()),
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }

        // DFC846-DFC875: Length checks (30 tests)
        let length_checks: &[(&str, usize, usize)] = &[
            ("npi", 10, 10),
            ("tin", 10, 10),
            ("ssn", 11, 11),
            ("zip_code", 5, 10),
            ("icd10_code", 3, 8),
            ("cpt_code", 5, 6),
            ("hcpcs_code", 5, 5),
            ("ndc_code", 13, 13),
            ("drg_code", 3, 3),
            ("revenue_code", 4, 4),
            ("place_of_service", 2, 2),
            ("modifier_code", 2, 2),
            ("state_code", 2, 2),
            ("country_code", 2, 3),
            ("currency_code", 3, 3),
            ("language_code", 2, 3),
            ("patient_name", 1, 200),
            ("provider_name", 1, 200),
            ("facility_name", 1, 200),
            ("address_line1", 1, 200),
            ("city", 1, 100),
            ("description", 0, 2000),
            ("notes", 0, 5000),
            ("comments", 0, 5000),
            ("reason_code", 1, 10),
            ("status_code", 1, 20),
            ("type_code", 1, 20),
            ("category_code", 1, 20),
            ("subcategory_code", 1, 20),
            ("source_system", 1, 50),
        ];
        for (i, (col, min_len, max_len)) in length_checks.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            kwargs.insert("min_value".to_string(), json!(min_len));
            kwargs.insert("max_value".to_string(), json!(max_len));
            e.push(ExpectationConfig {
                expectation_type: "expect_column_value_lengths_to_be_between".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 846 + i),
                    category: cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: Some(col.to_string()),
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }

        debug_assert_eq!(e.len(), 70, "DataFormatConsistencySuite must produce 70 tests");
        e
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::suites::SuiteGenerator;

    #[test]
    fn test_data_format_consistency_suite_count() {
        let suite = DataFormatConsistencySuite.build_suite(&DqConfig::default());
        assert_eq!(suite.expectations.len(), 70, "DFC806-DFC875 must produce 70 tests");
    }
}
