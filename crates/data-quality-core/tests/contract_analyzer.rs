use data_quality_core::{generate_contract_suites, DqConfig};
use data_ingestion_core::{
    ContractField, DataClassification, DataContract, FieldConstraint, LogicalType,
};
use std::collections::HashMap;

fn make_test_contract(fields: Vec<ContractField>) -> DataContract {
    DataContract {
        id: "test-id".to_string(),
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        description: None,
        owner: None,
        domain: None,
        source_format: "json".to_string(),
        fields,
        metadata: HashMap::new(),
        sla: None,
        lineage: None,
        quality: None,
        created_at: None,
        tags: vec![],
    }
}

fn make_field(name: &str) -> ContractField {
    ContractField {
        name: name.to_string(),
        logical_type: LogicalType::String,
        physical_type: None,
        nullable: true,
        required: false,
        primary_key: false,
        foreign_key: None,
        unique: false,
        description: None,
        constraints: vec![],
        example: None,
        default_value: None,
        pii: false,
        classification: DataClassification::Internal,
        tags: vec![],
        metadata: HashMap::new(),
        nested_fields: vec![],
    }
}

#[test]
fn test_nullable_false_generates_not_null() {
    let field = ContractField {
        nullable: false,
        ..make_field("order_id")
    };
    let contract = make_test_contract(vec![field]);
    let config = DqConfig {
        include_baseline: false,
        ..Default::default()
    };
    let suites = generate_contract_suites(&contract, &config);
    let field_suite = suites.iter().find(|s| s.name.ends_with("_field_suite")).unwrap();
    assert!(field_suite.expectations.iter().any(|e| {
        e.expectation_type == "expect_column_values_to_not_be_null"
            && e.kwargs.get("column").and_then(|v| v.as_str()) == Some("order_id")
    }));
}

#[test]
fn test_primary_key_generates_unique_and_not_null() {
    let field = ContractField {
        primary_key: true,
        ..make_field("id")
    };
    let contract = make_test_contract(vec![field]);
    let config = DqConfig {
        include_baseline: false,
        ..Default::default()
    };
    let suites = generate_contract_suites(&contract, &config);
    let field_suite = suites.iter().find(|s| s.name.ends_with("_field_suite")).unwrap();
    let has_not_null = field_suite.expectations.iter().any(|e| {
        e.expectation_type == "expect_column_values_to_not_be_null"
            && e.kwargs.get("column").and_then(|v| v.as_str()) == Some("id")
    });
    let has_unique = field_suite.expectations.iter().any(|e| {
        e.expectation_type == "expect_column_values_to_be_unique"
            && e.kwargs.get("column").and_then(|v| v.as_str()) == Some("id")
    });
    assert!(has_not_null && has_unique);
}

#[test]
fn test_email_type_generates_valid_email_expectation() {
    let field = ContractField {
        logical_type: LogicalType::Email,
        ..make_field("email")
    };
    let contract = make_test_contract(vec![field]);
    let config = DqConfig {
        include_baseline: false,
        ..Default::default()
    };
    let suites = generate_contract_suites(&contract, &config);
    let field_suite = suites.iter().find(|s| s.name.ends_with("_field_suite")).unwrap();
    assert!(field_suite.expectations.iter().any(|e| {
        e.expectation_type == "expect_column_values_to_be_valid_email"
    }));
}

#[test]
fn test_uuid_type_generates_valid_uuid_expectation() {
    let field = ContractField {
        logical_type: LogicalType::Uuid,
        ..make_field("record_id")
    };
    let contract = make_test_contract(vec![field]);
    let config = DqConfig {
        include_baseline: false,
        ..Default::default()
    };
    let suites = generate_contract_suites(&contract, &config);
    let field_suite = suites.iter().find(|s| s.name.ends_with("_field_suite")).unwrap();
    assert!(field_suite.expectations.iter().any(|e| {
        e.expectation_type == "expect_column_values_to_be_valid_uuid"
    }));
}

#[test]
fn test_boolean_type_generates_in_set() {
    let field = ContractField {
        logical_type: LogicalType::Boolean,
        ..make_field("is_active")
    };
    let contract = make_test_contract(vec![field]);
    let config = DqConfig {
        include_baseline: false,
        ..Default::default()
    };
    let suites = generate_contract_suites(&contract, &config);
    let field_suite = suites.iter().find(|s| s.name.ends_with("_field_suite")).unwrap();
    assert!(field_suite.expectations.iter().any(|e| {
        e.expectation_type == "expect_column_values_to_be_in_set"
            && e.kwargs.get("column").and_then(|v| v.as_str()) == Some("is_active")
    }));
}

#[test]
fn test_pii_field_generates_pii_suite() {
    let field = ContractField {
        pii: true,
        ..make_field("ssn")
    };
    let contract = make_test_contract(vec![field]);
    let config = DqConfig {
        include_baseline: false,
        ..Default::default()
    };
    let suites = generate_contract_suites(&contract, &config);
    assert!(suites.iter().any(|s| s.name.ends_with("_pii_suite")));
}

#[test]
fn test_no_pii_fields_no_pii_suite() {
    let field = make_field("name");
    let contract = make_test_contract(vec![field]);
    let config = DqConfig {
        include_baseline: false,
        ..Default::default()
    };
    let suites = generate_contract_suites(&contract, &config);
    assert!(!suites.iter().any(|s| s.name.ends_with("_pii_suite")));
}

#[test]
fn test_min_max_constraint_merges_into_single_between() {
    let field = ContractField {
        constraints: vec![
            FieldConstraint::Minimum(0.0),
            FieldConstraint::Maximum(100.0),
        ],
        logical_type: LogicalType::Float,
        ..make_field("score")
    };
    let contract = make_test_contract(vec![field]);
    let config = DqConfig {
        include_baseline: false,
        ..Default::default()
    };
    let suites = generate_contract_suites(&contract, &config);
    let con_suite = suites
        .iter()
        .find(|s| s.name.ends_with("_constraints_suite"))
        .unwrap();
    let between_count = con_suite
        .expectations
        .iter()
        .filter(|e| {
            e.expectation_type == "expect_column_values_to_be_between"
                && e.kwargs.get("column").and_then(|v| v.as_str()) == Some("score")
        })
        .count();
    assert_eq!(between_count, 1, "Min+Max should merge into one between expectation");
}

#[test]
fn test_allowed_values_generates_in_set() {
    let field = ContractField {
        constraints: vec![FieldConstraint::AllowedValues(vec![
            "A".into(),
            "B".into(),
            "C".into(),
        ])],
        ..make_field("status")
    };
    let contract = make_test_contract(vec![field]);
    let config = DqConfig {
        include_baseline: false,
        ..Default::default()
    };
    let suites = generate_contract_suites(&contract, &config);
    let con_suite = suites
        .iter()
        .find(|s| s.name.ends_with("_constraints_suite"))
        .unwrap();
    assert!(con_suite.expectations.iter().any(|e| {
        e.expectation_type == "expect_column_values_to_be_in_set"
            && e.kwargs.get("column").and_then(|v| v.as_str()) == Some("status")
    }));
}

#[test]
fn test_restricted_classification_generates_encrypted() {
    let field = ContractField {
        classification: DataClassification::Restricted,
        pii: true,
        ..make_field("secret")
    };
    let contract = make_test_contract(vec![field]);
    let config = DqConfig {
        include_baseline: false,
        ..Default::default()
    };
    let suites = generate_contract_suites(&contract, &config);
    let pii_suite = suites
        .iter()
        .find(|s| s.name.ends_with("_pii_suite"))
        .unwrap();
    assert!(pii_suite
        .expectations
        .iter()
        .any(|e| e.expectation_type == "expect_column_values_to_be_encrypted"));
}

#[test]
fn test_schema_suite_always_generated() {
    let field = make_field("name");
    let contract = make_test_contract(vec![field]);
    let config = DqConfig {
        include_baseline: false,
        ..Default::default()
    };
    let suites = generate_contract_suites(&contract, &config);
    assert!(suites.iter().any(|s| s.name.ends_with("_schema_suite")));
}

#[test]
fn test_schema_suite_has_column_set_expectation() {
    let fields = vec![make_field("id"), make_field("name"), make_field("email")];
    let contract = make_test_contract(fields);
    let config = DqConfig {
        include_baseline: false,
        ..Default::default()
    };
    let suites = generate_contract_suites(&contract, &config);
    let schema_suite = suites.iter().find(|s| s.name.ends_with("_schema_suite")).unwrap();
    assert!(schema_suite.expectations.iter().any(|e| {
        e.expectation_type == "expect_table_columns_to_match_set"
    }));
    assert!(schema_suite.expectations.iter().any(|e| {
        e.expectation_type == "expect_table_column_count_to_equal"
    }));
}

#[test]
fn test_pattern_constraint_generates_regex() {
    let field = ContractField {
        constraints: vec![FieldConstraint::Pattern(r"^\d{5}$".to_string())],
        ..make_field("zip_code")
    };
    let contract = make_test_contract(vec![field]);
    let config = DqConfig {
        include_baseline: false,
        ..Default::default()
    };
    let suites = generate_contract_suites(&contract, &config);
    let con_suite = suites
        .iter()
        .find(|s| s.name.ends_with("_constraints_suite"))
        .unwrap();
    assert!(con_suite.expectations.iter().any(|e| {
        e.expectation_type == "expect_column_values_to_match_regex"
            && e.kwargs.get("column").and_then(|v| v.as_str()) == Some("zip_code")
    }));
}
