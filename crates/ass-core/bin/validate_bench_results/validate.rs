//! Performance target validation logic
//!
//! Compares parsed benchmark results against the project's incremental,
//! editor-simulation, full-parsing, and memory-ratio performance targets.

#[cfg(not(feature = "std"))]
use alloc::format;

use crate::types::{BenchmarkResult, PerformanceTargets, ValidationResults};

/// Validate incremental parsing performance targets
pub fn validate_incremental_performance(
    results: &[BenchmarkResult],
    targets: &PerformanceTargets,
    validation: &mut ValidationResults,
) {
    for result in results {
        let time_ms = result.time_ns / 1_000_000.0;

        // Check incremental parsing time target
        if result.name.contains("incremental") {
            // Allow more time for large-scale anime files
            let time_target = if result.name.contains("large_scale_anime") {
                // Scale target based on file size
                if result.name.contains("bdmv_full") {
                    50.0 // 50ms for 100k+ events
                } else if result.name.contains("ova_complex") {
                    25.0 // 25ms for 50k events
                } else if result.name.contains("movie_2hr") {
                    15.0 // 15ms for 30k events
                } else {
                    10.0 // 10ms for 10k events
                }
            } else {
                targets.max_incremental_time_ms
            };

            if time_ms <= time_target {
                validation.add_pass(format!(
                    "Incremental parsing '{0}': {time_ms:.2}ms ≤ {time_target}ms",
                    result.name
                ));
            } else {
                validation.add_fail(format!(
                    "Incremental parsing '{0}': {time_ms:.2}ms > {time_target}ms",
                    result.name
                ));
            }
        }

        // Check editor simulation performance
        if result.name.contains("editor_simulation") {
            if time_ms <= targets.max_incremental_time_ms * 2.0 {
                // Allow 2x for editor operations
                validation.add_pass(format!(
                    "Editor simulation '{0}': {time_ms:.2}ms ≤ {1}ms",
                    result.name,
                    targets.max_incremental_time_ms * 2.0
                ));
            } else {
                validation.add_fail(format!(
                    "Editor simulation '{0}': {time_ms:.2}ms > {1}ms",
                    result.name,
                    targets.max_incremental_time_ms * 2.0
                ));
            }
        }
    }
}

/// Validate parser performance targets
pub fn validate_parser_performance(
    results: &[BenchmarkResult],
    targets: &PerformanceTargets,
    validation: &mut ValidationResults,
) {
    for result in results {
        let time_ms = result.time_ns / 1_000_000.0;

        // Check full parsing performance (should be reasonable but not as strict)
        if result.name.contains("parsing") && !result.name.contains("incremental") {
            if time_ms <= 50.0 {
                // 50ms for full parsing is reasonable
                validation.add_pass(format!(
                    "Full parsing '{0}': {time_ms:.2}ms ≤ 50ms",
                    result.name
                ));
            } else {
                validation.add_warning(format!(
                    "Full parsing '{0}': {time_ms:.2}ms > 50ms (not critical)",
                    result.name
                ));
            }
        }

        // Check memory usage if available
        if let Some(memory) = result.memory_usage {
            if let Some(throughput) = result.throughput_bytes_per_sec {
                let input_size = throughput / 1000.0; // Rough estimate
                #[allow(clippy::cast_precision_loss)]
                let ratio = memory as f64 / input_size;

                if ratio <= targets.max_memory_ratio {
                    validation.add_pass(format!(
                        "Memory ratio '{0}': {ratio:.2}x ≤ {1}x",
                        result.name, targets.max_memory_ratio
                    ));
                } else {
                    validation.add_fail(format!(
                        "Memory ratio '{0}': {ratio:.2}x > {1}x",
                        result.name, targets.max_memory_ratio
                    ));
                }
            }
        }
    }
}
