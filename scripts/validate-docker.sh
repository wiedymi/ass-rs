#!/bin/bash

# Docker Configuration Validator
# Validates Docker setup without requiring Docker to be running

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Logging functions
log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Validation counters
TOTAL_CHECKS=0
PASSED_CHECKS=0
FAILED_CHECKS=0

run_check() {
    local check_name="$1"
    local check_command="$2"
    
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    log_info "Checking: $check_name"
    
    if eval "$check_command"; then
        PASSED_CHECKS=$((PASSED_CHECKS + 1))
        log_success "$check_name ✓"
        return 0
    else
        FAILED_CHECKS=$((FAILED_CHECKS + 1))
        log_error "$check_name ✗"
        return 1
    fi
}

# Check if files exist
check_file_exists() {
    [ -f "$1" ]
}

check_file_executable() {
    [ -x "$1" ]
}

# Check if file contains required content
check_file_contains() {
    local file="$1"
    local pattern="$2"
    grep -q "$pattern" "$file" 2>/dev/null
}

# Validate Dockerfile syntax
validate_dockerfile() {
    local dockerfile="$1"
    
    # Check basic syntax elements
    check_file_contains "$dockerfile" "FROM rust:" && \
    check_file_contains "$dockerfile" "WORKDIR /workspace" && \
    check_file_contains "$dockerfile" "COPY" && \
    check_file_contains "$dockerfile" "RUN"
}

# Validate docker-compose.yml syntax
validate_compose() {
    local compose_file="$1"
    
    # Check basic YAML structure and required services
    check_file_contains "$compose_file" "version:" && \
    check_file_contains "$compose_file" "services:" && \
    check_file_contains "$compose_file" "tests:" && \
    check_file_contains "$compose_file" "benchmarks:" && \
    check_file_contains "$compose_file" "volumes:"
}

# Validate script syntax
validate_script() {
    local script="$1"
    
    # Check if script has valid shebang and basic structure
    head -1 "$script" | grep -q "^#!" && \
    check_file_contains "$script" "set -e"
}

# Check workspace structure
check_workspace_structure() {
    [ -f "Cargo.toml" ] && \
    [ -f "test_runner.sh" ] && \
    [ -d "crates" ] && \
    [ -d "tests" ]
}

echo "🐳 Docker Configuration Validator"
echo "=================================="

# File existence checks
run_check "Dockerfile exists" "check_file_exists Dockerfile"
run_check "docker-compose.yml exists" "check_file_exists docker-compose.yml"
run_check ".dockerignore exists" "check_file_exists .dockerignore"
run_check "docker-test.sh exists" "check_file_exists scripts/docker-test.sh"
run_check "ci-docker.sh exists" "check_file_exists scripts/ci-docker.sh"
run_check "validate-docker.sh exists" "check_file_exists scripts/validate-docker.sh"

# Permission checks
run_check "docker-test.sh is executable" "check_file_executable scripts/docker-test.sh"
run_check "ci-docker.sh is executable" "check_file_executable scripts/ci-docker.sh"
run_check "test_runner.sh is executable" "check_file_executable test_runner.sh"

# Content validation
run_check "Dockerfile syntax" "validate_dockerfile Dockerfile"
run_check "docker-compose.yml syntax" "validate_compose docker-compose.yml"
run_check "docker-test.sh syntax" "validate_script scripts/docker-test.sh"
run_check "ci-docker.sh syntax" "validate_script scripts/ci-docker.sh"

# Workspace structure
run_check "Workspace structure" "check_workspace_structure"

# Docker Compose service validation
echo ""
log_info "Validating Docker Compose services..."

services=("tests" "unit-tests" "integration-tests" "benchmarks" "wasm-tests" "quality" "coverage" "audit" "ci" "dev")
for service in "${services[@]}"; do
    run_check "Service '$service' defined" "check_file_contains docker-compose.yml '$service:'"
done

# Check multi-stage Dockerfile targets
echo ""
log_info "Validating Dockerfile targets..."

targets=("base" "test" "benchmark" "ci" "dev")
for target in "${targets[@]}"; do
    run_check "Docker target '$target'" "check_file_contains Dockerfile 'FROM .* as $target'"
done

# Validate environment variables and dependencies
echo ""
log_info "Validating dependencies and configuration..."

run_check "Rust tools installation" "check_file_contains Dockerfile 'cargo install'"
run_check "System dependencies" "check_file_contains Dockerfile 'apt-get install'"
run_check "WASM target" "check_file_contains Dockerfile 'wasm32-unknown-unknown'"
run_check "Font handling" "check_file_contains Dockerfile 'BENCH_FONT'"
run_check "Volume caching" "check_file_contains docker-compose.yml 'cargo-cache'"

# Validate GitHub Actions workflow
if [ -f ".github/workflows/docker-tests.yml" ]; then
    run_check "GitHub Actions workflow exists" "true"
    run_check "Workflow uses docker-test.sh" "check_file_contains .github/workflows/docker-tests.yml 'docker-test.sh'"
    run_check "Workflow has matrix strategy" "check_file_contains .github/workflows/docker-tests.yml 'matrix:'"
else
    run_check "GitHub Actions workflow exists" "false"
fi

# Check documentation
run_check "Docker documentation exists" "check_file_exists DOCKER.md"
if [ -f "DOCKER.md" ]; then
    run_check "Documentation covers quick start" "check_file_contains DOCKER.md 'Quick Start'"
    run_check "Documentation covers troubleshooting" "check_file_contains DOCKER.md 'Troubleshooting'"
fi

# Validate .dockerignore
echo ""
log_info "Validating .dockerignore configuration..."

run_check "Ignores target directory" "check_file_contains .dockerignore 'target/'"
run_check "Ignores git files" "check_file_contains .dockerignore '.git/'"
run_check "Includes essential files" "check_file_contains .dockerignore '!Cargo.toml'"

# Summary
echo ""
echo "📊 Validation Summary"
echo "===================="
echo "Total checks: $TOTAL_CHECKS"
echo "Passed: $PASSED_CHECKS"
echo "Failed: $FAILED_CHECKS"

if [ $FAILED_CHECKS -eq 0 ]; then
    log_success "All Docker configuration checks passed! 🎉"
    echo ""
    echo "✅ The Docker setup is ready for use!"
    echo ""
    echo "Next steps:"
    echo "  1. Install Docker and Docker Compose if not already installed"
    echo "  2. Run './scripts/docker-test.sh build' to build images"
    echo "  3. Run './scripts/docker-test.sh all' to test the complete setup"
    echo "  4. Use './scripts/docker-test.sh --help' for all available options"
    exit 0
else
    log_error "$FAILED_CHECKS validation check(s) failed"
    echo ""
    echo "❌ Please fix the issues above before using the Docker setup"
    exit 1
fi