#[cfg(not(feature = "nostd"))]
use std::time::Instant;

impl super::DebugPlayer {
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
        println!(
            "▶️  Playback started at {current_time}ms",
            current_time = self.current_time_ms
        );
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
        println!(
            "⏸️  Playback paused at {current_time}ms",
            current_time = self.current_time_ms
        );
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
        println!(
            "⏩ Seeked to {current_time}ms",
            current_time = self.current_time_ms
        );
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
        self.playback_speed = speed.clamp(0.1, 10.0);
        println!("🎚️  Playback speed: {speed}x", speed = self.playback_speed);
    }

    pub fn step_forward(&mut self) {
        self.current_time_ms =
            (self.current_time_ms + self.frame_interval_ms).min(self.end_time_ms);
        if self.is_playing {
            self.playback_start_instant = Some(Instant::now());
            self.playback_start_time_ms = self.current_time_ms;
            self.accumulated_time_ms = 0.0;
        }
        println!(
            "⏭️  Step forward to {current_time}ms",
            current_time = self.current_time_ms
        );
    }

    pub fn step_backward(&mut self) {
        self.current_time_ms = self.current_time_ms.saturating_sub(self.frame_interval_ms);
        if self.is_playing {
            self.playback_start_instant = Some(Instant::now());
            self.playback_start_time_ms = self.current_time_ms;
            self.accumulated_time_ms = 0.0;
        }
        println!(
            "⏮️  Step backward to {current_time}ms",
            current_time = self.current_time_ms
        );
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
            println!(
                "💾 Frame saving: ON (to {output_dir})",
                output_dir = self.output_dir
            );
        } else {
            println!("💾 Frame saving: OFF");
        }
    }

    pub fn set_loop(&mut self, enable: bool) {
        self.loop_playback = enable;
        println!(
            "🔁 Loop playback: {status}",
            status = if enable { "ON" } else { "OFF" }
        );
    }

    pub fn set_output_dir(&mut self, dir: &str) {
        self.output_dir = dir.to_string();
        println!("📁 Output directory set to: {dir}");
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    pub fn current_time(&self) -> u32 {
        self.current_time_ms
    }

    pub fn toggle_loop(&mut self) {
        self.loop_playback = !self.loop_playback;
        let loop_status = if self.loop_playback { "ON" } else { "OFF" };
        println!("🔁 Loop: {loop_status}");
    }

    pub fn increase_speed(&mut self) {
        let new_speed = (self.playback_speed * 1.5).min(10.0);
        self.set_speed(new_speed);
    }

    pub fn decrease_speed(&mut self) {
        let new_speed = (self.playback_speed / 1.5).max(0.1);
        self.set_speed(new_speed);
    }
}
