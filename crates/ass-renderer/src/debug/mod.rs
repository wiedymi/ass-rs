//! Debug and analysis tools for ASS subtitle rendering
//!
//! This module provides comprehensive debugging capabilities including:
//! - Frame analysis and benchmarking
//! - Visual comparison tools
//! - Performance profiling
//! - Compatibility testing with libass

#![allow(missing_docs)] // Debug module with many internal structures

use crate::{Frame, RenderError};

#[cfg(feature = "nostd")]
extern crate alloc;
#[cfg(feature = "nostd")]
use alloc::collections::BTreeMap as HashMap;
#[cfg(feature = "nostd")]
use alloc::{format, string::String, vec::Vec};
#[cfg(not(feature = "nostd"))]
use std::format;

/// Frame analysis and reporting tools
pub mod analyzer;
/// Color diagnostic utilities
pub mod color_diagnostic;
/// Frame inspection tools
pub mod inspector;
/// Debug player for subtitle playback
pub mod player;
/// Visual comparison utilities
pub mod visual_comparison;

/// Performance benchmarking tools
pub mod benchmarking;
/// libass compatibility testing
#[cfg(feature = "libass-compare")]
pub mod libass_compatibility;
/// libass FFI bindings
#[cfg(feature = "libass-compare")]
pub mod libass_ffi;
/// libass renderer wrapper
#[cfg(feature = "libass-compare")]
pub mod libass_renderer;
/// Pixel-perfect comparison tools
#[cfg(feature = "libass-compare")]
pub mod pixel_perfect_comparison;
/// Visual report generation
#[cfg(not(feature = "nostd"))]
pub mod visual_reporting;

pub use analyzer::{AnalysisReport, FrameAnalyzer};
pub use benchmarking::{
    quick_benchmark, BenchmarkConfig, BenchmarkResult, PerformanceBenchmark, PerformanceMetrics,
};
pub use inspector::FrameInspector;
#[cfg(feature = "libass-compare")]
pub use libass_compatibility::{
    CompatibilityResult, CompatibilityTestSuite, CompatibilityTester, DiffRegion, DiffType,
    TestConfig,
};
#[cfg(feature = "libass-compare")]
pub use libass_renderer::LibassRenderer;
#[cfg(feature = "libass-compare")]
pub use pixel_perfect_comparison::PixelPerfectComparison;
#[cfg(not(feature = "nostd"))]
pub use visual_reporting::{generate_compatibility_report, ReportConfig, VisualReportGenerator};
// #[cfg(feature = "libass-compare")]
// pub use pixel_perfect_comparison::{
//     PixelPerfectComparator, PixelComparisonResult, ComparisonConfig,
//     LibassCompatibilityTester, TestReport
// };
pub use player::{DebugPlayer, PlayerFrame};

/// Debug information for a rendered frame
#[derive(Debug, Clone)]
pub struct FrameDebugInfo {
    /// Frame timestamp in milliseconds
    pub timestamp_ms: u32,
    /// Number of active subtitle events
    pub active_events: usize,
    /// Dirty regions that need re-rendering
    pub dirty_regions: Vec<DirtyRegionInfo>,
    /// Time taken to render this frame in milliseconds
    pub render_time_ms: f64,
    /// Memory used by frame data in bytes
    pub memory_usage_bytes: usize,
    /// Number of cache hits during rendering
    pub cache_hits: usize,
    /// Number of cache misses during rendering
    pub cache_misses: usize,
    /// Name of the rendering backend used
    pub backend_type: String,
    /// Checksum of the frame data for comparison
    pub frame_checksum: u64,
    /// Number of non-transparent pixels
    pub non_transparent_pixels: usize,
    /// Bounding box of rendered content
    pub bounding_box: Option<BoundingBoxInfo>,
}

/// Information about a dirty region that needs re-rendering
#[derive(Debug, Clone)]
pub struct DirtyRegionInfo {
    /// X coordinate of the region
    pub x: u32,
    /// Y coordinate of the region
    pub y: u32,
    /// Width of the region
    pub width: u32,
    /// Height of the region
    pub height: u32,
    /// Reason why this region is dirty
    pub reason: String,
}

/// Bounding box information for rendered content
#[derive(Debug, Clone, PartialEq)]
pub struct BoundingBoxInfo {
    /// Minimum X coordinate
    pub min_x: u32,
    /// Minimum Y coordinate
    pub min_y: u32,
    /// Maximum X coordinate
    pub max_x: u32,
    /// Maximum Y coordinate
    pub max_y: u32,
}

/// Debug renderer that wraps a normal renderer and provides additional debugging info
pub struct DebugRenderer {
    /// The wrapped renderer
    inner: crate::Renderer,
    /// History of rendered frames with debug info
    frame_history: Vec<FrameDebugInfo>,
    /// Whether to add visual debugging overlay
    enable_visual_overlay: bool,
    /// Whether to output debug text to console
    enable_text_output: bool,
    /// Directory to save debug output files
    output_dir: Option<String>,
}

impl DebugRenderer {
    /// Create a new debug renderer wrapping the given renderer
    pub fn new(renderer: crate::Renderer) -> Self {
        Self {
            inner: renderer,
            frame_history: Vec::new(),
            enable_visual_overlay: false,
            enable_text_output: true,
            output_dir: None,
        }
    }

    /// Enable or disable visual debugging overlay on rendered frames
    pub fn enable_visual_overlay(&mut self, enable: bool) {
        self.enable_visual_overlay = enable;
    }

    /// Enable or disable text debug output to console
    pub fn enable_text_output(&mut self, enable: bool) {
        self.enable_text_output = enable;
    }

    /// Set the directory where debug output files will be saved
    pub fn set_output_dir(&mut self, dir: &str) {
        self.output_dir = Some(dir.to_string());
    }

    /// Render a frame with full debug instrumentation
    ///
    /// Returns both the rendered frame and detailed debug information
    pub fn render_frame_debug(
        &mut self,
        script: &ass_core::parser::Script,
        time_ms: u32,
    ) -> Result<(Frame, FrameDebugInfo), RenderError> {
        let start = std::time::Instant::now();

        // Render the frame
        let frame = self.inner.render_frame(script, time_ms)?;

        let render_time = start.elapsed().as_secs_f64() * 1000.0;

        // Collect debug info
        let debug_info = self.collect_debug_info(&frame, time_ms, render_time);

        // Store in history
        self.frame_history.push(debug_info.clone());

        // Output debug info if enabled
        if self.enable_text_output {
            self.output_text_debug(&debug_info);
        }

        // Save visual debug if output dir is set
        if let Some(ref dir) = self.output_dir {
            self.save_visual_debug(&frame, &debug_info, dir, time_ms)?;
        }

        Ok((frame, debug_info))
    }

    fn collect_debug_info(&self, frame: &Frame, time_ms: u32, render_time: f64) -> FrameDebugInfo {
        let pixels = frame.pixels();
        let mut non_transparent = 0;
        let mut min_x = frame.width();
        let mut min_y = frame.height();
        let mut max_x = 0u32;
        let mut max_y = 0u32;

        // Analyze pixels
        for y in 0..frame.height() {
            for x in 0..frame.width() {
                let idx = ((y * frame.width() + x) * 4) as usize;
                if idx + 3 < pixels.len() && pixels[idx + 3] > 0 {
                    non_transparent += 1;
                    min_x = min_x.min(x);
                    min_y = min_y.min(y);
                    max_x = max_x.max(x);
                    max_y = max_y.max(y);
                }
            }
        }

        let bounding_box = if non_transparent > 0 {
            Some(BoundingBoxInfo {
                min_x,
                min_y,
                max_x,
                max_y,
            })
        } else {
            None
        };

        // Calculate checksum
        let checksum = calculate_checksum(pixels);

        FrameDebugInfo {
            timestamp_ms: time_ms,
            active_events: 0,          // TODO: Get from renderer
            dirty_regions: Vec::new(), // TODO: Get from renderer
            render_time_ms: render_time,
            memory_usage_bytes: pixels.len(),
            cache_hits: 0,                        // TODO: Get from renderer
            cache_misses: 0,                      // TODO: Get from renderer
            backend_type: "Software".to_string(), // TODO: Get from renderer
            frame_checksum: checksum,
            non_transparent_pixels: non_transparent,
            bounding_box,
        }
    }

    fn output_text_debug(&self, info: &FrameDebugInfo) {
        println!("=== Frame Debug Info ===");
        println!("Timestamp: {}ms", info.timestamp_ms);
        println!("Render Time: {:.2}ms", info.render_time_ms);
        println!("Backend: {}", info.backend_type);
        println!("Active Events: {}", info.active_events);
        println!("Non-transparent Pixels: {}", info.non_transparent_pixels);

        if let Some(ref bbox) = info.bounding_box {
            println!(
                "Bounding Box: ({}, {}) to ({}, {})",
                bbox.min_x, bbox.min_y, bbox.max_x, bbox.max_y
            );
            println!(
                "  Size: {}x{}",
                bbox.max_x - bbox.min_x + 1,
                bbox.max_y - bbox.min_y + 1
            );
        }

        println!("Memory: {} KB", info.memory_usage_bytes / 1024);
        println!("Checksum: 0x{:016x}", info.frame_checksum);
        println!(
            "Cache: {} hits, {} misses",
            info.cache_hits, info.cache_misses
        );

        if !info.dirty_regions.is_empty() {
            println!("Dirty Regions:");
            for region in &info.dirty_regions {
                println!(
                    "  - {}x{} at ({}, {}): {}",
                    region.width, region.height, region.x, region.y, region.reason
                );
            }
        }
        println!("========================");
    }

    fn save_visual_debug(
        &self,
        frame: &Frame,
        info: &FrameDebugInfo,
        dir: &str,
        time_ms: u32,
    ) -> Result<(), RenderError> {
        // Create directory if it doesn't exist
        #[cfg(not(feature = "nostd"))]
        std::fs::create_dir_all(dir)
            .map_err(|e| RenderError::BackendError(format!("Failed to create debug dir: {}", e)))?;
        #[cfg(feature = "nostd")]
        return Err(RenderError::BackendError(
            "File operations not supported in no_std".into(),
        ));

        // Save frame as PNG with debug overlay if enabled
        if self.enable_visual_overlay {
            let debug_frame = self.create_debug_overlay(frame, info)?;
            save_frame_as_png(
                &debug_frame,
                &format!("{}/frame_{:06}_debug.png", dir, time_ms),
            )?;
        } else {
            save_frame_as_png(frame, &format!("{}/frame_{:06}.png", dir, time_ms))?;
        }

        // Save debug info as JSON
        #[cfg(all(not(feature = "nostd"), feature = "serde"))]
        {
            let json_path = format!("{}/frame_{:06}_info.json", dir, time_ms);
            let json = serde_json::to_string_pretty(&info).map_err(|e| {
                RenderError::BackendError(format!("Failed to serialize debug info: {}", e))
            })?;
            std::fs::write(json_path, json).map_err(|e| {
                RenderError::BackendError(format!("Failed to write debug info: {}", e))
            })?;
        }

        Ok(())
    }

    fn create_debug_overlay(
        &self,
        frame: &Frame,
        info: &FrameDebugInfo,
    ) -> Result<Frame, RenderError> {
        let mut debug_frame = frame.clone();

        // Draw bounding box
        if let Some(ref bbox) = info.bounding_box {
            draw_rectangle(
                &mut debug_frame,
                bbox.min_x,
                bbox.min_y,
                bbox.max_x - bbox.min_x + 1,
                bbox.max_y - bbox.min_y + 1,
                [255, 0, 0, 128],
            )?;
        }

        // Draw dirty regions
        for region in &info.dirty_regions {
            draw_rectangle(
                &mut debug_frame,
                region.x,
                region.y,
                region.width,
                region.height,
                [0, 255, 0, 128],
            )?;
        }

        // Draw info text overlay
        draw_text_overlay(
            &mut debug_frame,
            &format!(
                "Time: {}ms | Events: {} | Render: {:.1}ms",
                info.timestamp_ms, info.active_events, info.render_time_ms
            ),
            10,
            10,
        )?;

        Ok(debug_frame)
    }

    /// Get the history of rendered frames with debug information
    pub fn get_frame_history(&self) -> &[FrameDebugInfo] {
        &self.frame_history
    }

    /// Clear the frame history
    pub fn clear_history(&mut self) {
        self.frame_history.clear();
    }

    /// Compare two frames from the history by their indices
    ///
    /// Returns `None` if either index is out of bounds
    pub fn compare_frames(&self, idx1: usize, idx2: usize) -> Option<FrameComparison> {
        if idx1 >= self.frame_history.len() || idx2 >= self.frame_history.len() {
            return None;
        }

        let frame1 = &self.frame_history[idx1];
        let frame2 = &self.frame_history[idx2];

        Some(FrameComparison {
            checksum_match: frame1.frame_checksum == frame2.frame_checksum,
            pixel_diff: (frame1.non_transparent_pixels as i32
                - frame2.non_transparent_pixels as i32)
                .abs() as u32,
            render_time_diff: frame1.render_time_ms - frame2.render_time_ms,
            bbox_changed: frame1.bounding_box != frame2.bounding_box,
        })
    }
}

/// Result of comparing two frames
#[derive(Debug)]
pub struct FrameComparison {
    /// Whether the frame checksums match
    pub checksum_match: bool,
    /// Difference in non-transparent pixel count
    pub pixel_diff: u32,
    /// Difference in render time (milliseconds)
    pub render_time_diff: f64,
    /// Whether the bounding boxes differ
    pub bbox_changed: bool,
}

fn calculate_checksum(pixels: &[u8]) -> u64 {
    #[cfg(not(feature = "nostd"))]
    use std::collections::hash_map::DefaultHasher;
    #[cfg(not(feature = "nostd"))]
    use std::hash::{Hash, Hasher};

    #[cfg(feature = "nostd")]
    use core::hash::{Hash, Hasher};
    #[cfg(feature = "nostd")]
    struct DefaultHasher(u64);

    #[cfg(feature = "nostd")]
    impl DefaultHasher {
        fn new() -> Self {
            DefaultHasher(0)
        }
        fn finish(&self) -> u64 {
            self.0
        }
    }

    #[cfg(feature = "nostd")]
    impl Hasher for DefaultHasher {
        fn write(&mut self, bytes: &[u8]) {
            for &b in bytes {
                self.0 = self.0.wrapping_mul(31).wrapping_add(b as u64);
            }
        }

        fn finish(&self) -> u64 {
            self.0
        }
    }

    let mut hasher = DefaultHasher::new();
    pixels.hash(&mut hasher);
    hasher.finish()
}

fn save_frame_as_png(frame: &Frame, path: &str) -> Result<(), RenderError> {
    #[cfg(feature = "image")]
    {
        use image::{ImageBuffer, Rgba};

        let img = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(
            frame.width(),
            frame.height(),
            frame.pixels().to_vec(),
        )
        .ok_or_else(|| RenderError::BackendError("Failed to create image buffer".into()))?;

        img.save(path)
            .map_err(|e| RenderError::BackendError(format!("Failed to save PNG: {}", e)))?;
    }

    #[cfg(not(feature = "image"))]
    {
        let _ = (frame, path);
        // Silent no-op if image feature is not enabled
    }

    Ok(())
}

fn draw_rectangle(
    frame: &mut Frame,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    color: [u8; 4],
) -> Result<(), RenderError> {
    // TODO: Implement rectangle drawing
    let _ = (frame, x, y, width, height, color);
    Ok(())
}

fn draw_text_overlay(frame: &mut Frame, text: &str, x: u32, y: u32) -> Result<(), RenderError> {
    // TODO: Implement text overlay
    let _ = (frame, text, x, y);
    Ok(())
}

// Make FrameDebugInfo serializable
#[cfg(feature = "serde")]
impl serde::Serialize for FrameDebugInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("FrameDebugInfo", 11)?;
        state.serialize_field("timestamp_ms", &self.timestamp_ms)?;
        state.serialize_field("active_events", &self.active_events)?;
        state.serialize_field("dirty_regions", &self.dirty_regions)?;
        state.serialize_field("render_time_ms", &self.render_time_ms)?;
        state.serialize_field("memory_usage_bytes", &self.memory_usage_bytes)?;
        state.serialize_field("cache_hits", &self.cache_hits)?;
        state.serialize_field("cache_misses", &self.cache_misses)?;
        state.serialize_field("backend_type", &self.backend_type)?;
        state.serialize_field("frame_checksum", &format!("0x{:016x}", self.frame_checksum))?;
        state.serialize_field("non_transparent_pixels", &self.non_transparent_pixels)?;
        state.serialize_field("bounding_box", &self.bounding_box)?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for DirtyRegionInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("DirtyRegionInfo", 5)?;
        state.serialize_field("x", &self.x)?;
        state.serialize_field("y", &self.y)?;
        state.serialize_field("width", &self.width)?;
        state.serialize_field("height", &self.height)?;
        state.serialize_field("reason", &self.reason)?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for BoundingBoxInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("BoundingBoxInfo", 4)?;
        state.serialize_field("min_x", &self.min_x)?;
        state.serialize_field("min_y", &self.min_y)?;
        state.serialize_field("max_x", &self.max_x)?;
        state.serialize_field("max_y", &self.max_y)?;
        state.end()
    }
}
