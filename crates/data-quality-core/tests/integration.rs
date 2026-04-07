use data_quality_core::{
    generate_all_suites, generate_baseline_suites, generate_contract_suites, serialize_suite_set,
    DqConfig, DqOutputFormat,
};
use data_ingestion_core::{
    ContractField, DataClassification, DataContract, FieldConstraint, LogicalType,
};
use std::collections::HashMap;

fn make_full_contract() -> DataContract {
    DataContract {
        id: "integration-test-id".to_string(),
        name: "patient_record".to_string(),
        version: "1.0.0".to_string(),
        description: Some("Integration test patient record contract".to_string()),
        owner: Some("data-team".to_string()),
        domain: Some("healthcare".to_string()),
        source_format: "json_schema".to_string(),
        fields: vec![
            ContractField {
                name: "patient_id".to_string(),
                logical_type: LogicalType::Uuid,
                physical_type: Some("string".to_string()),
                nullable: false,
                required: true,
                primary_key: true,
                foreign_key: None,
                unique: true,
                description: Some("Unique patient identifier".to_string()),
                constraints: vec![],
                example: Some("550e8400-e29b-41d4-a716-446655440000".to_string()),
                default_value: None,
                pii: false,
                classification: DataClassification::Internal,
                tags: vec!["identifier".to_string()],
                metadata: HashMap::new(),
                nested_fields: vec![],
            },
            ContractField {
                name: "ssn".to_string(),
                logical_type: LogicalType::String,
                physical_type: Some("string".to_string()),
                nullable: true,
                required: false,
                primary_key: false,
                foreign_key: None,
                unique: false,
                description: Some("Social security number (masked)".to_string()),
                constraints: vec![FieldConstraint::Pattern(r"^\d{3}-\d{2}-\d{4}$".to_string())],
                example: None,
                default_value: None,
                pii: true,
                classification: DataClassification::Restricted,
                tags: vec!["pii".to_string(), "phi".to_string()],
                metadata: HashMap::new(),
                nested_fields: vec![],
            },
            ContractField {
                name: "age".to_string(),
                logical_type: LogicalType::Integer,
                physical_type: Some("integer".to_string()),
                nullable: true,
                required: false,
                primary_key: false,
                foreign_key: None,
                unique: false,
                description: Some("Patient age in years".to_string()),
                constraints: vec![
                    FieldConstraint::Minimum(0.0),
                    FieldConstraint::Maximum(150.0),
                ],
                example: Some("45".to_string()),
                default_value: None,
                pii: false,
                classification: DataClassification::Internal,
                tags: vec![],
                metadata: HashMap::new(),
                nested_fields: vec![],
            },
            ContractField {
                name: "status".to_string(),
                logical_type: LogicalType::String,
                physical_type: Some("string".to_string()),
                nullable: false,
                required: true,
                primary_key: false,
                foreign_key: None,
                unique: false,
                description: Some("Patient status".to_string()),
                constraints: vec![FieldConstraint::AllowedValues(vec![
                    "active".to_string(),
                    "inactive".to_string(),
                    "deceased".to_string(),
                ])],
                example: Some("active".to_string()),
                default_value: Some("active".to_string()),
                pii: false,
                classification: DataClassification::Internal,
                tags: vec![],
                metadata: HashMap::new(),
                nested_fields: vec![],
            },
        ],
        metadata: HashMap::new(),
        sla: None,
        lineage: None,
        quality: None,
        created_at: None,
        tags: vec![],
    }
}

#[test]
fn test_full_pipeline_baseline_plus_contract() {
    let contract = make_full_contract();
    let config = DqConfig::default();
    let suite_set = generate_all_suites(&contract, &config);

    // Verify baseline suites
    assert_eq!(suite_set.baseline_suites.len(), 16, "Must have 16 baseline suites");
    let baseline_total: usize = suite_set.baseline_suites.iter().map(|s| s.expectations.len()).sum();
    assert_eq!(baseline_total, 1328, "Baseline must have 1328 tests");

    // Verify contract suites exist
    assert!(!suite_set.contract_suites.is_empty(), "Must have contract-specific suites");

    // Verify total count
    assert!(suite_set.total_test_count >= 1328, "Total must be >= 1328");
    assert_eq!(
        suite_set.total_test_count,
        baseline_total + suite_set.contract_suites.iter().map(|s| s.expectations.len()).sum::<usize>()
    );

    // Verify PII suite exists (ssn field has pii=true)
    assert!(
        suite_set.contract_suites.iter().any(|s| s.name.ends_with("_pii_suite")),
        "Must have pii_suite for PII fields"
    );
}

#[test]
fn test_serialize_to_gx_json_valid_format() {
    let contract = make_full_contract();
    let config = DqConfig::default();
    let suite_set = generate_all_suites(&contract, &config);
    let files = serialize_suite_set(&suite_set, DqOutputFormat::GxJson)
        .expect("Serialization must succeed");

    assert!(!files.is_empty());

    // Verify each suite JSON file is valid GX 1.x format
    for file in files.iter().filter(|f| {
        f.filename.ends_with(".json")
            && !f.filename.ends_with("manifest.json")
    }) {
        let json: serde_json::Value = serde_json::from_slice(&file.content)
            .unwrap_or_else(|e| panic!("File {} must be valid JSON: {}", file.filename, e));

        assert!(json.get("name").is_some(), "Suite {} must have 'name'", file.filename);
        assert!(
            json.get("expectations").is_some(),
            "Suite {} must have 'expectations'",
            file.filename
        );
        let meta = json.get("meta").expect("Suite must have 'meta'");
        assert_eq!(
            meta.get("great_expectations_version").and_then(|v| v.as_str()),
            Some("1.11.3"),
            "GX version must be 1.11.3 in {}",
            file.filename
        );

        // Verify each expectation uses "type" key (GX 1.x format)
        for exp in json["expectations"].as_array().unwrap() {
            assert!(
                exp.get("type").is_some(),
                "Expectation in {} must use 'type' key",
                file.filename
            );
            assert!(
                exp.get("expectation_type").is_none(),
                "Expectation in {} must NOT use 'expectation_type' key",
                file.filename
            );
            assert!(
                exp.get("kwargs").is_some(),
                "Expectation in {} must have 'kwargs'",
                file.filename
            );
        }
    }
}

#[test]
fn test_manifest_json_is_valid() {
    let contract = make_full_contract();
    let config = DqConfig::default();
    let suite_set = generate_all_suites(&contract, &config);
    let files = serialize_suite_set(&suite_set, DqOutputFormat::GxJson)
        .expect("Serialization must succeed");

    let manifest_file = files
        .iter()
        .find(|f| f.filename.ends_with("manifest.json"))
        .expect("manifest.json must be generated");

    let manifest: serde_json::Value = serde_json::from_slice(&manifest_file.content)
        .expect("manifest.json must be valid JSON");

    assert!(manifest.get("total_test_count").is_some(), "Manifest must have total_test_count");
    assert!(manifest.get("suites").is_some(), "Manifest must have suites array");
    assert!(manifest.get("contract_name").is_some(), "Manifest must have contract_name");
}

#[test]
fn test_summary_csv_is_generated() {
    let contract = make_full_contract();
    let config = DqConfig::default();
    let suite_set = generate_all_suites(&contract, &config);
    let files = serialize_suite_set(&suite_set, DqOutputFormat::GxJson)
        .expect("Serialization must succeed");

    assert!(
        files.iter().any(|f| f.filename.ends_with("summary.csv")),
        "summary.csv must be generated"
    );
}

#[test]
fn test_baseline_only_config() {
    let contract = make_full_contract();
    let config = DqConfig {
        include_contract_specific: false,
        ..Default::default()
    };
    let suite_set = generate_all_suites(&contract, &config);
    assert_eq!(suite_set.baseline_suites.len(), 16);
    assert!(suite_set.contract_suites.is_empty());
}

#[test]
fn test_contract_only_config() {
    let contract = make_full_contract();
    let config = DqConfig {
        include_baseline: false,
        ..Default::default()
    };
    let suite_set = generate_all_suites(&contract, &config);
    assert!(suite_set.baseline_suites.is_empty());
    assert!(!suite_set.contract_suites.is_empty());
}

#[test]
fn test_generate_baseline_suites_standalone() {
    let config = DqConfig::default();
    let suites = generate_baseline_suites(&config);
    assert_eq!(suites.len(), 16);
    let total: usize = suites.iter().map(|s| s.expectations.len()).sum();
    assert_eq!(total, 1328);
}

#[test]
fn test_generate_contract_suites_standalone() {
    let contract = make_full_contract();
    let config = DqConfig::default();
    let suites = generate_contract_suites(&contract, &config);
    assert!(!suites.is_empty());
    // Should have schema, field, constraints, and pii suites
    assert!(suites.iter().any(|s| s.name.ends_with("_schema_suite")));
    assert!(suites.iter().any(|s| s.name.ends_with("_field_suite")));
    assert!(suites.iter().any(|s| s.name.ends_with("_pii_suite")));
    assert!(suites.iter().any(|s| s.name.ends_with("_constraints_suite")));
}

#[test]
fn test_output_filenames_follow_pattern() {
    let contract = make_full_contract();
    let config = DqConfig::default();
    let suite_set = generate_all_suites(&contract, &config);
    let files = serialize_suite_set(&suite_set, DqOutputFormat::GxJson)
        .expect("Serialization must succeed");

    let contract_name = "patient_record";

    // Baseline files should be in baseline/ subdirectory
    for suite in &suite_set.baseline_suites {
        let expected_prefix = format!("{}/baseline/", contract_name);
        assert!(
            files.iter().any(|f| f.filename.starts_with(&expected_prefix) && f.suite_name == suite.name),
            "Baseline suite {} must have file in {}/baseline/",
            suite.name,
            contract_name
        );
    }

    // Contract-specific files should be in contract_specific/ subdirectory
    for suite in &suite_set.contract_suites {
        let expected_prefix = format!("{}/contract_specific/", contract_name);
        assert!(
            files.iter().any(|f| f.filename.starts_with(&expected_prefix) && f.suite_name == suite.name),
            "Contract suite {} must have file in {}/contract_specific/",
            suite.name,
            contract_name
        );
    }
}
