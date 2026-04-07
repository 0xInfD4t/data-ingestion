# WASM Strategy

> Part of the [`data-ingestion`](ARCHITECTURE.md) architecture documentation.

This document covers the WebAssembly compilation strategy, `wasm-bindgen` API surface, JavaScript/TypeScript usage, and build configuration.

---

## Table of Contents

1. [Compilation Target](#1-compilation-target)
2. [Crate Structure](#2-crate-structure)
3. [Exported Functions](#3-exported-functions)
4. [WasmContractBuilder Class](#4-wasmcontractbuilder-class)
5. [Error Propagation to JavaScript](#5-error-propagation-to-javascript)
6. [TypeScript Type Definitions](#6-typescript-type-definitions)
7. [Build Configuration](#7-build-configuration)
8. [JavaScript Usage Examples](#8-javascript-usage-examples)
9. [WASM Binary Size Optimization](#9-wasm-binary-size-optimization)

---

## 1. Compilation Target

The WASM crate targets `wasm32-unknown-unknown`. This imposes the following constraints:

| Constraint | Reason | Solution |
|---|---|---|
| No `std::fs` | No filesystem in browser | Caller passes `&[u8]` |
| No `std::thread` | Single-threaded WASM | All operations are synchronous |
| No `std::time::SystemTime` | Not available in WASM | Accept timestamps as `Option<String>` parameter |
| UUID generation | Requires entropy source | `uuid` crate with `getrandom` + `js` feature |
| Panic handling | Panics are opaque in WASM | `console_error_panic_hook` crate |

### WASM Compatibility in `data-ingestion-core`

All code in `data-ingestion-core` must be `wasm32-unknown-unknown` safe. File I/O is gated behind `#[cfg(not(target_arch = "wasm32"))]`:

```rust
// Available on native only
#[cfg(not(target_arch = "wasm32"))]
pub fn ingest_file(path: &std::path::Path) -> Result<DataContract, IngestionError> {
    let bytes = std::fs::read(path)?;
    let hint = FormatHint {
        filename: path.file_name().and_then(|n| n.to_str()).map(String::from),
        ..Default::default()
    };
    ingest_bytes(&bytes, hint)
}

// Available on all targets including WASM
pub fn ingest_bytes(input: &[u8], hint: FormatHint) -> Result<DataContract, IngestionError> {
    // ... pure byte-slice logic
}
```

---

## 2. Crate Structure

**Location:** [`crates/data-ingestion-wasm/src/`](../crates/data-ingestion-wasm/src/)

```
crates/data-ingestion-wasm/
├── Cargo.toml
├── pkg/                    # wasm-pack output (gitignored)
└── src/
    ├── lib.rs              # Module root; sets up panic hook
    ├── bindings.rs         # All #[wasm_bindgen] exports
    └── utils.rs            # JS interop helpers (error conversion, etc.)
```

**`Cargo.toml`:**
```toml
[package]
name = "data-ingestion-wasm"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
data-ingestion-core = { path = "../data-ingestion-core" }
wasm-bindgen = "0.2"
js-sys = "0.3"
serde_json = "1"
console_error_panic_hook = { version = "0.1", optional = true }

[features]
default = ["console_error_panic_hook"]
```

**`src/lib.rs`:**
```rust
mod bindings;
mod utils;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
```

---

## 3. Exported Functions

All exported functions are defined in [`crates/data-ingestion-wasm/src/bindings.rs`](../crates/data-ingestion-wasm/src/bindings.rs).

All functions accept input as `&[u8]` (a `Uint8Array` in JavaScript) and return `Result<String, JsValue>`. On error, the `JsValue` is a JSON-serialized error object (see [Section 5](#5-error-propagation-to-javascript)).

```rust
use wasm_bindgen::prelude::*;
use data_ingestion_core::{Engine, OutputFormat, FormatHint};

/// Ingest input bytes and return a DataContract serialized as JSON.
/// filename_hint: optional original filename for format detection
/// config_json: optional JSON-encoded ContractBuilderConfig overrides
#[wasm_bindgen]
pub fn ingest_to_contract_json(
    input: &[u8],
    filename_hint: Option<String>,
    config_json: Option<String>,
) -> Result<String, JsValue>;

/// Ingest input bytes and return a DataContract serialized as YAML.
#[wasm_bindgen]
pub fn ingest_to_contract_yaml(
    input: &[u8],
    filename_hint: Option<String>,
    config_json: Option<String>,
) -> Result<String, JsValue>;

/// Ingest input bytes and return a DataContract serialized as XML.
#[wasm_bindgen]
pub fn ingest_to_contract_xml(
    input: &[u8],
    filename_hint: Option<String>,
    config_json: Option<String>,
) -> Result<String, JsValue>;

/// Ingest input bytes and return a DataContract serialized as CSV.
#[wasm_bindgen]
pub fn ingest_to_contract_csv(
    input: &[u8],
    filename_hint: Option<String>,
    config_json: Option<String>,
) -> Result<String, JsValue>;

/// Detect the source format of the input without full parsing.
/// Returns one of: "JsonDataset", "JsonSchema", "DataDictionary",
///                 "DataSchema", "DataStructure", "Xml", "Xsd"
#[wasm_bindgen]
pub fn detect_format(
    input: &[u8],
    filename_hint: Option<String>,
) -> String;

/// Convert an existing DataContract JSON string to another output format.
/// output_format: "json" | "yaml" | "xml" | "csv"
#[wasm_bindgen]
pub fn convert_contract(
    contract_json: &str,
    output_format: &str,
) -> Result<String, JsValue>;

/// Validate a DataContract JSON string.
/// Returns true if valid, throws JsValue error if invalid.
#[wasm_bindgen]
pub fn validate_contract_json(contract_json: &str) -> Result<bool, JsValue>;
```

### `config_json` Parameter Schema

The optional `config_json` parameter accepts a JSON object with any subset of these fields:

```json
{
  "contract_name": "MyContract",
  "contract_version": "1.0.0",
  "owner": {
    "team": "data-platform",
    "email": "data@example.com",
    "slack_channel": "#data-platform"
  },
  "domain": "identity",
  "tags": ["pii", "user-data"],
  "enrich_pii": true,
  "default_classification": "Internal",
  "primary_key_hints": ["user_id"],
  "sla": {
    "freshness_interval": "PT1H",
    "availability_percent": 99.9,
    "max_latency_ms": 500,
    "retention_days": 90
  },
  "lineage": {
    "source_system": "postgres",
    "source_table": "users",
    "transformation_notes": "Extracted from OLTP"
  }
}
```

---

## 4. `WasmContractBuilder` Class

A stateful JS class for incremental configuration, useful when building contracts interactively in a UI:

```rust
#[wasm_bindgen]
pub struct WasmContractBuilder {
    config: ContractBuilderConfig,
}

#[wasm_bindgen]
impl WasmContractBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmContractBuilder;

    /// Set the contract name
    pub fn set_name(&mut self, name: &str);

    /// Set the semantic version string (e.g. "1.0.0")
    pub fn set_version(&mut self, version: &str);

    /// Set owner team and email
    pub fn set_owner(&mut self, team: &str, email: &str);

    /// Set the data domain (e.g. "identity", "finance")
    pub fn set_domain(&mut self, domain: &str);

    /// Set tags as a comma-separated string
    pub fn set_tags(&mut self, tags: &str);

    /// Enable or disable automatic PII field detection
    pub fn enable_pii_detection(&mut self, enabled: bool);

    /// Set the default data classification
    /// Accepts: "Public" | "Internal" | "Confidential" | "Restricted"
    pub fn set_default_classification(&mut self, classification: &str);

    /// Add a primary key hint field name
    pub fn add_primary_key_hint(&mut self, field_name: &str);

    /// Ingest bytes and return the contract in the specified output format.
    /// output_format: "json" | "yaml" | "xml" | "csv"
    pub fn build_from_bytes(
        &self,
        input: &[u8],
        filename_hint: Option<String>,
        output_format: &str,
    ) -> Result<String, JsValue>;

    /// Serialize the current config as JSON (for debugging/inspection)
    pub fn config_json(&self) -> String;
}
```

---

## 5. Error Propagation to JavaScript

All `IngestionError`, `TransformError`, and `OutputError` values are converted to `JsValue` via a `From` implementation in [`crates/data-ingestion-wasm/src/utils.rs`](../crates/data-ingestion-wasm/src/utils.rs):

```rust
pub fn to_js_error(err: IngestionError) -> JsValue {
    let obj = js_sys::Object::new();
    // Serialize as JSON string for maximum compatibility
    let json = serde_json::json!({
        "code": err.code(),
        "message": err.to_string(),
        "details": err.details(),
    });
    JsValue::from_str(&json.to_string())
}
```

**Error JSON shape:**
```json
{
  "code": "PARSE_ERROR",
  "message": "Failed to parse JSON Schema: missing $schema keyword",
  "details": {
    "source_format": "JsonSchema",
    "field_path": "$.properties.user.properties.email"
  }
}
```

**Error codes:**

| Rust error variant | `code` string |
|---|---|
| `IngestionError::UnsupportedFormat` | `UNSUPPORTED_FORMAT` |
| `IngestionError::ParseError` | `PARSE_ERROR` |
| `IngestionError::DetectionFailed` | `DETECTION_FAILED` |
| `IngestionError::UnresolvedReference` | `UNRESOLVED_REFERENCE` |
| `IngestionError::CircularReference` | `CIRCULAR_REFERENCE` |
| `TransformError::InvalidIr` | `INVALID_IR` |
| `TransformError::TypeResolutionFailed` | `TYPE_RESOLUTION_FAILED` |
| `OutputError::SerializationFailed` | `SERIALIZATION_FAILED` |
| `OutputError::UnsupportedOutputFormat` | `UNSUPPORTED_OUTPUT_FORMAT` |

---

## 6. TypeScript Type Definitions

`wasm-pack` auto-generates `.d.ts` files. The generated types will be:

```typescript
/* data_ingestion_wasm.d.ts */

/**
 * Ingest input bytes and return a DataContract as a JSON string.
 * @param input - Raw file bytes as Uint8Array
 * @param filename_hint - Optional original filename for format detection
 * @param config_json - Optional JSON-encoded configuration overrides
 * @throws {string} JSON-encoded error object on failure
 */
export function ingest_to_contract_json(
  input: Uint8Array,
  filename_hint?: string,
  config_json?: string
): string;

export function ingest_to_contract_yaml(
  input: Uint8Array,
  filename_hint?: string,
  config_json?: string
): string;

export function ingest_to_contract_xml(
  input: Uint8Array,
  filename_hint?: string,
  config_json?: string
): string;

export function ingest_to_contract_csv(
  input: Uint8Array,
  filename_hint?: string,
  config_json?: string
): string;

export function detect_format(
  input: Uint8Array,
  filename_hint?: string
): string;

export function convert_contract(
  contract_json: string,
  output_format: "json" | "yaml" | "xml" | "csv"
): string;

export function validate_contract_json(contract_json: string): boolean;

export class WasmContractBuilder {
  constructor();
  set_name(name: string): void;
  set_version(version: string): void;
  set_owner(team: string, email: string): void;
  set_domain(domain: string): void;
  set_tags(tags: string): void;
  enable_pii_detection(enabled: boolean): void;
  set_default_classification(
    classification: "Public" | "Internal" | "Confidential" | "Restricted"
  ): void;
  add_primary_key_hint(field_name: string): void;
  build_from_bytes(
    input: Uint8Array,
    filename_hint: string | undefined,
    output_format: "json" | "yaml" | "xml" | "csv"
  ): string;
  config_json(): string;
  /** Must be called to free WASM memory when done */
  free(): void;
}

/** Initialize the WASM module. Must be awaited before calling any function. */
export default function init(input?: RequestInfo | URL | Response | BufferSource): Promise<InitOutput>;
```

---

## 7. Build Configuration

### Prerequisites

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
```

### `.cargo/config.toml`

```toml
[target.wasm32-unknown-unknown]
# Optimize for binary size
rustflags = ["-C", "opt-level=z", "-C", "lto=thin"]
```

### Build Commands

**For bundlers (webpack, vite, rollup) — recommended for web apps:**
```bash
wasm-pack build crates/data-ingestion-wasm \
  --target bundler \
  --out-dir ../../dist/wasm/bundler \
  --release
```

**For direct browser `<script type="module">` usage:**
```bash
wasm-pack build crates/data-ingestion-wasm \
  --target web \
  --out-dir ../../dist/wasm/web \
  --release
```

**For Node.js (CommonJS):**
```bash
wasm-pack build crates/data-ingestion-wasm \
  --target nodejs \
  --out-dir ../../dist/wasm/nodejs \
  --release
```

### Output Structure

```
dist/wasm/bundler/
├── data_ingestion_wasm.js          # ESM JS glue code
├── data_ingestion_wasm.d.ts        # TypeScript declarations
├── data_ingestion_wasm_bg.wasm     # WASM binary
├── data_ingestion_wasm_bg.wasm.d.ts
└── package.json                    # npm package manifest
```

### `package.json` (auto-generated by wasm-pack)

```json
{
  "name": "data-ingestion-wasm",
  "version": "0.1.0",
  "files": [
    "data_ingestion_wasm_bg.wasm",
    "data_ingestion_wasm.js",
    "data_ingestion_wasm.d.ts"
  ],
  "main": "data_ingestion_wasm.js",
  "types": "data_ingestion_wasm.d.ts",
  "sideEffects": false
}
```

---

## 8. JavaScript Usage Examples

### Browser (ESM bundler)

```javascript
import init, {
  ingest_to_contract_json,
  detect_format,
  WasmContractBuilder,
} from 'data-ingestion-wasm';

// Initialize WASM module once at app startup
await init();

// --- Simple one-liner ---
const fileInput = document.getElementById('schema-file');
fileInput.addEventListener('change', async (e) => {
  const file = e.target.files[0];
  const bytes = new Uint8Array(await file.arrayBuffer());

  try {
    const contractJson = ingest_to_contract_json(bytes, file.name);
    const contract = JSON.parse(contractJson);
    console.log('Contract:', contract);
  } catch (errJson) {
    const err = JSON.parse(errJson);
    console.error(`[${err.code}] ${err.message}`);
  }
});

// --- Builder pattern ---
const builder = new WasmContractBuilder();
builder.set_name('UserProfile');
builder.set_version('2.0.0');
builder.set_owner('data-team', 'data@example.com');
builder.set_domain('identity');
builder.enable_pii_detection(true);
builder.set_default_classification('Internal');

const response = await fetch('/schemas/user_profile.xsd');
const bytes = new Uint8Array(await response.arrayBuffer());
const yaml = builder.build_from_bytes(bytes, 'user_profile.xsd', 'yaml');

builder.free(); // Always free WASM memory
```

### Node.js

```javascript
const { readFileSync } = require('fs');
const {
  ingest_to_contract_json,
  convert_contract,
} = require('./dist/wasm/nodejs/data_ingestion_wasm.js');

const bytes = new Uint8Array(readFileSync('schema.json'));
const contractJson = ingest_to_contract_json(bytes, 'schema.json');

// Convert to YAML
const yaml = convert_contract(contractJson, 'yaml');
console.log(yaml);
```

### Wasmtime-py (Python WASM fallback)

For environments where native PyO3 wheels are unavailable (e.g., Pyodide in browser):

```python
from wasmtime import Store, Module, Instance, Linker, WasiConfig
import pathlib

store = Store()
wasi = WasiConfig()
wasi.inherit_stdout()
store.set_wasi(wasi)

wasm_path = pathlib.Path("dist/wasm/data_ingestion_wasm_bg.wasm")
module = Module.from_file(store.engine, str(wasm_path))
linker = Linker(store.engine)
linker.define_wasi()
instance = linker.instantiate(store, module)

# Note: For full WASM+Python integration, use the PyO3 wheel (see PYTHON_ABI.md)
# The wasmtime-py path is a fallback for restricted environments only
```

---

## 9. WASM Binary Size Optimization

Target binary size: **< 2 MB** (gzipped).

Strategies applied:

| Strategy | Configuration | Expected saving |
|---|---|---|
| Release mode | `--release` flag | ~60% vs debug |
| LTO | `lto = "thin"` in `.cargo/config.toml` | ~10–15% |
| Optimize for size | `opt-level = "z"` | ~5–10% |
| Strip debug symbols | `wasm-pack` does this automatically in release | ~20% |
| `wasm-opt` | `wasm-pack` runs `wasm-opt -Oz` automatically | ~10–15% |
| Avoid large deps | `serde_yaml` excluded from WASM if not needed | varies |
| Feature flags | Only enable needed format readers | varies |

**Checking binary size:**
```bash
ls -lh dist/wasm/bundler/data_ingestion_wasm_bg.wasm
gzip -k dist/wasm/bundler/data_ingestion_wasm_bg.wasm
ls -lh dist/wasm/bundler/data_ingestion_wasm_bg.wasm.gz
```

**`twiggy` for size profiling:**
```bash
cargo install twiggy
twiggy top dist/wasm/bundler/data_ingestion_wasm_bg.wasm
```
