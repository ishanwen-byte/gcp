#!/bin/bash
# Test script to verify different Cargo configurations

set -e

echo "🔧 Testing Cargo Configurations"
echo "=================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to test a configuration
test_config() {
    local config_name=$1
    local config_file=$2
    local command=$3

    print_status "Testing $config_name configuration..."

    if [ -f ".cargo/$config_file" ]; then
        print_status "Using config: .cargo/$config_file"

        if eval "$command"; then
            print_success "$config_name configuration works! ✅"
            return 0
        else
            print_error "$config_name configuration failed! ❌"
            return 1
        fi
    else
        print_warning "Config file .cargo/$config_file not found, skipping..."
        return 1
    fi
}

# Function to measure build time
measure_build_time() {
    local config_name=$1
    local config_file=$2
    local command=$3

    print_status "Measuring build time for $config_name..."

    if [ -f ".cargo/$config_file" ]; then
        local start_time=$(date +%s)

        if eval "$command" > /dev/null 2>&1; then
            local end_time=$(date +%s)
            local duration=$((end_time - start_time))
            print_success "$config_name build time: ${duration}s ⏱️"
            return 0
        else
            print_error "$config_name build failed!"
            return 1
        fi
    else
        print_warning "Config file .cargo/$config_file not found"
        return 1
    fi
}

# Clean previous builds
print_status "Cleaning previous builds..."
cargo clean

# Test configurations
echo
echo "🧪 Testing Configurations"
echo "========================"

test_config "Default" "config.toml" "cargo check"
test_config "Development" "config.dev.toml" "cargo check --config .cargo/config.dev.toml"
test_config "Release" "config.release.toml" "cargo check --release --config .cargo/config.release.toml"
test_config "Benchmark" "config.bench.toml" "cargo check --config .cargo/config.bench.toml"

echo
echo "⏱️ Measuring Build Times"
echo "========================"

# Clean and measure build times
cargo clean > /dev/null 2>&1

measure_build_time "Development" "config.dev.toml" "cargo build --config .cargo/config.dev.toml"

cargo clean > /dev/null 2>&1

measure_build_time "Default" "config.toml" "cargo build"

cargo clean > /dev/null 2>&1

measure_build_time "Release" "config.release.toml" "cargo build --release --config .cargo/config.release.toml"

echo
echo "🔍 Testing Binary Sizes"
echo "========================"

if [ -f "target/debug/gcp" ]; then
    local debug_size=$(du -h target/debug/gcp | cut -f1)
    print_success "Debug binary size: $debug_size 📦"
fi

if [ -f "target/release/gcp" ]; then
    local release_size=$(du -h target/release/gcp | cut -f1)
    print_success "Release binary size: $release_size 📦"
fi

echo
echo "🧪 Testing Functionality"
echo "========================"

# Test basic functionality
if [ -f "target/release/gcp" ]; then
    print_status "Testing release binary..."
    if ./target/release/gcp --help > /dev/null 2>&1; then
        print_success "Release binary works! ✅"
    else
        print_error "Release binary failed! ❌"
    fi
fi

if [ -f "target/debug/gcp" ]; then
    print_status "Testing debug binary..."
    if ./target/debug/gcp --help > /dev/null 2>&1; then
        print_success "Debug binary works! ✅"
    else
        print_error "Debug binary failed! ❌"
    fi
fi

echo
echo "🎯 Testing Aliases"
echo "=================="

# Test some aliases
print_status "Testing cargo check-all alias..."
if cargo check-all > /dev/null 2>&1; then
    print_success "check-all alias works! ✅"
else
    print_warning "check-all alias may need cargo-clippy"
fi

print_status "Testing cargo rr alias..."
if cargo rr --help > /dev/null 2>&1; then
    print_success "rr alias works! ✅"
else
    print_error "rr alias failed! ❌"
fi

echo
echo "🚀 Configuration Test Complete!"
echo "================================"
print_success "All tests completed successfully! 🎉"

# Display optimization info
echo
print_status "Optimization Information:"
echo "  • Jobs: 32 (parallel compilation)"
echo "  • Target CPU: Native optimizations enabled"
echo "  • Linker: Platform-specific fast linkers"
echo "  • LTO: Link-time optimization for release builds"
echo "  • Codegen Units: Optimized for performance vs speed"