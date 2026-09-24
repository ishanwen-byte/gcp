#!/usr/bin/env just
set shell := ["powershell.exe", "-NoProfile", "-ExecutionPolicy", "Bypass", "-Command"]

# Default: plain optimized build (no compression - keeps binary debuggable)
default: build

# Build optimized release binary
build:
    cargo build --release
    echo "Release build completed"

# Build and compress with UPX
build-upx:
    cargo build --release
    @upx --best --lzma target/release/gcp.exe

# UPX compression only (existing binary)
upx:
    @upx --best --lzma target/release/gcp.exe

# Show binary size
size:
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts/size.ps1

# Clean build artifacts
clean:
    cargo clean
    echo "Cleaned"

# Run tests
test:
    cargo test
    echo "Tests completed"

# Lint (fmt check + clippy)
lint:
    cargo fmt --check
    cargo clippy --all-targets

# Full verification pipeline
verify: clean build test lint size

# Smoke test against GitHub
smoke:
    cargo build --release
    ./target/release/gcp.exe https://github.com/octocat/Hello-World/blob/master/README just_smoke.txt
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts/smoke-check.ps1
