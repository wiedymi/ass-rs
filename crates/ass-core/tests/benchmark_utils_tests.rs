//! Tests for benchmark utility functions
//!
//! This module tests the synthetic data generation utilities used in benchmarks
//! to ensure they produce valid ASS content and cover all code paths.

#[cfg(feature = "benches")]
#[path = "benchmark_utils_tests/synthetic_basic.rs"]
mod synthetic_basic;

#[cfg(feature = "benches")]
#[path = "benchmark_utils_tests/synthetic_variations.rs"]
mod synthetic_variations;

#[cfg(feature = "benches")]
#[path = "benchmark_utils_tests/event_creation.rs"]
mod event_creation;

#[cfg(not(feature = "benches"))]
#[path = "benchmark_utils_tests/fallback.rs"]
mod fallback;
