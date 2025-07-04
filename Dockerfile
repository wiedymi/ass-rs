# Multi-stage Dockerfile for ASS Library Testing and Benchmarking
FROM rust:1.75-slim as base

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libfontconfig1-dev \
    libfreetype6-dev \
    libass-dev \
    wget \
    curl \
    git \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install additional Rust tools
RUN cargo install \
    cargo-audit \
    cargo-llvm-cov \
    critcmp \
    cargo-outdated \
    cargo-udeps \
    wasm-pack

# Install WASM target
RUN rustup target add wasm32-unknown-unknown

# Set working directory
WORKDIR /workspace

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./
COPY crates/*/Cargo.toml ./crates/

# Create dummy source files to build dependencies
RUN find crates -name "Cargo.toml" -exec dirname {} \; | \
    xargs -I {} sh -c 'mkdir -p {}/src && echo "fn main() {}" > {}/src/main.rs && echo "pub fn dummy() {}" > {}/src/lib.rs'

# Build dependencies
RUN cargo build --workspace --all-features
RUN cargo build --workspace --all-features --release

# Clean up dummy files
RUN find crates -name "src" -type d -exec rm -rf {} + 2>/dev/null || true

# Testing stage
FROM base as test

# Copy all source code
COPY . .

# Download test font if not present
RUN if [ ! -f "assets/NotoSans-Regular.ttf" ]; then \
    mkdir -p assets && \
    wget -O assets/NotoSans-Regular.ttf \
    "https://github.com/googlefonts/noto-fonts/raw/main/hinted/ttf/NotoSans/NotoSans-Regular.ttf"; \
    fi

# Set environment variables
ENV BENCH_FONT=/workspace/assets/NotoSans-Regular.ttf
ENV RUST_BACKTRACE=1
ENV RUST_LOG=info

# Make test runner executable
RUN chmod +x test_runner.sh

# Build all crates
RUN cargo build --workspace --all-features
RUN cargo build --workspace --all-features --release

# Benchmark stage
FROM test as benchmark

# Pre-build benchmarks
RUN cargo bench --workspace --no-run

# Install additional benchmarking tools
RUN apt-get update && apt-get install -y \
    valgrind \
    heaptrack \
    linux-perf \
    && rm -rf /var/lib/apt/lists/*

# CI stage - minimal environment for CI/CD
FROM base as ci

# Copy only necessary files for CI
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/
COPY tests/ ./tests/
COPY test_runner.sh ./
COPY assets/ ./assets/

# Set CI-specific environment variables
ENV CI=true
ENV RUST_BACKTRACE=1
ENV CARGO_TERM_COLOR=always

# Make test runner executable
RUN chmod +x test_runner.sh

# Build workspace
RUN cargo build --workspace --all-features

# Development stage - includes dev tools
FROM test as dev

# Install additional development tools
RUN cargo install \
    cargo-expand \
    cargo-fuzz \
    cargo-watch \
    cargo-nextest

# Install debugging tools
RUN apt-get update && apt-get install -y \
    gdb \
    strace \
    ltrace \
    && rm -rf /var/lib/apt/lists/*

# Default command
CMD ["./test_runner.sh"]