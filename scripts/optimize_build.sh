#!/bin/bash

# ASS-RS Performance Optimization Build Script
# Builds optimized versions for different targets and use cases

set -e

echo "🚀 ASS-RS Performance Optimization Build Script"
echo "==============================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if required tools are installed
check_dependencies() {
    print_status "Checking dependencies..."
    
    tools=("cargo" "wasm-pack" "twiggy" "wee_alloc")
    missing_tools=()
    
    for tool in "${tools[@]}"; do
        if ! command -v "$tool" &> /dev/null; then
            missing_tools+=("$tool")
        fi
    done
    
    if [ ${#missing_tools[@]} -gt 0 ]; then
        print_warning "Missing tools: ${missing_tools[*]}"
        print_status "Installing missing tools..."
        
        for tool in "${missing_tools[@]}"; do
            case $tool in
                "wasm-pack")
                    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
                    ;;
                "twiggy")
                    cargo install twiggy
                    ;;
                *)
                    print_warning "Please install $tool manually"
                    ;;
            esac
        done
    fi
}

# Set up RUSTFLAGS for maximum optimization
setup_rust_flags() {
    export RUSTFLAGS="-C target-cpu=native -C opt-level=3 -C lto=fat -C embed-bitcode=yes -C codegen-units=1"
    export CARGO_PROFILE_RELEASE_DEBUG=false
    export CARGO_PROFILE_RELEASE_STRIP=true
    export CARGO_PROFILE_RELEASE_PANIC=abort
    export CARGO_PROFILE_RELEASE_OVERFLOW_CHECKS=false
}

# Build native optimized version
build_native() {
    print_status "Building native optimized version..."
    
    setup_rust_flags
    
    # Build with maximum optimization
    cargo build --release --workspace
    
    # Build CLI with additional optimizations
    cargo build --release -p ass-cli
    
    print_status "Native build completed"
}

# Build WASM optimized version
build_wasm() {
    print_status "Building WASM optimized version..."
    
    # Set WASM-specific optimization flags
    export RUSTFLAGS="-C opt-level=s -C lto=fat -C panic=abort -C embed-bitcode=yes"
    
    # Build WASM package
    wasm-pack build crates/ass-wasm --target web --release --scope ass-rs
    
    # Optimize WASM binary size
    if command -v wasm-opt &> /dev/null; then
        print_status "Optimizing WASM binary with wasm-opt..."
        wasm-opt -Oz --enable-bulk-memory --enable-sign-ext \
                 crates/ass-wasm/pkg/ass_wasm_bg.wasm \
                 -o crates/ass-wasm/pkg/ass_wasm_bg.wasm
    fi
    
    # Analyze WASM bundle size
    if command -v twiggy &> /dev/null; then
        print_status "Analyzing WASM bundle size..."
        twiggy top crates/ass-wasm/pkg/ass_wasm_bg.wasm
    fi
    
    print_status "WASM build completed"
}

# Build with PGO (Profile-Guided Optimization)
build_pgo() {
    print_status "Building with Profile-Guided Optimization..."
    
    # Stage 1: Build instrumented binary
    export RUSTFLAGS="-C profile-generate=/tmp/pgo-data"
    cargo build --release -p ass-cli
    
    # Stage 2: Run training workload
    print_status "Running PGO training workload..."
    mkdir -p /tmp/pgo-training
    
    # Run benchmarks to generate profile data
    cargo bench --no-run
    timeout 60s cargo bench || true
    
    # Stage 3: Build optimized binary with profile data
    export RUSTFLAGS="-C profile-use=/tmp/pgo-data -C target-cpu=native -C opt-level=3 -C lto=fat"
    cargo build --release -p ass-cli
    
    print_status "PGO build completed"
    
    # Cleanup
    rm -rf /tmp/pgo-data
}

# Build benchmarks with optimizations
build_benchmarks() {
    print_status "Building optimized benchmarks..."
    
    setup_rust_flags
    export CARGO_PROFILE_BENCH_DEBUG=true  # Keep debug info for profiling
    
    cargo build --release --profile bench -p ass-benchmarks
    
    print_status "Benchmark build completed"
}

# Performance testing and validation
performance_test() {
    print_status "Running performance tests..."
    
    # Run quick benchmark to validate performance
    cargo bench --package ass-core -- --quick
    cargo bench --package ass-render -- --quick
    
    print_status "Performance tests completed"
}

# Generate size reports
size_analysis() {
    print_status "Generating size analysis reports..."
    
    # Native binary sizes
    if [ -f "target/release/ass-cli" ]; then
        native_size=$(ls -lh target/release/ass-cli | awk '{print $5}')
        print_status "Native CLI binary size: $native_size"
    fi
    
    # WASM bundle size
    if [ -f "crates/ass-wasm/pkg/ass_wasm_bg.wasm" ]; then
        wasm_size=$(ls -lh crates/ass-wasm/pkg/ass_wasm_bg.wasm | awk '{print $5}')
        print_status "WASM bundle size: $wasm_size"
    fi
    
    # Generate detailed size breakdown
    if command -v cargo-bloat &> /dev/null; then
        print_status "Generating cargo-bloat analysis..."
        cargo bloat --release --crates -p ass-core
        cargo bloat --release --crates -p ass-wasm
    else
        print_warning "cargo-bloat not found. Install with: cargo install cargo-bloat"
    fi
}

# Clean up build artifacts
cleanup() {
    print_status "Cleaning up build artifacts..."
    cargo clean
    rm -rf crates/ass-wasm/pkg
    rm -rf /tmp/pgo-training
}

# Main execution
main() {
    local targets=("$@")
    
    if [ ${#targets[@]} -eq 0 ]; then
        targets=("native" "wasm")
    fi
    
    check_dependencies
    
    for target in "${targets[@]}"; do
        case $target in
            "native")
                build_native
                ;;
            "wasm")
                build_wasm
                ;;
            "pgo")
                build_pgo
                ;;
            "benchmarks")
                build_benchmarks
                ;;
            "test")
                performance_test
                ;;
            "analyze")
                size_analysis
                ;;
            "clean")
                cleanup
                ;;
            "all")
                build_native
                build_wasm
                build_benchmarks
                performance_test
                size_analysis
                ;;
            *)
                print_error "Unknown target: $target"
                print_status "Available targets: native, wasm, pgo, benchmarks, test, analyze, clean, all"
                exit 1
                ;;
        esac
    done
    
    print_status "🎉 Build optimization complete!"
}

# Script usage
usage() {
    cat << EOF
Usage: $0 [targets...]

Available targets:
  native      - Build optimized native binaries
  wasm        - Build optimized WASM bundle
  pgo         - Build with Profile-Guided Optimization
  benchmarks  - Build optimized benchmarks
  test        - Run performance tests
  analyze     - Generate size analysis reports
  clean       - Clean build artifacts
  all         - Build all targets and run analysis

Examples:
  $0                    # Build native and wasm (default)
  $0 native wasm        # Build native and wasm
  $0 pgo                # Build with PGO
  $0 all                # Build everything and analyze
  $0 clean              # Clean up

EOF
}

# Handle command line arguments
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    if [[ "$1" == "--help" ]] || [[ "$1" == "-h" ]]; then
        usage
        exit 0
    fi
    
    main "$@"
fi