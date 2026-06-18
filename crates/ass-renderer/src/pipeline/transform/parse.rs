//! Parsing and easing helpers for transform (`\t`) animations.

#[cfg(feature = "nostd")]
use alloc::{string::String, vec::Vec};
#[cfg(not(feature = "nostd"))]
use std::{string::String, vec::Vec};

use super::AnimatableTag;

/// Apply acceleration curve to linear progress
///
/// Implements various easing functions based on acceleration value:
/// - accel = 1.0: Linear interpolation
/// - accel < 1.0: Ease-out (fast start, slow end)
/// - accel > 1.0: Ease-in (slow start, fast end)
/// - Special handling for common values like 0.5, 2.0, 3.0
pub(super) fn apply_acceleration_curve(t: f32, accel: f32) -> f32 {
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
pub(super) fn parse_animatable_tags(tag_string: &str) -> Vec<AnimatableTag> {
    let mut tags = Vec::new();
    let mut pos = 0;
    let chars: Vec<char> = tag_string.chars().collect();

    while pos < chars.len() {
        if chars[pos] == '\\' {
            pos += 1;
            let tag_start = pos;

            // Parse tag name - stop at first non-letter character after initial letter(s)
            // unless it's a tag that starts with a digit (like 1c, 2c, 3a, 4a)
            let first_char = if tag_start < chars.len() {
                chars[tag_start]
            } else {
                ' '
            };
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

                // Create animatable tag
                if let Some(tag) = create_animatable_tag(&tag_name, &args) {
                    tags.push(tag);
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
    u8::from_str_radix(hex, 16)
        .ok()
        .map(|ass_alpha| 255 - ass_alpha)
}
