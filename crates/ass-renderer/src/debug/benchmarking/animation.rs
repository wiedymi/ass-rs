//! Animation frame-rate benchmarking for the renderer

use crate::utils::RenderError;
use ass_core::parser::Script;

#[cfg(feature = "nostd")]
use alloc::{format, vec::Vec};
#[cfg(not(feature = "nostd"))]
use std::{format, time::Instant, vec::Vec};

use super::{BenchmarkResult, PerformanceBenchmark, PerformanceMetrics};

impl PerformanceBenchmark {
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

        eprintln!("Benchmarking animation frame rate: {total_frames} frames");

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
        eprintln!("  Average frame time: {avg_frame_time:.2}ms");
        eprintln!("  Theoretical FPS: {fps:.1}");
        eprintln!("  Actual FPS: {realtime_fps:.1}");

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
            test_name: format!("{test_name}_animation_fps"),
            resolution: (
                self.our_renderer.context().width(),
                self.our_renderer.context().height(),
            ),
            our_performance,
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
}
