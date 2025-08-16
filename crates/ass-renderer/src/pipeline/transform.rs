//! Transform animation support for \t tags

#[cfg(feature = "nostd")]
use alloc::{string::String, vec::Vec};
#[cfg(not(feature = "nostd"))]
use std::{string::String, vec::Vec};

use crate::utils::RenderError;
use ass_core::analysis::events::OverrideTag;

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

        #[cfg(all(debug_assertions, not(feature = "nostd")))]
        eprintln!("TransformAnimation::parse called with args: '{}'", args);

        let args = args.trim();
        if !args.starts_with('(') || !args.ends_with(')') {
            #[cfg(all(debug_assertions, not(feature = "nostd")))]
            eprintln!("TransformAnimation::parse failed - missing parentheses");
            return None;
        }

        let inner = &args[1..args.len() - 1];

        // Find where tags start (after numeric parameters)
        let mut tag_start = 0;
        let mut depth = 0;
        let mut in_tag = false;

        for (i, ch) in inner.chars().enumerate() {
            if ch == '\\' && !in_tag {
                tag_start = i;
                break;
            }
            if ch == '(' {
                depth += 1;
            } else if ch == ')' {
                depth -= 1;
            }
        }

        if tag_start == 0 {
            #[cfg(all(debug_assertions, not(feature = "nostd")))]
            eprintln!("TransformAnimation::parse failed - no tags found");
            return None;
        }

        let params = &inner[..tag_start];
        let tags = &inner[tag_start..];
        
        #[cfg(all(debug_assertions, not(feature = "nostd")))]
        eprintln!("TransformAnimation::parse - params: '{}', tags: '{}'", params, tags);

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

        #[cfg(all(debug_assertions, not(feature = "nostd")))]
        eprintln!("TransformAnimation::parse - parsed {} target tags", target_tags.len());

        if target_tags.is_empty() {
            #[cfg(all(debug_assertions, not(feature = "nostd")))]
            eprintln!("TransformAnimation::parse failed - no target tags found");
            return None;
        }

        Some(TransformAnimation {
            start_ms,
            end_ms,
            accel,
            target_tags,
        })
    }

    /// Calculate interpolation progress for current time
    pub fn calculate_progress(&self, current_ms: u32) -> f32 {
        if current_ms <= self.start_ms {
            return 0.0;
        }
        if self.end_ms > 0 && current_ms >= self.end_ms {
            return 1.0;
        }

        let duration = if self.end_ms > 0 {
            self.end_ms - self.start_ms
        } else {
            // Use full line duration if end time not specified
            current_ms - self.start_ms
        };

        if duration == 0 {
            return 1.0;
        }

        let linear_progress = (current_ms - self.start_ms) as f32 / duration as f32;

        // Apply acceleration curve
        apply_acceleration_curve(linear_progress, self.accel)
    }
}

/// Apply acceleration curve to linear progress
///
/// Implements various easing functions based on acceleration value:
/// - accel = 1.0: Linear interpolation
/// - accel < 1.0: Ease-out (fast start, slow end)
/// - accel > 1.0: Ease-in (slow start, fast end)
/// - Special handling for common values like 0.5, 2.0, 3.0
fn apply_acceleration_curve(t: f32, accel: f32) -> f32 {
    // Clamp t to [0, 1]
    let t = t.clamp(0.0, 1.0);

    if (accel - 1.0).abs() < 0.001 {
        // Linear interpolation
        t
    } else if (accel - 0.5).abs() < 0.001 {
        // Strong ease-out (quadratic)
        1.0 - (1.0 - t).powi(2)
    } else if (accel - 2.0).abs() < 0.001 {
        // Quadratic ease-in
        t * t
    } else if (accel - 3.0).abs() < 0.001 {
        // Cubic ease-in
        t * t * t
    } else if accel < 0.0 {
        // Negative acceleration: bounce or elastic effect
        // Simplified bounce for negative values
        let bounce = (t * core::f32::consts::PI).sin();
        t + bounce * 0.1 * accel.abs()
    } else {
        // General exponential easing
        // Uses pow for custom acceleration curves
        t.powf(accel)
    }
}

/// Parse tags that can be animated
fn parse_animatable_tags(tag_string: &str) -> Vec<AnimatableTag> {
    #[cfg(all(debug_assertions, not(feature = "nostd")))]
    eprintln!("parse_animatable_tags: input='{}'", tag_string);
    
    let mut tags = Vec::new();
    let mut pos = 0;
    let chars: Vec<char> = tag_string.chars().collect();

    while pos < chars.len() {
        if chars[pos] == '\\' {
            pos += 1;
            let tag_start = pos;

            // Parse tag name - stop at first non-letter character after initial letter(s)
            // unless it's a tag that starts with a digit (like 1c, 2c, 3a, 4a)
            let first_char = if tag_start < chars.len() { chars[tag_start] } else { ' ' };
            if first_char.is_ascii_digit() {
                // Tags like 1c, 2c, 3a, 4a
                pos += 1; // Skip the digit
                while pos < chars.len() && chars[pos].is_ascii_alphabetic() {
                    pos += 1;
                }
            } else {
                // Normal tags - only letters
                while pos < chars.len() && chars[pos].is_ascii_alphabetic() {
                    pos += 1;
                }
            }

            if pos > tag_start {
                let tag_name = chars[tag_start..pos].iter().collect::<String>();

                // Parse tag arguments
                let arg_start = pos;
                while pos < chars.len() && chars[pos] != '\\' {
                    pos += 1;
                }

                let args = chars[arg_start..pos].iter().collect::<String>();
                
                #[cfg(all(debug_assertions, not(feature = "nostd")))]
                eprintln!("  Found tag: name='{}', args='{}'", tag_name, args);

                // Create animatable tag
                if let Some(tag) = create_animatable_tag(&tag_name, &args) {
                    tags.push(tag);
                    #[cfg(all(debug_assertions, not(feature = "nostd")))]
                    eprintln!("  Created animatable tag for '{}'", tag_name);
                } else {
                    #[cfg(all(debug_assertions, not(feature = "nostd")))]
                    eprintln!("  Failed to create animatable tag for '{}'", tag_name);
                }
            }
        } else {
            pos += 1;
        }
    }

    tags
}

fn create_animatable_tag(name: &str, args: &str) -> Option<AnimatableTag> {
    match name {
        "fs" => args.parse::<f32>().ok().map(AnimatableTag::FontSize),
        "fscx" => args.parse::<f32>().ok().map(AnimatableTag::FontScaleX),
        "fscy" => args.parse::<f32>().ok().map(AnimatableTag::FontScaleY),
        "fsp" => args.parse::<f32>().ok().map(AnimatableTag::FontSpacing),
        "frz" | "fr" => args.parse::<f32>().ok().map(AnimatableTag::FontRotationZ),
        "frx" => args.parse::<f32>().ok().map(AnimatableTag::FontRotationX),
        "fry" => args.parse::<f32>().ok().map(AnimatableTag::FontRotationY),
        "c" | "1c" => parse_color_arg(args).map(AnimatableTag::PrimaryColor),
        "2c" => parse_color_arg(args).map(AnimatableTag::SecondaryColor),
        "3c" => parse_color_arg(args).map(AnimatableTag::OutlineColor),
        "4c" => parse_color_arg(args).map(AnimatableTag::ShadowColor),
        "alpha" => parse_alpha_arg(args).map(AnimatableTag::Alpha),
        "bord" => args.parse::<f32>().ok().map(AnimatableTag::BorderWidth),
        "shad" => args.parse::<f32>().ok().map(AnimatableTag::ShadowDepth),
        "blur" | "be" => args.parse::<f32>().ok().map(AnimatableTag::Blur),
        _ => None,
    }
}

fn parse_color_arg(args: &str) -> Option<[u8; 4]> {
    ass_core::utils::parse_bgr_color(args).ok()
}

fn parse_alpha_arg(args: &str) -> Option<u8> {
    let hex = args.trim_start_matches("&H").trim_start_matches("&h");
    // ASS uses inverted alpha: 0 = opaque, 255 = transparent
    // We need to invert it to match standard RGBA: 255 = opaque, 0 = transparent
    u8::from_str_radix(hex, 16).ok().map(|ass_alpha| 255 - ass_alpha)
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
