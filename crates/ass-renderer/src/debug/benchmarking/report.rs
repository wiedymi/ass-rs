//! Performance report and summary statistics generation

#[cfg(feature = "nostd")]
use alloc::{string::String, vec::Vec};
#[cfg(not(feature = "nostd"))]
use std::{string::String, vec::Vec};

use super::{PerformanceBenchmark, PerformanceReport, PerformanceSummary};

impl PerformanceBenchmark {
    /// Generate performance report
    pub fn generate_performance_report(&self) -> PerformanceReport {
        PerformanceReport {
            results: self.historical_results.clone(),
            summary: self.calculate_performance_summary(),
        }
    }

    /// Calculate performance summary statistics
    fn calculate_performance_summary(&self) -> PerformanceSummary {
        if self.historical_results.is_empty() {
            return PerformanceSummary::default();
        }

        let total_tests = self.historical_results.len();
        let performance_ratios: Vec<f64> = self
            .historical_results
            .iter()
            .filter_map(|r| r.performance_ratio)
            .collect();

        let avg_performance_ratio = if !performance_ratios.is_empty() {
            performance_ratios.iter().sum::<f64>() / performance_ratios.len() as f64
        } else {
            1.0
        };

        let avg_compatibility = self
            .historical_results
            .iter()
            .map(|r| r.compatibility_score)
            .sum::<f64>()
            / total_tests as f64;

        let avg_fps = self
            .historical_results
            .iter()
            .filter_map(|r| r.our_performance.fps)
            .sum::<f64>()
            / total_tests as f64;

        PerformanceSummary {
            total_tests,
            avg_performance_ratio,
            avg_compatibility_score: avg_compatibility,
            avg_fps,
            fastest_test: self.find_fastest_test(),
            slowest_test: self.find_slowest_test(),
        }
    }

    /// Find fastest test
    fn find_fastest_test(&self) -> Option<String> {
        self.historical_results
            .iter()
            .min_by(|a, b| {
                a.our_performance
                    .avg_render_time_ms
                    .partial_cmp(&b.our_performance.avg_render_time_ms)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|r| r.test_name.clone())
    }

    /// Find slowest test
    fn find_slowest_test(&self) -> Option<String> {
        self.historical_results
            .iter()
            .max_by(|a, b| {
                a.our_performance
                    .avg_render_time_ms
                    .partial_cmp(&b.our_performance.avg_render_time_ms)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|r| r.test_name.clone())
    }
}
