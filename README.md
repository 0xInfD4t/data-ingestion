# data-ingestion

A high-performance **data contract generation engine** written in Rust. Ingest JSON datasets, JSON Schema, XML, XSD, CSV data dictionaries, and YAML schemas — and emit structured data contracts in JSON, YAML, XML, and CSV formats.

Exposed as three build targets:

| Target | Crate | Use Case |
|--------|-------|----------|
| **`aytch`** CLI | `crates/aytch` | Shell / CI pipelines |
| **Python wheel** | `crates/data-ingestion-python` | Data engineering, notebooks |
| **WASM module** | `crates/data-ingestion-wasm` | Browser / Node.js tooling |

> For full system design, see [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).

---

## Quick Start — `aytch` CLI

### Install

```bash
cargo install --path crates/aytch
```

### Usage

```
aytch --ingest --src <PATH> --output <DIR> [OPTIONS]

Options:
  -i, --ingest              Ingest a source file and generate data contracts
  -s, --src <PATH>          Path to the source file
  -o, --output <PATH>       Output directory for generated contracts
  -t, --type <TYPE>         Generation type [default: datacontract]
  -f, --format <FORMAT>     Output format: json | yaml | xml | csv | all [default: all]
      --owner <OWNER>       Data contract owner (optional)
      --domain <DOMAIN>     Data contract domain (optional)
      --no-pii              Disable PII auto-detection
  -h, --help                Print help
  -V, --version             Print version
```

### Examples

```bash
# Ingest a JSON Schema → all output formats
aytch --ingest --src examples/sample_json_schema.json --output ./contracts

# Ingest an XSD with owner/domain metadata → YAML only
aytch --ingest --src examples/sample.xsd --output ./contracts --format yaml \
      --owner "hr-team" --domain "hr"

# Ingest a CSV data dictionary → JSON contract
aytch --ingest --src examples/sample_data_dictionary.csv --output ./contracts --format json

# Ingest an XML document → all formats, no PII detection
aytch --ingest --src examples/sample.xml --output ./contracts --no-pii
```

Each run writes one file per format: `<stem>.contract.<ext>` into the output directory.

---

## Supported Formats

### Input

| Format | Description |
|--------|-------------|
| JSON Dataset | Array of JSON objects — schema inferred from data |
| JSON Schema | Draft 4 / 7 / 2019-09 / 2020-12 |
| XML | XML document — structure inferred from elements and attributes |
| XSD | XML Schema Definition |
| CSV | Data dictionary (`field_name` / `type` columns) or raw tabular data |
| YAML | Data dictionary list or JSON Schema-like structure |

### Output

All outputs represent a **data contract** — a formal specification of a data source including field names, types, nullability, constraints, PII classification, and metadata.

| Format | Use Case |
|--------|----------|
| JSON | API responses, data catalog ingestion |
| YAML | Human-readable documentation, GitOps workflows |
| XML | Enterprise system integration |
| CSV | Spreadsheet tools, data lineage systems |

---

## Build Instructions

### Prerequisites

- Rust toolchain (`rustup`) — stable channel
- For Python wheel: [`maturin`](https://github.com/PyO3/maturin) (`pip install maturin`)
- For WASM: [`wasm-pack`](https://rustwasm.github.io/wasm-pack/) (`cargo install wasm-pack`)

### Build All Targets

```powershell
# Windows (PowerShell)
.\scripts\build_all.ps1
```

```bash
# Linux / macOS
bash scripts/build_all.sh
```

### Build Individual Targets

#### CLI (`aytch`)

```powershell
# Windows
.\scripts\build_aytch.ps1
```

```bash
# Linux / macOS
bash scripts/build_aytch.sh

# Or directly:
cargo build --release -p aytch
# Binary: target/release/aytch[.exe]
```

#### Python Wheel

```powershell
# Windows
.\scripts\build_python.ps1
```

```bash
# Linux / macOS
bash scripts/build_python.sh

# Or directly (development install):
maturin develop --manifest-path crates/data-ingestion-python/Cargo.toml
python examples/demo.py
```

#### WebAssembly (Node.js)

```powershell
# Windows
.\scripts\build_wasm.ps1
```

```bash
# Linux / macOS
bash scripts/build_wasm.sh

# Or directly:
wasm-pack build crates/data-ingestion-wasm --target nodejs --out-dir dist/wasm-nodejs
node examples/demo.js
```

---

## Testing

```bash
# Run all integration tests
cargo test -p data-ingestion-core

# Run the Rust demo example
cargo run --example demo -p data-ingestion-core
```

---

## Project Structure

```
data-ingestion/
├── Cargo.toml                        # Workspace manifest
├── Cargo.lock                        # Locked dependency versions
├── crates/
│   ├── aytch/                        # CLI binary (primary entry point)
│   │   └── src/main.rs
│   ├── data-ingestion-core/          # Core library — ingestion, IR, output
│   │   ├── src/
│   │   │   ├── ingestion/            # Format readers (JSON, XML, XSD, CSV, YAML)
│   │   │   ├── ir/                   # Intermediate representation + normalizer
│   │   │   ├── contract/             # Contract builder, enricher, validator
│   │   │   └── output/               # Serializers (JSON, YAML, XML, CSV)
│   │   └── tests/
│   ├── data-ingestion-python/        # PyO3 Python bindings
│   │   ├── src/lib.rs
│   │   ├── pyproject.toml
│   │   └── data_ingestion.pyi        # Type stubs
│   └── data-ingestion-wasm/          # wasm-bindgen WASM bindings
│       └── src/lib.rs
├── docs/                             # Architecture and API documentation
│   ├── ARCHITECTURE.md
│   ├── CRATES_AND_BUILD.md
│   ├── DATA_MODELS.md
│   ├── MODULES.md
│   ├── API_AND_ERRORS.md
│   ├── PYTHON_ABI.md
│   └── WASM_STRATEGY.md
├── examples/                         # Sample input files and demo scripts
│   ├── sample_json_schema.json
│   ├── sample_json_dataset.json
│   ├── sample.xml / sample.xsd
│   ├── sample_data_dictionary.csv
│   ├── sample_schema.yaml
│   ├── demo.py
│   └── demo.js
└── scripts/                          # Build scripts (PowerShell + Bash)
    ├── build_all.{ps1,sh}
    ├── build_aytch.{ps1,sh}
    ├── build_python.{ps1,sh}
    └── build_wasm.{ps1,sh}
```

---

## Documentation

| Document | Description |
|----------|-------------|
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | System design and pipeline overview |
| [`docs/CRATES_AND_BUILD.md`](docs/CRATES_AND_BUILD.md) | Build pipeline specification |
| [`docs/DATA_MODELS.md`](docs/DATA_MODELS.md) | IR and `DataContract` data models |
| [`docs/MODULES.md`](docs/MODULES.md) | Module-level documentation |
| [`docs/API_AND_ERRORS.md`](docs/API_AND_ERRORS.md) | Public API and error types |
| [`docs/PYTHON_ABI.md`](docs/PYTHON_ABI.md) | Python package specification |
| [`docs/WASM_STRATEGY.md`](docs/WASM_STRATEGY.md) | WASM build strategy |

---

## License

MIT — see [`Cargo.toml`](Cargo.toml) for details.
