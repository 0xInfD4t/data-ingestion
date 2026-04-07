$ErrorActionPreference = "Stop"

Write-Host "=== Building aytch CLI ===" -ForegroundColor Cyan

# Build release binary
Write-Host "Compiling aytch (release)..." -ForegroundColor Green
cargo build -p aytch --release

if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}

$binaryPath = "target\release\aytch.exe"

if (Test-Path $binaryPath) {
    $size = (Get-Item $binaryPath).Length / 1MB
    Write-Host "=== Build complete ===" -ForegroundColor Cyan
    Write-Host "Binary: $binaryPath ($([math]::Round($size, 2)) MB)" -ForegroundColor Green
    Write-Host ""
    Write-Host "Usage:" -ForegroundColor Yellow
    Write-Host "  .\target\release\aytch.exe --ingest --src <source-file> --output <output-dir> --type datacontract"
    Write-Host ""
    Write-Host "Examples:" -ForegroundColor Yellow
    Write-Host "  .\target\release\aytch.exe --ingest --src examples\sample_json_schema.json --output .\contracts --type datacontract"
    Write-Host "  .\target\release\aytch.exe --ingest --src examples\sample.xsd --output .\contracts --owner hr-team --domain hr"
    Write-Host "  .\target\release\aytch.exe --help"
} else {
    Write-Host "Binary not found at expected path: $binaryPath" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "To also build a pip-installable wheel:" -ForegroundColor Yellow
Write-Host "  .\scripts\build_aytch_wheel.ps1"
