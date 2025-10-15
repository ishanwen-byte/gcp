# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust project named "gcp" (Google Cloud Platform), currently in initial setup stage with a basic "Hello, world!" application. The project uses Rust edition 2024 and is set up with a standard Cargo project structure.

## Common Development Commands

### Building and Running
```bash
# Build the project
cargo build

# Build with optimizations for release
cargo build --release

# Run the application
cargo run

# Run tests
cargo test

# Check code without building (faster validation)
cargo check
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

## Project Structure

- `src/main.rs` - Entry point of the application (currently contains basic hello world)
- `Cargo.toml` - Project configuration and dependencies
- `.gitignore` - Git ignore configuration (excludes `/target` directory)

## Environment Setup

- Rust toolchain: nightly version 1.92.0 (as of 2025-10-05)
- Uses Rust edition 2024
- No external dependencies currently configured

## Development Notes

This is a minimal Rust project in early development stage. The codebase follows standard Rust project conventions and uses Cargo for dependency management and building.
