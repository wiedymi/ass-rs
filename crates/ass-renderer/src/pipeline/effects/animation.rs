//! Animation effect for interpolating values over time

use crate::pipeline::effects::Effect;
use crate::utils::{cubic_bezier, lerp, RenderError};

/// Animation interpolation type
#[derive(Debug, Clone, Copy)]
pub enum InterpolationType {
    /// Linear interpolation
    Linear,
    /// Ease-in (slow start)
    EaseIn,
    /// Ease-out (slow end)
    EaseOut,
    /// Ease-in-out (slow start and end)
    EaseInOut,
    /// Custom cubic bezier
    CubicBezier { p1: f32, p2: f32 },
}

impl InterpolationType {
    /// Calculate interpolation value
    pub fn interpolate(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);

        match self {
            Self::Linear => t,
            Self::EaseIn => t * t,
            Self::EaseOut => t * (2.0 - t),
            Self::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            }
            Self::CubicBezier { p1, p2 } => cubic_bezier(t, *p1, *p2),
        }
    }
}

/// Animation effect for time-based transformations
pub struct AnimationEffect {
    start_time: f32,
    end_time: f32,
    current_time: f32,
    interpolation: InterpolationType,
    property: AnimationProperty,
}

/// Animated property
#[derive(Debug, Clone)]
pub enum AnimationProperty {
    /// Opacity animation
    Opacity { start: f32, end: f32 },
    /// Position animation
    Position {
        start_x: f32,
        start_y: f32,
        end_x: f32,
        end_y: f32,
    },
    /// Scale animation
    Scale {
        start_x: f32,
        start_y: f32,
        end_x: f32,
        end_y: f32,
    },
    /// Rotation animation
    Rotation { start: f32, end: f32 },
    /// Color animation
    Color { start: [u8; 4], end: [u8; 4] },
}

impl AnimationEffect {
    /// Create a new animation effect
    pub fn new(
        start_time: f32,
        end_time: f32,
        interpolation: InterpolationType,
        property: AnimationProperty,
    ) -> Self {
        Self {
            start_time,
            end_time,
            current_time: 0.0,
            interpolation,
            property,
        }
    }

    /// Set current animation time
    pub fn set_time(&mut self, time: f32) {
        self.current_time = time;
    }

    /// Get animation progress (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        if self.current_time <= self.start_time {
            return 0.0;
        }
        if self.current_time >= self.end_time {
            return 1.0;
        }

        let duration = self.end_time - self.start_time;
        if duration <= 0.0 {
            return 1.0;
        }

        (self.current_time - self.start_time) / duration
    }

    /// Get interpolated value at current time
    pub fn interpolated_progress(&self) -> f32 {
        self.interpolation.interpolate(self.progress())
    }
}

impl Effect for AnimationEffect {
    fn apply(&self, pixels: &mut [u8], width: u32, height: u32) -> Result<(), RenderError> {
        if pixels.len() != (width * height * 4) as usize {
            return Err(RenderError::InvalidBufferSize {
                expected: (width * height * 4) as usize,
                actual: pixels.len(),
            });
        }

        let t = self.interpolated_progress();

        match &self.property {
            AnimationProperty::Opacity { start, end } => {
                let opacity = lerp(*start, *end, t);
                let opacity_byte = (opacity * 255.0) as u8;

                // Apply opacity to alpha channel
                for pixel in pixels.chunks_exact_mut(4) {
                    pixel[3] = ((pixel[3] as f32 * opacity) as u8).min(255);
                }
            }
            AnimationProperty::Color { start, end } => {
                // Blend colors
                for pixel in pixels.chunks_exact_mut(4) {
                    for i in 0..4 {
                        pixel[i] = lerp(start[i] as f32, end[i] as f32, t) as u8;
                    }
                }
            }
            AnimationProperty::Position { .. }
            | AnimationProperty::Scale { .. }
            | AnimationProperty::Rotation { .. } => {
                // These would require geometric transformations
                // which should be handled by TransformEffect
                // This is a placeholder
            }
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "Animation"
    }
}
