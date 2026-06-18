//! Animation timing, interpolation kinds, and `\t` tag representation

#[cfg(feature = "nostd")]
use alloc::{string::String, vec::Vec};
#[cfg(not(feature = "nostd"))]
use std::{string::String, vec::Vec};

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
