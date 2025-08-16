//! Animation system for ASS subtitle effects

#[cfg(feature = "nostd")]
use alloc::{string::{String, ToString}, vec::Vec};
#[cfg(not(feature = "nostd"))]
use std::{string::String, vec::Vec};

use crate::utils::RenderError;
use smallvec::SmallVec;

/// Animation interpolation types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationInterpolation {
    /// Linear interpolation
    Linear,
    /// Smooth step (ease-in-out)
    Smooth,
    /// Smoother step (smoother ease-in-out)
    Smoother,
}

/// Animation tag (\t) representation
#[derive(Debug, Clone)]
pub struct AnimationTag {
    /// Start time offset in centiseconds
    pub t1: Option<u32>,
    /// End time offset in centiseconds
    pub t2: Option<u32>,
    /// Acceleration factor
    pub accel: Option<f32>,
    /// Style modifiers to animate
    pub modifiers: Vec<String>,
}

/// Interpolation function type
pub type InterpolationFn = fn(f32) -> f32;

/// Animation timing parameters
#[derive(Debug, Clone)]
pub struct AnimationTiming {
    /// Start time in centiseconds
    pub start_cs: u32,
    /// End time in centiseconds
    pub end_cs: u32,
    /// Acceleration factor (default 1.0)
    pub accel: f32,
}

impl AnimationTiming {
    /// Create new animation timing
    pub fn new(start_cs: u32, end_cs: u32, accel: f32) -> Self {
        Self {
            start_cs,
            end_cs,
            accel,
        }
    }

    /// Calculate progress at given time (0.0 to 1.0)
    pub fn progress(&self, time_cs: u32) -> f32 {
        if time_cs <= self.start_cs {
            return 0.0;
        }
        if time_cs >= self.end_cs {
            return 1.0;
        }

        let duration = (self.end_cs - self.start_cs) as f32;
        let elapsed = (time_cs - self.start_cs) as f32;
        let linear_progress = elapsed / duration;

        // Apply acceleration
        if (self.accel - 1.0).abs() < 0.001 {
            linear_progress
        } else {
            linear_progress.powf(self.accel)
        }
    }
}

/// Animated property value
#[derive(Debug, Clone)]
pub enum AnimatedValue {
    /// Integer value animation
    Integer { from: i32, to: i32 },
    /// Float value animation
    Float { from: f32, to: f32 },
    /// Color animation (RGBA)
    Color { from: [u8; 4], to: [u8; 4] },
    /// Position animation
    Position { from: (f32, f32), to: (f32, f32) },
    /// Scale animation
    Scale { from: (f32, f32), to: (f32, f32) },
}

impl AnimatedValue {
    /// Interpolate value at given progress
    pub fn interpolate(&self, progress: f32) -> AnimatedResult {
        let t = progress.clamp(0.0, 1.0);

        match self {
            Self::Integer { from, to } => {
                let value = *from + ((to - from) as f32 * t) as i32;
                AnimatedResult::Integer(value)
            }
            Self::Float { from, to } => {
                let value = from + (to - from) * t;
                AnimatedResult::Float(value)
            }
            Self::Color { from, to } => {
                let r = from[0] as f32 + (to[0] as f32 - from[0] as f32) * t;
                let g = from[1] as f32 + (to[1] as f32 - from[1] as f32) * t;
                let b = from[2] as f32 + (to[2] as f32 - from[2] as f32) * t;
                let a = from[3] as f32 + (to[3] as f32 - from[3] as f32) * t;
                AnimatedResult::Color([r as u8, g as u8, b as u8, a as u8])
            }
            Self::Position { from, to } => {
                let x = from.0 + (to.0 - from.0) * t;
                let y = from.1 + (to.1 - from.1) * t;
                AnimatedResult::Position((x, y))
            }
            Self::Scale { from, to } => {
                let x = from.0 + (to.0 - from.0) * t;
                let y = from.1 + (to.1 - from.1) * t;
                AnimatedResult::Scale((x, y))
            }
        }
    }
}

/// Result of animation interpolation
#[derive(Debug, Clone)]
pub enum AnimatedResult {
    Integer(i32),
    Float(f32),
    Color([u8; 4]),
    Position((f32, f32)),
    Scale((f32, f32)),
}

/// Animation track for a single property
#[derive(Debug, Clone)]
pub struct AnimationTrack {
    /// Property name being animated
    pub property: String,
    /// Animation timing
    pub timing: AnimationTiming,
    /// Animated value
    pub value: AnimatedValue,
    /// Interpolation type
    pub interpolation: AnimationInterpolation,
}

impl AnimationTrack {
    /// Create a new animation track
    pub fn new(
        property: String,
        timing: AnimationTiming,
        value: AnimatedValue,
        interpolation: AnimationInterpolation,
    ) -> Self {
        Self {
            property,
            timing,
            value,
            interpolation,
        }
    }

    /// Evaluate animation at given time
    pub fn evaluate(&self, time_cs: u32) -> AnimatedResult {
        let progress = self.timing.progress(time_cs);
        let interpolated_progress = self.apply_interpolation(progress);
        self.value.interpolate(interpolated_progress)
    }

    /// Apply interpolation function to progress
    fn apply_interpolation(&self, progress: f32) -> f32 {
        match self.interpolation {
            AnimationInterpolation::Linear => progress,
            AnimationInterpolation::Smooth => smooth_step(progress),
            AnimationInterpolation::Smoother => smoother_step(progress),
        }
    }
}

/// Animation controller for managing multiple tracks
pub struct AnimationController {
    tracks: SmallVec<[AnimationTrack; 8]>,
}

impl AnimationController {
    /// Create a new animation controller
    pub fn new() -> Self {
        Self {
            tracks: SmallVec::new(),
        }
    }

    /// Add an animation track
    pub fn add_track(&mut self, track: AnimationTrack) {
        self.tracks.push(track);
    }

    /// Parse and add \t tag animations
    pub fn add_from_tag(
        &mut self,
        tag: &AnimationTag,
        event_start: u32,
        event_end: u32,
    ) -> Result<(), RenderError> {
        let timing = AnimationTiming::new(
            event_start + tag.t1.unwrap_or(0),
            event_start + tag.t2.unwrap_or(event_end - event_start),
            tag.accel.unwrap_or(1.0),
        );

        // Parse animated properties from tag modifiers
        for modifier in &tag.modifiers {
            let track = self.parse_modifier_animation(modifier, timing.clone())?;
            if let Some(track) = track {
                self.tracks.push(track);
            }
        }

        Ok(())
    }

    /// Parse a modifier into an animation track
    fn parse_modifier_animation(
        &self,
        modifier: &str,
        timing: AnimationTiming,
    ) -> Result<Option<AnimationTrack>, RenderError> {
        // Parse modifier format: \property(from,to) or \property(to)
        // Examples: \fscx(100,200), \c(&H0000FF&,&HFF0000&), \pos(0,0,100,100)

        // Simplified parsing - in production would use proper parser
        if modifier.starts_with("\\fscx") {
            // Font scale X animation
            if let Some(values) = extract_values(modifier) {
                if values.len() == 2 {
                    let from = values[0].parse::<f32>().unwrap_or(100.0);
                    let to = values[1].parse::<f32>().unwrap_or(100.0);
                    return Ok(Some(AnimationTrack::new(
                        "fscx".to_string(),
                        timing,
                        AnimatedValue::Float { from, to },
                        AnimationInterpolation::Linear,
                    )));
                }
            }
        } else if modifier.starts_with("\\fscy") {
            // Font scale Y animation
            if let Some(values) = extract_values(modifier) {
                if values.len() == 2 {
                    let from = values[0].parse::<f32>().unwrap_or(100.0);
                    let to = values[1].parse::<f32>().unwrap_or(100.0);
                    return Ok(Some(AnimationTrack::new(
                        "fscy".to_string(),
                        timing,
                        AnimatedValue::Float { from, to },
                        AnimationInterpolation::Linear,
                    )));
                }
            }
        } else if modifier.starts_with("\\fs") {
            // Font size animation
            if let Some(values) = extract_values(modifier) {
                if values.len() == 2 {
                    let from = values[0].parse::<f32>().unwrap_or(20.0);
                    let to = values[1].parse::<f32>().unwrap_or(20.0);
                    return Ok(Some(AnimationTrack::new(
                        "fs".to_string(),
                        timing,
                        AnimatedValue::Float { from, to },
                        AnimationInterpolation::Linear,
                    )));
                }
            }
        } else if modifier.starts_with("\\frz") || modifier.starts_with("\\fr") {
            // Rotation animation
            if let Some(values) = extract_values(modifier) {
                if values.len() == 2 {
                    let from = values[0].parse::<f32>().unwrap_or(0.0);
                    let to = values[1].parse::<f32>().unwrap_or(0.0);
                    return Ok(Some(AnimationTrack::new(
                        "frz".to_string(),
                        timing,
                        AnimatedValue::Float { from, to },
                        AnimationInterpolation::Linear,
                    )));
                }
            }
        } else if modifier.starts_with("\\c") || modifier.starts_with("\\1c") {
            // Color animation
            if let Some(values) = extract_values(modifier) {
                if values.len() == 2 {
                    let from = parse_color(values[0]).unwrap_or([255, 255, 255, 255]);
                    let to = parse_color(values[1]).unwrap_or([255, 255, 255, 255]);
                    return Ok(Some(AnimationTrack::new(
                        "c".to_string(),
                        timing,
                        AnimatedValue::Color { from, to },
                        AnimationInterpolation::Linear,
                    )));
                }
            }
        } else if modifier.starts_with("\\alpha") {
            // Alpha animation
            if let Some(values) = extract_values(modifier) {
                if values.len() == 2 {
                    let from = parse_alpha(values[0]).unwrap_or(255) as i32;
                    let to = parse_alpha(values[1]).unwrap_or(255) as i32;
                    return Ok(Some(AnimationTrack::new(
                        "alpha".to_string(),
                        timing,
                        AnimatedValue::Integer { from, to },
                        AnimationInterpolation::Linear,
                    )));
                }
            }
        }

        Ok(None)
    }

    /// Evaluate all animations at given time
    pub fn evaluate(&self, time_cs: u32) -> AnimationState {
        let mut state = AnimationState::new();

        for track in &self.tracks {
            let result = track.evaluate(time_cs);
            state.set_property(&track.property, result);
        }

        state
    }

    /// Check if any animations are active at given time
    pub fn is_active(&self, time_cs: u32) -> bool {
        self.tracks
            .iter()
            .any(|track| time_cs >= track.timing.start_cs && time_cs <= track.timing.end_cs)
    }

    /// Get all active tracks at given time
    pub fn active_tracks(&self, time_cs: u32) -> SmallVec<[&AnimationTrack; 8]> {
        self.tracks
            .iter()
            .filter(|track| time_cs >= track.timing.start_cs && time_cs <= track.timing.end_cs)
            .collect()
    }
}

impl Default for AnimationController {
    fn default() -> Self {
        Self::new()
    }
}

/// Current animation state
#[derive(Debug, Clone)]
pub struct AnimationState {
    properties: SmallVec<[(String, AnimatedResult); 16]>,
}

impl AnimationState {
    /// Create new animation state
    pub fn new() -> Self {
        Self {
            properties: SmallVec::new(),
        }
    }

    /// Set a property value
    pub fn set_property(&mut self, name: &str, value: AnimatedResult) {
        if let Some(entry) = self.properties.iter_mut().find(|(n, _)| n == name) {
            entry.1 = value;
        } else {
            self.properties.push((name.to_string(), value));
        }
    }

    /// Get a property value
    pub fn get_property(&self, name: &str) -> Option<&AnimatedResult> {
        self.properties
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v)
    }

    /// Get font size if animated
    pub fn font_size(&self) -> Option<f32> {
        self.get_property("fs").and_then(|v| {
            if let AnimatedResult::Float(size) = v {
                Some(*size)
            } else {
                None
            }
        })
    }

    /// Get scale if animated
    pub fn scale(&self) -> Option<(f32, f32)> {
        let scale_x = self.get_property("fscx").and_then(|v| {
            if let AnimatedResult::Float(scale) = v {
                Some(*scale)
            } else {
                None
            }
        });

        let scale_y = self.get_property("fscy").and_then(|v| {
            if let AnimatedResult::Float(scale) = v {
                Some(*scale)
            } else {
                None
            }
        });

        match (scale_x, scale_y) {
            (Some(x), Some(y)) => Some((x, y)),
            (Some(x), None) => Some((x, 100.0)),
            (None, Some(y)) => Some((100.0, y)),
            _ => None,
        }
    }

    /// Get rotation if animated
    pub fn rotation(&self) -> Option<f32> {
        self.get_property("frz").and_then(|v| {
            if let AnimatedResult::Float(rotation) = v {
                Some(*rotation)
            } else {
                None
            }
        })
    }

    /// Get color if animated
    pub fn color(&self) -> Option<[u8; 4]> {
        self.get_property("c").and_then(|v| {
            if let AnimatedResult::Color(color) = v {
                Some(*color)
            } else {
                None
            }
        })
    }
}

impl Default for AnimationState {
    fn default() -> Self {
        Self::new()
    }
}

// Helper functions

/// Smooth step interpolation (ease-in-out)
fn smooth_step(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Smoother step interpolation (smoother ease-in-out)
fn smoother_step(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Extract values from a modifier string
fn extract_values(modifier: &str) -> Option<Vec<&str>> {
    // Find content between parentheses
    let start = modifier.find('(')?;
    let end = modifier.find(')')?;
    let content = &modifier[start + 1..end];
    Some(content.split(',').collect())
}

/// Parse ASS color format
fn parse_color(color_str: &str) -> Option<[u8; 4]> {
    // Parse &HBBGGRR& or &HAABBGGRR& format
    let cleaned = color_str.trim_start_matches("&H").trim_end_matches('&');

    if cleaned.len() == 6 {
        // BGR format
        let bgr = u32::from_str_radix(cleaned, 16).ok()?;
        let b = ((bgr >> 16) & 0xFF) as u8;
        let g = ((bgr >> 8) & 0xFF) as u8;
        let r = (bgr & 0xFF) as u8;
        Some([r, g, b, 255])
    } else if cleaned.len() == 8 {
        // ABGR format
        let abgr = u32::from_str_radix(cleaned, 16).ok()?;
        let a = ((abgr >> 24) & 0xFF) as u8;
        let b = ((abgr >> 16) & 0xFF) as u8;
        let g = ((abgr >> 8) & 0xFF) as u8;
        let r = (abgr & 0xFF) as u8;
        Some([r, g, b, 255 - a]) // ASS uses inverse alpha
    } else {
        None
    }
}

/// Parse ASS alpha format
fn parse_alpha(alpha_str: &str) -> Option<u8> {
    // Parse &HXX& format
    let cleaned = alpha_str.trim_start_matches("&H").trim_end_matches('&');
    let alpha = u8::from_str_radix(cleaned, 16).ok()?;
    Some(255 - alpha) // ASS uses inverse alpha
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_timing() {
        let timing = AnimationTiming::new(100, 200, 1.0);

        assert_eq!(timing.progress(50), 0.0);
        assert_eq!(timing.progress(100), 0.0);
        assert_eq!(timing.progress(150), 0.5);
        assert_eq!(timing.progress(200), 1.0);
        assert_eq!(timing.progress(250), 1.0);
    }

    #[test]
    fn test_animated_value_interpolation() {
        let int_anim = AnimatedValue::Integer { from: 0, to: 100 };
        if let AnimatedResult::Integer(val) = int_anim.interpolate(0.5) {
            assert_eq!(val, 50);
        } else {
            panic!("Wrong result type");
        }

        let color_anim = AnimatedValue::Color {
            from: [0, 0, 0, 255],
            to: [255, 255, 255, 255],
        };
        if let AnimatedResult::Color(val) = color_anim.interpolate(0.5) {
            assert_eq!(val, [127, 127, 127, 255]);
        } else {
            panic!("Wrong result type");
        }
    }

    #[test]
    fn test_animation_controller() {
        let mut controller = AnimationController::new();

        let timing = AnimationTiming::new(0, 100, 1.0);
        let track = AnimationTrack::new(
            "test".to_string(),
            timing,
            AnimatedValue::Float {
                from: 0.0,
                to: 100.0,
            },
            AnimationInterpolation::Linear,
        );

        controller.add_track(track);

        let state = controller.evaluate(50);
        if let Some(AnimatedResult::Float(val)) = state.get_property("test") {
            assert!((val - 50.0).abs() < 0.001);
        } else {
            panic!("Property not found or wrong type");
        }
    }
}
