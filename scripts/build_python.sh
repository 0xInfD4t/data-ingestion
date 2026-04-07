#!/usr/bin/env bash
set -euo pipefail

echo "=== Building Python wheel ==="

# Determine maturin invocation
if command -v maturin &> /dev/null; then
    MATURIN="maturin"
elif python -m maturin --version &> /dev/null 2>&1; then
    MATURIN="python -m maturin"
elif python3 -m maturin --version &> /dev/null 2>&1; then
    MATURIN="python3 -m maturin"
else
    echo "maturin not found. Installing..."
    pip install maturin
    MATURIN="python -m maturin"
fi

echo "Using: $MATURIN"

# Create output directory
mkdir -p dist/python

# Build wheel
$MATURIN build \
    --manifest-path crates/data-ingestion-python/Cargo.toml \
    --release \
    --out dist/python/

echo "=== Python wheel built ==="
echo "Wheel in dist/python/"
