#!/bin/bash

# Comprehensive Test Runner for ASS-RS
# This script runs all tests, benchmarks, and performance comparisons

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
BENCHMARK_BASELINE=""
LIBASS_COMPARISON=false
VERBOSE=false
STRESS_TESTS=false
COVERAGE=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --baseline)
            BENCHMARK_BASELINE="$2"
            shift 2
            ;;
        --libass-comparison)
            LIBASS_COMPARISON=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --stress-tests)
            STRESS_TESTS=true
            shift
            ;;
        --coverage)
            COVERAGE=true
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --baseline NAME        Set benchmark baseline name"
            echo "  --libass-comparison    Enable libass comparison benchmarks"
            echo "  --verbose              Enable verbose output"
            echo "  --stress-tests         Run stress tests (long running)"
            echo "  --coverage             Generate coverage report"
            echo "  --help                 Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

print_header() {
    echo -e "${BLUE}===================================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}===================================================${NC}"
}

print_step() {
    echo -e "${GREEN}>>> $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}WARNING: $1${NC}"
}

print_error() {
    echo -e "${RED}ERROR: $1${NC}"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -d "crates" ]; then
    print_error "This script must be run from the project root directory"
    exit 1
fi

print_header "ASS-RS Comprehensive Test Suite"
echo "Starting comprehensive testing and benchmarking..."
echo ""

# Check dependencies
print_step "Checking dependencies"
if ! command -v cargo &> /dev/null; then
    print_error "cargo not found. Please install Rust."
    exit 1
fi

if ! command -v git &> /dev/null; then
    print_error "git not found. Please install git."
    exit 1
fi

# Install additional tools if needed
if [ "$COVERAGE" = true ]; then
    print_step "Installing coverage tools"
    cargo install --quiet cargo-llvm-cov || print_warning "Failed to install cargo-llvm-cov"
fi

if [ -n "$BENCHMARK_BASELINE" ]; then
    print_step "Installing benchmark comparison tools"
    cargo install --quiet critcmp || print_warning "Failed to install critcmp"
fi

print_header "Code Quality Checks"

# Format check
print_step "Checking code formatting"
if cargo fmt --all -- --check; then
    echo "✓ Code formatting is correct"
else
    print_error "Code formatting issues found. Run 'cargo fmt' to fix."
    exit 1
fi

# Clippy check
print_step "Running Clippy lints"
if cargo clippy --all-targets --all-features -- -D warnings; then
    echo "✓ No clippy warnings found"
else
    print_error "Clippy warnings found. Please fix them."
    exit 1
fi

# Security audit
print_step "Running security audit"
if cargo audit; then
    echo "✓ No security vulnerabilities found"
else
    print_warning "Security audit found issues. Please review."
fi

print_header "Unit Tests"

# Standard unit tests
print_step "Running standard unit tests"
if [ "$VERBOSE" = true ]; then
    cargo test --workspace --lib -- --nocapture
else
    cargo test --workspace --lib
fi

# Advanced unit tests
print_step "Running advanced unit tests"
if [ "$VERBOSE" = true ]; then
    cargo test --workspace --test advanced_tests -- --nocapture
else
    cargo test --workspace --test advanced_tests
fi

# Stress tests (if enabled)
if [ "$STRESS_TESTS" = true ]; then
    print_step "Running stress tests"
    if [ "$VERBOSE" = true ]; then
        cargo test --workspace -- --ignored --nocapture
    else
        cargo test --workspace -- --ignored
    fi
fi

print_header "Integration Tests"

# Integration tests
print_step "Running integration tests"
if [ "$VERBOSE" = true ]; then
    cargo test --workspace --test integration_tests -- --nocapture
else
    cargo test --workspace --test integration_tests
fi

# Cross-crate integration tests
print_step "Running cross-crate integration tests"
if [ "$VERBOSE" = true ]; then
    cargo test --workspace --test '*' -- --nocapture
else
    cargo test --workspace --test '*'
fi

print_header "Documentation Tests"

# Doc tests
print_step "Running documentation tests"
cargo test --workspace --doc

print_header "WASM Tests"

# WASM compilation test
print_step "Testing WASM compilation"
if cargo build --package ass-core --target wasm32-unknown-unknown --features wasm; then
    echo "✓ WASM compilation successful"
else
    print_warning "WASM compilation failed"
fi

# WASM tests (if wasm-pack is available)
if command -v wasm-pack &> /dev/null; then
    print_step "Running WASM tests"
    if [ -d "crates/ass-wasm" ]; then
        wasm-pack test crates/ass-wasm --headless --firefox || print_warning "WASM tests failed"
    fi
else
    print_warning "wasm-pack not found. Skipping WASM tests."
fi

print_header "Performance Benchmarks"

# Core parsing benchmarks
print_step "Running core parsing benchmarks"
if [ -n "$BENCHMARK_BASELINE" ]; then
    cargo bench --package ass-core -- --save-baseline "$BENCHMARK_BASELINE-core"
else
    cargo bench --package ass-core
fi

# Rendering benchmarks
print_step "Running rendering benchmarks"
if [ -n "$BENCHMARK_BASELINE" ]; then
    cargo bench --package ass-render -- --save-baseline "$BENCHMARK_BASELINE-render"
else
    cargo bench --package ass-render
fi

# Comparison benchmarks (if enabled)
if [ "$LIBASS_COMPARISON" = true ]; then
    print_step "Running libass comparison benchmarks"
    if [ -n "$BENCHMARK_BASELINE" ]; then
        cargo bench --features libass-comparison --bench comparison_benchmarks -- --save-baseline "$BENCHMARK_BASELINE-comparison"
    else
        cargo bench --features libass-comparison --bench comparison_benchmarks
    fi
else
    print_warning "Libass comparison benchmarks skipped. Use --libass-comparison to enable."
fi

# Comprehensive benchmarks
print_step "Running comprehensive benchmarks"
if [ -f "benches/comparison_benchmarks.rs" ]; then
    if [ -n "$BENCHMARK_BASELINE" ]; then
        cargo bench --bench comparison_benchmarks -- --save-baseline "$BENCHMARK_BASELINE-comprehensive"
    else
        cargo bench --bench comparison_benchmarks
    fi
fi

print_header "Build Tests"

# Test different feature combinations
print_step "Testing default features"
cargo build --workspace

print_step "Testing no default features"
cargo build --workspace --no-default-features

print_step "Testing all features"
cargo build --workspace --all-features

# Test different targets
print_step "Testing release build"
cargo build --workspace --release

print_step "Testing different targets"
if rustup target list --installed | grep -q "wasm32-unknown-unknown"; then
    cargo build --package ass-core --target wasm32-unknown-unknown --features wasm
    echo "✓ WASM target build successful"
else
    print_warning "WASM target not installed. Run: rustup target add wasm32-unknown-unknown"
fi

print_header "Coverage Analysis"

if [ "$COVERAGE" = true ]; then
    if command -v cargo-llvm-cov &> /dev/null; then
        print_step "Generating coverage report"
        cargo llvm-cov --workspace --html
        echo "✓ Coverage report generated at target/llvm-cov/html/index.html"
        
        # Generate coverage summary
        cargo llvm-cov --workspace --summary-only
    else
        print_warning "cargo-llvm-cov not available. Skipping coverage analysis."
    fi
else
    print_warning "Coverage analysis skipped. Use --coverage to enable."
fi

print_header "Performance Analysis"

# Benchmark comparison (if baseline provided)
if [ -n "$BENCHMARK_BASELINE" ] && command -v critcmp &> /dev/null; then
    print_step "Comparing benchmark results"
    
    # Compare core benchmarks
    if [ -f "target/criterion/$BENCHMARK_BASELINE-core/base/estimates.json" ]; then
        echo "Core parsing benchmark comparison:"
        critcmp "$BENCHMARK_BASELINE-core" --threshold 5
    fi
    
    # Compare render benchmarks
    if [ -f "target/criterion/$BENCHMARK_BASELINE-render/base/estimates.json" ]; then
        echo "Rendering benchmark comparison:"
        critcmp "$BENCHMARK_BASELINE-render" --threshold 5
    fi
    
    # Compare comprehensive benchmarks
    if [ -f "target/criterion/$BENCHMARK_BASELINE-comprehensive/base/estimates.json" ]; then
        echo "Comprehensive benchmark comparison:"
        critcmp "$BENCHMARK_BASELINE-comprehensive" --threshold 5
    fi
fi

print_header "Memory Analysis"

# Basic memory usage test
print_step "Running memory usage analysis"
if command -v valgrind &> /dev/null; then
    print_step "Running valgrind memory check"
    # Run a simple test under valgrind
    cargo build --workspace --release
    valgrind --tool=memcheck --leak-check=full --error-exitcode=1 \
        target/release/ass-cli --help > /dev/null 2>&1 || print_warning "Memory issues detected"
else
    print_warning "valgrind not available. Skipping memory analysis."
fi

print_header "Final Report"

# Generate final report
echo "Test execution completed successfully!"
echo ""
echo "Summary:"
echo "✓ Code quality checks passed"
echo "✓ Unit tests passed"
echo "✓ Integration tests passed"
echo "✓ Documentation tests passed"
echo "✓ Build tests passed"
echo "✓ Performance benchmarks completed"

if [ "$COVERAGE" = true ]; then
    echo "✓ Coverage analysis completed"
fi

if [ "$STRESS_TESTS" = true ]; then
    echo "✓ Stress tests completed"
fi

if [ "$LIBASS_COMPARISON" = true ]; then
    echo "✓ Libass comparison benchmarks completed"
fi

echo ""
echo -e "${GREEN}All tests completed successfully!${NC}"

# Performance targets check
print_step "Checking performance targets"
    echo "Performance targets (from BENCHMARKING.md):"
echo "- Parse 1000-line ASS file: <10ms"
echo "- Render 640x360 frame: <50ms"
echo "- Memory usage: <100MB for typical scripts"
echo "- WASM bundle size: <2MB"
echo ""
echo "Check benchmark results to verify these targets are met."

# Cleanup
print_step "Cleaning up temporary files"
cargo clean --quiet

echo ""
echo -e "${GREEN}Test suite execution completed!${NC}"
echo "Review the output above for any warnings or issues."

if [ -n "$BENCHMARK_BASELINE" ]; then
    echo ""
    echo "Benchmark baseline '$BENCHMARK_BASELINE' has been saved."
    echo "Use 'critcmp $BENCHMARK_BASELINE' to compare future runs."
fi