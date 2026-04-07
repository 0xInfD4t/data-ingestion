$ErrorActionPreference = "Stop"
Write-Host "=== Building data-quality WASM package ===" -ForegroundColor Cyan

if (-not (Get-Command wasm-pack -ErrorAction SilentlyContinue)) {
    Write-Host "wasm-pack not found, installing..." -ForegroundColor Yellow
    cargo install wasm-pack
    $env:PATH = [System.Environment]::GetEnvironmentVariable("PATH", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("PATH", "User")
}

wasm-pack build crates/data-quality-wasm --target bundler --out-dir ../../dist/dq-wasm-bundler --release
wasm-pack build crates/data-quality-wasm --target nodejs --out-dir ../../dist/dq-wasm-nodejs --release

Write-Host "=== data-quality WASM build complete ===" -ForegroundColor Cyan
