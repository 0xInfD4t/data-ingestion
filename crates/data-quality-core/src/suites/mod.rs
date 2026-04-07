use crate::config::DqConfig;
use crate::expectations::{ExpectationConfig, ExpectationSuite, SuiteMeta};

pub mod criminal_background;
pub mod cross_system_consistency;
pub mod data_accuracy;
pub mod data_business_rules;
pub mod data_completeness;
pub mod data_consistency;
pub mod data_dependency_checks;
pub mod data_format_consistency;
pub mod data_integrity;
pub mod data_profile;
pub mod data_sensitivity;
pub mod data_timeliness;
pub mod data_uniqueness;
pub mod data_validity;
pub mod data_volume_anomalies;
pub mod performance_metrics;
pub mod security_compliance;

pub use criminal_background::CriminalBackgroundSuite;
pub use cross_system_consistency::CrossSystemConsistencySuite;
pub use data_accuracy::DataAccuracySuite;
pub use data_business_rules::DataBusinessRulesSuite;
pub use data_completeness::DataCompletenessSuite;
pub use data_consistency::DataConsistencySuite;
pub use data_dependency_checks::DataDependencyChecksSuite;
pub use data_format_consistency::DataFormatConsistencySuite;
pub use data_integrity::DataIntegritySuite;
pub use data_profile::DataProfileSuite;
pub use data_sensitivity::DataSensitivitySuite;
pub use data_timeliness::DataTimelinessSuite;
pub use data_uniqueness::DataUniquenessSuite;
pub use data_validity::DataValiditySuite;
pub use data_volume_anomalies::DataVolumeAnomaliesSuite;
pub use performance_metrics::PerformanceMetricsSuite;
pub use security_compliance::SecurityComplianceSuite;

// ── SuiteGenerator trait ──────────────────────────────────────────────────────

/// Trait implemented by all 17 baseline suite generator structs.
pub trait SuiteGenerator {
    /// Canonical suite name, e.g. "data_validity_suite"
    fn suite_name(&self) -> &str;
    /// Quality dimension category, e.g. "validity"
    fn category(&self) -> &str;
    /// Test ID prefix, e.g. "DV" for data_validity
    fn test_id_prefix(&self) -> &str;
    /// Starting test ID number, e.g. 1 for DV001
    fn test_id_start(&self) -> usize;
    /// Generates all ExpectationConfig objects for this suite.
    fn generate(&self, config: &DqConfig) -> Vec<ExpectationConfig>;

    /// Builds the complete ExpectationSuite (default implementation).
    fn build_suite(&self, config: &DqConfig) -> ExpectationSuite {
        let expectations = self.generate(config);
        let count = expectations.len();
        ExpectationSuite {
            name: self.suite_name().to_string(),
            expectations,
            meta: SuiteMeta {
                great_expectations_version: config.gx_version.clone(),
                suite_id: uuid::Uuid::new_v4().to_string(),
                contract_id: None,
                generated_at: None,
                test_count: count,
            },
        }
    }
}

// ── BaselineSuiteSet ──────────────────────────────────────────────────────────

/// Generates all 17 baseline suites, respecting config filters.
pub struct BaselineSuiteSet;

impl BaselineSuiteSet {
    /// Returns all 17 suites filtered by config.enabled_suites / config.disabled_suites.
    pub fn generate_all(config: &DqConfig) -> Vec<ExpectationSuite> {
        let generators: Vec<Box<dyn SuiteGenerator>> = vec![
            Box::new(DataValiditySuite),
            Box::new(DataCompletenessSuite),
            Box::new(DataConsistencySuite),
            Box::new(DataAccuracySuite),
            Box::new(DataProfileSuite),
            Box::new(DataIntegritySuite),
            Box::new(DataTimelinessSuite),
            Box::new(DataSensitivitySuite),
            Box::new(DataUniquenessSuite),
            Box::new(DataBusinessRulesSuite),
            Box::new(DataFormatConsistencySuite),
            Box::new(DataVolumeAnomaliesSuite),
            Box::new(DataDependencyChecksSuite),
            Box::new(CrossSystemConsistencySuite),
            Box::new(PerformanceMetricsSuite),
            Box::new(SecurityComplianceSuite),
            Box::new(CriminalBackgroundSuite),
        ];

        generators
            .into_iter()
            .filter(|g| {
                let name = g.suite_name();
                let enabled = config
                    .enabled_suites
                    .as_ref()
                    .map(|list| list.iter().any(|s| s == name))
                    .unwrap_or(true);
                let disabled = config.disabled_suites.iter().any(|s| s == name);
                enabled && !disabled
            })
            .map(|g| g.build_suite(config))
            .collect()
    }
}

// ── Helper: format test ID ────────────────────────────────────────────────────

/// Format a test ID like "DV001", "DC096", "SC1328"
pub fn fmt_test_id(prefix: &str, n: usize) -> String {
    if n < 10 {
        format!("{}00{}", prefix, n)
    } else if n < 100 {
        format!("{}0{}", prefix, n)
    } else {
        format!("{}{}", prefix, n)
    }
}
