# aytch: Data Contract & Data Quality Engine

A high-performance **data contract generation and data quality testing engine** written in Rust. Ingest data schemas from any common format, emit structured data contracts, and automatically generate [Great Expectations](https://greatexpectations.io/)-compatible test suites — all from a single CLI or embeddable library.

Exposed as five build targets across a 7-crate workspace:

| Target | Crate | Use Case |
|--------|-------|----------|
| **`aytch`** CLI | `crates/aytch` | Shell / CI pipelines |
| **`data_ingestion`** Python wheel | `crates/data-ingestion-python` | Data engineering, notebooks |
| **`data_quality`** Python wheel | `crates/data-quality-python` | DQ testing from Python |
| **`data-ingestion`** WASM module | `crates/data-ingestion-wasm` | Browser / Node.js tooling |
| **`data-quality`** WASM module | `crates/data-quality-wasm` | Browser / Node.js DQ tooling |

> For full system design, see [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) and [`docs/DATA_QUALITY_ARCHITECTURE.md`](docs/DATA_QUALITY_ARCHITECTURE.md).

---

## Quick Start

### Step 1 — Generate data contracts from source files

```powershell
.\aytch.exe --ingest --src schema.json --output .\contracts --type datacontract
```

### Step 2 — Generate GX data quality test suites from contracts

```powershell
.\aytch.exe --dataquality --src .\contracts --output .\expectations --type greatexpectations
```

### Or do both in one command

```powershell
.\aytch.exe --ingest --dataquality --src schema.json --output .\output
```

### Process an entire folder recursively

```powershell
.\aytch.exe --ingest --dataquality --src .\schemas\ --output .\output --recursive
```

> **Best practice:** When using `--dataquality` on a folder with `--recursive`, keep source schema files and generated contracts in separate directories. Mixing them causes the engine to attempt re-ingesting its own output.

---

## Full CLI Reference

| Flag | Short | Description |
|------|-------|-------------|
| `--ingest` | `-i` | Ingest a source file and generate data contracts |
| `--dataquality` | `-q` | Generate Great Expectations data quality test suites |
| `--src <PATH>` | `-s` | Path to source file or directory |
| `--output <PATH>` | `-o` | Output directory for generated files |
| `--type <TYPE>` | `-t` | Generation type: `datacontract` \| `greatexpectations` (default: `datacontract`) |
| `--format <FORMAT>` | `-f` | Output format: `json` \| `yaml` \| `xml` \| `csv` \| `all` (default: `all`) |
| `--recursive` | `-r` | Process entire folder trees recursively |
| `--no-baseline` | | Skip the 1478 baseline GX tests; emit contract-specific suites only |
| `--suites <LIST>` | | Comma-separated list of suite names to include (filters baseline suites) |
| `--owner <OWNER>` | | Data contract owner metadata (optional) |
| `--domain <DOMAIN>` | | Data contract domain metadata (optional) |
| `--no-pii` | | Disable PII auto-detection |
| `--help` | `-h` | Print help |
| `--version` | `-V` | Print version |

### CLI Examples

```powershell
# Ingest a JSON Schema -> all output formats
.\aytch.exe --ingest --src examples\sample_json_schema.json --output .\contracts

# Ingest an XSD with owner/domain metadata -> YAML only
.\aytch.exe --ingest --src examples\sample.xsd --output .\contracts --format yaml `
            --owner "hr-team" --domain "hr"

# Ingest a CSV data dictionary -> JSON contract
.\aytch.exe --ingest --src examples\sample_data_dictionary.csv --output .\contracts --format json

# Generate GX suites for a single contract, skip baseline tests
.\aytch.exe --dataquality --src .\contracts\schema.contract.json --output .\expectations --no-baseline

# Generate only specific baseline suites
.\aytch.exe --dataquality --src .\contracts --output .\expectations `
            --suites "data_validity,data_completeness,criminal_background_check"

# Full pipeline, recursive, all suites
.\aytch.exe --ingest --dataquality --src .\schemas\ --output .\output --recursive
```

Each contract run writes one file per format: `<stem>.contract.<ext>` into the output directory.

---

## Supported Input Formats

| Format | Description |
|--------|-------------|
| JSON Dataset | Array of JSON objects — schema inferred from data |
| JSON Schema | Draft 4 / 7 / 2019-09 / 2020-12 |
| XML | XML document — structure inferred from elements and attributes |
| XSD | XML Schema Definition |
| CSV | Data dictionary (`field_name` / `type` columns) or raw tabular data |
| YAML | Data dictionary list or JSON Schema-like structure |

---

## Data Contract Output Formats

All outputs represent a **data contract** — a formal specification of a data source including field names, types, nullability, constraints, PII classification, and metadata.

| Format | Use Case |
|--------|----------|
| JSON | API responses, data catalog ingestion |
| YAML | Human-readable documentation, GitOps workflows |
| XML | Enterprise system integration |
| CSV | Spreadsheet tools, data lineage systems |

---

## Data Quality Suites

The `--dataquality` flag drives the `data-quality-core` engine, which produces **Great Expectations-compatible JSON suite files** ready to load directly into a GX checkpoint.

### Baseline Suites — 1478 Tests Across 17 Suites

Every run (unless `--no-baseline` is passed) emits a full baseline suite set covering general data quality dimensions:

| Suite | Tests | Description |
|-------|-------|-------------|
| `data_validity` | ~87 | Type correctness, range checks, regex patterns |
| `data_completeness` | ~87 | Null rates, required field presence |
| `data_consistency` | ~87 | Cross-field and cross-record consistency |
| `data_accuracy` | ~87 | Value accuracy against reference sets |
| `data_profile` | ~87 | Statistical profiling expectations |
| `data_integrity` | ~87 | Referential and relational integrity |
| `data_timeliness` | ~87 | Freshness and latency expectations |
| `data_sensitivity` | ~87 | PII and sensitivity classification checks |
| `data_uniqueness` | ~87 | Duplicate detection and key uniqueness |
| `data_business_rules` | ~87 | Domain-specific business rule enforcement |
| `data_format_consistency` | ~87 | Format standardization (dates, phones, codes) |
| `data_volume_anomalies` | ~87 | Row count and volume drift detection |
| `data_dependency_checks` | ~87 | Field dependency and conditional logic |
| `cross_system_consistency` | ~87 | Cross-dataset and cross-system alignment |
| `performance_metrics` | ~87 | Query and pipeline performance expectations |
| `security_compliance` | ~87 | Encryption, masking, and access control checks |
| `criminal_background_check` | **150** | Domain-specific CBC suite (see below) |

**Total: 1478 baseline tests**

### Criminal Background Check Domain Suite

The `criminal_background_check` suite contains **150 tests (CBC001–CBC150)** covering all **68 fields** of the national criminal background check schema, including:

- Personal identifiers (SSN, name, DOB, aliases)
- Address history and verification
- Criminal record fields (offense codes, disposition, sentence)
- Court and jurisdiction metadata
- Sex offender registry checks
- Federal and state watch-list flags
- Compliance and consent fields
- Result codes and adjudication outcomes

### Contract-Specific Test Generation

In addition to baseline suites, the engine analyzes each `DataContract` and generates **customized GX expectations** tailored to the actual schema:

| Analyzer | What It Generates |
|----------|-------------------|
| `schema_analyzer` | Table-level expectations (row count ranges, column set) |
| `field_analyzer` | Per-column type, nullability, and value expectations |
| `constraint_analyzer` | Min/max, regex, enum, and uniqueness expectations |
| `pii_analyzer` | Masking, encryption, and access-control expectations for PII fields |

### Output Structure

```
<output>/
  baseline/
    data_validity.json
    data_completeness.json
    data_consistency.json
    data_accuracy.json
    data_profile.json
    data_integrity.json
    data_timeliness.json
    data_sensitivity.json
    data_uniqueness.json
    data_business_rules.json
    data_format_consistency.json
    data_volume_anomalies.json
    data_dependency_checks.json
    cross_system_consistency.json
    performance_metrics.json
    security_compliance.json
    criminal_background_check.json
    summary.csv          ← suite name, test count, pass/fail totals
    manifest.json        ← metadata: generated_at, engine version, suite list
  contract_specific/
    <stem>_schema.json
    <stem>_fields.json
    <stem>_constraints.json
    <stem>_pii.json
```

All JSON files are **GX-compatible** and can be loaded directly via `great_expectations.core.ExpectationSuite.from_json()`.

---

## Architecture

The project is a **Cargo workspace** with 7 crates:

```
crates/
  aytch/                    ← CLI binary (primary entry point)
    src/main.rs
  data-ingestion-core/      ← Ingestion + contract generation engine
    src/
      ingestion/            ← Format readers (JSON, JSON Schema, XML, XSD, CSV, YAML)
      ir/                   ← Intermediate representation + normalizer
      contract/             ← Contract builder, enricher, validator
      output/               ← Serializers (JSON, YAML, XML, CSV)
    tests/
  data-ingestion-wasm/      ← wasm-bindgen WASM bindings for ingestion
    src/lib.rs
  data-ingestion-python/    ← PyO3 Python bindings (data_ingestion module)
    src/lib.rs
    pyproject.toml
    data_ingestion.pyi      ← Type stubs
  data-quality-core/        ← GX suite generation engine
    src/
      suites/               ← 17 baseline suite generators
      contract_analyzer/    ← Schema, field, constraint, PII analyzers
      expectations/         ← Expectation types + serializer
      output/               ← GX JSON/YAML writers, manifest, summary CSV
    tests/
  data-quality-wasm/        ← wasm-bindgen WASM bindings for DQ
    src/lib.rs
  data-quality-python/      ← PyO3 Python bindings (data_quality module)
    src/lib.rs
    pyproject.toml
    data_quality.pyi        ← Type stubs
```

### Pipeline

```
Source File(s)
     │
     ▼
data-ingestion-core
  ├── Format Reader  (JSON / JSON Schema / XML / XSD / CSV / YAML)
  ├── IR Normalizer  (unified intermediate representation)
  ├── Contract Builder + Enricher + Validator
  └── Output Serializer  (JSON / YAML / XML / CSV)
     │
     ▼
DataContract(s)
     │
     ▼
data-quality-core
  ├── Baseline Suite Generators  (16 general + 1 CBC domain suite)
  ├── Contract Analyzer          (schema / field / constraint / PII)
  └── GX Output Writers          (JSON suites + summary.csv + manifest.json)
     │
     ▼
Great Expectations Suite Files
```

---

## Build Instructions

### Prerequisites

- Rust toolchain (`rustup`) — stable channel
- For Python wheels: [`maturin`](https://github.com/PyO3/maturin) (`pip install maturin`)
- For WASM: [`wasm-pack`](https://rustwasm.github.io/wasm-pack/) (`cargo install wasm-pack`)

### Build All Targets

```powershell
# Windows (PowerShell) — builds CLI + both Python wheels + both WASM packages
.\scripts\build_all.ps1
```

```bash
# Linux / macOS
bash scripts/build_all.sh
```

### Build Individual Targets

#### CLI (`aytch`)

```powershell
.\scripts\build_aytch.ps1
# Binary: target/release/aytch.exe
```

```bash
bash scripts/build_aytch.sh
# Or: cargo build --release -p aytch
```

#### Python Wheels

```powershell
.\scripts\build_python.ps1              # data_ingestion wheel
.\scripts\build_dataquality_python.ps1  # data_quality wheel
```

```bash
bash scripts/build_python.sh
# Or (dev install): maturin develop --manifest-path crates/data-ingestion-python/Cargo.toml
```

#### WASM Packages

```powershell
.\scripts\build_wasm.ps1            # data-ingestion WASM (Node.js target)
.\scripts\build_dataquality_wasm.ps1    # data-quality WASM (Node.js target)
```

```bash
bash scripts/build_wasm.sh
# Or: wasm-pack build crates/data-ingestion-wasm --target nodejs --out-dir dist/wasm-nodejs
```

---

## Testing

```powershell
# Ingestion engine — 83 integration tests
cargo test -p data-ingestion-core

# Data quality engine — 83 tests (verifies all 1478 baseline GX tests)
cargo test -p data-quality-core

# Run the Rust demo example
cargo run --example demo -p data-ingestion-core
```

---

## Python API

### Installation

```bash
pip install data_ingestion data_quality   # from built wheels
# or dev install:
maturin develop --manifest-path crates/data-ingestion-python/Cargo.toml
maturin develop --manifest-path crates/data-quality-python/Cargo.toml
```

### Usage

```python
import data_ingestion, data_quality

# --- Ingest a schema and generate a data contract ---
engine = data_ingestion.ContractEngine()
contract = engine.process_bytes(
    open("schema.json", "rb").read(),
    format_hint="json_schema"
)
engine.write_contract(contract, output_dir="./contracts", formats=["json", "yaml"])

# --- Generate GX data quality suites from the contract ---
dq = data_quality.DqEngine()
suite_set = dq.generate_from_contract(contract)
dq.write_suite_set(suite_set, output_dir="./expectations")

# --- Or generate baseline suites only ---
baseline = dq.generate_baseline()
dq.write_suite_set(baseline, output_dir="./expectations/baseline")
```

See [`crates/data-ingestion-python/data_ingestion.pyi`](crates/data-ingestion-python/data_ingestion.pyi) and [`crates/data-quality-python/data_quality.pyi`](crates/data-quality-python/data_quality.pyi) for full type stubs.

---

## Documentation

| Document | Description |
|----------|-------------|
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | Ingestion system design and pipeline overview |
| [`docs/DATA_QUALITY_ARCHITECTURE.md`](docs/DATA_QUALITY_ARCHITECTURE.md) | Data quality engine design and suite structure |
| [`docs/CRATES_AND_BUILD.md`](docs/CRATES_AND_BUILD.md) | Build pipeline specification for all 7 crates |
| [`docs/DATA_MODELS.md`](docs/DATA_MODELS.md) | IR and `DataContract` data models |
| [`docs/DQ_DATA_MODELS.md`](docs/DQ_DATA_MODELS.md) | Data quality expectation and suite data models |
| [`docs/DQ_EXPECTATIONS.md`](docs/DQ_EXPECTATIONS.md) | GX expectation types and serialization |
| [`docs/DQ_OUTPUT_AND_BINDINGS.md`](docs/DQ_OUTPUT_AND_BINDINGS.md) | DQ output writers and Python/WASM bindings |
| [`docs/DQ_SUITES_AND_ANALYZER.md`](docs/DQ_SUITES_AND_ANALYZER.md) | Suite generators and contract analyzer modules |
| [`docs/MODULES.md`](docs/MODULES.md) | Module-level documentation for ingestion crates |
| [`docs/API_AND_ERRORS.md`](docs/API_AND_ERRORS.md) | Public API and error types |
| [`docs/PYTHON_ABI.md`](docs/PYTHON_ABI.md) | Python package ABI specification |
| [`docs/WASM_STRATEGY.md`](docs/WASM_STRATEGY.md) | WASM build strategy and browser/Node.js usage |

---

## License

MIT — see [`Cargo.toml`](Cargo.toml) for details.
