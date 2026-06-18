//! Argument parsers for ASS override tags

use super::types::{ClipData, FadeData};
#[cfg(feature = "nostd")]
use alloc::vec::Vec;
#[cfg(not(feature = "nostd"))]
use std::vec::Vec;

/// Parse position arguments from ASS \pos tag
pub fn parse_pos_args(args: &str) -> Option<(f32, f32)> {
    let args = args.trim_start_matches('(').trim_end_matches(')');
    let parts: Vec<&str> = args.split(',').collect();
    if parts.len() == 2 {
        if let (Ok(x), Ok(y)) = (
            parts[0].trim().parse::<f32>(),
            parts[1].trim().parse::<f32>(),
        ) {
            return Some((x, y));
        }
    }
    None
}

/// Parse movement arguments from ASS \move tag
pub fn parse_move_args(args: &str) -> Option<(f32, f32, f32, f32, u32, u32)> {
    let args = args.trim_start_matches('(').trim_end_matches(')');
    let parts: Vec<&str> = args.split(',').collect();

    if parts.len() >= 4 {
        let x1 = parts[0].trim().parse::<f32>().ok()?;
        let y1 = parts[1].trim().parse::<f32>().ok()?;
        let x2 = parts[2].trim().parse::<f32>().ok()?;
        let y2 = parts[3].trim().parse::<f32>().ok()?;

        // Times in \move are in milliseconds, need to convert to centiseconds
        let t1_ms = if parts.len() > 4 {
            parts[4].trim().parse::<u32>().unwrap_or(0)
        } else {
            0
        };

        let t2_ms = if parts.len() > 5 {
            parts[5].trim().parse::<u32>().unwrap_or(0)
        } else {
            0
        };

        // Convert milliseconds to centiseconds
        let t1 = t1_ms / 10;
        let t2 = t2_ms / 10;

        return Some((x1, y1, x2, y2, t1, t2));
    }

    None
}

/// Strip any stray prefix before the `&H` colour/alpha introducer.
///
/// Real-world scripts often emit a malformed `\1cH&H2A4F5D&` (a stray letter
/// between the tag and the `&H` value); libass parses these, so the value
/// parsers slice from the `&H`/`&h` introducer rather than assuming it leads.
/// Returns the original string unchanged when no `&H` is present (e.g. legacy
/// decimal colours), so well-formed input is unaffected.
fn strip_to_ampersand_hex(args: &str) -> &str {
    let bytes = args.as_bytes();
    for i in 0..bytes.len().saturating_sub(1) {
        if bytes[i] == b'&' && (bytes[i + 1] | 0x20) == b'h' {
            return &args[i..];
        }
    }
    args
}

/// Parse color arguments from ASS color tags
pub fn parse_color(args: &str) -> Option<[u8; 4]> {
    // Use ass-core's color parsing
    ass_core::utils::parse_bgr_color(strip_to_ampersand_hex(args)).ok()
}

/// Parse alpha arguments from ASS alpha tags
pub fn parse_alpha(args: &str) -> Option<u8> {
    let hex = strip_to_ampersand_hex(args)
        .trim_start_matches("&H")
        .trim_start_matches("&h")
        .trim_end_matches('&');
    // ASS uses inverted alpha: 0 = opaque, 255 = transparent
    // We need to invert it to match standard RGBA: 255 = opaque, 0 = transparent
    u8::from_str_radix(hex, 16)
        .ok()
        .map(|ass_alpha| 255 - ass_alpha)
}

/// Parse rectangular clip arguments from an ASS `\clip`/`\iclip` tag
/// (`(x1,y1,x2,y2)`). Vector (drawing) clips are not handled here.
pub fn parse_clip_args(args: &str) -> Option<ClipData> {
    let args = args.trim_start_matches('(').trim_end_matches(')');
    let parts: Vec<&str> = args.split(',').collect();

    if parts.len() == 4 {
        let x1 = parts[0].trim().parse::<f32>().ok()?;
        let y1 = parts[1].trim().parse::<f32>().ok()?;
        let x2 = parts[2].trim().parse::<f32>().ok()?;
        let y2 = parts[3].trim().parse::<f32>().ok()?;

        return Some(ClipData {
            x1,
            y1,
            x2,
            y2,
            inverse: false,
        });
    }

    None
}

/// Parse fade arguments from ASS \fade tag
pub fn parse_fade_args(args: &str) -> Option<FadeData> {
    let args = args.trim_start_matches('(').trim_end_matches(')');
    let parts: Vec<&str> = args.split(',').collect();

    if parts.len() >= 2 {
        // Simple fade: fade_in_time, fade_out_time (in milliseconds)
        if parts.len() == 2 {
            let fade_in_ms = parts[0].trim().parse::<u32>().ok()?;
            let fade_out_ms = parts[1].trim().parse::<u32>().ok()?;

            // Convert milliseconds to centiseconds
            let time_start = fade_in_ms / 10;
            let time_end = fade_out_ms / 10;

            // For simple fade, we store durations not alpha values
            // The actual alpha calculation happens during rendering
            return Some(FadeData {
                alpha_start: 0, // Not used for simple fade
                alpha_end: 0,   // Not used for simple fade
                time_start,     // Fade-in duration in centiseconds
                time_end,       // Fade-out duration in centiseconds
                alpha_middle: None,
                time_fade_in: None,
                time_fade_out: None,
            });
        }

        // Complex fade: alpha1, alpha2, alpha3, t1, t2, t3, t4
        // In ASS format, alpha values are INVERTED: 00=opaque, FF=transparent
        // alpha1: alpha before fade in (00-FF in hex, where 00=opaque, FF=transparent)
        // alpha2: alpha during main display
        // alpha3: alpha after fade out
        // t1-t2: fade in period (in milliseconds)
        // t2-t3: fully visible period (in milliseconds)
        // t3-t4: fade out period (in milliseconds)
        if parts.len() >= 7 {
            let alpha1 = parts[0].trim().parse::<u8>().ok()?;
            let alpha2 = parts[1].trim().parse::<u8>().ok()?;
            let alpha3 = parts[2].trim().parse::<u8>().ok()?;
            let t1_ms = parts[3].trim().parse::<u32>().ok()?;
            let t2_ms = parts[4].trim().parse::<u32>().ok()?;
            let t3_ms = parts[5].trim().parse::<u32>().ok()?;
            let t4_ms = parts[6].trim().parse::<u32>().ok()?;

            // Convert milliseconds to centiseconds
            let t1 = t1_ms / 10;
            let t2 = t2_ms / 10;
            let t3 = t3_ms / 10;
            let t4 = t4_ms / 10;

            // Store the ASS alpha values as-is (00=opaque, FF=transparent)
            // They'll be inverted when applied
            return Some(FadeData {
                alpha_start: alpha1,
                alpha_end: alpha3,
                time_start: t1,
                time_end: t4,
                alpha_middle: Some(alpha2),
                time_fade_in: Some(t2 - t1),
                time_fade_out: Some(t4 - t3),
            });
        }
    }

    None
}
