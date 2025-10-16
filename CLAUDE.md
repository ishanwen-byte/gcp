# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

GCP (GitHub Copy) is a minimal command-line tool for downloading files and folders from public GitHub repositories. The project is designed as a lightweight, no_std-compatible binary with extreme optimization for small size. It uses custom JSON parsing instead of heavy dependencies and features a custom allocator (wee_alloc) for memory efficiency.

## Common Development Commands

### Building and Running
```bash
# Build the project
cargo build

# Build optimized release binary with extreme size optimization
cargo build --release

# Run the application
cargo run -- <github_url> [destination]

# Run tests
cargo test

# Check code without building (faster validation)
cargo check
```

### Build System Commands
```bash
# Use Just (recommended)
just build          # Build optimized release binary
just upx            # Build and compress with UPX
just size           # Show binary sizes
just clean          # Clean build artifacts
just test           # Run tests
just all            # Full build pipeline (clean → build → compress → test)

# Use Make (alternative)
make build          # Build optimized release binary
make upx            # Build and compress with UPX
make size           # Show binary sizes
make clean          # Clean build artifacts
make test           # Run tests
make all            # Full build pipeline

# Use PowerShell (Windows)
./build.ps1         # Build optimized release binary
./build.ps1 -Compress # Build and compress with UPX
./build.ps1 -All    # Full build pipeline
```

### Development Tools
```bash
# Format code according to Rust standards
cargo fmt

# Run Clippy linter for code quality checks
cargo clippy

# Generate documentation
cargo doc --open

# Update dependencies
cargo update
```

## Architecture Overview

### Core Components

**src/main.rs**: CLI entry point with minimal argument parsing
- Handles GitHub URL validation
- Manages command-line arguments (URL, destination)
- Provides help text and error handling

**src/lib.rs**: Public API surface
- Exports main `download_from_github()` function
- Integrates all modules

**src/github.rs**: GitHub URL parsing and API construction
- Parses GitHub.com and raw.githubusercontent.com URLs
- Supports blob (file) and tree (folder) URL types
- Generates GitHub API endpoints for content retrieval
- Extracts filenames for automatic destination naming

**src/downloader.rs**: Minimal HTTP client and file operations
- Uses attohttpc for lightweight HTTP requests
- Custom JSON parsing without serde dependency
- Handles base64 content decoding from GitHub API
- Recursive folder download with directory creation
- Falls back to raw URLs when API content unavailable

**src/error.rs**: Minimal error handling without thiserror
- Custom error types for different failure modes
- Implements standard error traits
- Provides conversion from common error types (io, base64, attohttpc)

### Key Design Principles

1. **Minimal Dependencies**: Uses only essential crates (attohttpc, base64, wee_alloc)
2. **Custom JSON Parsing**: Avoids serde dependency through manual string parsing
3. **No_std Compatibility**: Designed to work in embedded environments
4. **Extreme Size Optimization**: Configured for minimal binary size
5. **Memory Efficiency**: Uses wee_alloc allocator for small footprint

### Build Configuration

The project includes extensive release optimizations in Cargo.toml:
- Link-time optimization (LTO) enabled
- Panic mode set to abort (no unwinding)
- Single code generation unit
- Symbol stripping and size optimization
- Overflow checks and debug assertions disabled
- Incremental compilation disabled

## Project Structure

- `src/main.rs` - CLI entry point and argument handling
- `src/lib.rs` - Library interface and module organization
- `src/github.rs` - GitHub URL parsing and API integration
- `src/downloader.rs` - File downloading and JSON parsing
- `src/error.rs` - Minimal error types and handling
- `Cargo.toml` - Project configuration with extreme optimizations
- `justfile` - Just build system commands (recommended)
- `Makefile` - Alternative build system using make
- `build.ps1` - PowerShell build script for Windows
- `clippy.toml` - Clippy linting configuration
- `.rustfmt.toml` - Rust formatting configuration

## Environment Setup

- Rust toolchain: nightly version 1.92.0+ required for edition 2024
- Uses Rust edition 2024
- Minimal external dependencies: attohttpc, base64, wee_alloc
- Optional: UPX for further binary compression

## Usage Examples

```bash
# Download single file
gcp "https://github.com/owner/repo/blob/main/file.txt"

# Download file with custom destination
gcp "https://github.com/owner/repo/blob/main/file.txt" my_file.txt

# Download folder
gcp "https://github.com/owner/repo/tree/main/folder" ./local_folder/

# Show help
gcp --help
```

## Development Notes

- The codebase avoids heavy dependencies and uses custom implementations for JSON parsing
- Error handling is minimal but complete, supporting common failure modes
- Binary size optimization takes priority over convenience features
- Only supports public repositories (no authentication in minimal version)
- Uses wee_alloc allocator to reduce memory footprint
