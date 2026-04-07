//! Integration tests for the full data-ingestion-core pipeline.
//!
//! Each test exercises the complete path:
//!   raw bytes → FormatDetector → FormatReader → IrDocument → IrNormalizer
//!             → ContractBuilder → DataContract → serialize (JSON | YAML | XML | CSV)

use data_ingestion_core::{
    contract::builder::ContractBuilderConfig,
    output::OutputFormat,
    process, to_format,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn default_config() -> ContractBuilderConfig {
    ContractBuilderConfig {
        version: "1.0.0".to_string(),
        owner: Some("test-team".to_string()),
        domain: Some("testing".to_string()),
        enrich_pii: true,
        ..ContractBuilderConfig::default()
    }
}

fn config_no_pii() -> ContractBuilderConfig {
    ContractBuilderConfig {
        enrich_pii: false,
        ..ContractBuilderConfig::default()
    }
}

// ── Test 1: JSON Dataset → DataContract → all 4 output formats ────────────────

#[test]
fn test_json_dataset_to_contract_and_all_formats() {
    let input = br#"[
        {
            "user_id": 1,
            "email": "alice@example.com",
            "first_name": "Alice",
            "is_active": true,
            "balance": 1250.75,
            "created_at": "2024-01-15T10:30:00Z"
        },
        {
            "user_id": 2,
            "email": "bob@example.com",
            "first_name": "Bob",
            "is_active": false,
            "balance": 0.0,
            "created_at": "2024-02-20T14:00:00Z"
        }
    ]"#;

    let contract = process(input, Some("json"), Some("users.json"), default_config())
        .expect("JSON dataset processing failed");

    // Assert field count and names
    assert!(
        contract.fields.len() >= 5,
        "Expected at least 5 fields, got {}",
        contract.fields.len()
    );

    let field_names: Vec<&str> = contract.fields.iter().map(|f| f.name.as_str()).collect();
    assert!(
        field_names.contains(&"user_id"),
        "Missing field 'user_id'; fields: {:?}",
        field_names
    );
    assert!(
        field_names.contains(&"email"),
        "Missing field 'email'; fields: {:?}",
        field_names
    );
    assert!(
        field_names.contains(&"is_active"),
        "Missing field 'is_active'; fields: {:?}",
        field_names
    );

    // JSON output
    let json_bytes = to_format(&contract, OutputFormat::Json).expect("JSON serialization failed");
    let json_str = String::from_utf8(json_bytes).expect("JSON output is not valid UTF-8");
    assert!(!json_str.is_empty(), "JSON output is empty");
    // Must be valid JSON
    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).expect("JSON output is not valid JSON");
    assert!(
        parsed.get("fields").is_some(),
        "JSON output missing 'fields' key"
    );
    // Field names appear in the JSON
    assert!(
        json_str.contains("user_id"),
        "JSON output missing 'user_id'"
    );
    assert!(json_str.contains("email"), "JSON output missing 'email'");

    // YAML output
    let yaml_bytes = to_format(&contract, OutputFormat::Yaml).expect("YAML serialization failed");
    let yaml_str = String::from_utf8(yaml_bytes).expect("YAML output is not valid UTF-8");
    assert!(!yaml_str.is_empty(), "YAML output is empty");
    assert!(
        yaml_str.contains("user_id"),
        "YAML output missing 'user_id'"
    );

    // XML output
    let xml_bytes = to_format(&contract, OutputFormat::Xml).expect("XML serialization failed");
    let xml_str = String::from_utf8(xml_bytes).expect("XML output is not valid UTF-8");
    assert!(!xml_str.is_empty(), "XML output is empty");
    assert!(
        xml_str.contains("<DataContract") || xml_str.contains("<data_contract"),
        "XML output missing DataContract element; got: {}",
        &xml_str[..xml_str.len().min(200)]
    );
    assert!(
        xml_str.contains("<Field") || xml_str.contains("<field"),
        "XML output missing Field elements"
    );

    // CSV output
    let csv_bytes = to_format(&contract, OutputFormat::Csv).expect("CSV serialization failed");
    let csv_str = String::from_utf8(csv_bytes).expect("CSV output is not valid UTF-8");
    assert!(!csv_str.is_empty(), "CSV output is empty");
    // Check header row contains expected columns
    let first_line = csv_str.lines().next().expect("CSV output has no lines");
    assert!(
        first_line.contains("field_name"),
        "CSV header missing 'field_name'; header: {}",
        first_line
    );
    assert!(
        first_line.contains("logical_type"),
        "CSV header missing 'logical_type'; header: {}",
        first_line
    );
    assert!(
        first_line.contains("nullable"),
        "CSV header missing 'nullable'; header: {}",
        first_line
    );
    assert!(
        first_line.contains("pii"),
        "CSV header missing 'pii'; header: {}",
        first_line
    );
}

// ── Test 2: JSON Schema → DataContract ────────────────────────────────────────

#[test]
fn test_json_schema_required_and_optional_fields() {
    let input = br#"{
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "Order",
        "description": "An e-commerce order",
        "type": "object",
        "required": ["order_id", "customer_id", "total_amount"],
        "properties": {
            "order_id": {
                "type": "string",
                "description": "Unique order identifier"
            },
            "customer_id": {
                "type": "integer",
                "description": "Customer identifier"
            },
            "total_amount": {
                "type": "number",
                "description": "Total order amount in USD"
            },
            "discount_code": {
                "type": ["string", "null"],
                "description": "Optional discount code"
            },
            "notes": {
                "type": "string",
                "description": "Optional order notes"
            }
        }
    }"#;

    let contract = process(
        input,
        Some("json_schema"),
        Some("order.json"),
        default_config(),
    )
    .expect("JSON Schema processing failed");

    assert!(
        !contract.fields.is_empty(),
        "Contract has no fields"
    );

    // Find required fields — they should have nullable=false
    let order_id_field = contract
        .fields
        .iter()
        .find(|f| f.name == "order_id")
        .expect("Field 'order_id' not found");
    assert!(
        !order_id_field.nullable,
        "Required field 'order_id' should have nullable=false"
    );

    let customer_id_field = contract
        .fields
        .iter()
        .find(|f| f.name == "customer_id")
        .expect("Field 'customer_id' not found");
    assert!(
        !customer_id_field.nullable,
        "Required field 'customer_id' should have nullable=false"
    );

    let total_amount_field = contract
        .fields
        .iter()
        .find(|f| f.name == "total_amount")
        .expect("Field 'total_amount' not found");
    assert!(
        !total_amount_field.nullable,
        "Required field 'total_amount' should have nullable=false"
    );

    // Optional fields should have nullable=true
    let discount_field = contract
        .fields
        .iter()
        .find(|f| f.name == "discount_code")
        .expect("Field 'discount_code' not found");
    assert!(
        discount_field.nullable,
        "Optional field 'discount_code' should have nullable=true"
    );

    // Descriptions should be preserved
    assert_eq!(
        order_id_field.description.as_deref(),
        Some("Unique order identifier"),
        "Field 'order_id' description not preserved"
    );
    assert_eq!(
        customer_id_field.description.as_deref(),
        Some("Customer identifier"),
        "Field 'customer_id' description not preserved"
    );
}

// ── Test 3: XML → DataContract ────────────────────────────────────────────────

#[test]
fn test_xml_to_contract() {
    let input = br#"<?xml version="1.0" encoding="UTF-8"?>
<products>
  <product id="P001" category="electronics">
    <name>Wireless Headphones</name>
    <price currency="USD">79.99</price>
    <stock>150</stock>
    <available>true</available>
  </product>
  <product id="P002" category="books">
    <name>Rust Programming Language</name>
    <price currency="USD">39.99</price>
    <stock>75</stock>
    <available>true</available>
  </product>
</products>"#;

    let contract = process(input, Some("xml"), Some("products.xml"), default_config())
        .expect("XML processing failed");

    assert!(
        !contract.fields.is_empty(),
        "XML contract has no fields"
    );

    // The contract should have fields for the XML elements
    let field_names: Vec<&str> = contract.fields.iter().map(|f| f.name.as_str()).collect();
    // At minimum, the root element or child elements should appear
    assert!(
        !field_names.is_empty(),
        "No fields extracted from XML; fields: {:?}",
        field_names
    );
}

// ── Test 4: XSD → DataContract ────────────────────────────────────────────────

#[test]
fn test_xsd_to_contract() {
    // Use a named complexType so the XSD reader's two-pass parser can resolve it.
    // Pass 1 collects "EmployeeType" into complex_types; pass 2 resolves the
    // xs:element's type="EmployeeType" reference and populates nested_fields.
    let input = br#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:complexType name="EmployeeType">
    <xs:sequence>
      <xs:element name="employee_id" type="xs:integer"/>
      <xs:element name="first_name" type="xs:string"/>
      <xs:element name="last_name" type="xs:string"/>
      <xs:element name="email" type="xs:string"/>
      <xs:element name="department" type="xs:string"/>
      <xs:element name="salary" type="xs:decimal" minOccurs="0"/>
      <xs:element name="hire_date" type="xs:date"/>
      <xs:element name="is_manager" type="xs:boolean" minOccurs="0"/>
    </xs:sequence>
  </xs:complexType>
  <xs:element name="Employee" type="EmployeeType"/>
</xs:schema>"#;

    let contract = process(input, Some("xsd"), Some("employee.xsd"), default_config())
        .expect("XSD processing failed");

    assert!(
        !contract.fields.is_empty(),
        "XSD contract has no fields"
    );

    // The XSD reader produces the root xs:element ("Employee") as a top-level
    // field whose ir_type is IrType::Object(EmployeeType). The builder stores
    // child elements in nested_fields when include_nested=true.
    let top_level_names: Vec<String> = contract
        .fields
        .iter()
        .map(|f| f.name.to_lowercase())
        .collect();

    // Collect nested field names one level deep
    let mut all_names: Vec<String> = top_level_names.clone();
    for field in &contract.fields {
        for nested in &field.nested_fields {
            all_names.push(nested.name.to_lowercase());
            // Also handle dot-prefixed names from qualified_name
            all_names.push(
                nested.name
                    .split('.')
                    .last()
                    .unwrap_or(&nested.name)
                    .to_lowercase(),
            );
        }
    }

    // Root element "Employee" must appear as a top-level field
    assert!(
        top_level_names.iter().any(|n| n == "employee"),
        "Expected root 'employee' field from XSD; top-level fields: {:?}",
        top_level_names
    );

    // Child element names must appear in nested_fields
    assert!(
        all_names.iter().any(|n| n == "employee_id" || n.ends_with(".employee_id")),
        "Missing 'employee_id' in XSD contract nested fields; all names: {:?}",
        all_names
    );
    assert!(
        all_names.iter().any(|n| n == "email" || n.ends_with(".email")),
        "Missing 'email' in XSD contract nested fields; all names: {:?}",
        all_names
    );
    assert!(
        all_names.iter().any(|n| n == "first_name" || n.ends_with(".first_name")),
        "Missing 'first_name' in XSD contract nested fields; all names: {:?}",
        all_names
    );
}

// ── Test 5: CSV Data Dictionary → DataContract ────────────────────────────────

#[test]
fn test_csv_data_dictionary_to_contract() {
    let input = b"field_name,data_type,description,nullable\n\
transaction_id,string,Unique transaction identifier,false\n\
account_id,integer,Account identifier,false\n\
amount,decimal,Transaction amount in USD,false\n\
description,string,Transaction description,true\n\
merchant_id,string,Merchant identifier,true\n";

    let contract = process(input, Some("csv"), Some("transactions.csv"), default_config())
        .expect("CSV data dictionary processing failed");

    assert!(
        !contract.fields.is_empty(),
        "CSV contract has no fields"
    );

    let field_names: Vec<&str> = contract.fields.iter().map(|f| f.name.as_str()).collect();

    assert!(
        field_names.contains(&"transaction_id"),
        "Missing field 'transaction_id'; fields: {:?}",
        field_names
    );
    assert!(
        field_names.contains(&"account_id"),
        "Missing field 'account_id'; fields: {:?}",
        field_names
    );
    assert!(
        field_names.contains(&"amount"),
        "Missing field 'amount'; fields: {:?}",
        field_names
    );
}

// ── Test 6: YAML Schema → DataContract ────────────────────────────────────────

#[test]
fn test_yaml_schema_to_contract() {
    let input = b"- name: event_id\n  \
  type: string\n  \
  description: Unique event identifier\n  \
  nullable: false\n\
- name: event_type\n  \
  type: string\n  \
  description: Type of event\n  \
  nullable: false\n\
- name: user_id\n  \
  type: integer\n  \
  description: User identifier\n  \
  nullable: true\n\
- name: timestamp\n  \
  type: datetime\n  \
  description: Event timestamp in UTC\n  \
  nullable: false\n";

    let contract = process(input, Some("yaml"), Some("events.yaml"), default_config())
        .expect("YAML schema processing failed");

    assert!(
        !contract.fields.is_empty(),
        "YAML contract has no fields"
    );

    let field_names: Vec<&str> = contract.fields.iter().map(|f| f.name.as_str()).collect();

    assert!(
        field_names.contains(&"event_id"),
        "Missing field 'event_id'; fields: {:?}",
        field_names
    );
    assert!(
        field_names.contains(&"event_type"),
        "Missing field 'event_type'; fields: {:?}",
        field_names
    );
    assert!(
        field_names.contains(&"user_id"),
        "Missing field 'user_id'; fields: {:?}",
        field_names
    );
    assert!(
        field_names.contains(&"timestamp"),
        "Missing field 'timestamp'; fields: {:?}",
        field_names
    );
}

// ── Test 7: PII Detection ─────────────────────────────────────────────────────

#[test]
fn test_pii_detection() {
    let input = br#"{
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "UserProfile",
        "type": "object",
        "properties": {
            "email": {
                "type": "string",
                "description": "User email address"
            },
            "ssn": {
                "type": "string",
                "description": "Social security number"
            },
            "product_id": {
                "type": "string",
                "description": "Product identifier"
            },
            "order_count": {
                "type": "integer",
                "description": "Number of orders"
            }
        }
    }"#;

    let contract = process(
        input,
        Some("json_schema"),
        Some("user_profile.json"),
        default_config(),
    )
    .expect("PII detection test processing failed");

    // email should be PII
    let email_field = contract
        .fields
        .iter()
        .find(|f| f.name == "email")
        .expect("Field 'email' not found");
    assert!(
        email_field.pii,
        "Field 'email' should have pii=true"
    );

    // ssn should be PII
    let ssn_field = contract
        .fields
        .iter()
        .find(|f| f.name == "ssn")
        .expect("Field 'ssn' not found");
    assert!(
        ssn_field.pii,
        "Field 'ssn' should have pii=true"
    );

    // product_id should NOT be PII
    let product_field = contract
        .fields
        .iter()
        .find(|f| f.name == "product_id")
        .expect("Field 'product_id' not found");
    assert!(
        !product_field.pii,
        "Field 'product_id' should have pii=false"
    );

    // order_count should NOT be PII
    let order_field = contract
        .fields
        .iter()
        .find(|f| f.name == "order_count")
        .expect("Field 'order_count' not found");
    assert!(
        !order_field.pii,
        "Field 'order_count' should have pii=false"
    );
}

// ── Test 8: PII disabled ──────────────────────────────────────────────────────

#[test]
fn test_pii_disabled_no_pii_flagged() {
    let input = br#"[{"email": "test@example.com", "ssn": "123-45-6789", "product_id": "P001"}]"#;

    let contract = process(input, Some("json"), Some("data.json"), config_no_pii())
        .expect("Processing with PII disabled failed");

    for field in &contract.fields {
        assert!(
            !field.pii,
            "Field '{}' should have pii=false when PII enrichment is disabled",
            field.name
        );
    }
}

// ── Test 9: Full pipeline via process() function ──────────────────────────────

#[test]
fn test_full_pipeline_process_function() {
    let input = br#"[
        {"id": 1, "name": "Alice", "score": 95.5, "active": true},
        {"id": 2, "name": "Bob",   "score": 87.0, "active": false}
    ]"#;

    let config = ContractBuilderConfig {
        version: "2.0.0".to_string(),
        owner: Some("pipeline-test".to_string()),
        domain: Some("analytics".to_string()),
        enrich_pii: true,
        ..ContractBuilderConfig::default()
    };

    let result = process(input, Some("json"), Some("scores.json"), config);
    assert!(result.is_ok(), "process() returned error: {:?}", result.err());

    let contract = result.unwrap();
    assert_eq!(contract.version, "2.0.0");
    assert_eq!(contract.owner.as_deref(), Some("pipeline-test"));
    assert_eq!(contract.domain.as_deref(), Some("analytics"));
    assert!(!contract.fields.is_empty(), "Contract has no fields");
    assert!(!contract.id.is_empty(), "Contract id is empty");
    assert!(!contract.name.is_empty(), "Contract name is empty");
}

// ── Test 10: JSON Schema → CSV output has correct header ─────────────────────

#[test]
fn test_json_schema_csv_output_header() {
    let input = br#"{
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "Product",
        "type": "object",
        "required": ["product_id", "name"],
        "properties": {
            "product_id": {"type": "string"},
            "name": {"type": "string"},
            "price": {"type": "number"}
        }
    }"#;

    let contract = process(
        input,
        Some("json_schema"),
        Some("product.json"),
        default_config(),
    )
    .expect("JSON Schema processing failed");

    let csv_bytes = to_format(&contract, OutputFormat::Csv).expect("CSV serialization failed");
    let csv_str = String::from_utf8(csv_bytes).expect("CSV is not valid UTF-8");

    let header = csv_str.lines().next().expect("CSV has no lines");

    // Verify all 16 expected columns are present
    let expected_columns = [
        "field_name",
        "logical_type",
        "physical_type",
        "nullable",
        "required",
        "primary_key",
        "foreign_key_table",
        "foreign_key_column",
        "unique",
        "pii",
        "classification",
        "description",
        "example",
        "default_value",
        "constraints",
        "tags",
    ];

    for col in &expected_columns {
        assert!(
            header.contains(col),
            "CSV header missing column '{}'; header: {}",
            col,
            header
        );
    }

    // Verify data rows exist
    let row_count = csv_str.lines().count();
    assert!(
        row_count > 1,
        "CSV has only the header row, no data rows"
    );
}

// ── Test 11: JSON Schema → YAML output is valid YAML ─────────────────────────

#[test]
fn test_json_schema_yaml_output_is_valid() {
    let input = br#"{
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "Event",
        "type": "object",
        "properties": {
            "event_id": {"type": "string"},
            "timestamp": {"type": "string", "format": "date-time"}
        }
    }"#;

    let contract = process(
        input,
        Some("json_schema"),
        Some("event.json"),
        default_config(),
    )
    .expect("JSON Schema processing failed");

    let yaml_bytes = to_format(&contract, OutputFormat::Yaml).expect("YAML serialization failed");
    let yaml_str = String::from_utf8(yaml_bytes).expect("YAML is not valid UTF-8");

    assert!(!yaml_str.is_empty(), "YAML output is empty");
    // YAML should contain the contract name
    assert!(
        yaml_str.contains("Event") || yaml_str.contains("event"),
        "YAML output missing contract name; yaml: {}",
        &yaml_str[..yaml_str.len().min(300)]
    );
    // YAML should contain field names
    assert!(
        yaml_str.contains("event_id"),
        "YAML output missing 'event_id'"
    );
}

// ── Test 12: JSON Schema → XML output contains expected elements ──────────────

#[test]
fn test_json_schema_xml_output_structure() {
    let input = br#"{
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "Sensor",
        "type": "object",
        "properties": {
            "sensor_id": {"type": "string"},
            "reading": {"type": "number"},
            "unit": {"type": "string"}
        }
    }"#;

    let contract = process(
        input,
        Some("json_schema"),
        Some("sensor.json"),
        default_config(),
    )
    .expect("JSON Schema processing failed");

    let xml_bytes = to_format(&contract, OutputFormat::Xml).expect("XML serialization failed");
    let xml_str = String::from_utf8(xml_bytes).expect("XML is not valid UTF-8");

    assert!(!xml_str.is_empty(), "XML output is empty");
    // XML must contain a DataContract root element
    assert!(
        xml_str.contains("DataContract") || xml_str.contains("data_contract"),
        "XML output missing DataContract element"
    );
    // XML must contain Field elements
    assert!(
        xml_str.contains("Field") || xml_str.contains("field"),
        "XML output missing Field elements"
    );
    // Field names should appear
    assert!(
        xml_str.contains("sensor_id"),
        "XML output missing 'sensor_id'"
    );
}

// ── Test 13: Contract metadata is populated ───────────────────────────────────

#[test]
fn test_contract_metadata_populated() {
    let input = br#"[{"id": 1, "value": "test"}]"#;

    let config = ContractBuilderConfig {
        version: "3.1.0".to_string(),
        owner: Some("metadata-owner".to_string()),
        domain: Some("metadata-domain".to_string()),
        enrich_pii: false,
        ..ContractBuilderConfig::default()
    };

    let contract = process(input, Some("json"), Some("meta_test.json"), config)
        .expect("Metadata test processing failed");

    assert_eq!(
        contract.version, "3.1.0",
        "Contract version not set correctly"
    );
    assert_eq!(
        contract.owner.as_deref(),
        Some("metadata-owner"),
        "Contract owner not set correctly"
    );
    assert_eq!(
        contract.domain.as_deref(),
        Some("metadata-domain"),
        "Contract domain not set correctly"
    );
    // source_format should be set
    assert!(
        !contract.source_format.is_empty(),
        "Contract source_format is empty"
    );
    // id should be a non-empty UUID-like string
    assert!(
        !contract.id.is_empty(),
        "Contract id is empty"
    );
    assert_eq!(
        contract.id.len(),
        36,
        "Contract id does not look like a UUID (expected 36 chars)"
    );
}

// ── Test 14: Format detection via filename extension ─────────────────────────

#[test]
fn test_format_detection_via_filename() {
    // JSON array — detected via .json extension
    let json_input = br#"[{"key": "value"}]"#;
    let result = process(json_input, None, Some("data.json"), default_config());
    assert!(
        result.is_ok(),
        "Format detection via .json extension failed: {:?}",
        result.err()
    );

    // YAML — detected via .yaml extension
    let yaml_input = b"- name: field_a\n  type: string\n  nullable: false\n";
    let result = process(yaml_input, None, Some("schema.yaml"), default_config());
    assert!(
        result.is_ok(),
        "Format detection via .yaml extension failed: {:?}",
        result.err()
    );
}

// ── Test 15: JSON Schema with nested object ───────────────────────────────────

#[test]
fn test_json_schema_nested_object() {
    let input = br#"{
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "User",
        "type": "object",
        "properties": {
            "user_id": {"type": "integer"},
            "address": {
                "type": "object",
                "properties": {
                    "street": {"type": "string"},
                    "city": {"type": "string"},
                    "zip": {"type": "string"}
                }
            }
        }
    }"#;

    let contract = process(
        input,
        Some("json_schema"),
        Some("user.json"),
        default_config(),
    )
    .expect("Nested JSON Schema processing failed");

    assert!(
        !contract.fields.is_empty(),
        "Contract has no fields"
    );

    // user_id should be a top-level field
    let user_id = contract.fields.iter().find(|f| f.name == "user_id");
    assert!(user_id.is_some(), "Field 'user_id' not found");

    // address should appear either as a top-level field with nested_fields,
    // or flattened as address.street etc.
    let has_address = contract.fields.iter().any(|f| {
        f.name == "address"
            || f.name.starts_with("address.")
            || f.name.starts_with("address/")
    });
    assert!(
        has_address,
        "No address-related fields found; fields: {:?}",
        contract.fields.iter().map(|f| &f.name).collect::<Vec<_>>()
    );
}
