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
        let frac_str = &seconds_parts[1];
        let frac_val: u32 = frac_str
            .parse()
            .map_err(|_| CoreError::InvalidTime(format!("Invalid centiseconds: {frac_str}")))?;

        match frac_str.len() {
            1 => frac_val * 10,
            2 => frac_val,
            _ => {
                return Err(CoreError::InvalidTime(format!(
                    "Too many decimal places: {frac_str}"
                )))
            }
        }
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
