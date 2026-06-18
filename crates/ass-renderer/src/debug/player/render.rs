use super::PlayerFrame;
use crate::RenderError;
use ass_core::parser::Script;

#[cfg(not(feature = "nostd"))]
use std::time::{Duration, Instant};

#[cfg(feature = "nostd")]
use alloc::vec::Vec;

impl super::DebugPlayer {
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
                    println!("📦 Using cached frame for {cache_key}ms");
                }

                return Ok(player_frame);
            }
        }

        let script_content = self
            .script_content
            .as_ref()
            .ok_or_else(|| RenderError::InvalidInput("No script loaded".into()))?;

        let script = Script::parse(script_content)
            .map_err(|e| RenderError::ParseError(format!("Failed to parse script: {e:?}")))?;

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
        println!("│ Visible pixels: {non_transparent:6}            │");
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
                .map_err(|e| RenderError::BackendError(format!("Failed to save frame: {e}")))?;
        }

        Ok(())
    }
}
