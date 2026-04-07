$ErrorActionPreference = "Stop"

Write-Host "=== Building Python wheel ===" -ForegroundColor Cyan

# Determine maturin invocation: prefer direct command, fall back to python -m maturin
function Invoke-Maturin {
    param([string[]]$Arguments)
    
    if (Get-Command maturin -ErrorAction SilentlyContinue) {
        & maturin @Arguments
    } elseif (Get-Command python -ErrorAction SilentlyContinue) {
        # Try python -m maturin (works when maturin is installed but not on PATH)
        & python -m maturin @Arguments
        if ($LASTEXITCODE -ne 0) {
            # maturin not installed, install it and retry
            Write-Host "Installing maturin via pip..." -ForegroundColor Yellow
            & pip install maturin
            & python -m maturin @Arguments
        }
    } else {
        throw "Neither 'maturin' nor 'python' found on PATH. Install Python and run: pip install maturin"
    }
}

# Create output directory
New-Item -ItemType Directory -Force -Path "dist\python" | Out-Null

# Build wheel
Write-Host "Building Python wheel..." -ForegroundColor Green
Invoke-Maturin @(
    "build",
    "--manifest-path", "crates\data-ingestion-python\Cargo.toml",
    "--release",
    "--out", "dist\python"
)

Write-Host "=== Python wheel built ===" -ForegroundColor Cyan
Write-Host "Wheel in dist\python\"
