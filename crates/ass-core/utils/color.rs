//! ASS BGR color parsing helpers.
//!
//! Converts ASS BGR/ABGR hex color strings into standard RGBA byte arrays
//! suitable for rendering.

#[cfg(not(feature = "std"))]
use alloc::format;
#[cfg(feature = "std")]
use std::format;

use super::CoreError;

/// Parse ASS BGR color format to RGBA bytes
///
/// ASS uses BGR format like `&H00FF00FF&` (blue, green, red, alpha).
/// Converts to standard RGBA format for rendering.
///
/// # Arguments
///
/// * `color_str` - Color string in ASS format
///
/// # Returns
///
/// RGBA bytes `[red, green, blue, alpha]` or error if invalid format.
///
/// # Example
///
/// ```rust
/// # use ass_core::utils::parse_bgr_color;
/// // Pure red in ASS format
/// let rgba = parse_bgr_color("&H000000FF&")?;
/// assert_eq!(rgba, [255, 0, 0, 0]);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// Returns an error if the color string format is invalid or cannot be parsed.
pub fn parse_bgr_color(color_str: &str) -> Result<[u8; 4], CoreError> {
    let trimmed = color_str.trim();

    let hex_part =
        if (trimmed.starts_with("&H") || trimmed.starts_with("&h")) && trimmed.ends_with('&') {
            &trimmed[2..trimmed.len() - 1]
        } else if let Some(stripped) = trimmed.strip_prefix("&H") {
            stripped
        } else if let Some(stripped) = trimmed.strip_prefix("&h") {
            stripped
        } else if let Some(stripped) = trimmed.strip_prefix("0x") {
            stripped
        } else if trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
            trimmed
        } else {
            return Err(CoreError::InvalidColor(format!(
                "Invalid color format: {color_str}"
            )));
        };

    let hex_value = u32::from_str_radix(hex_part, 16)
        .map_err(|_| CoreError::InvalidColor(format!("Invalid hex value: {hex_part}")))?;

    let color_array = match hex_part.len() {
        6 => {
            // ASS uses BGR format: &HBBGGRR
            let red = (hex_value & 0xFF) as u8;
            let green = ((hex_value >> 8) & 0xFF) as u8;
            let blue = ((hex_value >> 16) & 0xFF) as u8;
            [red, green, blue, 0] // Default alpha to 0 for 6-char format
        }
        8 => {
            // ASS uses ABGR format: &HAABBGGRR
            let alpha = ((hex_value >> 24) & 0xFF) as u8; // ASS alpha is direct (0=transparent, FF=opaque)
            let red = (hex_value & 0xFF) as u8;
            let green = ((hex_value >> 8) & 0xFF) as u8;
            let blue = ((hex_value >> 16) & 0xFF) as u8;
            [red, green, blue, alpha]
        }
        _ => {
            return Err(CoreError::InvalidColor(format!(
                "Invalid color length: {}",
                hex_part.len()
            )))
        }
    };

    Ok(color_array)
}
