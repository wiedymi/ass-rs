use crate::Frame;

#[cfg(not(feature = "nostd"))]
use std::time::Duration;

#[cfg(feature = "nostd")]
use alloc::vec::Vec;

pub struct PlayerFrame {
    pub frame: Frame,
    pub timestamp_ms: u32,
    pub render_time: Duration,
    pub frame_number: u32,
}

#[derive(Debug)]
pub struct TestReport {
    pub test_points: Vec<TestPoint>,
    pub total_render_time: Duration,
    pub frames_with_content: usize,
    pub frames_empty: usize,
    pub average_render_time_ms: f64,
}

impl TestReport {
    pub fn print_summary(&self) {
        println!("\n╔════════════════════════════════════════╗");
        println!("║         Test Report Summary            ║");
        println!("╚════════════════════════════════════════╝");

        println!("\n📈 Overall Statistics:");
        let test_points_len = self.test_points.len();
        println!("  • Test points: {test_points_len}");
        println!(
            "  • Frames with content: {} ({:.1}%)",
            self.frames_with_content,
            (self.frames_with_content as f32 / self.test_points.len() as f32) * 100.0
        );
        println!(
            "  • Empty frames: {} ({:.1}%)",
            self.frames_empty,
            (self.frames_empty as f32 / self.test_points.len() as f32) * 100.0
        );
        println!(
            "  • Average render time: {:.2}ms",
            self.average_render_time_ms
        );
        println!(
            "  • Total render time: {:.2}ms",
            self.total_render_time.as_secs_f64() * 1000.0
        );

        println!("\n📊 Performance Distribution:");
        let mut fast = 0;
        let mut normal = 0;
        let mut slow = 0;

        for point in &self.test_points {
            let ms = point.render_time.as_secs_f64() * 1000.0;
            if ms < 5.0 {
                fast += 1;
            } else if ms < 15.0 {
                normal += 1;
            } else {
                slow += 1;
            }
        }

        println!("  • Fast (<5ms): {fast}");
        println!("  • Normal (5-15ms): {normal}");
        println!("  • Slow (>15ms): {slow}");

        println!("\n🔍 Individual Test Points:");
        for point in &self.test_points {
            println!(
                "  • {:6}ms: {:.2}ms render | {}",
                point.timestamp_ms,
                point.render_time.as_secs_f64() * 1000.0,
                if point.has_visible_content {
                    "✓ visible"
                } else {
                    "✗ empty"
                }
            );
        }

        println!("\n═══════════════════════════════════════");
    }
}

#[derive(Debug)]
pub struct TestPoint {
    pub timestamp_ms: u32,
    pub render_time: Duration,
    pub has_visible_content: bool,
}
