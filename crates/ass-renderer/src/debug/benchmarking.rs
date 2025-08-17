//! Benchmarking and performance analysis for compatibility testing

use crate::debug::{CompatibilityResult, LibassRenderer};
use crate::renderer::{RenderContext, Renderer};
use crate::utils::RenderError;
use ass_core::parser::Script;

#[cfg(feature = "nostd")]
use alloc::{format, string::String, vec::Vec};
#[cfg(not(feature = "nostd"))]
use std::{format, string::String, time::Instant, vec::Vec};

/// Performance benchmarking system for renderer comparison
pub struct PerformanceBenchmark {
    /// Our renderer instance
    our_renderer: Renderer,
    /// Libass renderer for comparison
    #[cfg(feature = "libass-compare")]
    libass_renderer: LibassRenderer,
    /// Benchmark configuration
    config: BenchmarkConfig,
    /// Historical results for regression analysis
    historical_results: Vec<BenchmarkResult>,
}

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
    /// libass performance for comparison
    #[cfg(feature = "libass-compare")]
    pub libass_performance: Option<PerformanceMetrics>,
    /// Performance ratio (our_time / libass_time)
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

impl PerformanceBenchmark {
    /// Create new performance benchmark
    #[cfg(feature = "libass-compare")]
    pub fn new(context: RenderContext, config: BenchmarkConfig) -> Result<Self, RenderError> {
        let our_renderer = Renderer::new(crate::backends::BackendType::Software, context.clone())?;
        let libass_renderer = LibassRenderer::new(context.width(), context.height())?;

        Ok(Self {
            our_renderer,
            libass_renderer,
            config,
            historical_results: Vec::new(),
        })
    }

    /// Create benchmark without libass comparison
    #[cfg(not(feature = "libass-compare"))]
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
                "Benchmarking {} at {}x{}",
                test_name, resolution.0, resolution.1
            );

            // Update context resolution
            let context = RenderContext::new(resolution.0, resolution.1);
            self.our_renderer =
                Renderer::new(crate::backends::BackendType::Software, context)?;

            #[cfg(feature = "libass-compare")]
            {
                self.libass_renderer
                    .set_frame_size(resolution.0, resolution.1);
            }

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

        // Benchmark libass if available
        #[cfg(feature = "libass-compare")]
        let libass_performance = self.benchmark_libass_renderer(script, test_time_cs).ok();
        #[cfg(not(feature = "libass-compare"))]
        let libass_performance = None;

        // Calculate ratios
        let (performance_ratio, memory_ratio) = if let Some(ref libass_perf) = libass_performance {
            let perf_ratio = our_performance.avg_render_time_ms / libass_perf.avg_render_time_ms;
            let mem_ratio = match (
                our_performance.avg_memory_bytes,
                libass_perf.avg_memory_bytes,
            ) {
                (Some(our_mem), Some(libass_mem)) => Some(our_mem as f64 / libass_mem as f64),
                _ => None,
            };
            (Some(perf_ratio), mem_ratio)
        } else {
            (None, None)
        };

        // Calculate compatibility score (simplified - would need actual pixel comparison)
        let compatibility_score = 0.95; // Placeholder

        Ok(BenchmarkResult {
            test_name: format!(
                "{}_{}_{}x{}",
                test_name, "single", resolution.0, resolution.1
            ),
            resolution,
            our_performance,
            #[cfg(feature = "libass-compare")]
            libass_performance,
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

    /// Benchmark libass renderer performance
    #[cfg(feature = "libass-compare")]
    fn benchmark_libass_renderer(
        &mut self,
        script: &Script,
        time_cs: u32,
    ) -> Result<PerformanceMetrics, RenderError> {
        let mut render_times = Vec::new();
        let mut memory_usage = Vec::new();

        // Warmup
        for _ in 0..self.config.warmup_iterations {
            let _ = self.libass_renderer.render_frame(script, time_cs)?;
        }

        // Actual measurements
        for _ in 0..self.config.iterations {
            let start_memory = self.measure_memory_usage();

            #[cfg(not(feature = "nostd"))]
            let start_time = Instant::now();

            let _frame = self.libass_renderer.render_frame(script, time_cs)?;

            #[cfg(not(feature = "nostd"))]
            let elapsed = start_time.elapsed();
            #[cfg(feature = "nostd")]
            let elapsed = std::time::Duration::from_millis(1); // Placeholder for no_std

            let end_memory = self.measure_memory_usage();

            render_times.push(elapsed.as_secs_f64() * 1000.0);

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

    /// Run frame rate benchmark for animations
    pub fn benchmark_animation_framerate(
        &mut self,
        script: &Script,
        test_name: &str,
    ) -> Result<BenchmarkResult, RenderError> {
        if !self.config.measure_frame_rate {
            return Err(RenderError::UnsupportedOperation(
                "Frame rate benchmarking disabled".to_string(),
            ));
        }

        let (start_time, end_time) = self.find_animation_timerange(script);
        let total_frames = ((end_time - start_time) / 4) as usize; // 25 FPS
        let mut frame_times = Vec::new();

        eprintln!("Benchmarking animation frame rate: {} frames", total_frames);

        #[cfg(not(feature = "nostd"))]
        let start_benchmark = Instant::now();

        // Render all frames
        for frame_idx in 0..total_frames {
            let time_cs = start_time + (frame_idx as u32 * 4); // 25 FPS

            #[cfg(not(feature = "nostd"))]
            let frame_start = Instant::now();

            let _frame = self.our_renderer.render_frame(script, time_cs)?;

            #[cfg(not(feature = "nostd"))]
            let frame_time = frame_start.elapsed();
            #[cfg(feature = "nostd")]
            let frame_time = std::time::Duration::from_millis(1);

            frame_times.push(frame_time.as_secs_f64() * 1000.0);
        }

        #[cfg(not(feature = "nostd"))]
        let total_time = start_benchmark.elapsed().as_secs_f64();
        #[cfg(feature = "nostd")]
        let total_time = 1.0;

        let avg_frame_time = frame_times.iter().sum::<f64>() / frame_times.len() as f64;
        let fps = 1000.0 / avg_frame_time;
        let realtime_fps = total_frames as f64 / total_time;

        eprintln!("Animation benchmark results:");
        eprintln!("  Average frame time: {:.2}ms", avg_frame_time);
        eprintln!("  Theoretical FPS: {:.1}", fps);
        eprintln!("  Actual FPS: {:.1}", realtime_fps);

        let our_performance = PerformanceMetrics {
            avg_render_time_ms: avg_frame_time,
            min_render_time_ms: frame_times.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
            max_render_time_ms: frame_times.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)),
            render_time_std_dev: {
                let variance = frame_times
                    .iter()
                    .map(|&t| (t - avg_frame_time).powi(2))
                    .sum::<f64>()
                    / frame_times.len() as f64;
                variance.sqrt()
            },
            fps: Some(fps),
            peak_memory_bytes: None,
            avg_memory_bytes: None,
            cache_hit_rate: None,
        };

        Ok(BenchmarkResult {
            test_name: format!("{}_animation_fps", test_name),
            resolution: (
                self.our_renderer.context().width(),
                self.our_renderer.context().height(),
            ),
            our_performance,
            #[cfg(feature = "libass-compare")]
            libass_performance: None, // TODO: Implement libass animation benchmark
            performance_ratio: None,
            memory_ratio: None,
            compatibility_score: 1.0, // Placeholder
        })
    }

    /// Find animation time range
    fn find_animation_timerange(&self, script: &Script) -> (u32, u32) {
        let mut min_time = u32::MAX;
        let mut max_time = 0;

        for section in script.sections() {
            if let ass_core::parser::Section::Events(events) = section {
                for event in events {
                    let start = event.start_time_cs().unwrap_or(0);
                    let end = event.end_time_cs().unwrap_or(0);
                    min_time = min_time.min(start);
                    max_time = max_time.max(end);
                }
            }
        }

        if min_time == u32::MAX {
            (0, self.config.animation_duration_cs)
        } else {
            let duration = (max_time - min_time).min(self.config.animation_duration_cs);
            (min_time, min_time + duration)
        }
    }

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
