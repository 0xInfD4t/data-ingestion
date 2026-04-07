use indexmap::IndexMap;
use serde_json::json;

use crate::config::DqConfig;
use crate::expectations::{ExpectationConfig, ExpectationMeta, GeneratedFrom};
use crate::suites::{fmt_test_id, SuiteGenerator};

pub struct DataValiditySuite;

impl SuiteGenerator for DataValiditySuite {
    fn suite_name(&self) -> &str { "data_validity_suite" }
    fn category(&self) -> &str { "validity" }
    fn test_id_prefix(&self) -> &str { "DV" }
    fn test_id_start(&self) -> usize { 1 }

    fn generate(&self, _config: &DqConfig) -> Vec<ExpectationConfig> {
        let suite = self.suite_name();
        let cat = self.category();
        let pfx = self.test_id_prefix();
        let mut e = Vec::new();

        // DV001-DV005: Schema column set validations for standard healthcare schemas
        let schemas = [
            ("HL7", vec!["patient_id","encounter_id","message_type","event_type","sending_facility","receiving_facility","message_datetime","patient_name","patient_dob","patient_gender"]),
            ("NCQA", vec!["member_id","plan_id","measure_id","numerator","denominator","exclusion","reporting_period","provider_npi","facility_id","data_source"]),
            ("Medicare", vec!["beneficiary_id","claim_id","claim_type","service_date","provider_npi","diagnosis_code","procedure_code","allowed_amount","paid_amount","place_of_service"]),
            ("Medicaid", vec!["recipient_id","claim_id","service_date","provider_id","diagnosis_code","procedure_code","paid_amount","program_type","eligibility_category","county_code"]),
            ("Hybrid", vec!["record_id","source_system","patient_id","encounter_id","service_date","provider_id","diagnosis_code","procedure_code","amount","status"]),
        ];
        for (i, (schema_name, cols)) in schemas.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column_set".to_string(), json!(cols));
            kwargs.insert("exact_match".to_string(), json!(false));
            e.push(ExpectationConfig {
                expectation_type: "expect_table_columns_to_match_set".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, i + 1),
                    category: cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: None,
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
            let _ = schema_name;
        }

        // DV006-DV020: URN format validations
        let urn_fields = [
            ("patient_id", r"^[A-Z]{2}\d{9}$"),
            ("encounter_id", r"^ENC-\d{8}-[A-Z0-9]{4}$"),
            ("member_id", r"^MBR-\d{10}$"),
            ("claim_id", r"^CLM-\d{12}$"),
            ("provider_npi", r"^\d{10}$"),
            ("facility_id", r"^FAC-[A-Z]{3}-\d{6}$"),
            ("diagnosis_code", r"^[A-Z]\d{2}(\.\d{1,4})?$"),
            ("procedure_code", r"^\d{5}[A-Z0-9]?$"),
            ("beneficiary_id", r"^\d{11}[A-Z]$"),
            ("recipient_id", r"^[A-Z]{2}\d{8}$"),
            ("plan_id", r"^PLN-\d{8}$"),
            ("measure_id", r"^[A-Z]{3,6}-\d{4}$"),
            ("message_type", r"^[A-Z]{3}_[A-Z]\d{2}$"),
            ("record_id", r"^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$"),
            ("source_system", r"^[A-Z][A-Z0-9_]{2,19}$"),
        ];
        for (i, (col, regex)) in urn_fields.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            kwargs.insert("regex".to_string(), json!(regex));
            e.push(ExpectationConfig {
                expectation_type: "expect_column_values_to_match_regex".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 6 + i),
                    category: cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: Some(col.to_string()),
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }

        // DV021-DV040: Entity validation (name, address, DOB, SSN, etc.)
        let entity_checks: &[(&str, &str, &str)] = &[
            ("patient_name", "expect_column_values_to_not_be_null", "completeness"),
            ("patient_dob", "expect_column_values_to_be_dateutil_parseable", "validity"),
            ("patient_gender", "expect_column_values_to_be_in_set", "validity"),
            ("ssn", "expect_column_values_to_be_valid_ssn", "validity"),
            ("phone_number", "expect_column_values_to_be_valid_phone_number", "validity"),
            ("email_address", "expect_column_values_to_be_valid_email", "validity"),
            ("postal_code", "expect_column_values_to_be_valid_postal_code", "validity"),
            ("address_line1", "expect_column_values_to_not_be_null", "completeness"),
            ("city", "expect_column_values_to_not_be_null", "completeness"),
            ("state_code", "expect_column_values_to_match_regex", "validity"),
            ("country_code", "expect_column_values_to_match_regex", "validity"),
            ("date_of_birth", "expect_column_values_to_be_dateutil_parseable", "validity"),
            ("admission_date", "expect_column_values_to_be_dateutil_parseable", "validity"),
            ("discharge_date", "expect_column_values_to_be_dateutil_parseable", "validity"),
            ("service_date", "expect_column_values_to_be_dateutil_parseable", "validity"),
            ("created_at", "expect_column_values_to_be_dateutil_parseable", "validity"),
            ("updated_at", "expect_column_values_to_be_dateutil_parseable", "validity"),
            ("first_name", "expect_column_value_lengths_to_be_between", "validity"),
            ("last_name", "expect_column_value_lengths_to_be_between", "validity"),
            ("middle_name", "expect_column_value_lengths_to_be_between", "validity"),
        ];
        for (i, (col, exp_type, exp_cat)) in entity_checks.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            match *exp_type {
                "expect_column_values_to_be_in_set" => {
                    kwargs.insert("value_set".to_string(), json!(["M","F","U","O"]));
                }
                "expect_column_values_to_match_regex" if *col == "state_code" => {
                    kwargs.insert("regex".to_string(), json!(r"^[A-Z]{2}$"));
                }
                "expect_column_values_to_match_regex" => {
                    kwargs.insert("regex".to_string(), json!(r"^[A-Z]{2,3}$"));
                }
                "expect_column_value_lengths_to_be_between" => {
                    kwargs.insert("min_value".to_string(), json!(1));
                    kwargs.insert("max_value".to_string(), json!(100));
                }
                _ => {}
            }
            e.push(ExpectationConfig {
                expectation_type: exp_type.to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 21 + i),
                    category: exp_cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: Some(col.to_string()),
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }

        // DV041-DV060: Attribute validation (age, weight, BMI, vitals)
        let attr_checks: &[(&str, f64, f64)] = &[
            ("age", 0.0, 150.0),
            ("weight_kg", 0.5, 500.0),
            ("height_cm", 30.0, 300.0),
            ("bmi", 10.0, 80.0),
            ("systolic_bp", 50.0, 300.0),
            ("diastolic_bp", 20.0, 200.0),
            ("heart_rate", 20.0, 300.0),
            ("temperature_c", 30.0, 45.0),
            ("oxygen_saturation", 50.0, 100.0),
            ("respiratory_rate", 4.0, 60.0),
            ("glucose_mg_dl", 20.0, 1000.0),
            ("hemoglobin_g_dl", 1.0, 25.0),
            ("creatinine_mg_dl", 0.1, 30.0),
            ("cholesterol_mg_dl", 50.0, 1000.0),
            ("triglycerides_mg_dl", 10.0, 5000.0),
            ("sodium_meq_l", 100.0, 200.0),
            ("potassium_meq_l", 1.0, 10.0),
            ("wbc_k_ul", 0.1, 100.0),
            ("platelet_k_ul", 10.0, 2000.0),
            ("inr", 0.5, 20.0),
        ];
        for (i, (col, min_v, max_v)) in attr_checks.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            kwargs.insert("min_value".to_string(), json!(min_v));
            kwargs.insert("max_value".to_string(), json!(max_v));
            e.push(ExpectationConfig {
                expectation_type: "expect_column_values_to_be_between".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 41 + i),
                    category: cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: Some(col.to_string()),
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }

        // DV061-DV075: Regex pattern variants
        let regex_checks: &[(&str, &str)] = &[
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
            ("place_of_service", r"^\d{2}$"),
            ("type_of_bill", r"^\d{3,4}$"),
            ("modifier_code", r"^[A-Z0-9]{2}$"),
            ("taxonomy_code", r"^\d{10}[A-Z]$"),
            ("zip_code", r"^\d{5}(-\d{4})?$"),
        ];
        for (i, (col, regex)) in regex_checks.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            kwargs.insert("regex".to_string(), json!(regex));
            e.push(ExpectationConfig {
                expectation_type: "expect_column_values_to_match_regex".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 61 + i),
                    category: cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: Some(col.to_string()),
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }

        // DV076-DV095: Cross-field and semantic checks
        let semantic_checks: &[(&str, &str, &str)] = &[
            ("claim_status", "expect_column_values_to_be_in_set", "validity"),
            ("encounter_type", "expect_column_values_to_be_in_set", "validity"),
            ("admission_type", "expect_column_values_to_be_in_set", "validity"),
            ("discharge_disposition", "expect_column_values_to_be_in_set", "validity"),
            ("claim_type", "expect_column_values_to_be_in_set", "validity"),
            ("coverage_type", "expect_column_values_to_be_in_set", "validity"),
            ("benefit_type", "expect_column_values_to_be_in_set", "validity"),
            ("network_indicator", "expect_column_values_to_be_in_set", "validity"),
            ("authorization_status", "expect_column_values_to_be_in_set", "validity"),
            ("payment_method", "expect_column_values_to_be_in_set", "validity"),
            ("allowed_amount", "expect_column_values_to_be_between", "validity"),
            ("paid_amount", "expect_column_values_to_be_between", "validity"),
            ("copay_amount", "expect_column_values_to_be_between", "validity"),
            ("deductible_amount", "expect_column_values_to_be_between", "validity"),
            ("coinsurance_amount", "expect_column_values_to_be_between", "validity"),
            ("units_of_service", "expect_column_values_to_be_between", "validity"),
            ("days_supply", "expect_column_values_to_be_between", "validity"),
            ("quantity_dispensed", "expect_column_values_to_be_between", "validity"),
            ("refill_number", "expect_column_values_to_be_between", "validity"),
            ("days_in_hospital", "expect_column_values_to_be_between", "validity"),
        ];
        for (i, (col, exp_type, exp_cat)) in semantic_checks.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            match *exp_type {
                "expect_column_values_to_be_in_set" => {
                    let values = match *col {
                        "claim_status" => json!(["paid","denied","pending","adjusted","voided"]),
                        "encounter_type" => json!(["inpatient","outpatient","emergency","observation","telehealth"]),
                        "admission_type" => json!(["emergency","urgent","elective","newborn","trauma"]),
                        "discharge_disposition" => json!(["home","snf","rehab","expired","ama","transfer"]),
                        "claim_type" => json!(["professional","institutional","dental","pharmacy","vision"]),
                        "coverage_type" => json!(["medical","dental","vision","pharmacy","behavioral"]),
                        "benefit_type" => json!(["inpatient","outpatient","professional","ancillary","pharmacy"]),
                        "network_indicator" => json!(["in_network","out_of_network","unknown"]),
                        "authorization_status" => json!(["approved","denied","pending","not_required"]),
                        "payment_method" => json!(["eft","check","virtual_card","capitation"]),
                        _ => json!(["Y","N"]),
                    };
                    kwargs.insert("value_set".to_string(), values);
                }
                "expect_column_values_to_be_between" => {
                    kwargs.insert("min_value".to_string(), json!(0.0));
                    kwargs.insert("max_value".to_string(), json!(1000000.0));
                }
                _ => {}
            }
            e.push(ExpectationConfig {
                expectation_type: exp_type.to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 76 + i),
                    category: exp_cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: Some(col.to_string()),
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }

        debug_assert_eq!(e.len(), 95, "DataValiditySuite must produce 95 tests");
        e
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::suites::SuiteGenerator;

    #[test]
    fn test_data_validity_suite_count() {
        let suite = DataValiditySuite.build_suite(&DqConfig::default());
        assert_eq!(suite.expectations.len(), 95, "DV001-DV095 must produce 95 tests");
    }
}
