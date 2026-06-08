//! Validated parsing helpers for raw style field strings.
//!
//! Shared `pub(super)` parsers that convert raw ASS style fields into typed
//! values with range validation, used across the resolution submodules.

use crate::{utils::CoreError, Result};

/// Parse font size with validation
pub(super) fn parse_font_size(size_str: &str) -> Result<f32> {
    let size = parse_float(size_str)?;
    if size <= 0.0 || size > 1000.0 {
        Err(CoreError::parse("Invalid font size"))
    } else {
        Ok(size)
    }
}

/// Parse color value with default handling for empty strings
pub(super) fn parse_color_with_default(color_str: &str) -> Result<[u8; 4]> {
    if color_str.trim().is_empty() {
        Ok([255, 255, 255, 255]) // Default white with full alpha
    } else {
        crate::utils::parse_bgr_color(color_str)
    }
}

/// Parse boolean flag (0 or 1)
pub(super) fn parse_bool_flag(flag_str: &str) -> Result<bool> {
    match flag_str {
        "0" => Ok(false),
        "1" => Ok(true),
        _ => Err(CoreError::parse("Invalid boolean flag")),
    }
}

/// Parse percentage value
pub(super) fn parse_percentage(percent_str: &str) -> Result<f32> {
    let value = parse_float(percent_str)?;
    if (0.0..=1000.0).contains(&value) {
        Ok(value)
    } else {
        Err(CoreError::parse("Invalid percentage"))
    }
}

/// Parse float value with validation
pub(super) fn parse_float(float_str: &str) -> Result<f32> {
    float_str
        .parse::<f32>()
        .map_err(|_| CoreError::parse("Invalid float value"))
}

/// Parse u8 value with validation
pub(super) fn parse_u8(u8_str: &str) -> Result<u8> {
    u8_str
        .parse::<u8>()
        .map_err(|_| CoreError::parse("Invalid u8 value"))
}

/// Parse u16 value with validation
pub(super) fn parse_u16(u16_str: &str) -> Result<u16> {
    u16_str
        .parse::<u16>()
        .map_err(|_| CoreError::parse("Invalid u16 value"))
}
