$ErrorActionPreference = "Stop"

Write-Host "=== Building WASM package ===" -ForegroundColor Cyan

# Check wasm-pack is installed; if not, install via cargo and retry
if (-not (Get-Command wasm-pack -ErrorAction SilentlyContinue)) {
    Write-Host "Installing wasm-pack..." -ForegroundColor Yellow
    cargo install wasm-pack
    # Refresh PATH for current session so the newly installed binary is found
    $env:PATH = [System.Environment]::GetEnvironmentVariable("PATH", "User") + ";" + [System.Environment]::GetEnvironmentVariable("PATH", "Machine")
    if (-not (Get-Command wasm-pack -ErrorAction SilentlyContinue)) {
        throw "wasm-pack was installed but is still not on PATH. Add '$env:USERPROFILE\.cargo\bin' to your PATH and re-run."
    }
}

# Build for bundler target
Write-Host "Building for bundler target..." -ForegroundColor Green
wasm-pack build crates/data-ingestion-wasm `
    --target bundler `
    --out-dir ..\..\dist\wasm-bundler `
    --release

# Build for web target
Write-Host "Building for web target..." -ForegroundColor Green
wasm-pack build crates/data-ingestion-wasm `
    --target web `
    --out-dir ..\..\dist\wasm-web `
    --release

# Build for nodejs target
Write-Host "Building for nodejs target..." -ForegroundColor Green
wasm-pack build crates/data-ingestion-wasm `
    --target nodejs `
    --out-dir ..\..\dist\wasm-nodejs `
    --release

Write-Host "=== WASM build complete ===" -ForegroundColor Cyan
Write-Host "Artifacts in dist/wasm-bundler/, dist/wasm-web/, dist/wasm-nodejs/"
