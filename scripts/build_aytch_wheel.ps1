$ErrorActionPreference = "Stop"

Write-Host "=== Building aytch Python wheel ===" -ForegroundColor Cyan

function Invoke-Maturin {
    param([string[]]$Arguments)
    if (Get-Command maturin -ErrorAction SilentlyContinue) {
        & maturin @Arguments
        if ($LASTEXITCODE -ne 0) { throw "maturin exited with code $LASTEXITCODE" }
    } elseif (Get-Command python -ErrorAction SilentlyContinue) {
        & python -m maturin @Arguments
        if ($LASTEXITCODE -ne 0) {
            Write-Host "maturin module not found, installing..." -ForegroundColor Yellow
            pip install maturin
            & python -m maturin @Arguments
            if ($LASTEXITCODE -ne 0) { throw "maturin exited with code $LASTEXITCODE" }
        }
    } else {
        throw "Neither 'maturin' nor 'python' found on PATH."
    }
}

New-Item -ItemType Directory -Force -Path "dist\aytch" | Out-Null

Invoke-Maturin @(
    "build",
    "--manifest-path", "crates\aytch\Cargo.toml",
    "--release",
    "--out", "dist\aytch"
)

$wheel = Get-ChildItem "dist\aytch\*.whl" | Select-Object -First 1
if ($wheel) {
    Write-Host "=== aytch wheel built ===" -ForegroundColor Cyan
    Write-Host "Wheel: $($wheel.FullName)" -ForegroundColor Green
    Write-Host ""
    Write-Host "Install with:" -ForegroundColor Yellow
    Write-Host "  pip install `"$($wheel.FullName)`""
    Write-Host ""
    Write-Host "Or install in development mode:" -ForegroundColor Yellow
    Write-Host "  maturin develop --manifest-path crates\aytch\Cargo.toml"
} else {
    Write-Host "No wheel found in dist\aytch\" -ForegroundColor Red
    exit 1
}
