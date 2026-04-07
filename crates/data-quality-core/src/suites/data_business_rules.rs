use indexmap::IndexMap;
use serde_json::json;

use crate::config::DqConfig;
use crate::expectations::{ExpectationConfig, ExpectationMeta, GeneratedFrom};
use crate::suites::{fmt_test_id, SuiteGenerator};

pub struct DataBusinessRulesSuite;

impl SuiteGenerator for DataBusinessRulesSuite {
    fn suite_name(&self) -> &str { "data_business_rules_suite" }
    fn category(&self) -> &str { "business_rules" }
    fn test_id_prefix(&self) -> &str { "DBR" }
    fn test_id_start(&self) -> usize { 726 }

    fn generate(&self, _config: &DqConfig) -> Vec<ExpectationConfig> {
        let suite = self.suite_name();
        let cat = self.category();
        let pfx = self.test_id_prefix();
        let mut e = Vec::new();

        // DBR726-DBR765: Allowed value set checks for business codes (40 tests)
        let code_sets: &[(&str, &[&str])] = &[
            ("claim_status", &["paid","denied","pending","adjusted","voided","suspended"]),
            ("claim_type", &["professional","institutional","dental","pharmacy","vision","behavioral"]),
            ("encounter_type", &["inpatient","outpatient","emergency","observation","telehealth","home_health"]),
            ("admission_type", &["emergency","urgent","elective","newborn","trauma","transfer"]),
            ("discharge_disposition", &["home","snf","rehab","expired","ama","transfer","hospice"]),
            ("coverage_type", &["medical","dental","vision","pharmacy","behavioral","supplemental"]),
            ("benefit_type", &["inpatient","outpatient","professional","ancillary","pharmacy","mental_health"]),
            ("network_indicator", &["in_network","out_of_network","unknown","not_applicable"]),
            ("authorization_status", &["approved","denied","pending","not_required","cancelled","expired"]),
            ("payment_method", &["eft","check","virtual_card","capitation","withheld"]),
            ("payer_type", &["primary","secondary","tertiary","self_pay","charity"]),
            ("claim_frequency", &["1","2","3","4","5","6","7","8","9"]),
            ("place_of_service", &["11","12","21","22","23","24","31","32","41","42","49","50","51","52","53","54","55","56","57","58","60","61","62","65","71","72","81","99"]),
            ("type_of_bill", &["011","012","013","021","022","023","031","032","033","041","042","043","071","072","073","081","082","083"]),
            ("revenue_code", &["0100","0110","0120","0130","0200","0210","0220","0250","0260","0270","0300","0301","0302","0303","0304","0305","0306","0307","0308","0309"]),
            ("gender_code", &["M","F","U","O","X"]),
            ("race_code", &["1","2","3","4","5","6","7","8","9"]),
            ("ethnicity_code", &["H","N","U"]),
            ("marital_status", &["S","M","D","W","U","P","A"]),
            ("tobacco_use", &["current","former","never","unknown"]),
            ("alcohol_use", &["none","light","moderate","heavy","unknown"]),
            ("drug_use", &["none","current","former","unknown"]),
            ("employment_status", &["employed","unemployed","retired","student","disabled","unknown"]),
            ("insurance_type", &["commercial","medicare","medicaid","self_pay","other","tricare","chip"]),
            ("language_code", &["ENG","SPA","CHI","FRE","GER","ITA","POR","RUS","ARA","OTH"]),
            ("state_code", &["AL","AK","AZ","AR","CA","CO","CT","DE","FL","GA","HI","ID","IL","IN","IA","KS","KY","LA","ME","MD","MA","MI","MN","MS","MO","MT","NE","NV","NH","NJ","NM","NY","NC","ND","OH","OK","OR","PA","RI","SC","SD","TN","TX","UT","VT","VA","WA","WV","WI","WY","DC"]),
            ("country_code", &["US","CA","MX","GB","AU","DE","FR","JP","CN","IN"]),
            ("currency_code", &["USD","CAD","EUR","GBP","AUD","JPY","CNY","INR"]),
            ("time_zone", &["America/New_York","America/Chicago","America/Denver","America/Los_Angeles","America/Anchorage","Pacific/Honolulu"]),
            ("data_source", &["ehr","claims","pharmacy","lab","registry","survey","wearable","manual"]),
            ("record_type", &["original","adjustment","void","replacement","correction"]),
            ("submission_type", &["initial","resubmission","appeal","corrected","void"]),
            ("processing_status", &["pending","processing","completed","failed","cancelled","on_hold"]),
            ("priority_level", &["low","medium","high","critical","urgent"]),
            ("risk_level", &["low","medium","high","very_high","critical"]),
            ("care_setting", &["inpatient","outpatient","home","snf","ltc","hospice","palliative"]),
            ("program_type", &["disease_management","case_management","wellness","prevention","chronic_care","complex_care"]),
            ("outreach_method", &["phone","mail","email","text","portal","in_person","telehealth"]),
            ("consent_status", &["consented","declined","pending","revoked","expired","not_required"]),
            ("data_quality_flag", &["pass","fail","warning","review","unknown"]),
        ];
        for (i, (col, values)) in code_sets.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            kwargs.insert("value_set".to_string(), json!(values));
            e.push(ExpectationConfig {
                expectation_type: "expect_column_values_to_be_in_set".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 726 + i),
                    category: cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: Some(col.to_string()),
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }

        // DBR766-DBR805: Business rule range checks (40 tests)
        let rule_ranges: &[(&str, f64, f64)] = &[
            ("copay_percentage", 0.0, 100.0),
            ("coinsurance_percentage", 0.0, 100.0),
            ("deductible_percentage", 0.0, 100.0),
            ("out_of_pocket_max", 0.0, 100_000.0),
            ("deductible_max", 0.0, 50_000.0),
            ("premium_amount", 0.0, 50_000.0),
            ("capitation_pmpm", 0.0, 5_000.0),
            ("withhold_percentage", 0.0, 30.0),
            ("risk_corridor_percentage", 0.0, 50.0),
            ("mlr_percentage", 0.0, 100.0),
            ("admin_ratio", 0.0, 50.0),
            ("medical_loss_ratio", 50.0, 100.0),
            ("quality_bonus_percentage", 0.0, 20.0),
            ("penalty_percentage", 0.0, 10.0),
            ("interest_rate", 0.0, 30.0),
            ("discount_rate", 0.0, 50.0),
            ("inflation_rate", -5.0, 20.0),
            ("trend_factor", 0.5, 2.0),
            ("credibility_factor", 0.0, 1.0),
            ("experience_factor", 0.0, 5.0),
            ("age_factor", 0.1, 10.0),
            ("gender_factor", 0.5, 2.0),
            ("geographic_factor", 0.5, 3.0),
            ("industry_factor", 0.5, 3.0),
            ("group_size_factor", 0.5, 2.0),
            ("benefit_richness_factor", 0.5, 2.0),
            ("network_discount", 0.0, 80.0),
            ("provider_discount", 0.0, 80.0),
            ("drug_discount", 0.0, 90.0),
            ("rebate_percentage", 0.0, 50.0),
            ("formulary_tier", 1.0, 6.0),
            ("step_therapy_step", 1.0, 5.0),
            ("prior_auth_days", 0.0, 365.0),
            ("concurrent_review_days", 0.0, 365.0),
            ("retrospective_review_days", 0.0, 365.0),
            ("appeal_days", 0.0, 180.0),
            ("grievance_days", 0.0, 60.0),
            ("coordination_of_benefits_order", 1.0, 3.0),
            ("subrogation_percentage", 0.0, 100.0),
            ("recovery_percentage", 0.0, 100.0),
        ];
        for (i, (col, min_v, max_v)) in rule_ranges.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            kwargs.insert("min_value".to_string(), json!(min_v));
            kwargs.insert("max_value".to_string(), json!(max_v));
            e.push(ExpectationConfig {
                expectation_type: "expect_column_values_to_be_between".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 766 + i),
                    category: cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: Some(col.to_string()),
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }

        debug_assert_eq!(e.len(), 80, "DataBusinessRulesSuite must produce 80 tests");
        e
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::suites::SuiteGenerator;

    #[test]
    fn test_data_business_rules_suite_count() {
        let suite = DataBusinessRulesSuite.build_suite(&DqConfig::default());
        assert_eq!(suite.expectations.len(), 80, "DBR726-DBR805 must produce 80 tests");
    }
}
