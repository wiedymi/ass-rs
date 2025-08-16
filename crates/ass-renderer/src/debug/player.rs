use crate::{BackendType, Frame, RenderContext, RenderError, Renderer};
use ass_core::parser::Script;

#[cfg(not(feature = "nostd"))]
use std::collections::HashMap;
#[cfg(not(feature = "nostd"))]
use std::path::Path;
#[cfg(not(feature = "nostd"))]
use std::time::{Duration, Instant};

#[cfg(feature = "nostd")]
use alloc::collections::BTreeMap as HashMap;
#[cfg(feature = "nostd")]
use alloc::string::String;
#[cfg(feature = "nostd")]
use alloc::vec::Vec;

/// Debug player for visual verification of subtitle rendering
pub struct DebugPlayer {
    renderer: Renderer,
    script_content: Option<String>,
    parsed_script: Option<Script<'static>>,
    current_time_ms: u32,
    playback_speed: f32,
    is_playing: bool,
    frame_interval_ms: u32,
    output_dir: String,
    save_frames: bool,
    show_stats: bool,
    loop_playback: bool,
    start_time_ms: u32,
    end_time_ms: u32,
    frame_cache: HashMap<u32, Frame>,
    cache_enabled: bool,
    max_cache_size: usize,
    // Timing fields for proper synchronization
    playback_start_instant: Option<Instant>,
    playback_start_time_ms: u32,
    accumulated_time_ms: f32,
    width: u32,
    height: u32,
}

impl DebugPlayer {
    pub fn new(backend_type: BackendType, width: u32, height: u32) -> Result<Self, RenderError> {
        let mut context = RenderContext::new(width, height);
        context.font_database_mut().load_system_fonts();

        let renderer = Renderer::new(backend_type, context)?;

        Ok(Self {
            renderer,
            script_content: None,
            parsed_script: None,
            current_time_ms: 0,
            playback_speed: 1.0,
            is_playing: false,
            frame_interval_ms: 40, // ~25 FPS for subtitle granularity
            output_dir: "debug_player_output".to_string(),
            save_frames: false,
            show_stats: true,
            loop_playback: false,
            start_time_ms: 0,
            end_time_ms: 0, // Default to 0, will be set when script is loaded
            frame_cache: HashMap::new(),
            cache_enabled: true,
            max_cache_size: 100, // Cache up to 100 frames
            playback_start_instant: None,
            playback_start_time_ms: 0,
            accumulated_time_ms: 0.0,
            width,
            height,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// Check if a script is currently loaded
    pub fn has_script(&self) -> bool {
        self.script_content.is_some()
    }

    /// Get the current end time in milliseconds
    pub fn duration_ms(&self) -> u32 {
        self.end_time_ms
    }

    pub fn load_script(&mut self, script_content: &str) -> Result<(), RenderError> {
        // Parse and store the script to avoid re-parsing on every frame
        let owned_content = script_content.to_string();
        let script = Script::parse(&owned_content)
            .map_err(|e| RenderError::ParseError(format!("Failed to parse script: {:?}", e)))?;

        // Calculate actual end time from events
        let mut max_time = 0u32;
        let mut event_count = 0;
        for section in script.sections() {
            if let ass_core::parser::Section::Events(events) = section {
                for event in events.iter() {
                    event_count += 1;
                    if let Ok(end) = event.end_time_cs() {
                        max_time = max_time.max(end * 10); // Convert centiseconds to milliseconds
                    }
                }
            }
        }

        if max_time > 0 {
            self.end_time_ms = max_time;
        } else {
            // If no events found or all events have no end time,
            // set a reasonable default duration (10 seconds)
            self.end_time_ms = 10000;
        }

        // Clear cache when loading new script
        self.frame_cache.clear();

        // Store both the content and parsed script
        self.script_content = Some(owned_content);
        // Note: We can't store the parsed script due to lifetime issues
        // We'll need to parse on demand but cache rendered frames

        println!(
            "Script loaded. Duration: {}ms, Events: {}",
            self.end_time_ms, event_count
        );

        Ok(())
    }

    pub fn set_cache_enabled(&mut self, enabled: bool) {
        self.cache_enabled = enabled;
        if !enabled {
            self.frame_cache.clear();
        }
    }

    pub fn set_max_cache_size(&mut self, size: usize) {
        self.max_cache_size = size;
        // Trim cache if needed
        while self.frame_cache.len() > size {
            // Remove oldest entries (not optimal but simple)
            if let Some(&min_key) = self.frame_cache.keys().min() {
                self.frame_cache.remove(&min_key);
            }
        }
    }

    pub fn play(&mut self) {
        self.is_playing = true;
        self.playback_start_instant = Some(Instant::now());
        self.playback_start_time_ms = self.current_time_ms;
        self.accumulated_time_ms = 0.0;
        println!("▶️  Playback started at {}ms", self.current_time_ms);
    }

    pub fn pause(&mut self) {
        if self.is_playing {
            // Save accumulated time
            if let Some(start) = self.playback_start_instant {
                let elapsed = start.elapsed().as_secs_f32() * 1000.0 * self.playback_speed;
                self.accumulated_time_ms += elapsed;
            }
        }
        self.is_playing = false;
        self.playback_start_instant = None;
        println!("⏸️  Playback paused at {}ms", self.current_time_ms);
    }

    pub fn stop(&mut self) {
        self.is_playing = false;
        self.current_time_ms = self.start_time_ms;
        self.playback_start_instant = None;
        self.accumulated_time_ms = 0.0;
        println!("⏹️  Playback stopped");
    }

    pub fn seek(&mut self, time_ms: u32) {
        self.current_time_ms = time_ms.min(self.end_time_ms);
        if self.is_playing {
            self.playback_start_instant = Some(Instant::now());
            self.playback_start_time_ms = self.current_time_ms;
            self.accumulated_time_ms = 0.0;
        }
        println!("⏩ Seeked to {}ms", self.current_time_ms);
    }

    pub fn set_speed(&mut self, speed: f32) {
        // If playing, update the accumulated time before changing speed
        if self.is_playing {
            if let Some(start) = self.playback_start_instant {
                let elapsed = start.elapsed().as_secs_f32() * 1000.0 * self.playback_speed;
                self.accumulated_time_ms += elapsed;
                self.playback_start_instant = Some(Instant::now());
                self.playback_start_time_ms = self.current_time_ms;
            }
        }
        self.playback_speed = speed.max(0.1).min(10.0);
        println!("🎚️  Playback speed: {}x", self.playback_speed);
    }

    pub fn step_forward(&mut self) {
        self.current_time_ms =
            (self.current_time_ms + self.frame_interval_ms).min(self.end_time_ms);
        if self.is_playing {
            self.playback_start_instant = Some(Instant::now());
            self.playback_start_time_ms = self.current_time_ms;
            self.accumulated_time_ms = 0.0;
        }
        println!("⏭️  Step forward to {}ms", self.current_time_ms);
    }

    pub fn step_backward(&mut self) {
        self.current_time_ms = self.current_time_ms.saturating_sub(self.frame_interval_ms);
        if self.is_playing {
            self.playback_start_instant = Some(Instant::now());
            self.playback_start_time_ms = self.current_time_ms;
            self.accumulated_time_ms = 0.0;
        }
        println!("⏮️  Step backward to {}ms", self.current_time_ms);
    }

    pub fn toggle_stats(&mut self) {
        self.show_stats = !self.show_stats;
        println!(
            "📊 Stats display: {}",
            if self.show_stats { "ON" } else { "OFF" }
        );
    }

    pub fn toggle_frame_saving(&mut self) {
        self.save_frames = !self.save_frames;
        if self.save_frames {
            #[cfg(not(feature = "nostd"))]
            std::fs::create_dir_all(&self.output_dir).ok();
            println!("💾 Frame saving: ON (to {})", self.output_dir);
        } else {
            println!("💾 Frame saving: OFF");
        }
    }

    pub fn set_loop(&mut self, enable: bool) {
        self.loop_playback = enable;
        println!("🔁 Loop playback: {}", if enable { "ON" } else { "OFF" });
    }

    pub fn set_output_dir(&mut self, dir: &str) {
        self.output_dir = dir.to_string();
        println!("📁 Output directory set to: {}", dir);
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    pub fn current_time(&self) -> u32 {
        self.current_time_ms
    }

    pub fn toggle_loop(&mut self) {
        self.loop_playback = !self.loop_playback;
        println!("🔁 Loop: {}", if self.loop_playback { "ON" } else { "OFF" });
    }

    pub fn increase_speed(&mut self) {
        let new_speed = (self.playback_speed * 1.5).min(10.0);
        self.set_speed(new_speed);
    }

    pub fn decrease_speed(&mut self) {
        let new_speed = (self.playback_speed / 1.5).max(0.1);
        self.set_speed(new_speed);
    }

    pub fn render_current_frame(&mut self) -> Result<PlayerFrame, RenderError> {
        // Check cache first if enabled
        if self.cache_enabled {
            // Round to nearest frame interval for better cache hits
            let cache_key =
                (self.current_time_ms / self.frame_interval_ms) * self.frame_interval_ms;

            if let Some(cached_frame) = self.frame_cache.get(&cache_key) {
                let player_frame = PlayerFrame {
                    frame: cached_frame.clone(),
                    timestamp_ms: self.current_time_ms,
                    render_time: Duration::from_micros(100), // Cached, so very fast
                    frame_number: (self.current_time_ms / self.frame_interval_ms),
                };

                if self.show_stats {
                    println!("📦 Using cached frame for {}ms", cache_key);
                }

                return Ok(player_frame);
            }
        }

        let script_content = self
            .script_content
            .as_ref()
            .ok_or_else(|| RenderError::InvalidInput("No script loaded".into()))?;

        let script = Script::parse(script_content)
            .map_err(|e| RenderError::ParseError(format!("Failed to parse script: {:?}", e)))?;

        let start = Instant::now();
        // Convert milliseconds to centiseconds for the renderer
        let time_cs = self.current_time_ms / 10;
        let frame = self.renderer.render_frame(&script, time_cs)?;
        let render_time = start.elapsed();

        // Cache the frame if enabled
        if self.cache_enabled {
            let cache_key =
                (self.current_time_ms / self.frame_interval_ms) * self.frame_interval_ms;

            // Maintain cache size limit
            if self.frame_cache.len() >= self.max_cache_size {
                // Remove oldest entry
                if let Some(&min_key) = self.frame_cache.keys().min() {
                    self.frame_cache.remove(&min_key);
                }
            }

            self.frame_cache.insert(cache_key, frame.clone());
        }

        let player_frame = PlayerFrame {
            frame: frame.clone(),
            timestamp_ms: self.current_time_ms,
            render_time,
            frame_number: (self.current_time_ms / self.frame_interval_ms),
        };

        if self.show_stats {
            self.print_frame_stats(&player_frame);
        }

        if self.save_frames {
            self.save_frame(&player_frame)?;
        }

        Ok(player_frame)
    }

    pub fn update(&mut self, _delta_time: Duration) -> Result<Option<PlayerFrame>, RenderError> {
        if !self.is_playing {
            return Ok(None);
        }

        // Calculate the actual timeline position based on real elapsed time
        if let Some(start_instant) = self.playback_start_instant {
            let elapsed_ms = start_instant.elapsed().as_secs_f32() * 1000.0 * self.playback_speed;
            self.current_time_ms =
                self.playback_start_time_ms + (elapsed_ms + self.accumulated_time_ms) as u32;

            if self.current_time_ms >= self.end_time_ms {
                if self.loop_playback {
                    self.current_time_ms = self.start_time_ms;
                    self.playback_start_instant = Some(Instant::now());
                    self.playback_start_time_ms = self.start_time_ms;
                    self.accumulated_time_ms = 0.0;
                    println!("🔁 Looping playback");
                } else {
                    self.is_playing = false;
                    self.current_time_ms = self.end_time_ms;
                    self.playback_start_instant = None;
                    println!("✅ Playback finished");
                    return Ok(None);
                }
            }
        }

        self.render_current_frame().map(Some)
    }

    fn print_frame_stats(&self, player_frame: &PlayerFrame) {
        let pixels = player_frame.frame.pixels();
        let mut non_transparent = 0;

        for chunk in pixels.chunks(4) {
            if chunk.len() == 4 && chunk[3] > 0 {
                non_transparent += 1;
            }
        }

        println!("┌────────────────────────────────────┐");
        println!(
            "│ Frame #{:06} @ {:02}:{:02}.{:03}        │",
            player_frame.frame_number,
            self.current_time_ms / 60000,
            (self.current_time_ms / 1000) % 60,
            self.current_time_ms % 1000
        );
        println!("├────────────────────────────────────┤");
        println!(
            "│ Render: {:.2}ms                    │",
            player_frame.render_time.as_secs_f64() * 1000.0
        );
        println!("│ Visible pixels: {:6}            │", non_transparent);
        println!(
            "│ Speed: {:.1}x | Progress: {:3.1}%    │",
            self.playback_speed,
            (self.current_time_ms as f32 / self.end_time_ms as f32) * 100.0
        );
        println!("└────────────────────────────────────┘");
    }

    fn save_frame(&self, player_frame: &PlayerFrame) -> Result<(), RenderError> {
        let path = format!(
            "{}/frame_{:06}_{:06}ms.png",
            self.output_dir, player_frame.frame_number, player_frame.timestamp_ms
        );

        #[cfg(feature = "image")]
        {
            use image::{ImageBuffer, Rgba};

            let img = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(
                player_frame.frame.width(),
                player_frame.frame.height(),
                player_frame.frame.pixels().to_vec(),
            )
            .ok_or_else(|| RenderError::BackendError("Failed to create image buffer".into()))?;

            img.save(&path)
                .map_err(|e| RenderError::BackendError(format!("Failed to save frame: {}", e)))?;
        }

        Ok(())
    }

    pub fn run_automatic_test(&mut self, test_points: Vec<u32>) -> Result<TestReport, RenderError> {
        let script_content = self
            .script_content
            .as_ref()
            .ok_or_else(|| RenderError::InvalidInput("No script loaded".into()))?;

        let script = Script::parse(script_content)
            .map_err(|e| RenderError::ParseError(format!("Failed to parse script: {:?}", e)))?;

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
            println!("  Testing at {}ms...", time_ms);

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
        println!("  • Test points: {}", self.test_points.len());
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

        println!("  • Fast (<5ms): {}", fast);
        println!("  • Normal (5-15ms): {}", normal);
        println!("  • Slow (>15ms): {}", slow);

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
