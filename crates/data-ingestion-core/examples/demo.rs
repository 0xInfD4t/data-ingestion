//! Demo: Full pipeline from file to data contract in all 4 output formats.
//!
//! Run with:
//!   cargo run --example demo -p data-ingestion-core

use data_ingestion_core::{contract::builder::ContractBuilderConfig, output::OutputFormat, process, to_format};

fn main() {
    // ── Example 1: JSON Schema ────────────────────────────────────────────────
    let schema = include_bytes!("../../../examples/sample_json_schema.json");

    let config = ContractBuilderConfig {
        version: "1.0.0".to_string(),
        owner: Some("data-team".to_string()),
        domain: Some("e-commerce".to_string()),
        enrich_pii: true,
        ..ContractBuilderConfig::default()
    };

    let contract = process(
        schema,
        Some("json_schema"),
        Some("sample_json_schema.json"),
        config.clone(),
    )
    .expect("Failed to process JSON Schema");

    println!("=== Contract: {} ===", contract.name);
    println!("Fields: {}", contract.fields.len());
    for field in &contract.fields {
        println!(
            "  - {} ({}) nullable={} pii={}",
            field.name, field.logical_type, field.nullable, field.pii
        );
    }

    // Output as JSON
    let json = to_format(&contract, OutputFormat::Json).expect("JSON serialization failed");
    let json_str = String::from_utf8_lossy(&json);
    let preview_len = json_str.len().min(500);
    println!("\n=== JSON Output (first 500 chars) ===");
    println!("{}", &json_str[..preview_len]);

    // Output as CSV
    let csv = to_format(&contract, OutputFormat::Csv).expect("CSV serialization failed");
    println!("\n=== CSV Output ===");
    println!("{}", String::from_utf8_lossy(&csv));

    // ── Example 2: CSV Data Dictionary ───────────────────────────────────────
    let dict = include_bytes!("../../../examples/sample_data_dictionary.csv");
    let contract2 = process(
        dict,
        Some("csv"),
        Some("sample_data_dictionary.csv"),
        config,
    )
    .expect("Failed to process CSV data dictionary");

    println!("\n=== Contract: {} ===", contract2.name);
    println!("Fields: {}", contract2.fields.len());

    let yaml = to_format(&contract2, OutputFormat::Yaml).expect("YAML serialization failed");
    println!("\n=== YAML Output ===");
    println!("{}", String::from_utf8_lossy(&yaml));
}
