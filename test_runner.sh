#!/bin/bash

# Comprehensive Test Runner for ASS Library
# This script runs all types of tests: unit tests, integration tests, benchmarks, and build tests

set -e  # Exit on any error

echo "🚀 Starting Comprehensive Test Suite for ASS Library"
echo "=================================================="

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

# Function to run a command and capture its output
run_test() {
    local test_name="$1"
    local command="$2"
    
    print_status "Running $test_name..."
    
    if eval "$command"; then
        print_success "$test_name passed"
        return 0
    else
        print_error "$test_name failed"
        return 1
    fi
}

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Function to update test counters
update_counters() {
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    if [ $1 -eq 0 ]; then
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
}

echo ""
echo "📋 Test Plan:"
echo "  1. Code formatting and linting"
echo "  2. Unit tests for all crates"
echo "  3. Integration tests"
echo "  4. Performance benchmarks"
echo "  5. WASM tests"
echo "  6. Build tests"
echo "  7. Documentation tests"
echo ""

# 1. Code formatting and linting
echo "🔍 Code Quality Checks"
echo "====================="

run_test "Code formatting check" "cargo fmt --all -- --check"
update_counters $?

run_test "Clippy linting" "cargo clippy --all-targets --all-features -- -D warnings"
update_counters $?

# 2. Unit tests for all crates
echo ""
echo "🧪 Unit Tests"
echo "============="

# Test each crate individually
for crate in ass-core ass-render ass-io ass-cli ass-wasm; do
    run_test "Unit tests for $crate" "cargo test --package $crate"
    update_counters $?
done

# Test all crates together
run_test "All unit tests" "cargo test --workspace --lib"
update_counters $?

# 3. Integration tests
echo ""
echo "🔗 Integration Tests"
echo "==================="

run_test "Integration tests" "cargo test --test integration_tests"
update_counters $?

# 4. Performance benchmarks
echo ""
echo "📊 Performance Benchmarks"
echo "========================="

# Check if criterion is available and run benchmarks
if cargo bench --help >/dev/null 2>&1; then
    run_test "Core parsing benchmarks" "cargo bench --package ass-core"
    update_counters $?
    
    # Only run render benchmarks if font file exists
    if [ -f "assets/NotoSans-Regular.ttf" ] || [ ! -z "$BENCH_FONT" ]; then
        run_test "Render benchmarks" "cargo bench --package ass-render"
        update_counters $?
    else
        print_warning "Skipping render benchmarks (no font file found)"
        print_warning "Set BENCH_FONT environment variable to run render benchmarks"
    fi
else
    print_warning "Criterion not available, skipping benchmarks"
fi

# 5. WASM tests
echo ""
echo "🌐 WASM Tests"
echo "============"

# Check if wasm-pack is available
if command -v wasm-pack >/dev/null 2>&1; then
    run_test "WASM build test" "wasm-pack build crates/ass-wasm --target web"
    update_counters $?
    
    run_test "WASM tests (node)" "wasm-pack test crates/ass-wasm --node"
    update_counters $?
else
    print_warning "wasm-pack not found, skipping WASM tests"
    print_warning "Install wasm-pack to run WASM tests: https://rustwasm.github.io/wasm-pack/"
fi

# 6. Build tests
echo ""
echo "🔨 Build Tests"
echo "=============="

# Test different build configurations
run_test "Debug build" "cargo build --workspace"
update_counters $?

run_test "Release build" "cargo build --workspace --release"
update_counters $?

# Test feature combinations
run_test "No default features build" "cargo build --workspace --no-default-features"
update_counters $?

run_test "All features build" "cargo build --workspace --all-features"
update_counters $?

# Test individual crate builds
for crate in ass-core ass-render ass-io ass-cli ass-wasm; do
    run_test "Build $crate" "cargo build --package $crate"
    update_counters $?
done

# 7. Documentation tests
echo ""
echo "📚 Documentation Tests"
echo "======================"

run_test "Documentation build" "cargo doc --workspace --no-deps"
update_counters $?

run_test "Documentation tests" "cargo test --workspace --doc"
update_counters $?

# 8. Additional checks
echo ""
echo "🔍 Additional Checks"
echo "==================="

# Check for security vulnerabilities
if command -v cargo-audit >/dev/null 2>&1; then
    run_test "Security audit" "cargo audit"
    update_counters $?
else
    print_warning "cargo-audit not found, skipping security audit"
    print_warning "Install with: cargo install cargo-audit"
fi

# Check for outdated dependencies
if command -v cargo-outdated >/dev/null 2>&1; then
    run_test "Dependency check" "cargo outdated --exit-code 1"
    update_counters $?
else
    print_warning "cargo-outdated not found, skipping dependency check"
    print_warning "Install with: cargo install cargo-outdated"
fi

# Check for unused dependencies
if command -v cargo-udeps >/dev/null 2>&1; then
    run_test "Unused dependencies check" "cargo +nightly udeps --all-targets"
    update_counters $?
else
    print_warning "cargo-udeps not found, skipping unused dependency check"
    print_warning "Install with: cargo install cargo-udeps"
fi

# Final summary
echo ""
echo "📊 Test Summary"
echo "==============="
echo "Total tests run: $TOTAL_TESTS"
echo "Passed: $PASSED_TESTS"
echo "Failed: $FAILED_TESTS"

if [ $FAILED_TESTS -eq 0 ]; then
    print_success "All tests passed! 🎉"
    echo ""
    echo "✅ The ASS library is ready for use!"
    echo ""
    echo "Next steps:"
    echo "  - Run 'cargo build --release' for optimized builds"
    echo "  - Check 'cargo doc --open' for documentation"
    echo "  - Use 'cargo run --bin ass-cli -- --help' for CLI usage"
    exit 0
else
    print_error "$FAILED_TESTS test(s) failed"
    echo ""
    echo "❌ Please fix the failing tests before proceeding"
    echo ""
    echo "Common fixes:"
    echo "  - Run 'cargo fmt' to fix formatting"
    echo "  - Run 'cargo clippy --fix' to fix linting issues"
    echo "  - Check individual test output for specific errors"
    exit 1
fi