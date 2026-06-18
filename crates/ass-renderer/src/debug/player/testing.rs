use super::{TestPoint, TestReport};
use crate::RenderError;
use ass_core::parser::Script;

#[cfg(not(feature = "nostd"))]
use std::time::{Duration, Instant};

#[cfg(feature = "nostd")]
use alloc::vec::Vec;

impl super::DebugPlayer {
    pub fn run_automatic_test(&mut self, test_points: Vec<u32>) -> Result<TestReport, RenderError> {
        let script_content = self
            .script_content
            .as_ref()
            .ok_or_else(|| RenderError::InvalidInput("No script loaded".into()))?;

        let script = Script::parse(script_content)
            .map_err(|e| RenderError::ParseError(format!("Failed to parse script: {e:?}")))?;

        let mut report = TestReport {
            test_points: Vec::new(),
            total_render_time: Duration::ZERO,
            frames_with_content: 0,
            frames_empty: 0,
            average_render_time_ms: 0.0,
        };

        println!(
            "\n🧪 Running automatic test at {} points",
            test_points.len()
        );

        for time_ms in test_points {
            println!("  Testing at {time_ms}ms...");

            let start = Instant::now();
            // Convert milliseconds to centiseconds for the renderer
            let time_cs = time_ms / 10;
            let frame = self.renderer.render_frame(&script, time_cs)?;
            let render_time = start.elapsed();

            let pixels = frame.pixels();
            let mut has_content = false;

            for chunk in pixels.chunks(4) {
                if chunk.len() == 4 && chunk[3] > 0 {
                    has_content = true;
                    break;
                }
            }

            if has_content {
                report.frames_with_content += 1;
            } else {
                report.frames_empty += 1;
            }

            report.test_points.push(TestPoint {
                timestamp_ms: time_ms,
                render_time,
                has_visible_content: has_content,
            });

            report.total_render_time += render_time;
        }

        if !report.test_points.is_empty() {
            report.average_render_time_ms =
                report.total_render_time.as_secs_f64() * 1000.0 / report.test_points.len() as f64;
        }

        report.print_summary();

        Ok(report)
    }
}
