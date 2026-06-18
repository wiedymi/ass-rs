//! Per-property animation tracks and their interpolation helpers

#[cfg(feature = "nostd")]
use alloc::string::String;
#[cfg(not(feature = "nostd"))]
use std::string::String;

use super::timing::{AnimationInterpolation, AnimationTiming};
use super::value::{AnimatedResult, AnimatedValue};

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

/// Smooth step interpolation (ease-in-out)
fn smooth_step(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Smoother step interpolation (smoother ease-in-out)
fn smoother_step(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}
