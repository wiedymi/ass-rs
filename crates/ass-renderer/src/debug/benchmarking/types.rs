//! Data structures for benchmark configuration and results

#[cfg(feature = "nostd")]
use alloc::{string::String, vec::Vec};
#[cfg(not(feature = "nostd"))]
use std::{string::String, vec::Vec};

/// Configuration for performance benchmarking
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    /// Number of iterations per test
    pub iterations: usize,
    /// Warmup iterations before measuring
    pub warmup_iterations: usize,
    /// Include memory usage measurement
    pub measure_memory: bool,
    /// Include frame rate measurement for animations
    pub measure_frame_rate: bool,
    /// Test different video resolutions
    pub test_resolutions: Vec<(u32, u32)>,
    /// Animation duration for frame rate tests (in centiseconds)
    pub animation_duration_cs: u32,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            iterations: 10,
            warmup_iterations: 3,
            measure_memory: true,
            measure_frame_rate: true,
            test_resolutions: vec![
                (1280, 720),  // 720p
                (1920, 1080), // 1080p
                (3840, 2160), // 4K
            ],
            animation_duration_cs: 1000, // 10 seconds
        }
    }
}

/// Result of performance benchmark
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// Test identifier
    pub test_name: String,
    /// Video resolution used
    pub resolution: (u32, u32),
    /// Our renderer performance
    pub our_performance: PerformanceMetrics,
    /// Performance ratio (our_time / reference_time), if a reference is available
    pub performance_ratio: Option<f64>,
    /// Memory usage comparison
    pub memory_ratio: Option<f64>,
    /// Compatibility score (0.0 = completely different, 1.0 = identical)
    pub compatibility_score: f64,
}

/// Detailed performance metrics
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Average rendering time per frame (milliseconds)
    pub avg_render_time_ms: f64,
    /// Minimum rendering time (milliseconds)
    pub min_render_time_ms: f64,
    /// Maximum rendering time (milliseconds)
    pub max_render_time_ms: f64,
    /// Standard deviation of render times
    pub render_time_std_dev: f64,
    /// Frames per second (for animation tests)
    pub fps: Option<f64>,
    /// Peak memory usage (bytes)
    pub peak_memory_bytes: Option<usize>,
    /// Average memory usage (bytes)
    pub avg_memory_bytes: Option<usize>,
    /// Cache hit rate (0.0 to 1.0)
    pub cache_hit_rate: Option<f64>,
}

/// Performance report containing all benchmark results
#[derive(Debug, Clone)]
pub struct PerformanceReport {
    /// Individual test results
    pub results: Vec<BenchmarkResult>,
    /// Summary statistics
    pub summary: PerformanceSummary,
}

/// Summary of performance characteristics
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    /// Total number of tests run
    pub total_tests: usize,
    /// Average performance ratio vs libass
    pub avg_performance_ratio: f64,
    /// Average compatibility score
    pub avg_compatibility_score: f64,
    /// Average frames per second
    pub avg_fps: f64,
    /// Name of fastest test
    pub fastest_test: Option<String>,
    /// Name of slowest test
    pub slowest_test: Option<String>,
}

impl Default for PerformanceSummary {
    fn default() -> Self {
        Self {
            total_tests: 0,
            avg_performance_ratio: 1.0,
            avg_compatibility_score: 1.0,
            avg_fps: 0.0,
            fastest_test: None,
            slowest_test: None,
        }
    }
}
