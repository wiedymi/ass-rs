//! ASS timing conversion helpers.
//!
//! Parses ASS `H:MM:SS.CC` time strings to centiseconds and formats
//! centiseconds back into the ASS time representation.

#[cfg(not(feature = "std"))]
use alloc::{format, string::String, vec::Vec};
#[cfg(feature = "std")]
use std::{format, string::String, vec::Vec};

use super::CoreError;

/// Parse ASS time format (H:MM:SS.CC) to centiseconds
///
/// ASS uses centiseconds (1/100th second) for timing.
/// Supports various formats including fractional seconds.
///
/// # Example
///
/// ```rust
/// # use ass_core::utils::parse_ass_time;
/// assert_eq!(parse_ass_time("0:01:30.50")?, 9050); // 1:30.5 = 9050 centiseconds
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// Returns an error if the time format is invalid or cannot be parsed.
pub fn parse_ass_time(time_str: &str) -> Result<u32, CoreError> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() != 3 {
        return Err(CoreError::InvalidTime(format!(
            "Invalid time format: {time_str}"
        )));
    }

    let hours: u32 = parts[0]
        .parse()
        .map_err(|_| CoreError::InvalidTime(format!("Invalid hours: {}", parts[0])))?;

    let minutes: u32 = parts[1]
        .parse()
        .map_err(|_| CoreError::InvalidTime(format!("Invalid minutes: {}", parts[1])))?;

    let seconds_parts: Vec<&str> = parts[2].split('.').collect();
    let seconds: u32 = seconds_parts[0]
        .parse()
        .map_err(|_| CoreError::InvalidTime(format!("Invalid seconds: {}", seconds_parts[0])))?;

    let centiseconds = if seconds_parts.len() > 1 {
        let frac_str = seconds_parts[1];
        if frac_str.is_empty() || !frac_str.bytes().all(|b| b.is_ascii_digit()) {
            return Err(CoreError::InvalidTime(format!(
                "Invalid centiseconds: {frac_str}"
            )));
        }
        // Fractional seconds -> centiseconds. ASS specifies two digits, but real
        // files (and libass) tolerate millisecond (3-digit) precision, so scale by
        // the digit count and truncate, considering at most the first three digits
        // (`.c` = tenths, `.cc` = centiseconds, `.mmm` = milliseconds).
        let frac = &frac_str[..frac_str.len().min(3)];
        let frac_val: u32 = frac
            .parse()
            .map_err(|_| CoreError::InvalidTime(format!("Invalid centiseconds: {frac_str}")))?;
        // 10^len for len in 1..=3: tenths, centiseconds, milliseconds.
        let scale = match frac.len() {
            1 => 10,
            2 => 100,
            _ => 1000,
        };
        frac_val * 100 / scale
    } else {
        0
    };

    if minutes >= 60 {
        return Err(CoreError::InvalidTime(format!(
            "Minutes must be < 60: {minutes}"
        )));
    }
    if seconds >= 60 {
        return Err(CoreError::InvalidTime(format!(
            "Seconds must be < 60: {seconds}"
        )));
    }
    if centiseconds >= 100 {
        return Err(CoreError::InvalidTime(format!(
            "Centiseconds must be < 100: {centiseconds}"
        )));
    }

    Ok(hours * 360_000 + minutes * 6_000 + seconds * 100 + centiseconds)
}

/// Parse ASS time format (`H:MM:SS.CC`) to milliseconds.
///
/// Preserves the full fractional precision real scripts use (`.c` tenths, `.cc`
/// centiseconds, `.mmm` milliseconds). Unlike [`parse_ass_time`], which truncates
/// to centiseconds, this keeps millisecond resolution — needed so frame-stepped
/// karaoke/typesetting (syllable events only milliseconds apart) is sampled at the
/// right instant rather than collapsing onto centisecond boundaries.
///
/// # Errors
///
/// Returns an error if the time format is invalid or cannot be parsed.
pub fn parse_ass_time_ms(time_str: &str) -> Result<u32, CoreError> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() != 3 {
        return Err(CoreError::InvalidTime(format!(
            "Invalid time format: {time_str}"
        )));
    }

    let hours: u32 = parts[0]
        .parse()
        .map_err(|_| CoreError::InvalidTime(format!("Invalid hours: {}", parts[0])))?;
    let minutes: u32 = parts[1]
        .parse()
        .map_err(|_| CoreError::InvalidTime(format!("Invalid minutes: {}", parts[1])))?;

    let seconds_parts: Vec<&str> = parts[2].split('.').collect();
    let seconds: u32 = seconds_parts[0]
        .parse()
        .map_err(|_| CoreError::InvalidTime(format!("Invalid seconds: {}", seconds_parts[0])))?;

    let milliseconds = if seconds_parts.len() > 1 {
        let frac_str = seconds_parts[1];
        if frac_str.is_empty() || !frac_str.bytes().all(|b| b.is_ascii_digit()) {
            return Err(CoreError::InvalidTime(format!(
                "Invalid fractional seconds: {frac_str}"
            )));
        }
        // Consider at most the first three digits: `.c` tenths, `.cc` centiseconds,
        // `.mmm` milliseconds. Scale each to milliseconds.
        let frac = &frac_str[..frac_str.len().min(3)];
        let frac_val: u32 = frac.parse().map_err(|_| {
            CoreError::InvalidTime(format!("Invalid fractional seconds: {frac_str}"))
        })?;
        let scale = match frac.len() {
            1 => 100, // tenths -> ms
            2 => 10,  // centiseconds -> ms
            _ => 1,   // milliseconds
        };
        frac_val * scale
    } else {
        0
    };

    if minutes >= 60 {
        return Err(CoreError::InvalidTime(format!(
            "Minutes must be < 60: {minutes}"
        )));
    }
    if seconds >= 60 {
        return Err(CoreError::InvalidTime(format!(
            "Seconds must be < 60: {seconds}"
        )));
    }

    Ok(hours * 3_600_000 + minutes * 60_000 + seconds * 1_000 + milliseconds)
}

/// Format centiseconds back to ASS time format
///
/// Converts internal centisecond representation back to H:MM:SS.CC format.
#[must_use]
pub fn format_ass_time(centiseconds: u32) -> String {
    let hours = centiseconds / 360_000;
    let remainder = centiseconds % 360_000;
    let minutes = remainder / 6000;
    let remainder = remainder % 6000;
    let seconds = remainder / 100;
    let cs = remainder % 100;

    format!("{hours}:{minutes:02}:{seconds:02}.{cs:02}")
}
