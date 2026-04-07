#!/usr/bin/env bash
set -euo pipefail

echo "=== Building all targets ==="

# Run tests first
echo "Running tests..."
cargo test -p data-ingestion-core

# Build WASM
bash "$(dirname "$0")/build_wasm.sh"

# Build Python
bash "$(dirname "$0")/build_python.sh"

echo "=== All builds complete ==="
