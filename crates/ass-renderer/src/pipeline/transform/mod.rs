//! Transform animation support for \t tags

mod parse;

#[cfg(feature = "nostd")]
use alloc::vec::Vec;
#[cfg(not(feature = "nostd"))]
use std::vec::Vec;

use parse::{apply_acceleration_curve, parse_animatable_tags};

/// Transform animation data
#[derive(Debug, Clone)]
pub struct TransformAnimation {
    /// Start time in milliseconds (relative to line start)
    pub start_ms: u32,
    /// End time in milliseconds (relative to line start)
    pub end_ms: u32,
    /// Acceleration factor (1.0 = linear, >1 = ease-in, <1 = ease-out)
    pub accel: f32,
    /// Tags to animate to
    pub target_tags: Vec<AnimatableTag>,
}

/// Tags that can be animated
#[derive(Debug, Clone)]
pub enum AnimatableTag {
    /// Font size
    FontSize(f32),
    /// Font scale X
    FontScaleX(f32),
    /// Font scale Y  
    FontScaleY(f32),
    /// Font spacing
    FontSpacing(f32),
    /// Font rotation Z
    FontRotationZ(f32),
    /// Font rotation X (3D)
    FontRotationX(f32),
    /// Font rotation Y (3D)
    FontRotationY(f32),
    /// Primary color
    PrimaryColor([u8; 4]),
    /// Secondary color
    SecondaryColor([u8; 4]),
    /// Outline color
    OutlineColor([u8; 4]),
    /// Shadow color
    ShadowColor([u8; 4]),
    /// Alpha
    Alpha(u8),
    /// Border width
    BorderWidth(f32),
    /// Shadow depth
    ShadowDepth(f32),
    /// Blur
    Blur(f32),
}

impl TransformAnimation {
    /// Parse transform tag arguments
    pub fn parse(args: &str) -> Option<Self> {
        // Transform can have formats:
        // \t(tags) - animate over full duration
        // \t(accel,tags) - with acceleration
        // \t(t1,t2,tags) - with time range
        // \t(t1,t2,accel,tags) - with time range and acceleration

        let args = args.trim();
        if !args.starts_with('(') || !args.ends_with(')') {
            return None;
        }

        let inner = &args[1..args.len() - 1];

        // Find where the override tags start (the first backslash). Everything
        // before it is the numeric parameter list, which is empty for the
        // `\t(<tags>)` form (tags begin at index 0). No backslash means there are
        // no tags to animate, so this is not a transform.
        let tag_start = inner.find('\\')?;

        let params = &inner[..tag_start];
        let tags = &inner[tag_start..];

        // Parse parameters - need to be careful since last param might be tags not accel
        let parts: Vec<&str> = params
            .trim_end_matches(',')
            .split(',')
            .map(|s| s.trim())
            .collect();

        let (start_ms, end_ms, accel) =
            if parts.is_empty() || (parts.len() == 1 && parts[0].is_empty()) {
                // No parameters, just tags
                (0, 0, 1.0)
            } else if parts.len() == 1 {
                // Could be acceleration or time1
                if let Ok(val) = parts[0].parse::<f32>() {
                    if val < 10.0 {
                        // Likely acceleration (values typically 0.1 to 5)
                        (0, 0, val)
                    } else {
                        // Likely time value
                        (0, val as u32, 1.0)
                    }
                } else {
                    (0, 0, 1.0)
                }
            } else if parts.len() == 2 {
                // Two params: t1,t2
                let t1 = parts[0].parse::<u32>().unwrap_or(0);
                let t2 = parts[1].parse::<u32>().unwrap_or(0);
                (t1, t2, 1.0)
            } else {
                // Three or more params: t1,t2,accel
                let t1 = parts[0].parse::<u32>().unwrap_or(0);
                let t2 = parts[1].parse::<u32>().unwrap_or(0);
                // Only parse third param as accel if it's a valid float
                let accel = parts[2].parse::<f32>().unwrap_or(1.0);
                (t1, t2, accel)
            };

        // Parse target tags
        let target_tags = parse_animatable_tags(tags);

        if target_tags.is_empty() {
            return None;
        }

        Some(TransformAnimation {
            start_ms,
            end_ms,
            accel,
            target_tags,
        })
    }

    /// Calculate interpolation progress for the current time (all in milliseconds
    /// relative to the event start).
    ///
    /// `full_duration_ms` is the event's duration: a `\t` with no explicit end time
    /// (`\t(tags)` or `\t(accel,tags)`) animates across the whole event, matching
    /// libass, rather than snapping straight to the target.
    pub fn calculate_progress(&self, current_ms: u32, full_duration_ms: u32) -> f32 {
        let end_ms = if self.end_ms > 0 {
            self.end_ms
        } else {
            full_duration_ms
        };

        if current_ms <= self.start_ms {
            return 0.0;
        }
        if end_ms <= self.start_ms || current_ms >= end_ms {
            return 1.0;
        }

        let linear_progress = (current_ms - self.start_ms) as f32 / (end_ms - self.start_ms) as f32;

        // Apply acceleration curve
        apply_acceleration_curve(linear_progress, self.accel)
    }
}

/// Interpolate between two values based on progress
pub fn interpolate_f32(from: f32, to: f32, progress: f32) -> f32 {
    from + (to - from) * progress
}

/// Interpolate between two colors
pub fn interpolate_color(from: [u8; 4], to: [u8; 4], progress: f32) -> [u8; 4] {
    [
        (from[0] as f32 + (to[0] as f32 - from[0] as f32) * progress) as u8,
        (from[1] as f32 + (to[1] as f32 - from[1] as f32) * progress) as u8,
        (from[2] as f32 + (to[2] as f32 - from[2] as f32) * progress) as u8,
        (from[3] as f32 + (to[3] as f32 - from[3] as f32) * progress) as u8,
    ]
}

/// Interpolate alpha value
pub fn interpolate_alpha(from: u8, to: u8, progress: f32) -> u8 {
    (from as f32 + (to as f32 - from as f32) * progress) as u8
}
