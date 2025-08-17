//! Comprehensive libass compatibility testing framework
//!
//! This module provides tools for pixel-perfect comparison between our renderer
//! and libass, including visual diff generation, animation testing, and regression detection.

use crate::debug::{LibassRenderer, PixelPerfectComparison};
use crate::renderer::{RenderContext, Renderer};
use crate::utils::RenderError;
use ass_core::parser::Script;

#[cfg(feature = "nostd")]
use alloc::{format, string::String, vec::Vec};
#[cfg(not(feature = "nostd"))]
use std::{collections::HashMap, string::String, time::Instant, vec::Vec};

/// Compatibility test result
#[derive(Debug, Clone)]
pub struct CompatibilityResult {
    /// Test name/identifier
    pub test_name: String,
    /// Whether the test passed (pixel-perfect match)
    pub passed: bool,
    /// Pixel difference percentage (0.0 = perfect match, 1.0 = completely different)
    pub pixel_diff_percentage: f64,
    /// Maximum pixel value difference (0-255)
    pub max_pixel_diff: u8,
    /// Areas where differences occur
    pub diff_regions: Vec<DiffRegion>,
    /// Rendering time comparison (ours vs libass)
    pub performance_ratio: Option<f64>,
    /// Memory usage comparison
    pub memory_ratio: Option<f64>,
}

/// Region where pixel differences were detected
#[derive(Debug, Clone)]
pub struct DiffRegion {
    /// Bounding box (x, y, width, height)
    pub bounds: (u32, u32, u32, u32),
    /// Average difference in this region
    pub avg_diff: f64,
    /// Maximum difference in this region
    pub max_diff: u8,
    /// Type of difference detected
    pub diff_type: DiffType,
}

/// Types of differences that can be detected
#[derive(Debug, Clone, PartialEq)]
pub enum DiffType {
    /// Color/pixel value differences
    PixelValue,
    /// Text positioning differences
    TextPosition,
    /// Animation timing differences
    AnimationTiming,
    /// Font rendering differences
    FontRendering,
    /// Effect application differences
    Effects,
    /// Geometric transformation differences
    Transform,
}

/// Comprehensive compatibility tester
pub struct CompatibilityTester {
    /// Our renderer instance
    our_renderer: Renderer,
    /// Libass renderer for reference
    libass_renderer: LibassRenderer,
    /// Test configuration
    config: TestConfig,
    /// Results cache for regression testing
    #[cfg(not(feature = "nostd"))]
    #[allow(dead_code)] // Cache for future regression testing functionality
    results_cache: HashMap<String, CompatibilityResult>,
}

/// Configuration for compatibility testing
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// Tolerance for pixel differences (0-255)
    pub pixel_tolerance: u8,
    /// Minimum difference percentage to consider significant
    pub significance_threshold: f64,
    /// Whether to generate visual diff images
    pub generate_visual_diffs: bool,
    /// Output directory for test results
    #[cfg(not(feature = "nostd"))]
    pub output_dir: String,
    /// Whether to test animations frame-by-frame
    pub test_animations: bool,
    /// Animation frame step (in centiseconds)
    pub animation_step_cs: u32,
    /// Maximum animation duration to test (in centiseconds)
    pub max_animation_duration_cs: u32,
    /// Whether to measure performance
    pub measure_performance: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            pixel_tolerance: 1,            // Allow 1 pixel value difference
            significance_threshold: 0.001, // 0.1% difference threshold
            generate_visual_diffs: true,
            #[cfg(not(feature = "nostd"))]
            output_dir: "test_output/compatibility".to_string(),
            test_animations: true,
            animation_step_cs: 4,            // 40ms steps (25 FPS)
            max_animation_duration_cs: 3000, // 30 seconds max
            measure_performance: true,
        }
    }
}

impl CompatibilityTester {
    /// Create new compatibility tester
    pub fn new(context: RenderContext, config: TestConfig) -> Result<Self, RenderError> {
        let our_renderer = Renderer::new(crate::backends::BackendType::Software, context.clone())?;
        let libass_renderer = LibassRenderer::new(context.width(), context.height())?;

        Ok(Self {
            our_renderer,
            libass_renderer,
            config,
            #[cfg(not(feature = "nostd"))]
            results_cache: HashMap::new(),
        })
    }

    /// Run comprehensive compatibility test on a script
    pub fn test_script_compatibility(
        &mut self,
        script: &Script,
        test_name: &str,
    ) -> Result<CompatibilityResult, RenderError> {
        // Test static frames first
        let static_result = self.test_static_compatibility(script, test_name)?;

        if !self.config.test_animations {
            return Ok(static_result);
        }

        // Test animations if enabled
        let animation_results = self.test_animation_compatibility(script, test_name)?;

        // Combine results
        Ok(self.combine_test_results(static_result, animation_results))
    }

    /// Test static frame compatibility
    fn test_static_compatibility(
        &mut self,
        script: &Script,
        test_name: &str,
    ) -> Result<CompatibilityResult, RenderError> {
        // Find a representative time to test (middle of first event)
        let test_time = self.find_representative_time(script);

        let start_time = Instant::now();
        let our_frame = self.our_renderer.render_frame(script, test_time)?;
        let our_render_time = start_time.elapsed();

        let start_time = Instant::now();
        let libass_frame = self.libass_renderer.render_frame(script, test_time)?;
        let libass_render_time = start_time.elapsed();

        // Compare frames pixel by pixel
        let comparison = PixelPerfectComparison::compare_frames(
            our_frame.data(),
            libass_frame.data(),
            our_frame.width(),
            our_frame.height(),
        );

        let result = CompatibilityResult {
            test_name: format!("{test_name}_static"),
            passed: comparison.pixel_diff_percentage <= self.config.significance_threshold,
            pixel_diff_percentage: comparison.pixel_diff_percentage,
            max_pixel_diff: comparison.max_pixel_diff,
            diff_regions: self.analyze_diff_regions(&comparison),
            performance_ratio: Some(
                our_render_time.as_nanos() as f64 / libass_render_time.as_nanos() as f64,
            ),
            memory_ratio: None, // TODO: Implement memory measurement
        };

        // Generate visual diff if requested
        #[cfg(all(not(feature = "nostd"), feature = "image"))]
        if self.config.generate_visual_diffs && !result.passed {
            self.generate_visual_diff(&our_frame, &libass_frame, &comparison, test_name)?;
        }

        Ok(result)
    }

    /// Test animation compatibility frame by frame
    fn test_animation_compatibility(
        &mut self,
        script: &Script,
        test_name: &str,
    ) -> Result<Vec<CompatibilityResult>, RenderError> {
        let mut results = Vec::new();
        let (start_time, end_time) = self.find_animation_timerange(script);

        let end_time = end_time.min(start_time + self.config.max_animation_duration_cs);

        for time_cs in (start_time..=end_time).step_by(self.config.animation_step_cs as usize) {
            let frame_result = self.test_frame_at_time(script, time_cs, test_name)?;
            results.push(frame_result);

            // Early exit if we find major differences
            if let Some(last_result) = results.last() {
                if last_result.pixel_diff_percentage > 0.1 {
                    eprintln!("Major difference detected at {time_cs}cs, stopping animation test");
                    break;
                }
            }
        }

        Ok(results)
    }

    /// Test a single frame at specific time
    fn test_frame_at_time(
        &mut self,
        script: &Script,
        time_cs: u32,
        test_name: &str,
    ) -> Result<CompatibilityResult, RenderError> {
        let our_frame = self.our_renderer.render_frame(script, time_cs)?;
        let libass_frame = self.libass_renderer.render_frame(script, time_cs)?;

        let comparison = PixelPerfectComparison::compare_frames(
            our_frame.data(),
            libass_frame.data(),
            our_frame.width(),
            our_frame.height(),
        );

        Ok(CompatibilityResult {
            test_name: format!("{test_name}_frame_{time_cs}"),
            passed: comparison.pixel_diff_percentage <= self.config.significance_threshold,
            pixel_diff_percentage: comparison.pixel_diff_percentage,
            max_pixel_diff: comparison.max_pixel_diff,
            diff_regions: self.analyze_diff_regions(&comparison),
            performance_ratio: None,
            memory_ratio: None,
        })
    }

    /// Analyze difference regions to categorize types of differences
    fn analyze_diff_regions(&self, comparison: &PixelPerfectComparison) -> Vec<DiffRegion> {
        let mut regions = Vec::new();

        // Simple clustering of different pixels into regions
        // TODO: Implement proper connected component analysis

        if let Some(diff_map) = &comparison.difference_map {
            // Find clusters of different pixels
            let clusters = self.find_difference_clusters(diff_map);

            for cluster in clusters {
                let diff_type = self.classify_difference_type(&cluster);
                regions.push(DiffRegion {
                    bounds: cluster.bounds,
                    avg_diff: cluster.avg_diff,
                    max_diff: cluster.max_diff,
                    diff_type,
                });
            }
        }

        regions
    }

    /// Find clusters of different pixels
    fn find_difference_clusters(&self, _diff_map: &[u8]) -> Vec<DiffCluster> {
        // TODO: Implement connected component analysis
        // For now, return empty clusters
        Vec::new()
    }

    /// Classify the type of difference in a cluster
    fn classify_difference_type(&self, _cluster: &DiffCluster) -> DiffType {
        // TODO: Implement heuristics to classify difference types
        // - Text positioning: differences in glyph positions
        // - Font rendering: differences in glyph shapes
        // - Effects: differences in shadows, outlines, etc.
        // - Transform: differences in rotation, scaling
        // - Animation timing: differences that vary with time
        DiffType::PixelValue
    }

    /// Generate visual diff image for debugging
    #[cfg(all(not(feature = "nostd"), feature = "image"))]
    fn generate_visual_diff(
        &self,
        our_frame: &crate::renderer::Frame,
        libass_frame: &crate::renderer::Frame,
        comparison: &PixelPerfectComparison,
        test_name: &str,
    ) -> Result<(), RenderError> {
        // Create output directory
        let output_dir = std::path::Path::new(&self.config.output_dir);
        std::fs::create_dir_all(output_dir)
            .map_err(|e| RenderError::IOError(format!("Failed to create output directory: {e}")))?;

        // Save our frame
        let our_path = output_dir.join(format!("{test_name}_ours.png"));
        self.save_frame_as_image(our_frame, &our_path)?;

        // Save libass frame
        let libass_path = output_dir.join(format!("{test_name}_libass.png"));
        self.save_frame_as_image(libass_frame, &libass_path)?;

        // Generate and save diff visualization
        if let Some(diff_map) = &comparison.difference_map {
            let diff_path = output_dir.join(format!("{test_name}_diff.png"));
            self.save_diff_visualization(
                diff_map,
                our_frame.width(),
                our_frame.height(),
                &diff_path,
            )?;
        }

        Ok(())
    }

    /// Save frame data as PNG image
    #[cfg(all(not(feature = "nostd"), feature = "image"))]
    fn save_frame_as_image(
        &self,
        frame: &crate::renderer::Frame,
        path: &std::path::Path,
    ) -> Result<(), RenderError> {
        use image::{ImageBuffer, RgbImage};

        let width = frame.width();
        let height = frame.height();
        let data = frame.data();

        // Convert RGBA to RGB
        let mut rgb_data = Vec::with_capacity(width as usize * height as usize * 3);
        for chunk in data.chunks(4) {
            rgb_data.push(chunk[0]); // R
            rgb_data.push(chunk[1]); // G
            rgb_data.push(chunk[2]); // B
                                     // Skip alpha channel
        }

        let img: RgbImage = ImageBuffer::from_raw(width, height, rgb_data).ok_or_else(|| {
            RenderError::BackendError("Failed to create image buffer".to_string())
        })?;

        img.save(path)
            .map_err(|e| RenderError::IOError(format!("Failed to save image: {e}")))?;

        Ok(())
    }

    /// Save difference visualization
    #[cfg(all(not(feature = "nostd"), feature = "image"))]
    fn save_diff_visualization(
        &self,
        _diff_map: &[u8],
        width: u32,
        height: u32,
        path: &std::path::Path,
    ) -> Result<(), RenderError> {
        use image::{ImageBuffer, RgbImage};

        // Create heatmap visualization of differences
        let mut rgb_data = Vec::with_capacity(width as usize * height as usize * 3);

        for &diff_value in _diff_map {
            if diff_value == 0 {
                // No difference - black
                rgb_data.extend_from_slice(&[0, 0, 0]);
            } else {
                // Difference - red intensity based on difference magnitude
                rgb_data.extend_from_slice(&[diff_value, 0, 0]);
            }
        }

        let img: RgbImage = ImageBuffer::from_raw(width, height, rgb_data).ok_or_else(|| {
            RenderError::BackendError("Failed to create diff image buffer".to_string())
        })?;

        img.save(path)
            .map_err(|e| RenderError::IOError(format!("Failed to save diff image: {e}")))?;

        Ok(())
    }

    /// Combine multiple test results into a summary
    fn combine_test_results(
        &self,
        static_result: CompatibilityResult,
        animation_results: Vec<CompatibilityResult>,
    ) -> CompatibilityResult {
        let animation_passed = animation_results.iter().all(|r| r.passed);
        let max_animation_diff = animation_results
            .iter()
            .map(|r| r.pixel_diff_percentage)
            .fold(0.0, f64::max);

        CompatibilityResult {
            test_name: static_result.test_name.replace("_static", "_combined"),
            passed: static_result.passed && animation_passed,
            pixel_diff_percentage: static_result.pixel_diff_percentage.max(max_animation_diff),
            max_pixel_diff: static_result.max_pixel_diff.max(
                animation_results
                    .iter()
                    .map(|r| r.max_pixel_diff)
                    .max()
                    .unwrap_or(0),
            ),
            diff_regions: static_result.diff_regions, // TODO: Combine all diff regions
            performance_ratio: static_result.performance_ratio,
            memory_ratio: static_result.memory_ratio,
        }
    }

    /// Find representative time for static testing
    fn find_representative_time(&self, script: &Script) -> u32 {
        // Find middle of first event that has visible text
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

        // Default to 1 second
        100
    }

    /// Find animation time range to test
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
            (0, 1000) // Default 10 second range
        } else {
            (min_time, max_time)
        }
    }
}

/// Cluster of different pixels for analysis
#[derive(Debug)]
struct DiffCluster {
    bounds: (u32, u32, u32, u32),
    avg_diff: f64,
    max_diff: u8,
    #[allow(dead_code)] // Stored for future diff analysis features
    pixel_count: usize,
}

/// Comprehensive test suite runner
pub struct CompatibilityTestSuite {
    tester: CompatibilityTester,
    test_cases: Vec<TestCase>,
}

/// Individual test case
#[derive(Debug, Clone)]
pub struct TestCase {
    pub name: String,
    pub script_content: String,
    pub expected_issues: Vec<DiffType>,
    pub tolerance: Option<f64>,
}

impl CompatibilityTestSuite {
    /// Create new test suite
    pub fn new(context: RenderContext, config: TestConfig) -> Result<Self, RenderError> {
        let tester = CompatibilityTester::new(context, config)?;
        Ok(Self {
            tester,
            test_cases: Vec::new(),
        })
    }

    /// Add test case from ASS content
    pub fn add_test_case(&mut self, name: String, script_content: String) {
        self.test_cases.push(TestCase {
            name,
            script_content,
            expected_issues: Vec::new(),
            tolerance: None,
        });
    }

    /// Run all test cases
    pub fn run_all_tests(&mut self) -> Result<Vec<CompatibilityResult>, RenderError> {
        let mut results = Vec::new();

        for test_case in &self.test_cases {
            eprintln!("Running compatibility test: {}", test_case.name);

            let script = ass_core::parser::Script::parse(&test_case.script_content)
                .map_err(|e| RenderError::ParseError(format!("Failed to parse script: {e}")))?;

            let result = self
                .tester
                .test_script_compatibility(&script, &test_case.name)?;

            eprintln!(
                "Test {} - Passed: {}, Diff: {:.3}%",
                test_case.name,
                result.passed,
                result.pixel_diff_percentage * 100.0
            );

            results.push(result);
        }

        Ok(results)
    }
}
