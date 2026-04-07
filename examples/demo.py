#!/usr/bin/env python3
"""
Demo: Full pipeline from bytes to data contract using the Python bindings.

Run after building with maturin:
    maturin develop --manifest-path crates/data-ingestion-python/Cargo.toml
    python examples/demo.py
"""

import sys
import os

# Try to import the built module
try:
    import data_ingestion
except ImportError:
    print("ERROR: data_ingestion module not found.")
    print("Build it first with: maturin develop --manifest-path crates/data-ingestion-python/Cargo.toml")
    sys.exit(1)

print(f"data_ingestion version: {data_ingestion.__version__}")

# Create engine
engine = data_ingestion.ContractEngine()
engine.set_owner("data-team")
engine.set_domain("e-commerce")
engine.set_enrich_pii(True)

examples_dir = os.path.dirname(os.path.abspath(__file__))

# Example 1: JSON Schema
print("\n=== Example 1: JSON Schema ===")
with open(os.path.join(examples_dir, "sample_json_schema.json"), "rb") as f:
    content = f.read()

contract = engine.process_bytes(content, format_hint="json_schema", source_path="sample_json_schema.json")
print(f"Contract name: {contract['name']}")
print(f"Fields ({len(contract['fields'])}):")
for field in contract["fields"]:
    pii_marker = " [PII]" if field.get("pii") else ""
    print(f"  - {field['name']} ({field['logical_type']}) nullable={field['nullable']}{pii_marker}")

# CSV output
csv_output = engine.process_to_format(content, format_hint="json_schema", output_format="csv")
print("\nCSV Output:")
print(csv_output)

# Example 2: CSV Data Dictionary
print("\n=== Example 2: CSV Data Dictionary ===")
with open(os.path.join(examples_dir, "sample_data_dictionary.csv"), "rb") as f:
    content = f.read()

contract2 = engine.process_bytes(content, format_hint="csv", source_path="sample_data_dictionary.csv")
print(f"Contract name: {contract2['name']}")
print(f"Fields: {len(contract2['fields'])}")

yaml_output = engine.process_to_format(content, format_hint="csv", output_format="yaml")
print("\nYAML Output:")
print(yaml_output[:1000])  # First 1000 chars

# Example 3: Validation
print("\n=== Example 3: Validation ===")
validation = engine.validate_contract(contract)
print(f"Valid: {validation['valid']}")
print(f"Warnings: {validation['warnings']}")
print(f"Errors: {validation['errors']}")

print("\n=== All examples complete ===")
