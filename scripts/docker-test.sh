#!/bin/bash

# Docker Test Runner for ASS Library
# Provides convenient commands to run various test suites in Docker containers

set -e

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

# Function to check if Docker is running
check_docker() {
    if ! docker info >/dev/null 2>&1; then
        print_error "Docker is not running. Please start Docker and try again."
        exit 1
    fi
}

# Function to check if docker-compose is available
check_compose() {
    if ! command -v docker-compose >/dev/null 2>&1; then
        print_error "docker-compose is not installed. Please install it and try again."
        exit 1
    fi
}

# Function to build Docker images
build_images() {
    print_status "Building Docker images..."
    docker-compose build
    print_success "Docker images built successfully"
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [COMMAND] [OPTIONS]"
    echo ""
    echo "Commands:"
    echo "  all                 Run all tests (equivalent to ./test_runner.sh)"
    echo "  unit                Run unit tests only"
    echo "  integration         Run integration tests only"
    echo "  benchmarks          Run performance benchmarks"
    echo "  wasm                Run WASM tests"
    echo "  quality             Run code quality checks (format + clippy)"
    echo "  coverage            Generate test coverage report"
    echo "  audit               Run security audit"
    echo "  ci                  Simulate CI environment"
    echo "  dev                 Start development container"
    echo "  build               Build Docker images"
    echo "  clean               Clean Docker containers and volumes"
    echo ""
    echo "Options:"
    echo "  --build             Force rebuild of Docker images"
    echo "  --no-cache          Build without using cache"
    echo "  --parallel          Run tests in parallel (where applicable)"
    echo "  --verbose           Enable verbose output"
    echo "  --help              Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 all                    # Run all tests"
    echo "  $0 unit --build           # Build images and run unit tests"
    echo "  $0 benchmarks --verbose   # Run benchmarks with verbose output"
    echo "  $0 dev                    # Start interactive development container"
}

# Parse command line arguments
COMMAND=""
BUILD_FLAG=""
VERBOSE_FLAG=""
PARALLEL_FLAG=""

while [[ $# -gt 0 ]]; do
    case $1 in
        all|unit|integration|benchmarks|wasm|quality|coverage|audit|ci|dev|build|clean)
            COMMAND="$1"
            shift
            ;;
        --build)
            BUILD_FLAG="--build"
            shift
            ;;
        --no-cache)
            BUILD_FLAG="--build --no-cache"
            shift
            ;;
        --parallel)
            PARALLEL_FLAG="--parallel"
            shift
            ;;
        --verbose)
            VERBOSE_FLAG="--verbose"
            shift
            ;;
        --help)
            show_usage
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Default command if none provided
if [[ -z "$COMMAND" ]]; then
    COMMAND="all"
fi

# Check prerequisites
check_docker
check_compose

# Set docker-compose environment variables
export COMPOSE_PROJECT_NAME="ass-library-tests"

if [[ -n "$VERBOSE_FLAG" ]]; then
    export RUST_LOG="debug"
    export VERBOSE="1"
fi

# Main command execution
case $COMMAND in
    build)
        build_images
        ;;
    
    all)
        print_status "Running comprehensive test suite..."
        docker-compose run $BUILD_FLAG --rm tests
        ;;
    
    unit)
        print_status "Running unit tests..."
        docker-compose run $BUILD_FLAG --rm unit-tests
        ;;
    
    integration)
        print_status "Running integration tests..."
        docker-compose run $BUILD_FLAG --rm integration-tests
        ;;
    
    benchmarks)
        print_status "Running performance benchmarks..."
        docker-compose run $BUILD_FLAG --rm benchmarks
        ;;
    
    wasm)
        print_status "Running WASM tests..."
        docker-compose run $BUILD_FLAG --rm wasm-tests
        ;;
    
    quality)
        print_status "Running code quality checks..."
        docker-compose run $BUILD_FLAG --rm quality
        ;;
    
    coverage)
        print_status "Generating test coverage report..."
        docker-compose run $BUILD_FLAG --rm coverage
        print_status "Coverage report generated in target/llvm-cov/html/"
        ;;
    
    audit)
        print_status "Running security audit..."
        docker-compose run $BUILD_FLAG --rm audit
        ;;
    
    ci)
        print_status "Simulating CI environment..."
        docker-compose run $BUILD_FLAG --rm ci
        ;;
    
    dev)
        print_status "Starting development container..."
        docker-compose run $BUILD_FLAG --rm dev
        ;;
    
    clean)
        print_status "Cleaning Docker containers and volumes..."
        docker-compose down --volumes --remove-orphans
        docker system prune -f
        print_success "Cleanup completed"
        ;;
    
    *)
        print_error "Unknown command: $COMMAND"
        show_usage
        exit 1
        ;;
esac

print_success "Command '$COMMAND' completed successfully"