//! Benchmarking and performance analysis for the renderer

use crate::renderer::{RenderContext, Renderer};
use crate::utils::RenderError;
use ass_core::parser::Script;

#[cfg(feature = "nostd")]
use alloc::{format, vec::Vec};
#[cfg(not(feature = "nostd"))]
use std::{format, time::Instant, vec::Vec};

mod animation;
mod report;
mod types;

pub use types::{
    BenchmarkConfig, BenchmarkResult, PerformanceMetrics, PerformanceReport, PerformanceSummary,
};

/// Performance benchmarking system for renderer comparison
pub struct PerformanceBenchmark {
    /// Our renderer instance
    our_renderer: Renderer,
    /// Benchmark configuration
    config: BenchmarkConfig,
    /// Historical results for regression analysis
    historical_results: Vec<BenchmarkResult>,
}

impl PerformanceBenchmark {
    /// Create new performance benchmark
    pub fn new(context: RenderContext, config: BenchmarkConfig) -> Result<Self, RenderError> {
        let our_renderer = Renderer::new(crate::backends::BackendType::Software, context)?;

        Ok(Self {
            our_renderer,
            config,
            historical_results: Vec::new(),
        })
    }

    /// Run comprehensive benchmark on a script
    pub fn benchmark_script(
        &mut self,
        script: &Script,
        test_name: &str,
    ) -> Result<Vec<BenchmarkResult>, RenderError> {
        let mut results = Vec::new();
        let test_resolutions = self.config.test_resolutions.clone();

        for &resolution in &test_resolutions {
            eprintln!(
                "Benchmarking {test_name} at {}x{}",
                resolution.0, resolution.1
            );

            // Update context resolution
            let context = RenderContext::new(resolution.0, resolution.1);
            self.our_renderer = Renderer::new(crate::backends::BackendType::Software, context)?;

            let result = self.benchmark_single_resolution(script, test_name, resolution)?;
            results.push(result);
        }

        // Store results for regression analysis
        self.historical_results.extend(results.clone());

        Ok(results)
    }

    /// Benchmark at a single resolution
    fn benchmark_single_resolution(
        &mut self,
        script: &Script,
        test_name: &str,
        resolution: (u32, u32),
    ) -> Result<BenchmarkResult, RenderError> {
        let test_time_cs = self.find_representative_time(script);

        // Benchmark our renderer
        let our_performance = self.benchmark_our_renderer(script, test_time_cs)?;

        // No reference renderer to compare against, so ratios are unavailable.
        let (performance_ratio, memory_ratio): (Option<f64>, Option<f64>) = (None, None);

        // Calculate compatibility score (simplified - would need actual pixel comparison)
        let compatibility_score = 0.95; // Placeholder

        Ok(BenchmarkResult {
            test_name: format!("{test_name}_{}_{}x{}", "single", resolution.0, resolution.1),
            resolution,
            our_performance,
            performance_ratio,
            memory_ratio,
            compatibility_score,
        })
    }

    /// Benchmark our renderer performance
    fn benchmark_our_renderer(
        &mut self,
        script: &Script,
        time_cs: u32,
    ) -> Result<PerformanceMetrics, RenderError> {
        let mut render_times = Vec::new();
        let mut memory_usage = Vec::new();

        // Warmup
        for _ in 0..self.config.warmup_iterations {
            let _ = self.our_renderer.render_frame(script, time_cs)?;
        }

        // Actual measurements
        for _ in 0..self.config.iterations {
            let start_memory = self.measure_memory_usage();

            #[cfg(not(feature = "nostd"))]
            let start_time = Instant::now();

            let _frame = self.our_renderer.render_frame(script, time_cs)?;

            #[cfg(not(feature = "nostd"))]
            let elapsed = start_time.elapsed();
            #[cfg(feature = "nostd")]
            let elapsed = std::time::Duration::from_millis(1); // Placeholder for no_std

            let end_memory = self.measure_memory_usage();

            render_times.push(elapsed.as_secs_f64() * 1000.0); // Convert to milliseconds

            if let (Some(start), Some(end)) = (start_memory, end_memory) {
                memory_usage.push(end.saturating_sub(start));
            }
        }

        self.calculate_performance_metrics(render_times, memory_usage)
    }

    /// Calculate performance metrics from raw measurements
    fn calculate_performance_metrics(
        &self,
        render_times: Vec<f64>,
        memory_usage: Vec<usize>,
    ) -> Result<PerformanceMetrics, RenderError> {
        if render_times.is_empty() {
            return Err(RenderError::InvalidInput(
                "No render time measurements".to_string(),
            ));
        }

        let avg_render_time = render_times.iter().sum::<f64>() / render_times.len() as f64;
        let min_render_time = render_times.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_render_time = render_times
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        // Calculate standard deviation
        let variance = render_times
            .iter()
            .map(|&t| (t - avg_render_time).powi(2))
            .sum::<f64>()
            / render_times.len() as f64;
        let std_dev = variance.sqrt();

        // Calculate FPS (frames per second)
        let fps = if avg_render_time > 0.0 {
            Some(1000.0 / avg_render_time)
        } else {
            None
        };

        // Memory statistics
        let (peak_memory, avg_memory) = if !memory_usage.is_empty() {
            let peak = memory_usage.iter().max().copied();
            let avg = Some(memory_usage.iter().sum::<usize>() / memory_usage.len());
            (peak, avg)
        } else {
            (None, None)
        };

        Ok(PerformanceMetrics {
            avg_render_time_ms: avg_render_time,
            min_render_time_ms: min_render_time,
            max_render_time_ms: max_render_time,
            render_time_std_dev: std_dev,
            fps,
            peak_memory_bytes: peak_memory,
            avg_memory_bytes: avg_memory,
            cache_hit_rate: None, // TODO: Implement cache metrics
        })
    }

    /// Measure current memory usage (simplified)
    fn measure_memory_usage(&self) -> Option<usize> {
        // This is a simplified implementation
        // In practice, you'd want to use a proper memory profiler
        #[cfg(not(feature = "nostd"))]
        {
            // Use system APIs to get memory usage
            // This is a placeholder - actual implementation would vary by platform
            None
        }
        #[cfg(feature = "nostd")]
        {
            None
        }
    }

    /// Find representative time for testing
    fn find_representative_time(&self, script: &Script) -> u32 {
        // Find middle of first event with text
        for section in script.sections() {
            if let ass_core::parser::Section::Events(events) = section {
                for event in events {
                    if !event.text.trim().is_empty() {
                        let start = event.start_time_cs().unwrap_or(0);
                        let end = event.end_time_cs().unwrap_or(0);
                        return start + (end - start) / 2;
                    }
                }
            }
        }
        100 // Default to 1 second
    }
}

/// Quick benchmark function for single script
pub fn quick_benchmark(script: &Script, test_name: &str) -> Result<BenchmarkResult, RenderError> {
    let context = RenderContext::new(1920, 1080);
    let config = BenchmarkConfig {
        iterations: 5,
        warmup_iterations: 1,
        test_resolutions: vec![(1920, 1080)],
        ..Default::default()
    };

    let mut benchmark = PerformanceBenchmark::new(context, config)?;
    let results = benchmark.benchmark_script(script, test_name)?;

    results
        .into_iter()
        .next()
        .ok_or_else(|| RenderError::InvalidState("No benchmark results".to_string()))
}
