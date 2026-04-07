$ErrorActionPreference = "Stop"
Write-Host "=== Building data-quality Python wheel ===" -ForegroundColor Cyan

function Invoke-Maturin {
    param([string[]]$Arguments)
    if (Get-Command maturin -ErrorAction SilentlyContinue) {
        & maturin @Arguments
    } elseif (Get-Command python -ErrorAction SilentlyContinue) {
        & python -m maturin @Arguments
        if ($LASTEXITCODE -ne 0) {
            pip install maturin
            & python -m maturin @Arguments
        }
    } else {
        throw "Neither 'maturin' nor 'python' found on PATH."
    }
}

New-Item -ItemType Directory -Force -Path "dist\python" | Out-Null
Invoke-Maturin @("build", "--manifest-path", "crates\data-quality-python\Cargo.toml", "--release", "--out", "dist\python")
Write-Host "=== data-quality Python wheel built ===" -ForegroundColor Cyan
