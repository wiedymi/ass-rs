use crate::{BackendType, Frame, RenderContext, RenderError, Renderer};
use ass_core::parser::Script;

#[cfg(not(feature = "nostd"))]
use std::collections::HashMap;
#[cfg(not(feature = "nostd"))]
use std::time::Instant;

#[cfg(feature = "nostd")]
use alloc::collections::BTreeMap as HashMap;
#[cfg(feature = "nostd")]
use alloc::string::String;

mod playback;
mod render;
mod report;
mod testing;

pub use report::{PlayerFrame, TestPoint, TestReport};

/// Debug player for visual verification of subtitle rendering
pub struct DebugPlayer {
    renderer: Renderer,
    script_content: Option<String>,
    #[allow(dead_code)] // Debug feature - may be used for future player functionality
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
            .map_err(|e| RenderError::ParseError(format!("Failed to parse script: {e:?}")))?;

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
            "Script loaded. Duration: {duration}ms, Events: {events}",
            duration = self.end_time_ms,
            events = event_count
        );

        Ok(())
    }
}
