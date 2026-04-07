#!/usr/bin/env bash
set -euo pipefail

echo "=== Building aytch CLI ==="

# Build release binary
echo "Compiling aytch (release)..."
cargo build -p aytch --release

BINARY="target/release/aytch"

if [ -f "$BINARY" ]; then
    SIZE=$(du -sh "$BINARY" | cut -f1)
    echo "=== Build complete ==="
    echo "Binary: $BINARY ($SIZE)"
    echo ""
    echo "Usage:"
    echo "  ./target/release/aytch --ingest --src <source-file> --output <output-dir> --type datacontract"
    echo ""
    echo "Examples:"
    echo "  ./target/release/aytch --ingest --src examples/sample_json_schema.json --output ./contracts --type datacontract"
    echo "  ./target/release/aytch --ingest --src examples/sample.xsd --output ./contracts --owner hr-team --domain hr"
    echo "  ./target/release/aytch --help"
else
    echo "Binary not found at expected path: $BINARY"
    exit 1
fi
