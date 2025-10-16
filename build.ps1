#!/usr/bin/env pwsh

param(
    [switch]$Compress = $false,
    [switch]$Clean = $false,
    [switch]$Test = $false,
    [switch]$Help = $false,
    [switch]$Size = $false,
    [switch]$All = $false
)

function Write-Status {
    param([string]$Message, [string]$Status = "[OK]")
    Write-Host "$Status $Message" -ForegroundColor Green
}

function Write-Warning {
    param([string]$Message)
    Write-Host "[WARN] $Message" -ForegroundColor Yellow
}

function Write-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

function Show-Help {
    Write-Host "Build script for GCP - GitHub Copier" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Usage: ./build.ps1 [OPTIONS]"
    Write-Host ""
    Write-Host "OPTIONS:"
    Write-Host "  -Compress    Build and compress with UPX"
    Write-Host "  -Clean       Clean build artifacts"
    Write-Host "  -Test        Run tests"
    Write-Host "  -Help        Show this help message"
    Write-Host "  -Size        Show binary sizes"
    Write-Host "  -All         Clean, build, compress, test and show size"
    Write-Host ""
    Write-Host "EXAMPLES:"
    Write-Host "  ./build.ps1              # Build only"
    Write-Host "  ./build.ps1 -Compress    # Build and compress"
    Write-Host "  ./build.ps1 -All         # Full build pipeline"
}

function Show-Size {
    if (Test-Path "target/release/gcp.exe") {
        $size = (Get-Item "target/release/gcp.exe").Length / 1KB
        Write-Host "=== Binary Sizes ===" -ForegroundColor Cyan
        Write-Host "Release binary: $([math]::Round($size, 2)) KB"
    } else {
        Write-Warning "Release binary not found. Run './build.ps1' first."
    }
}

function Test-UPX {
    try {
        upx --version | Out-Null
        return $true
    } catch {
        return $false
    }
}

function Build-Release {
    Write-Host "Building optimized release binary..." -ForegroundColor Cyan
    cargo build --release
    if ($LASTEXITCODE -eq 0) {
        Write-Status "Release build completed"
    } else {
        Write-Error "Build failed"
        exit 1
    }
}

function Apply-UPX {
    Write-Host "Applying UPX compression..." -ForegroundColor Cyan
    if (Test-UPX) {
        upx --best --lzma target/release/gcp.exe
        if ($LASTEXITCODE -eq 0) {
            Write-Status "UPX compression applied successfully"
        } else {
            Write-Warning "UPX compression failed"
        }
    } else {
        Write-Warning "UPX not found. Install UPX for smaller binary size"
        Write-Host "Download from: https://upx.github.io/" -ForegroundColor Gray
    }
}

function Run-Tests {
    Write-Host "Running tests..." -ForegroundColor Cyan
    cargo test
    if ($LASTEXITCODE -eq 0) {
        Write-Status "Tests completed"
    } else {
        Write-Error "Tests failed"
        exit 1
    }
}

function Test-Help {
    Write-Host "Testing help command..." -ForegroundColor Cyan
    cargo run -- --help
}

# Main execution logic
if ($Help) {
    Show-Help
    exit 0
}

if ($All) {
    Write-Host "Running full build pipeline..." -ForegroundColor Cyan
    cargo clean
    Write-Status "Build artifacts cleaned"
    Build-Release
    Apply-UPX
    Test-Help
    Show-Size
    Write-Status "Full pipeline completed"
    exit 0
}

if ($Clean) {
    Write-Host "Cleaning build artifacts..." -ForegroundColor Cyan
    cargo clean
    Write-Status "Build artifacts cleaned"
    exit 0
}

if ($Test) {
    Run-Tests
    exit 0
}

if ($Size) {
    Show-Size
    exit 0
}

# Default behavior: build
Build-Release

if ($Compress) {
    Apply-UPX
}

Write-Status "Build completed"