#!/usr/bin/env just
set shell := ["powershell.exe", "-NoProfile", "-ExecutionPolicy", "Bypass", "-Command"]

# Default recipe - build with UPX compression by default
default: build-upx

# Build optimized release binary only
build:
    cargo build --release
    echo "✓ Release build completed"

# Build and compress with UPX (default behavior)
build-upx:
    pwsh -ExecutionPolicy Bypass -File scripts/build-upx.ps1

# UPX compression only (for existing binary)
upx:
    pwsh -ExecutionPolicy Bypass -File scripts/upx-only.ps1

# Show binary sizes with compression comparison
size:
    echo "=== Binary Size Analysis ==="
    pwsh -ExecutionPolicy Bypass -File scripts/size.ps1

# Detailed size comparison with backup
size-compare:
    echo "=== Detailed Size Comparison ==="
    pwsh -ExecutionPolicy Bypass -File scripts/size-compare.ps1

# Clean build artifacts
clean:
    cargo clean
    echo "✓ Build artifacts cleaned"

# Test the application
test:
    cargo test
    echo "✓ Tests completed"

# Run help command
help-test:
    cargo run -- --help

# Build and test (with compression)
all: clean build-upx help-test size-compare

# Build and test without compression
all-no-compress: clean build help-test size

# Alternative simple commands for better compatibility
build-simple:
    cargo build --release

clean-simple:
    cargo clean

test-simple:
    cargo test