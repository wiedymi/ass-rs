//! Tag parameter validation module

#[cfg(feature = "nostd")]
use alloc::{format, string::ToString};
#[cfg(not(feature = "nostd"))]
use std::string::ToString;

use crate::utils::RenderError;

/// Validate alignment value (1-9 for numpad positions)
pub fn validate_alignment(value: u8) -> Result<u8, RenderError> {
    if value >= 1 && value <= 9 {
        Ok(value)
    } else {
        Err(RenderError::InvalidScript(format!(
            "Invalid alignment value: {}. Must be 1-9",
            value
        )))
    }
}

/// Validate wrap style (0-3)
pub fn validate_wrap_style(value: u8) -> Result<u8, RenderError> {
    if value <= 3 {
        Ok(value)
    } else {
        Err(RenderError::InvalidScript(format!(
            "Invalid wrap style: {}. Must be 0-3",
            value
        )))
    }
}

/// Validate drawing mode level (0-4)
pub fn validate_drawing_mode(value: u8) -> Result<u8, RenderError> {
    if value <= 4 {
        Ok(value)
    } else {
        Err(RenderError::InvalidScript(format!(
            "Invalid drawing mode: {}. Must be 0-4",
            value
        )))
    }
}

/// Validate font size (must be positive)
pub fn validate_font_size(value: f32) -> Result<f32, RenderError> {
    if value > 0.0 && value < 1000.0 {
        Ok(value)
    } else {
        Err(RenderError::InvalidScript(format!(
            "Invalid font size: {}. Must be positive and reasonable",
            value
        )))
    }
}

/// Validate scale percentage (0-1000%)
pub fn validate_scale(value: f32) -> Result<f32, RenderError> {
    if value >= 0.0 && value <= 1000.0 {
        Ok(value)
    } else {
        Err(RenderError::InvalidScript(format!(
            "Invalid scale: {}%. Must be 0-1000",
            value
        )))
    }
}

/// Validate rotation angle (degrees)
pub fn validate_rotation(value: f32) -> Result<f32, RenderError> {
    // Normalize to 0-360 range
    let normalized = value % 360.0;
    Ok(if normalized < 0.0 {
        normalized + 360.0
    } else {
        normalized
    })
}

/// Validate border width (non-negative)
pub fn validate_border_width(value: f32) -> Result<f32, RenderError> {
    if value >= 0.0 && value < 100.0 {
        Ok(value)
    } else {
        Err(RenderError::InvalidScript(format!(
            "Invalid border width: {}. Must be non-negative",
            value
        )))
    }
}

/// Validate blur amount (non-negative)
pub fn validate_blur(value: f32) -> Result<f32, RenderError> {
    if value >= 0.0 && value < 100.0 {
        Ok(value)
    } else {
        Err(RenderError::InvalidScript(format!(
            "Invalid blur: {}. Must be non-negative",
            value
        )))
    }
}

/// Validate color array (BGRA format)
pub fn validate_color(color: [u8; 4]) -> Result<[u8; 4], RenderError> {
    // Colors are already in valid u8 range
    Ok(color)
}

/// Validate alpha value (0-255)
pub fn validate_alpha(value: u8) -> Result<u8, RenderError> {
    // u8 is already in valid range
    Ok(value)
}

/// Validate time in milliseconds
pub fn validate_time_ms(value: u32) -> Result<u32, RenderError> {
    // Allow any u32 value for time
    Ok(value)
}

/// Validate karaoke duration (centiseconds)
pub fn validate_karaoke_duration(value: u32) -> Result<u32, RenderError> {
    if value < 100000 {
        // Max ~16 minutes per syllable
        Ok(value)
    } else {
        Err(RenderError::InvalidScript(format!(
            "Invalid karaoke duration: {} centiseconds",
            value
        )))
    }
}

/// Validate position coordinates
pub fn validate_position(x: f32, y: f32) -> Result<(f32, f32), RenderError> {
    // Allow negative values for off-screen positioning
    if x.is_finite() && y.is_finite() {
        Ok((x, y))
    } else {
        Err(RenderError::InvalidScript(
            "Invalid position: coordinates must be finite".to_string(),
        ))
    }
}

/// Validate clipping rectangle
pub fn validate_clip_rect(
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
) -> Result<(f32, f32, f32, f32), RenderError> {
    if x1.is_finite() && y1.is_finite() && x2.is_finite() && y2.is_finite() {
        // Ensure x2 > x1 and y2 > y1
        let (x1, x2) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
        let (y1, y2) = if y1 <= y2 { (y1, y2) } else { (y2, y1) };
        Ok((x1, y1, x2, y2))
    } else {
        Err(RenderError::InvalidScript(
            "Invalid clip rectangle: coordinates must be finite".to_string(),
        ))
    }
}

/// Validate font encoding (0-255, though only specific values are meaningful)
pub fn validate_font_encoding(value: u8) -> Result<u8, RenderError> {
    // Common encodings: 0 (ANSI), 1 (Default), 128 (Shift-JIS), 134 (GB2312), etc.
    Ok(value)
}

/// Validate shear/perspective factor
pub fn validate_shear(value: f32) -> Result<f32, RenderError> {
    if value >= -2.0 && value <= 2.0 {
        Ok(value)
    } else {
        Err(RenderError::InvalidScript(format!(
            "Invalid shear factor: {}. Should be between -2 and 2",
            value
        )))
    }
}

/// Validate acceleration factor for transforms
pub fn validate_acceleration(value: f32) -> Result<f32, RenderError> {
    if value > 0.0 && value < 100.0 {
        Ok(value)
    } else {
        Err(RenderError::InvalidScript(format!(
            "Invalid acceleration: {}. Must be positive",
            value
        )))
    }
}
