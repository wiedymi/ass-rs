//! Animation controller that drives multiple tracks and `\t` tag parsing

#[cfg(feature = "nostd")]
use alloc::{string::ToString, vec::Vec};
#[cfg(not(feature = "nostd"))]
use std::vec::Vec;

use crate::utils::RenderError;
use smallvec::SmallVec;

use super::state::AnimationState;
use super::timing::{AnimationInterpolation, AnimationTag, AnimationTiming};
use super::track::AnimationTrack;
use super::value::AnimatedValue;

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

// Helper functions

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
