# Makefile for GCP - GitHub Copier

.PHONY: build upx clean test help size all

# Default target
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
		echo "Release binary: $$(( $(shell stat -c%s target/release/gcp.exe) / 1024 )) KB"; \
	else \
		echo "Release binary not found. Run 'make build' first."; \
	fi

# Clean build artifacts
clean:
	cargo clean
	@echo "✓ Build artifacts cleaned"

# Run tests
test:
	cargo test
	@echo "✓ Tests completed"

# Test help command
help-test: build
	./target/release/gcp.exe --help

# Full build pipeline
all: clean build upx help-test size

# Show help
help:
	@echo "Makefile for GCP - GitHub Copier"
	@echo ""
	@echo "Targets:"
	@echo "  build      - Build optimized release binary"
	@echo "  upx        - Build and compress with UPX"
	@echo "  clean      - Clean build artifacts"
	@echo "  test       - Run tests"
	@echo "  help-test  - Test help command"
	@echo "  size       - Show binary sizes"
	@echo "  all        - Full build pipeline"
	@echo "  help       - Show this help"