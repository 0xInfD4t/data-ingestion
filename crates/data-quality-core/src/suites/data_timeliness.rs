use indexmap::IndexMap;
use serde_json::json;

use crate::config::DqConfig;
use crate::expectations::{ExpectationConfig, ExpectationMeta, GeneratedFrom};
use crate::suites::{fmt_test_id, SuiteGenerator};

pub struct DataTimelinessSuite;

impl SuiteGenerator for DataTimelinessSuite {
    fn suite_name(&self) -> &str { "data_timeliness_suite" }
    fn category(&self) -> &str { "timeliness" }
    fn test_id_prefix(&self) -> &str { "DT" }
    fn test_id_start(&self) -> usize { 536 }

    fn generate(&self, _config: &DqConfig) -> Vec<ExpectationConfig> {
        let suite = self.suite_name();
        let cat = self.category();
        let pfx = self.test_id_prefix();
        let mut e = Vec::new();

        // DT536-DT575: Date parseable checks for timestamp fields (40 tests)
        let timestamp_fields = [
            "created_at", "updated_at", "deleted_at", "processed_at", "submitted_at",
            "received_at", "approved_at", "denied_at", "paid_at", "voided_at",
            "adjudicated_at", "posted_at", "exported_at", "imported_at", "validated_at",
            "enriched_at", "normalized_at", "archived_at", "purged_at", "restored_at",
            "claim_received_date", "claim_processed_date", "claim_paid_date", "claim_denied_date", "claim_adjusted_date",
            "auth_requested_date", "auth_approved_date", "auth_denied_date", "auth_expired_date", "auth_cancelled_date",
            "enrollment_date", "disenrollment_date", "effective_date", "termination_date", "renewal_date",
            "last_updated_date", "last_verified_date", "last_contact_date", "next_review_date", "expiration_date",
        ];
        for (i, col) in timestamp_fields.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            e.push(ExpectationConfig {
                expectation_type: "expect_column_values_to_be_dateutil_parseable".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 536 + i),
                    category: cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: Some(col.to_string()),
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }

        // DT576-DT615: Strftime format checks (40 tests)
        let format_checks: &[(&str, &str)] = &[
            ("service_date", "%Y-%m-%d"),
            ("admission_date", "%Y-%m-%d"),
            ("discharge_date", "%Y-%m-%d"),
            ("claim_from_date", "%Y-%m-%d"),
            ("claim_through_date", "%Y-%m-%d"),
            ("coverage_start_date", "%Y-%m-%d"),
            ("coverage_end_date", "%Y-%m-%d"),
            ("prescription_date", "%Y-%m-%d"),
            ("fill_date", "%Y-%m-%d"),
            ("dispense_date", "%Y-%m-%d"),
            ("order_date", "%Y-%m-%d"),
            ("result_date", "%Y-%m-%d"),
            ("report_date", "%Y-%m-%d"),
            ("review_date", "%Y-%m-%d"),
            ("audit_date", "%Y-%m-%d"),
            ("birth_date", "%Y-%m-%d"),
            ("death_date", "%Y-%m-%d"),
            ("surgery_date", "%Y-%m-%d"),
            ("procedure_date", "%Y-%m-%d"),
            ("diagnosis_date", "%Y-%m-%d"),
            ("created_at", "%Y-%m-%dT%H:%M:%S"),
            ("updated_at", "%Y-%m-%dT%H:%M:%S"),
            ("processed_at", "%Y-%m-%dT%H:%M:%S"),
            ("submitted_at", "%Y-%m-%dT%H:%M:%S"),
            ("received_at", "%Y-%m-%dT%H:%M:%S"),
            ("approved_at", "%Y-%m-%dT%H:%M:%S"),
            ("denied_at", "%Y-%m-%dT%H:%M:%S"),
            ("paid_at", "%Y-%m-%dT%H:%M:%S"),
            ("adjudicated_at", "%Y-%m-%dT%H:%M:%S"),
            ("posted_at", "%Y-%m-%dT%H:%M:%S"),
            ("exported_at", "%Y-%m-%dT%H:%M:%S"),
            ("imported_at", "%Y-%m-%dT%H:%M:%S"),
            ("validated_at", "%Y-%m-%dT%H:%M:%S"),
            ("enriched_at", "%Y-%m-%dT%H:%M:%S"),
            ("normalized_at", "%Y-%m-%dT%H:%M:%S"),
            ("archived_at", "%Y-%m-%dT%H:%M:%S"),
            ("purged_at", "%Y-%m-%dT%H:%M:%S"),
            ("restored_at", "%Y-%m-%dT%H:%M:%S"),
            ("last_updated_date", "%Y-%m-%d"),
            ("last_verified_date", "%Y-%m-%d"),
        ];
        for (i, (col, fmt)) in format_checks.iter().enumerate() {
            let mut kwargs = IndexMap::new();
            kwargs.insert("column".to_string(), json!(col));
            kwargs.insert("strftime_format".to_string(), json!(fmt));
            e.push(ExpectationConfig {
                expectation_type: "expect_column_values_to_match_strftime_format".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, 576 + i),
                    category: cat.to_string(),
                    suite: suite.to_string(),
                    contract_field: Some(col.to_string()),
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
        }

        debug_assert_eq!(e.len(), 80, "DataTimelinessSuite must produce 80 tests");
        e
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::suites::SuiteGenerator;

    #[test]
    fn test_data_timeliness_suite_count() {
        let suite = DataTimelinessSuite.build_suite(&DqConfig::default());
        assert_eq!(suite.expectations.len(), 80, "DT536-DT615 must produce 80 tests");
    }
}
