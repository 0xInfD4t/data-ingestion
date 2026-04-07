# aytch

A high-performance data contract and data quality suite generator CLI, written in Rust.

## Installation

```bash
pip install aytch
```

## Usage

```bash
# Generate data contracts from source files
aytch --ingest --src schema.json --output ./contracts --type datacontract

# Generate Great Expectations test suites
aytch --dataquality --src ./contracts --output ./expectations --type greatexpectations

# Process entire folder recursively
aytch --ingest --dataquality --src ./schemas/ --output ./output --recursive

# Get help
aytch --help
```

## Supported Input Formats

- JSON datasets
- JSON Schema (Draft 4/7/2019-09/2020-12)
- XML documents
- XSD (XML Schema Definition)
- CSV data dictionaries
- YAML schemas

## Output

- **Data contracts**: JSON, YAML, XML, CSV
- **Data quality suites**: Great Expectations-compatible JSON suite files (1478+ baseline tests)
