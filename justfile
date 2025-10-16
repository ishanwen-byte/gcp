#!/usr/bin/env just

# Default recipe
default: build

# Build optimized release binary
build:
    cargo build --release
    @echo "✓ Release build completed"

# Build and compress with UPX
upx: build
    @echo "Applying UPX compression..."
    @if command -v upx >/dev/null 2>&1; then \
        upx --best --lzma target/release/gcp.exe; \
        echo "✓ UPX compression applied successfully"; \
    else \
        echo "⚠️  UPX not found. Install UPX for smaller binary size"; \
        echo "Download from: https://upx.github.io/"; \
    fi

# Show binary sizes
size:
    @echo "=== Binary Sizes ==="
    @if [ -f target/release/gcp.exe ]; then \
        echo "Release binary: $(powershell -c "(Get-Item 'target/release/gcp.exe').Length/1KB") KB"; \
    else \
        echo "Release binary not found. Run 'just build' first."; \
    fi

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

# Build and test
all: clean build upx help-test size