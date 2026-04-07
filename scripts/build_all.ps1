$ErrorActionPreference = "Stop"

Write-Host "=== Building all targets ===" -ForegroundColor Cyan

# Run tests first
Write-Host "Running tests..." -ForegroundColor Yellow
cargo test -p data-ingestion-core
cargo test -p data-quality-core

# Build WASM (data-ingestion)
& "$PSScriptRoot\build_wasm.ps1"

# Build Python (data-ingestion)
& "$PSScriptRoot\build_python.ps1"

# Build aytch CLI
& "$PSScriptRoot\build_aytch.ps1"

# Build aytch Python wheel
& "$PSScriptRoot\build_aytch_wheel.ps1"

# Build data-quality WASM
& "$PSScriptRoot\build_dataquality_wasm.ps1"

# Build data-quality Python wheel
& "$PSScriptRoot\build_dataquality_python.ps1"

Write-Host "=== All builds complete ===" -ForegroundColor Green
