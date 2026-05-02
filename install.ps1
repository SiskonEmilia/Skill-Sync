$ErrorActionPreference = "Stop"

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Error "cargo is not available. Please install Rust from https://rustup.rs"
    exit 1
}

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Push-Location $scriptDir

try {
    cargo build --release
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    .\target\release\sync install
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
} finally {
    Pop-Location
}

Write-Host "skill-sync installed successfully."
