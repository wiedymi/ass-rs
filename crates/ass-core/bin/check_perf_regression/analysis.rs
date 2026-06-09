//! Performance analysis against absolute targets and baseline comparisons.
//!
//! Compares loaded benchmark timings against fixed performance targets and
//! classifies criterion change estimates into regressions, improvements, or
//! results that stay within the acceptable threshold.

#[cfg(not(feature = "std"))]
use alloc::format;
use std::collections::HashMap;

use crate::types::{BenchmarkResult, RegressionAnalysis};

/// Performance regression threshold percentage
const REGRESSION_THRESHOLD_PERCENT: f64 = 10.0;
/// Incremental parsing target in milliseconds
const INCREMENTAL_PARSING_TARGET_MS: f64 = 5.0;

/// Analyze performance against absolute targets
pub fn analyze_performance_targets(
    results: &HashMap<String, BenchmarkResult>,
    analysis: &mut RegressionAnalysis,
) {
    for (name, result) in results {
        let time_ms = result.time_ms();

        // Check incremental parsing targets
        if name.contains("incremental") {
            if time_ms > INCREMENTAL_PARSING_TARGET_MS {
                analysis.target_violations.push(format!(
                    "❌ {name}: {time_ms:.2}ms > {INCREMENTAL_PARSING_TARGET_MS}ms target"
                ));
            } else {
                analysis.within_threshold.push(format!(
                    "✅ {name}: {time_ms:.2}ms ≤ {INCREMENTAL_PARSING_TARGET_MS}ms target"
                ));
            }
        }
    }
}

/// Analyze performance regressions against baseline
pub fn analyze_regressions(comparisons: &HashMap<String, f64>, analysis: &mut RegressionAnalysis) {
    for (name, change_percent) in comparisons {
        let abs_change = change_percent.abs();

        if abs_change > REGRESSION_THRESHOLD_PERCENT {
            if *change_percent > 0.0 {
                // Performance got worse
                analysis.regressions.push(format!(
                    "📉 {name}: {change_percent:.1}% slower (>{REGRESSION_THRESHOLD_PERCENT}% threshold)"
                ));
            } else {
                // Performance improved significantly
                analysis
                    .improvements
                    .push(format!("📈 {name}: {abs_change:.1}% faster"));
            }
        } else {
            analysis.within_threshold.push(format!(
                "✅ {name}: {change_percent:.1}% change (within {REGRESSION_THRESHOLD_PERCENT}% threshold)"
            ));
        }
    }
}
