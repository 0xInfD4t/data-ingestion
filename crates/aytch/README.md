# aytch

A high-performance data contract and data quality suite generator CLI, written in Rust.

## Installation

```bash
pip install aytch
```

## Usage

```bash
# Step 1: Generate data contracts from source files
aytch --action ingest --src schema.json --output ./contracts --type datacontract

# Step 2: Generate Great Expectations test suites from contracts
aytch --action dataquality --src ./contracts --output ./expectations --type greatexpectations

# Step 3: Run GX test suites against actual data
aytch --action dqrun --src ./data/transactions.csv --tests ./expectations/schema --output ./results --type greatexpectations

# With contracts for auditing/traceability
aytch --action dqrun --src ./data/ --tests ./expectations/ --contracts ./contracts/ --output ./results --type greatexpectations --recursive

# Process entire folder recursively
aytch --action ingest --src ./schemas/ --output ./contracts --recursive
aytch --action dataquality --src ./contracts --output ./expectations --recursive

# Get help
aytch --help
```

## CLI Reference

| Flag | Short | Description |
|------|-------|-------------|
| `--action <ACTION>` | `-a` | Action: `ingest`, `dataquality`, or `dqrun` |
| `--src <PATH>` | `-s` | Source file or folder |
| `--output <PATH>` | `-o` | Output directory |
| `--type <TYPE>` | `-t` | `datacontract` or `greatexpectations` |
| `--format <FORMAT>` | `-f` | `json`, `yaml`, `xml`, `csv`, or `all` (ingest only) |
| `--tests <PATH>` | | GX suite files to execute (dqrun only) |
| `--contracts <PATH>` | | Data contracts for auditing/traceability (dqrun only) |
| `--recursive` | `-r` | Process folder recursively |
| `--no-baseline` | | Skip 1478 baseline tests (dataquality only) |
| `--suites <LIST>` | | Comma-separated suite names (dataquality only) |
| `--owner <OWNER>` | | Contract owner metadata |
| `--domain <DOMAIN>` | | Contract domain metadata |
| `--python <EXE>` | | Python executable for dqrun (default: auto-detect). Use to specify a venv Python |
| `--no-pii` | | Disable PII auto-detection |

> **Note**: The `--ingest` and `--dataquality` flags from earlier versions are still supported but deprecated. Use `--action ingest` and `--action dataquality` instead.

## Supported Input Formats

- JSON datasets
- JSON Schema (Draft 4/7/2019-09/2020-12)
- XML documents
- XSD (XML Schema Definition)
- CSV data dictionaries
- YAML schemas

## Output

- **Data contracts** (`--action ingest`): JSON, YAML, XML, CSV
- **Data quality suites** (`--action dataquality`): Great Expectations-compatible JSON suite files (1478+ baseline tests)
- **Validation results** (`--action dqrun`): Per-suite `*_results.json` + `validation_summary.json`

## Prerequisites for `--action dqrun`

**Option A: Install system-wide**
```bash
pip install great-expectations pandas
```

**Option B: Use a dedicated venv (recommended)**
```bash
python -m venv gx-venv
gx-venv\Scripts\activate
pip install great-expectations pandas
# Then use --python to point aytch at this venv:
aytch --action dqrun --src ./data --tests ./expectations --output ./results \
      --type greatexpectations --python gx-venv\Scripts\python.exe
```

Or install from the dev requirements file:
```bash
pip install -r requirements-dev.txt
```

## Specifying a Python environment for `--action dqrun`

Use `--python` to point `aytch` at a specific Python executable (e.g. a venv with `great-expectations` installed):

**Specifying a Python environment:**
```bash
# Use a specific venv with great-expectations installed
aytch --action dqrun \
  --src ./data/transactions.csv \
  --tests ./expectations/transactions \
  --output ./results \
  --type greatexpectations \
  --python .venv/Scripts/python.exe
```
