#!/usr/bin/env bash
set -euo pipefail

echo "=== Building WASM package ==="

# Check wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "Installing wasm-pack..."
    cargo install wasm-pack
fi

# Build for bundler target (webpack, rollup, vite)
echo "Building for bundler target..."
wasm-pack build crates/data-ingestion-wasm \
    --target bundler \
    --out-dir ../../dist/wasm-bundler \
    --release

# Build for web target (native ES modules)
echo "Building for web target..."
wasm-pack build crates/data-ingestion-wasm \
    --target web \
    --out-dir ../../dist/wasm-web \
    --release

# Build for nodejs target
echo "Building for nodejs target..."
wasm-pack build crates/data-ingestion-wasm \
    --target nodejs \
    --out-dir ../../dist/wasm-nodejs \
    --release

echo "=== WASM build complete ==="
echo "Artifacts in dist/wasm-bundler/, dist/wasm-web/, dist/wasm-nodejs/"
