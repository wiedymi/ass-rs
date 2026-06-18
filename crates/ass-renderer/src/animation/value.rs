//! Animated property values and their interpolation results

/// Animated property value
#[derive(Debug, Clone)]
pub enum AnimatedValue {
    /// Integer value animation
    Integer {
        /// Starting value
        from: i32,
        /// Ending value
        to: i32,
    },
    /// Float value animation
    Float {
        /// Starting value
        from: f32,
        /// Ending value
        to: f32,
    },
    /// Color animation (RGBA)
    Color {
        /// Starting color [R, G, B, A]
        from: [u8; 4],
        /// Ending color [R, G, B, A]
        to: [u8; 4],
    },
    /// Position animation
    Position {
        /// Starting position (x, y)
        from: (f32, f32),
        /// Ending position (x, y)
        to: (f32, f32),
    },
    /// Scale animation
    Scale {
        /// Starting scale (x, y)
        from: (f32, f32),
        /// Ending scale (x, y)
        to: (f32, f32),
    },
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
    /// Integer result value
    Integer(i32),
    /// Float result value
    Float(f32),
    /// Color result value [R, G, B, A]
    Color([u8; 4]),
    /// Position result value (x, y)
    Position((f32, f32)),
    /// Scale result value (x, y)
    Scale((f32, f32)),
}
