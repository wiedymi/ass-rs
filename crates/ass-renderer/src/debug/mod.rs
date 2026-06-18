//! Debug and analysis tools for ASS subtitle rendering
//!
//! This module provides comprehensive debugging capabilities including:
//! - Frame analysis and benchmarking
//! - Visual comparison tools
//! - Performance profiling

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
/// Debug information data structures
pub mod info;
/// Frame inspection tools
pub mod inspector;
/// Debug player for subtitle playback
pub mod player;
/// Frame helper utilities for the debug renderer
pub mod util;
/// Visual comparison utilities
pub mod visual_comparison;

/// Performance benchmarking tools
pub mod benchmarking;

/// libass FFI bridge for A/B comparison (dev-only, requires native libass).
#[cfg(feature = "libass-compare")]
pub mod libass;

pub use analyzer::{AnalysisReport, FrameAnalyzer};
pub use benchmarking::{
    quick_benchmark, BenchmarkConfig, BenchmarkResult, PerformanceBenchmark, PerformanceMetrics,
};
pub use info::{BoundingBoxInfo, DirtyRegionInfo, FrameComparison, FrameDebugInfo};
pub use inspector::FrameInspector;
pub use player::{DebugPlayer, PlayerFrame};

use util::{calculate_checksum, draw_rectangle, draw_text_overlay, save_frame_as_png};

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
            .map_err(|e| RenderError::BackendError(format!("Failed to create debug dir: {e}")))?;
        #[cfg(feature = "nostd")]
        return Err(RenderError::BackendError(
            "File operations not supported in no_std".into(),
        ));

        // Save frame as PNG with debug overlay if enabled
        if self.enable_visual_overlay {
            let debug_frame = self.create_debug_overlay(frame, info)?;
            save_frame_as_png(&debug_frame, &format!("{dir}/frame_{time_ms:06}_debug.png"))?;
        } else {
            save_frame_as_png(frame, &format!("{dir}/frame_{time_ms:06}.png"))?;
        }

        // Save debug info as JSON
        #[cfg(all(not(feature = "nostd"), feature = "serde"))]
        {
            let json_path = format!("{dir}/frame_{time_ms:06}_info.json");
            let json = serde_json::to_string_pretty(&info).map_err(|e| {
                RenderError::BackendError(format!("Failed to serialize debug info: {e}"))
            })?;
            std::fs::write(json_path, json).map_err(|e| {
                RenderError::BackendError(format!("Failed to write debug info: {e}"))
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
                .unsigned_abs(),
            render_time_diff: frame1.render_time_ms - frame2.render_time_ms,
            bbox_changed: frame1.bounding_box != frame2.bounding_box,
        })
    }
}
