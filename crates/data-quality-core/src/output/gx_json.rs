use crate::error::DqError;
use crate::expectations::ExpectationSuite;

/// Serialize an ExpectationSuite to GX 1.x-compatible JSON bytes.
pub fn to_gx_json(suite: &ExpectationSuite) -> Result<Vec<u8>, DqError> {
    serde_json::to_vec_pretty(suite)
        .map_err(|e| DqError::SerializationError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::suites::{BaselineSuiteSet, SuiteGenerator};
    use crate::suites::data_validity::DataValiditySuite;
    use crate::config::DqConfig;

    #[test]
    fn test_gx_json_uses_type_key_not_expectation_type() {
        let suite = DataValiditySuite.build_suite(&DqConfig::default());
        let bytes = to_gx_json(&suite).expect("serialization must succeed");
        let json: serde_json::Value = serde_json::from_slice(&bytes).expect("must be valid JSON");

        // Verify suite structure
        assert!(json.get("name").is_some(), "Suite must have 'name'");
        assert!(json.get("expectations").is_some(), "Suite must have 'expectations'");
        assert!(json.get("meta").is_some(), "Suite must have 'meta'");

        // Verify each expectation uses "type" not "expectation_type"
        for exp in json["expectations"].as_array().unwrap() {
            assert!(
                exp.get("type").is_some(),
                "Expectation must use 'type' key (GX 1.x format)"
            );
            assert!(
                exp.get("expectation_type").is_none(),
                "Expectation must NOT use 'expectation_type' key"
            );
            assert!(exp.get("kwargs").is_some(), "Expectation must have 'kwargs'");
            assert!(exp.get("meta").is_some(), "Expectation must have 'meta'");
        }
    }

    #[test]
    fn test_gx_json_meta_has_gx_version() {
        let config = DqConfig::default();
        let suite = DataValiditySuite.build_suite(&config);
        let bytes = to_gx_json(&suite).expect("serialization must succeed");
        let json: serde_json::Value = serde_json::from_slice(&bytes).expect("must be valid JSON");
        let meta = json.get("meta").unwrap();
        assert_eq!(
            meta.get("great_expectations_version").and_then(|v| v.as_str()),
            Some("1.11.3"),
            "GX version must be 1.11.3"
        );
    }

    #[test]
    fn test_all_baseline_suites_serialize_to_valid_json() {
        let config = DqConfig::default();
        let suites = BaselineSuiteSet::generate_all(&config);
        for suite in &suites {
            let bytes = to_gx_json(suite).expect("serialization must succeed");
            let json: serde_json::Value =
                serde_json::from_slice(&bytes).expect("must be valid JSON");
            assert!(json.get("name").is_some());
            assert!(json.get("expectations").is_some());
        }
    }
}
