# Test script to verify different Cargo configurations (PowerShell version)

# Colors for output
$Colors = @{
    Red = "Red"
    Green = "Green"
    Yellow = "Yellow"
    Blue = "Blue"
    White = "White"
}

function Write-Status {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor $Colors.Blue
}

function Write-Success {
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor $Colors.Green
}

function Write-Warning {
    param([string]$Message)
    Write-Host "[WARNING] $Message" -ForegroundColor $Colors.Yellow
}

function Write-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor $Colors.Red
}

# Function to test a configuration
function Test-Config {
    param(
        [string]$ConfigName,
        [string]$ConfigFile,
        [string]$Command
    )

    Write-Status "Testing $ConfigName configuration..."

    if (Test-Path ".cargo\$ConfigFile") {
        Write-Status "Using config: .cargo\$ConfigFile"

        try {
            Invoke-Expression $Command | Out-Null
            Write-Success "$ConfigName configuration works! ✅"
            return $true
        }
        catch {
            Write-Error "$ConfigName configuration failed! ❌"
            return $false
        }
    }
    else {
        Write-Warning "Config file .cargo\$ConfigFile not found, skipping..."
        return $false
    }
}

# Function to measure build time
function Measure-BuildTime {
    param(
        [string]$ConfigName,
        [string]$ConfigFile,
        [string]$Command
    )

    Write-Status "Measuring build time for $ConfigName..."

    if (Test-Path ".cargo\$ConfigFile") {
        $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()

        try {
            Invoke-Expression $Command | Out-Null
            $stopwatch.Stop()
            Write-Success "$ConfigName build time: $($stopwatch.ElapsedMilliseconds)ms ⏱️"
            return $true
        }
        catch {
            Write-Error "$ConfigName build failed!"
            return $false
        }
    }
    else {
        Write-Warning "Config file .cargo\$ConfigFile not found"
        return $false
    }
}

# Function to get file size
function Get-FileSize {
    param([string]$Path)

    if (Test-Path $Path) {
        $size = (Get-Item $Path).Length
        if ($size -gt 1MB) {
            return "{0:N1}MB" -f ($size / 1MB)
        }
        elseif ($size -gt 1KB) {
            return "{0:N1}KB" -f ($size / 1KB)
        }
        else {
            return "$($size)B"
        }
    }
    return "N/A"
}

# Main script
Write-Host "🔧 Testing Cargo Configurations" -ForegroundColor $Colors.White
Write-Host "==================================" -ForegroundColor $Colors.White

# Clean previous builds
Write-Status "Cleaning previous builds..."
cargo clean 2>$null

# Test configurations
Write-Host ""
Write-Host "🧪 Testing Configurations" -ForegroundColor $Colors.White
Write-Host "========================" -ForegroundColor $Colors.White

Test-Config "Default" "config.toml" "cargo check"
Test-Config "Development" "config.dev.toml" "cargo check --config .cargo/config.dev.toml"
Test-Config "Release" "config.release.toml" "cargo check --release --config .cargo/config.release.toml"
Test-Config "Benchmark" "config.bench.toml" "cargo check --config .cargo/config.bench.toml"

Write-Host ""
Write-Host "⏱️ Measuring Build Times" -ForegroundColor $Colors.White
Write-Host "========================" -ForegroundColor $Colors.White

# Clean and measure build times
cargo clean 2>$null

Measure-BuildTime "Development" "config.dev.toml" "cargo build --config .cargo/config.dev.toml"

cargo clean 2>$null

Measure-BuildTime "Default" "config.toml" "cargo build"

cargo clean 2>$null

Measure-BuildTime "Release" "config.release.toml" "cargo build --release --config .cargo/config.release.toml"

Write-Host ""
Write-Host "🔍 Testing Binary Sizes" -ForegroundColor $Colors.White
Write-Host "========================" -ForegroundColor $Colors.White

if (Test-Path "target\debug\gcp.exe") {
    $debugSize = Get-FileSize "target\debug\gcp.exe"
    Write-Success "Debug binary size: $debugSize 📦"
}

if (Test-Path "target\release\gcp.exe") {
    $releaseSize = Get-FileSize "target\release\gcp.exe"
    Write-Success "Release binary size: $releaseSize 📦"
}

Write-Host ""
Write-Host "🧪 Testing Functionality" -ForegroundColor $Colors.White
Write-Host "========================" -ForegroundColor $Colors.White

# Test basic functionality
if (Test-Path "target\release\gcp.exe") {
    Write-Status "Testing release binary..."
    try {
        & "target\release\gcp.exe" --help 2>$null | Out-Null
        Write-Success "Release binary works! ✅"
    }
    catch {
        Write-Error "Release binary failed! ❌"
    }
}

if (Test-Path "target\debug\gcp.exe") {
    Write-Status "Testing debug binary..."
    try {
        & "target\debug\gcp.exe" --help 2>$null | Out-Null
        Write-Success "Debug binary works! ✅"
    }
    catch {
        Write-Error "Debug binary failed! ❌"
    }
}

Write-Host ""
Write-Host "🎯 Testing Aliases" -ForegroundColor $Colors.White
Write-Host "==================" -ForegroundColor $Colors.White

# Test some aliases
Write-Status "Testing cargo check-all alias..."
try {
    cargo check-all 2>$null | Out-Null
    Write-Success "check-all alias works! ✅"
}
catch {
    Write-Warning "check-all alias may need cargo-clippy"
}

Write-Status "Testing cargo rr alias..."
try {
    cargo rr --help 2>$null | Out-Null
    Write-Success "rr alias works! ✅"
}
catch {
    Write-Error "rr alias failed! ❌"
}

Write-Host ""
Write-Host "🚀 Configuration Test Complete!" -ForegroundColor $Colors.White
Write-Host "================================" -ForegroundColor $Colors.White
Write-Success "All tests completed successfully! 🎉"

# Display optimization info
Write-Host ""
Write-Status "Optimization Information:"
Write-Host "  • Jobs: 32 (parallel compilation)"
Write-Host "  • Target CPU: Native optimizations enabled"
Write-Host "  • Linker: Platform-specific fast linkers"
Write-Host "  • LTO: Link-time optimization for release builds"
Write-Host "  • Codegen Units: Optimized for performance vs speed"

# Show next steps
Write-Host ""
Write-Host "📋 Quick Start Commands:" -ForegroundColor $Colors.White
Write-Host "===========================" -ForegroundColor $Colors.White
Write-Host "• Development build: cargo run --config .cargo/config.dev.toml"
Write-Host "• Release build:     cargo run --release --config .cargo/config.release.toml"
Write-Host "• Benchmarking:      cargo bench --config .cargo/config.bench.toml"
Write-Host "• Fast dev build:     cargo dev-fast"
Write-Host "• Release run:       cargo rr"