$ErrorActionPreference = "Stop"

Write-Host "=== Building all targets ===" -ForegroundColor Cyan

# Run tests first
Write-Host "Running tests..." -ForegroundColor Yellow
cargo test -p data-ingestion-core

# Build WASM
& "$PSScriptRoot\build_wasm.ps1"

# Build Python
& "$PSScriptRoot\build_python.ps1"

# Build aytch CLI
& "$PSScriptRoot\build_aytch.ps1"

Write-Host "=== All builds complete ===" -ForegroundColor Green
