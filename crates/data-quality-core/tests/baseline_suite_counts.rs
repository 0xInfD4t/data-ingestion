use data_quality_core::{
    generate_baseline_suites,
    suites::{
        BaselineSuiteSet, CriminalBackgroundSuite, CrossSystemConsistencySuite, DataAccuracySuite,
        DataBusinessRulesSuite, DataCompletenessSuite, DataConsistencySuite,
        DataDependencyChecksSuite, DataFormatConsistencySuite, DataIntegritySuite, DataProfileSuite,
        DataSensitivitySuite, DataTimelinessSuite, DataUniquenessSuite, DataValiditySuite,
        DataVolumeAnomaliesSuite, PerformanceMetricsSuite, SecurityComplianceSuite, SuiteGenerator,
    },
    DqConfig,
};

#[test]
fn test_data_validity_suite_count() {
    let suite = DataValiditySuite.build_suite(&DqConfig::default());
    assert_eq!(suite.expectations.len(), 95, "DV001-DV095 must produce 95 tests");
}

#[test]
fn test_data_completeness_suite_count() {
    let suite = DataCompletenessSuite.build_suite(&DqConfig::default());
    assert_eq!(suite.expectations.len(), 70, "DC096-DC165 must produce 70 tests");
}

#[test]
fn test_data_consistency_suite_count() {
    let suite = DataConsistencySuite.build_suite(&DqConfig::default());
    assert_eq!(suite.expectations.len(), 90, "DCN166-DCN255 must produce 90 tests");
}

#[test]
fn test_data_accuracy_suite_count() {
    let suite = DataAccuracySuite.build_suite(&DqConfig::default());
    assert_eq!(suite.expectations.len(), 100, "DA256-DA355 must produce 100 tests");
}

#[test]
fn test_data_profile_suite_count() {
    let suite = DataProfileSuite.build_suite(&DqConfig::default());
    assert_eq!(suite.expectations.len(), 120, "DP356-DP475 must produce 120 tests");
}

#[test]
fn test_data_integrity_suite_count() {
    let suite = DataIntegritySuite.build_suite(&DqConfig::default());
    assert_eq!(suite.expectations.len(), 60, "DI476-DI535 must produce 60 tests");
}

#[test]
fn test_data_timeliness_suite_count() {
    let suite = DataTimelinessSuite.build_suite(&DqConfig::default());
    assert_eq!(suite.expectations.len(), 80, "DT536-DT615 must produce 80 tests");
}

#[test]
fn test_data_sensitivity_suite_count() {
    let suite = DataSensitivitySuite.build_suite(&DqConfig::default());
    assert_eq!(suite.expectations.len(), 50, "DS616-DS665 must produce 50 tests");
}

#[test]
fn test_data_uniqueness_suite_count() {
    let suite = DataUniquenessSuite.build_suite(&DqConfig::default());
    assert_eq!(suite.expectations.len(), 60, "DU666-DU725 must produce 60 tests");
}

#[test]
fn test_data_business_rules_suite_count() {
    let suite = DataBusinessRulesSuite.build_suite(&DqConfig::default());
    assert_eq!(suite.expectations.len(), 80, "DBR726-DBR805 must produce 80 tests");
}

#[test]
fn test_data_format_consistency_suite_count() {
    let suite = DataFormatConsistencySuite.build_suite(&DqConfig::default());
    assert_eq!(suite.expectations.len(), 70, "DFC806-DFC875 must produce 70 tests");
}

#[test]
fn test_data_volume_anomalies_suite_count() {
    let suite = DataVolumeAnomaliesSuite.build_suite(&DqConfig::default());
    assert_eq!(suite.expectations.len(), 90, "DVA876-DVA965 must produce 90 tests");
}

#[test]
fn test_data_dependency_checks_suite_count() {
    let suite = DataDependencyChecksSuite.build_suite(&DqConfig::default());
    assert_eq!(suite.expectations.len(), 50, "DDC966-DDC1015 must produce 50 tests");
}

#[test]
fn test_cross_system_consistency_suite_count() {
    let suite = CrossSystemConsistencySuite.build_suite(&DqConfig::default());
    assert_eq!(suite.expectations.len(), 100, "CSC1016-CSC1115 must produce 100 tests");
}

#[test]
fn test_performance_metrics_suite_count() {
    let suite = PerformanceMetricsSuite.build_suite(&DqConfig::default());
    assert_eq!(suite.expectations.len(), 60, "PM1116-PM1175 must produce 60 tests");
}

#[test]
fn test_security_compliance_suite_count() {
    let suite = SecurityComplianceSuite.build_suite(&DqConfig::default());
    assert_eq!(suite.expectations.len(), 153, "SC1176-SC1328 must produce 153 tests");
}

#[test]
fn test_criminal_background_suite_count() {
    let suite = CriminalBackgroundSuite.build_suite(&DqConfig::default());
    assert_eq!(
        suite.expectations.len(),
        150,
        "CBC001-CBC150 must produce 150 tests"
    );
    assert!(
        suite.expectations.len() >= 120,
        "criminal_background_suite must have at least 120 tests"
    );
}

#[test]
fn test_total_baseline_count() {
    let suites = BaselineSuiteSet::generate_all(&DqConfig::default());
    let total: usize = suites.iter().map(|s| s.expectations.len()).sum();
    assert_eq!(total, 1478, "Total baseline tests must equal 1478 (1328 + 150)");
}

#[test]
fn test_baseline_suite_count_is_17() {
    let suites = BaselineSuiteSet::generate_all(&DqConfig::default());
    assert_eq!(suites.len(), 17, "Must have exactly 17 baseline suites");
}

#[test]
fn test_suite_filtering_by_enabled_suites() {
    let config = DqConfig {
        enabled_suites: Some(vec!["data_validity_suite".to_string()]),
        ..Default::default()
    };
    let suites = BaselineSuiteSet::generate_all(&config);
    assert_eq!(suites.len(), 1);
    assert_eq!(suites[0].name, "data_validity_suite");
}

#[test]
fn test_suite_filtering_by_disabled_suites() {
    let config = DqConfig {
        disabled_suites: vec!["security_compliance_suite".to_string()],
        ..Default::default()
    };
    let suites = BaselineSuiteSet::generate_all(&config);
    assert_eq!(suites.len(), 16);
    assert!(!suites.iter().any(|s| s.name == "security_compliance_suite"));
}

#[test]
fn test_generate_baseline_suites_api() {
    let config = DqConfig::default();
    let suites = generate_baseline_suites(&config);
    assert_eq!(suites.len(), 17);
    let total: usize = suites.iter().map(|s| s.expectations.len()).sum();
    assert_eq!(total, 1478);
}

#[test]
fn test_all_suites_have_gx_version() {
    let config = DqConfig::default();
    let suites = BaselineSuiteSet::generate_all(&config);
    for suite in &suites {
        assert_eq!(
            suite.meta.great_expectations_version, "1.11.3",
            "Suite {} must have GX version 1.11.3",
            suite.name
        );
    }
}

#[test]
fn test_all_suites_have_suite_id() {
    let config = DqConfig::default();
    let suites = BaselineSuiteSet::generate_all(&config);
    for suite in &suites {
        assert!(
            !suite.meta.suite_id.is_empty(),
            "Suite {} must have a non-empty suite_id",
            suite.name
        );
    }
}

#[test]
fn test_all_suites_test_count_matches_expectations_len() {
    let config = DqConfig::default();
    let suites = BaselineSuiteSet::generate_all(&config);
    for suite in &suites {
        assert_eq!(
            suite.meta.test_count,
            suite.expectations.len(),
            "Suite {} meta.test_count must match expectations.len()",
            suite.name
        );
    }
}
