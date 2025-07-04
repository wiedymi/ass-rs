#!/bin/bash

# CI/CD Docker Test Runner for ASS Library
# Optimized for CI/CD environments with better error handling and reporting

set -euo pipefail

# Environment variables with defaults
CI=${CI:-false}
GITHUB_ACTIONS=${GITHUB_ACTIONS:-false}
BUILD_NUMBER=${BUILD_NUMBER:-"local"}
RUST_LOG=${RUST_LOG:-"info"}

# Colors for output (disabled in CI)
if [[ "$CI" == "true" ]]; then
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    NC=''
else
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    BLUE='\033[0;34m'
    NC='\033[0m'
fi

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $1"
}

# Function to handle errors
error_handler() {
    local line_number=$1
    log_error "Script failed at line $line_number"
    
    if [[ "$CI" == "true" ]]; then
        # In CI, also log Docker system info for debugging
        log_info "Docker system information:"
        docker system df || true
        docker system events --since 1m --until now || true
    fi
    
    exit 1
}

# Set error handler
trap 'error_handler $LINENO' ERR

# Function to check system requirements
check_requirements() {
    log_info "Checking system requirements..."
    
    # Check Docker
    if ! command -v docker >/dev/null 2>&1; then
        log_error "Docker is not installed"
        exit 1
    fi
    
    if ! docker info >/dev/null 2>&1; then
        log_error "Docker daemon is not running"
        exit 1
    fi
    
    # Check Docker Compose
    if ! command -v docker-compose >/dev/null 2>&1; then
        log_error "docker-compose is not installed"
        exit 1
    fi
    
    # Log versions for debugging
    log_info "Docker version: $(docker --version)"
    log_info "Docker Compose version: $(docker-compose --version)"
    
    # Check available disk space (warn if less than 5GB)
    if command -v df >/dev/null 2>&1; then
        available_space=$(df . | tail -1 | awk '{print $4}')
        if [[ $available_space -lt 5242880 ]]; then  # 5GB in KB
            log_warning "Low disk space detected: $(( available_space / 1024 / 1024 ))GB available"
        fi
    fi
    
    log_success "System requirements check passed"
}

# Function to clean up old Docker resources
cleanup_docker() {
    log_info "Cleaning up Docker resources..."
    
    # Remove old containers
    docker container prune -f >/dev/null 2>&1 || true
    
    # Remove unused images (but keep recent ones)
    docker image prune -f >/dev/null 2>&1 || true
    
    # In CI, be more aggressive with cleanup
    if [[ "$CI" == "true" ]]; then
        # Remove dangling volumes
        docker volume prune -f >/dev/null 2>&1 || true
        
        # Remove build cache older than 7 days
        docker builder prune --filter until=168h -f >/dev/null 2>&1 || true
    fi
    
    log_success "Docker cleanup completed"
}

# Function to run tests with retry logic
run_tests_with_retry() {
    local test_type="$1"
    local max_retries=3
    local retry_count=0
    
    while [[ $retry_count -lt $max_retries ]]; do
        log_info "Running $test_type tests (attempt $((retry_count + 1))/$max_retries)..."
        
        if ./scripts/docker-test.sh "$test_type" --build; then
            log_success "$test_type tests passed"
            return 0
        else
            retry_count=$((retry_count + 1))
            if [[ $retry_count -lt $max_retries ]]; then
                log_warning "$test_type tests failed, retrying in 30 seconds..."
                sleep 30
                cleanup_docker
            else
                log_error "$test_type tests failed after $max_retries attempts"
                return 1
            fi
        fi
    done
}

# Function to collect artifacts
collect_artifacts() {
    log_info "Collecting test artifacts..."
    
    # Create artifacts directory
    mkdir -p artifacts
    
    # Collect coverage reports
    if [[ -d "target/llvm-cov" ]]; then
        cp -r target/llvm-cov artifacts/ || true
        log_info "Coverage reports collected"
    fi
    
    # Collect benchmark results
    if [[ -d "target/criterion" ]]; then
        cp -r target/criterion artifacts/ || true
        log_info "Benchmark results collected"
    fi
    
    # Collect test outputs
    if [[ -d "test_output" ]]; then
        cp -r test_output artifacts/ || true
        log_info "Test outputs collected"
    fi
    
    # Generate summary report
    cat > artifacts/test-summary.txt << EOF
Test Run Summary
================
Build Number: $BUILD_NUMBER
Date: $(date)
Git Commit: $(git rev-parse HEAD 2>/dev/null || echo "unknown")
Git Branch: $(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")
Docker Version: $(docker --version)
Docker Compose Version: $(docker-compose --version)

Test Results:
- All tests completed
- Artifacts collected in artifacts/ directory

System Information:
$(uname -a)
EOF
    
    log_success "Artifacts collected successfully"
}

# Main execution
main() {
    log_info "Starting CI Docker test execution (Build: $BUILD_NUMBER)"
    
    # Check requirements
    check_requirements
    
    # Initial cleanup
    cleanup_docker
    
    # Set compose project name for isolation
    export COMPOSE_PROJECT_NAME="ass-library-ci-$BUILD_NUMBER"
    
    # Track test results
    local failed_tests=()
    
    # Run test suites
    test_suites=("quality" "unit" "integration" "wasm" "audit")
    
    for test_suite in "${test_suites[@]}"; do
        if ! run_tests_with_retry "$test_suite"; then
            failed_tests+=("$test_suite")
        fi
    done
    
    # Run benchmarks (allowed to fail in CI)
    log_info "Running benchmarks (non-blocking)..."
    if ./scripts/docker-test.sh benchmarks --build; then
        log_success "Benchmarks completed"
    else
        log_warning "Benchmarks failed (non-critical in CI)"
    fi
    
    # Generate coverage report
    log_info "Generating coverage report..."
    if ./scripts/docker-test.sh coverage --build; then
        log_success "Coverage report generated"
    else
        log_warning "Coverage report generation failed"
    fi
    
    # Collect artifacts
    collect_artifacts
    
    # Final cleanup
    cleanup_docker
    
    # Report results
    if [[ ${#failed_tests[@]} -eq 0 ]]; then
        log_success "All tests passed! 🎉"
        
        # In GitHub Actions, set output
        if [[ "$GITHUB_ACTIONS" == "true" ]]; then
            echo "test_result=success" >> $GITHUB_OUTPUT
        fi
        
        exit 0
    else
        log_error "The following test suites failed: ${failed_tests[*]}"
        
        # In GitHub Actions, set output
        if [[ "$GITHUB_ACTIONS" == "true" ]]; then
            echo "test_result=failure" >> $GITHUB_OUTPUT
            echo "failed_tests=${failed_tests[*]}" >> $GITHUB_OUTPUT
        fi
        
        exit 1
    fi
}

# Handle script arguments
case "${1:-}" in
    --help)
        echo "Usage: $0 [--help]"
        echo ""
        echo "CI/CD Docker Test Runner for ASS Library"
        echo ""
        echo "Environment Variables:"
        echo "  CI                 Set to 'true' for CI mode"
        echo "  GITHUB_ACTIONS     Set to 'true' for GitHub Actions"
        echo "  BUILD_NUMBER       Build identifier (default: 'local')"
        echo "  RUST_LOG           Rust logging level (default: 'info')"
        exit 0
        ;;
    "")
        main
        ;;
    *)
        log_error "Unknown argument: $1"
        exit 1
        ;;
esac