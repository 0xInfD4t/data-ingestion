# data-ingestion

A high-performance data contract generation engine written in Rust, exposed as a Python library.

## Installation

```bash
pip install data-ingestion
```

Or build from source:
```bash
pip install maturin
maturin develop
```

## Quick Start

```python
import data_ingestion

engine = data_ingestion.ContractEngine()
engine.set_owner("data-team")
engine.set_domain("finance")
engine.set_enrich_pii(True)

# Process a JSON Schema file
with open("schema.json", "rb") as f:
    content = f.read()

# Get contract as Python dict
contract = engine.process_bytes(content, format_hint="json_schema")

# Get contract as CSV string
csv_output = engine.process_to_format(content, format_hint="json_schema", output_format="csv")

# Get contract as YAML string
yaml_output = engine.process_to_format(content, output_format="yaml")
```

## Supported Input Formats

| Format | `format_hint` value | Description |
|--------|---------------------|-------------|
| JSON Dataset | `"json"` | Array of JSON objects |
| JSON Schema | `"json_schema"` | Draft 4/7/2019-09/2020-12 |
| XML | `"xml"` | XML document |
| XSD | `"xsd"` | XML Schema Definition |
| CSV | `"csv"` | Data dictionary or raw data |
| YAML | `"yaml"` | YAML data dictionary or schema |

## Output Formats

- `"json"` — Full DataContract as JSON
- `"yaml"` — Full DataContract as YAML
- `"xml"` — Full DataContract as XML
- `"csv"` — Flat field list as CSV (16 columns)
