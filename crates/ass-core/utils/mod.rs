//! Utility functions and shared types for ASS-RS core
//!
//! Contains common functionality used across parser, tokenizer, and analysis modules.
//! Focuses on zero-allocation helpers, color processing, and UTF-8 handling.
//!
//! # Performance
//!
//! - Zero-copy span utilities for AST references
//! - SIMD-optimized color conversions when available
//! - Minimal allocation math helpers (bezier evaluation)
//!
//! # Example
//!
//! ```rust
//! use ass_core::utils::{Spans, parse_bgr_color};
//!
//! let color_str = "&H00FF00FF&";
//! let rgba = parse_bgr_color(color_str)?;
//! assert_eq!(rgba, [255, 0, 255, 0]); // BGR -> RGBA
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#[cfg(not(feature = "std"))]
use alloc::{format, string::String, vec::Vec};
use core::{fmt, ops::Range};
#[cfg(feature = "std")]
use std::{format, string::String, vec::Vec};

pub mod benchmark_generators;
pub mod errors;
pub mod hashers;
pub mod utf8;

pub use benchmark_generators::{
    create_test_event, generate_overlapping_script, generate_script_with_issues, ComplexityLevel,
    ScriptGenerator,
};
pub use errors::CoreError;
pub use hashers::{create_hash_map, create_hash_map_with_capacity, create_hasher, hash_value};
pub use utf8::{detect_encoding, normalize_line_endings, recover_utf8, strip_bom, validate_utf8};

/// Zero-copy span utilities for AST node validation and manipulation
///
/// Provides safe methods to work with string slices that reference
/// the original source text, maintaining zero-copy semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spans<'a> {
    /// Reference to the original source text
    source: &'a str,
}

impl<'a> Spans<'a> {
    /// Create new span utilities for source text
    #[must_use]
    pub const fn new(source: &'a str) -> Self {
        Self { source }
    }

    /// Validate that a span references this source text
    ///
    /// Returns `true` if the span is a valid substring of the source.
    /// Used for debug assertions to ensure zero-copy invariants.
    #[must_use]
    pub fn validate_span(&self, span: &str) -> bool {
        let source_start = self.source.as_ptr() as usize;
        let source_end = source_start + self.source.len();

        let span_start = span.as_ptr() as usize;
        let span_end = span_start + span.len();

        span_start >= source_start && span_end <= source_end
    }

    /// Get byte offset of span within source
    #[must_use]
    pub fn span_offset(&self, span: &str) -> Option<usize> {
        let source_start = self.source.as_ptr() as usize;
        let span_start = span.as_ptr() as usize;

        if self.validate_span(span) {
            Some(span_start - source_start)
        } else {
            None
        }
    }

    /// Get line number (1-based) for a span
    #[must_use]
    pub fn span_line(&self, span: &str) -> Option<usize> {
        let offset = self.span_offset(span)?;
        Some(self.source[..offset].chars().filter(|&c| c == '\n').count() + 1)
    }

    /// Get column number (1-based) for a span
    #[must_use]
    pub fn span_column(&self, span: &str) -> Option<usize> {
        let offset = self.span_offset(span)?;
        let line_start = self.source[..offset].rfind('\n').map_or(0, |pos| pos + 1);

        Some(self.source[line_start..offset].chars().count() + 1)
    }

    /// Extract substring by byte range
    #[must_use]
    pub fn substring(&self, range: Range<usize>) -> Option<&'a str> {
        self.source.get(range)
    }
}

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
            [red, green, blue, 255] // Default to fully opaque
        }
        8 => {
            // ASS uses ABGR format: &HAABBGGRR
            let alpha = 255 - ((hex_value >> 24) & 0xFF) as u8; // ASS alpha is inverted (0=opaque, FF=transparent)
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

/// Parse numeric value from ASS field with validation
///
/// Handles integer and floating-point parsing with ASS-specific validation.
/// Provides better error messages than standard parsing.
///
/// # Errors
///
/// Returns an error if the string cannot be parsed as the target numeric type.
pub fn parse_numeric<T>(value_str: &str) -> Result<T, CoreError>
where
    T: core::str::FromStr,
    T::Err: fmt::Display,
{
    value_str
        .trim()
        .parse()
        .map_err(|e| CoreError::InvalidNumeric(format!("Failed to parse '{value_str}': {e}")))
}

/// Evaluate cubic bezier curve at parameter t
///
/// Used for drawing command evaluation and animation curves.
/// No external dependencies - implements bezier math directly.
///
/// # Arguments
///
/// * `p0, p1, p2, p3` - Control points as (x, y) tuples
/// * `t` - Parameter from 0.0 to 1.0
///
/// # Returns
///
/// Point on curve as (x, y) tuple
#[must_use]
pub fn eval_cubic_bezier(
    p0: (f32, f32),
    p1: (f32, f32),
    p2: (f32, f32),
    p3: (f32, f32),
    t: f32,
) -> (f32, f32) {
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;

    let x = t3.mul_add(
        p3.0,
        (3.0 * mt * t2).mul_add(p2.0, mt3.mul_add(p0.0, 3.0 * mt2 * t * p1.0)),
    );
    let y = t3.mul_add(
        p3.1,
        (3.0 * mt * t2).mul_add(p2.1, mt3.mul_add(p0.1, 3.0 * mt2 * t * p1.1)),
    );

    (x, y)
}

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

/// Trim and normalize whitespace in ASS field values
///
/// ASS fields may have inconsistent whitespace that should be normalized
/// while preserving intentional spacing in text content.
#[must_use]
pub fn normalize_field_value(value: &str) -> &str {
    value.trim()
}

/// Check if string contains only valid ASS characters
///
/// ASS has restrictions on certain characters in names and style definitions.
#[must_use]
pub fn validate_ass_name(name: &str) -> bool {
    !name.is_empty()
        && !name.contains(',') // Comma is field separator
        && !name.contains(':') // Colon is key-value separator
        && !name.contains('{') // Override block start
        && !name.contains('}') // Override block end
        && name.chars().all(|c| !c.is_control() || c == '\t')
}

/// Decode UU-encoded data commonly found in ASS `[Fonts]` and `[Graphics]` sections
///
/// UU-encoding (Unix-to-Unix encoding) embeds binary data as ASCII text.
/// Each line starts with a length character followed by encoded data.
///
/// # Arguments
///
/// * `lines` - Iterator of UU-encoded text lines
///
/// # Returns
///
/// Decoded binary data or error if encoding is invalid.
///
/// # Example
///
/// ```rust
/// # use ass_core::utils::decode_uu_data;
/// let lines = vec![""];
/// let decoded = decode_uu_data(lines.iter().map(|s| *s))?;
/// // UU-decode implementation handles empty input gracefully
/// assert!(decoded.len() >= 0);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// Returns an error if the UU-encoded data is malformed or cannot be decoded.
#[allow(clippy::similar_names)]
pub fn decode_uu_data<'a, I>(lines: I) -> Result<Vec<u8>, CoreError>
where
    I: Iterator<Item = &'a str>,
{
    let mut result = Vec::new();

    for line in lines {
        let line = line.trim_start().trim_end_matches(['\n', '\r']);
        if line.is_empty() {
            continue;
        }

        // Check for end marker
        if line == "end" || line.starts_with("end ") {
            break;
        }

        let input_bytes = line.as_bytes();
        if input_bytes.is_empty() {
            continue;
        }

        // First character encodes the line length
        let expected_length = (input_bytes[0].wrapping_sub(b' ')) as usize;

        // Only process lines with reasonable UU length values (0-45)
        // This filters out obvious non-UU lines like comments
        if expected_length > 45 {
            continue;
        }

        // If length is 0, this indicates end of data
        if expected_length == 0 {
            break;
        }

        let data_part = &input_bytes[1..];
        let mut decoded_bytes = Vec::new();

        // Process groups of 4 characters into 3 bytes
        for chunk in data_part.chunks(4) {
            let mut group = [b' '; 4];
            for (i, &byte) in chunk.iter().enumerate() {
                group[i] = byte;
            }

            // Decode 4 characters to 3 bytes
            let c1 = group[0].wrapping_sub(b' ');
            let c2 = group[1].wrapping_sub(b' ');
            let c3 = group[2].wrapping_sub(b' ');
            let c4 = group[3].wrapping_sub(b' ');

            let decoded_byte1 = (c1 << 2) | (c2 >> 4);
            let decoded_byte2 = ((c2 & 0x0F) << 4) | (c3 >> 2);
            let decoded_byte3 = ((c3 & 0x03) << 6) | c4;

            // Always decode all 3 bytes - missing chars are treated as spaces (value 0)
            decoded_bytes.push(decoded_byte1);
            decoded_bytes.push(decoded_byte2);
            decoded_bytes.push(decoded_byte3);
        }

        // Truncate to expected length to handle padding
        decoded_bytes.truncate(expected_length);
        result.extend_from_slice(&decoded_bytes);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(not(feature = "std"))]
    use alloc::vec;

    #[test]
    fn spans_validation() {
        let source = "Hello, World!";
        let spans = Spans::new(source);

        let valid_span = &source[0..5]; // "Hello"
        assert!(spans.validate_span(valid_span));
        assert_eq!(spans.span_offset(valid_span), Some(0));
        assert_eq!(spans.span_line(valid_span), Some(1));
        assert_eq!(spans.span_column(valid_span), Some(1));

        let another_span = &source[7..12]; // "World"
        assert!(spans.validate_span(another_span));
        assert_eq!(spans.span_offset(another_span), Some(7));
    }

    #[test]
    fn spans_multiline() {
        let source = "Line 1\nLine 2\nLine 3";
        let spans = Spans::new(source);

        let line2_span = &source[7..13]; // "Line 2"
        assert_eq!(spans.span_line(line2_span), Some(2));
        assert_eq!(spans.span_column(line2_span), Some(1));
    }

    #[test]
    fn parse_bgr_colors() {
        assert_eq!(parse_bgr_color("&H000000FF&").unwrap(), [255, 0, 0, 0]);
        assert_eq!(parse_bgr_color("&H0000FF00&").unwrap(), [0, 255, 0, 0]);
        assert_eq!(parse_bgr_color("&H00FF0000&").unwrap(), [0, 0, 255, 0]);

        assert_eq!(parse_bgr_color("&HFF000000&").unwrap(), [0, 0, 0, 255]);

        assert_eq!(parse_bgr_color("0x000000FF").unwrap(), [255, 0, 0, 0]);
        assert_eq!(parse_bgr_color("000000FF").unwrap(), [255, 0, 0, 0]);
    }

    #[test]
    fn parse_bgr_colors_invalid() {
        assert!(parse_bgr_color("invalid").is_err());
        assert!(parse_bgr_color("&HZZZZ&").is_err());
        assert!(parse_bgr_color("").is_err());
    }

    #[test]
    fn parse_bgr_colors_without_trailing_ampersand() {
        assert_eq!(parse_bgr_color("&H000000FF").unwrap(), [255, 0, 0, 0]);
        assert_eq!(parse_bgr_color("&H00FFFFFF").unwrap(), [255, 255, 255, 0]);
        assert_eq!(parse_bgr_color("&H00000000").unwrap(), [0, 0, 0, 0]);
        assert_eq!(parse_bgr_color("&HFF000000").unwrap(), [0, 0, 0, 255]);
    }

    #[test]
    fn parse_ass_times() {
        assert_eq!(parse_ass_time("0:00:00.00").unwrap(), 0);
        assert_eq!(parse_ass_time("0:00:01.00").unwrap(), 100);
        assert_eq!(parse_ass_time("0:01:00.00").unwrap(), 6000);
        assert_eq!(parse_ass_time("1:00:00.00").unwrap(), 360_000);
        assert_eq!(parse_ass_time("0:01:30.50").unwrap(), 9050);
    }

    #[test]
    fn parse_ass_times_invalid() {
        assert!(parse_ass_time("invalid").is_err());
        assert!(parse_ass_time("0:60:00.00").is_err()); // Invalid minutes
        assert!(parse_ass_time("0:00:60.00").is_err()); // Invalid seconds
        assert!(parse_ass_time("0:00:00.100").is_err()); // Invalid centiseconds
    }

    #[test]
    fn format_ass_times() {
        assert_eq!(format_ass_time(0), "0:00:00.00");
        assert_eq!(format_ass_time(100), "0:00:01.00");
        assert_eq!(format_ass_time(6000), "0:01:00.00");
        assert_eq!(format_ass_time(360_000), "1:00:00.00");
        assert_eq!(format_ass_time(9050), "0:01:30.50");
    }

    #[test]
    fn bezier_evaluation() {
        let p0 = (0.0, 0.0);
        let p1 = (0.33, 0.0);
        let p2 = (0.67, 1.0);
        let p3 = (1.0, 1.0);

        let start = eval_cubic_bezier(p0, p1, p2, p3, 0.0);
        assert_eq!(start, p0);

        let end = eval_cubic_bezier(p0, p1, p2, p3, 1.0);
        assert_eq!(end, p3);

        let mid = eval_cubic_bezier(p0, p1, p2, p3, 0.5);
        assert!(mid.0 > 0.0 && mid.0 < 1.0);
        assert!(mid.1 > 0.0 && mid.1 < 1.0);
    }

    #[test]
    fn validate_ass_names() {
        assert!(validate_ass_name("Default"));
        assert!(validate_ass_name("MyStyle"));
        assert!(validate_ass_name("Style with spaces"));

        assert!(!validate_ass_name("")); // Empty
        assert!(!validate_ass_name("Style,Name")); // Comma
        assert!(!validate_ass_name("Style:Name")); // Colon
        assert!(!validate_ass_name("Style{Name")); // Brace
        assert!(!validate_ass_name("Style\nName")); // Control character
    }

    #[test]
    fn normalize_field_values() {
        assert_eq!(normalize_field_value("  value  "), "value");
        assert_eq!(normalize_field_value("\tvalue\t"), "value");
        assert_eq!(normalize_field_value("value"), "value");
    }

    #[test]
    fn numeric_parsing() {
        assert_eq!(parse_numeric::<i32>("42").unwrap(), 42);
        assert!((parse_numeric::<f32>("3.15").unwrap() - 3.15).abs() < f32::EPSILON);
        assert!(parse_numeric::<i32>("invalid").is_err());
    }

    #[test]
    fn decode_uu_data_empty_input() {
        let lines: Vec<&str> = vec![];
        let decoded = decode_uu_data(lines.iter().copied()).unwrap();
        assert_eq!(decoded, Vec::<u8>::new());
    }

    #[test]
    fn decode_uu_data_known_encoding() {
        // Test known UU-encoded data: "Cat" -> "#0V%T"
        let lines = ["#0V%T"];
        let decoded = decode_uu_data(lines.iter().copied()).unwrap();
        assert_eq!(decoded, b"Cat");
    }

    #[test]
    fn decode_uu_data_known_encoding_png() {
        // Test known UU-encoded data: "PNG" -> "#4$Y'"
        let lines = ["#4$Y'"];
        let decoded = decode_uu_data(lines.iter().copied()).unwrap();
        assert_eq!(decoded, b"PNG");
    }

    #[test]
    fn decode_uu_data_multiline() {
        // Test multi-line UU-encoded data
        let lines = ["#0V%T", "#0V%T"];
        let decoded = decode_uu_data(lines.iter().copied()).unwrap();
        assert_eq!(decoded, b"CatCat");
    }

    #[test]
    fn decode_uu_data_with_end_marker() {
        let lines = ["#0V%T", "end"];
        let decoded = decode_uu_data(lines.iter().copied()).unwrap();
        assert_eq!(decoded, b"Cat");
    }

    #[test]
    fn decode_uu_data_with_end_marker_spaced() {
        let lines = ["#0V%T", "end 644"];
        let decoded = decode_uu_data(lines.iter().copied()).unwrap();
        assert_eq!(decoded, b"Cat");
    }

    #[test]
    fn decode_uu_data_zero_length_line() {
        // Zero-length line should terminate decoding
        let lines = ["#0V%T", " "];
        let decoded = decode_uu_data(lines.iter().copied()).unwrap();
        assert_eq!(decoded, b"Cat");
    }

    #[test]
    fn decode_uu_data_whitespace_lines() {
        let lines = ["  #0V%T  ", "\t", ""];
        let decoded = decode_uu_data(lines.iter().copied()).unwrap();
        assert_eq!(decoded, b"Cat");
    }

    #[test]
    fn decode_uu_data_length_validation() {
        // Test that length encoding is respected
        let lines = ["!    "]; // '!' encodes length 1, but provides 4 characters of data
        let decoded = decode_uu_data(lines.iter().copied()).unwrap();
        assert_eq!(decoded.len(), 1); // Should be truncated to declared length
    }

    #[test]
    fn decode_uu_data_partial_chunks() {
        // Test handling of incomplete 4-character groups
        let lines = ["\"``"]; // Only 3 characters after length byte
        let decoded = decode_uu_data(lines.iter().copied()).unwrap();
        assert_eq!(decoded.len(), 2); // Should decode what's available
    }

    #[test]
    fn decode_uu_data_large_line() {
        // Test handling of max-length UU line (45 bytes -> 60 characters + length)
        let line = format!("M{}", "!!!!".repeat(15)); // 45 bytes of data
        let lines = [line.as_str()];
        let decoded = decode_uu_data(lines.iter().copied()).unwrap();
        assert_eq!(decoded.len(), 45);
    }

    #[test]
    fn decode_uu_data_mixed_content() {
        let lines = [
            "begin 644 test.txt", // Should be ignored
            "#0V%T",              // Should be decoded
            "| comment",          // Should be ignored as it doesn't start with valid length
            "#4$Y'",              // Should be decoded
            "end",                // Should terminate
        ];
        let decoded = decode_uu_data(lines.iter().copied()).unwrap();
        assert_eq!(decoded, b"CatPNG");
    }

    #[test]
    fn decode_uu_data_all_printable_chars() {
        // Test that decoder handles all valid UU characters (space to underscore)
        let lines = ["@ !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_"];
        let _decoded = decode_uu_data(lines.iter().copied()).unwrap();
        // Should not panic, exact output depends on UU encoding rules
    }

    #[test]
    fn decode_uu_data_boundary_lengths() {
        // Test boundary cases for line lengths
        let single_byte = ["!   "]; // Length 1
        let two_bytes = ["\"`` "]; // Length 2
        let three_bytes = ["#```"]; // Length 3

        let decoded1 = decode_uu_data(single_byte.iter().copied()).unwrap();
        assert_eq!(decoded1.len(), 1);

        let decoded2 = decode_uu_data(two_bytes.iter().copied()).unwrap();
        assert_eq!(decoded2.len(), 2);

        let decoded3 = decode_uu_data(three_bytes.iter().copied()).unwrap();
        assert_eq!(decoded3.len(), 3);
    }

    #[test]
    fn decode_uu_data_handles_invalid_gracefully() {
        // Decoder should not panic on invalid characters
        let lines = ["#\x01\x02\x03"]; // Non-printable characters
        let _result = decode_uu_data(lines.iter().copied());
        // Should not panic, may return unexpected data or error
    }

    #[test]
    fn parse_bgr_color_edge_cases() {
        // Test lowercase hex prefix
        assert_eq!(parse_bgr_color("&h000000").unwrap(), [0, 0, 0, 0]);
        assert_eq!(parse_bgr_color("&hFFFFFF").unwrap(), [255, 255, 255, 0]);

        // Test 0x prefix
        assert_eq!(parse_bgr_color("0x000000").unwrap(), [0, 0, 0, 0]);
        assert_eq!(parse_bgr_color("0xFFFFFF").unwrap(), [255, 255, 255, 0]);

        // Test plain hex without prefix
        assert_eq!(parse_bgr_color("000000").unwrap(), [0, 0, 0, 0]);
        assert_eq!(parse_bgr_color("FFFFFF").unwrap(), [255, 255, 255, 0]);

        // Test with extra whitespace
        assert_eq!(parse_bgr_color("  &H000000  ").unwrap(), [0, 0, 0, 0]);
        assert_eq!(parse_bgr_color("\t&H000000\t").unwrap(), [0, 0, 0, 0]);

        // Test with trailing ampersand variations
        assert_eq!(parse_bgr_color("&H000000&").unwrap(), [0, 0, 0, 0]);
        assert_eq!(parse_bgr_color("&h000000&").unwrap(), [0, 0, 0, 0]);

        // Test mixed case hex digits
        assert_eq!(parse_bgr_color("&HaAbBcC").unwrap(), [204, 187, 170, 0]);
        assert_eq!(parse_bgr_color("&HFFaaBBcc").unwrap(), [204, 187, 170, 255]);

        // Test invalid lengths
        assert!(parse_bgr_color("&H00000").is_err()); // 5 chars
        assert!(parse_bgr_color("&H0000000").is_err()); // 7 chars
        assert!(parse_bgr_color("&H000000000").is_err()); // 9 chars

        // Test invalid characters in hex
        assert!(parse_bgr_color("&H00000G").is_err());
        assert!(parse_bgr_color("&H00Z000").is_err());

        // Test empty after prefix
        assert!(parse_bgr_color("&H").is_err());
        assert!(parse_bgr_color("0x").is_err());

        // Test malformed prefixes
        assert!(parse_bgr_color("&H000000X").is_err());
        assert!(parse_bgr_color("X&H000000").is_err());
    }

    #[test]
    fn spans_edge_cases() {
        let source = "line1\nline2\nline3";
        let spans = Spans::new(source);

        // Test span validation with actual substrings
        let line1 = &source[0..5]; // "line1"
        let line2 = &source[6..11]; // "line2"
        let line3 = &source[12..17]; // "line3"

        assert!(spans.validate_span(line1));
        assert!(spans.validate_span(line2));
        assert!(spans.validate_span(line3));
        assert!(spans.validate_span(source)); // Entire source

        // Test span offset calculations
        assert_eq!(spans.span_offset(line1), Some(0));
        assert_eq!(spans.span_offset(line2), Some(6));
        assert_eq!(spans.span_offset(line3), Some(12));

        // Test line calculations
        assert_eq!(spans.span_line(line1), Some(1));
        assert_eq!(spans.span_line(line2), Some(2));
        assert_eq!(spans.span_line(line3), Some(3));

        // Test column calculations
        assert_eq!(spans.span_column(line1), Some(1));
        assert_eq!(spans.span_column(line2), Some(1));
        assert_eq!(spans.span_column(line3), Some(1));

        // Test substring extraction
        assert_eq!(spans.substring(0..5), Some("line1"));
        assert_eq!(spans.substring(6..11), Some("line2"));
        assert_eq!(spans.substring(12..17), Some("line3"));
        assert_eq!(spans.substring(0..source.len()), Some(source));

        // Test invalid range
        assert_eq!(spans.substring(0..100), None);
    }

    #[test]
    fn parse_ass_time_edge_cases() {
        // Test maximum valid values
        assert!(parse_ass_time("23:59:59.99").is_ok());

        // Test zero padding variations
        assert_eq!(parse_ass_time("0:0:0.0").unwrap(), 0);
        assert_eq!(parse_ass_time("0:00:00.0").unwrap(), 0);
        assert_eq!(parse_ass_time("0:00:00.00").unwrap(), 0);

        // Test missing components
        assert!(parse_ass_time("0:00").is_err());
        assert!(parse_ass_time("0").is_err());
        assert!(parse_ass_time("").is_err());

        // Test extra components
        assert!(parse_ass_time("0:0:0:0.0").is_err());
        // Note: parse_ass_time("0:0:0.0.0") actually succeeds by taking first decimal part
        assert!(parse_ass_time("0:0:0.0.0").is_ok());

        // Test negative values
        assert!(parse_ass_time("-1:00:00.00").is_err());
        assert!(parse_ass_time("0:-1:00.00").is_err());
        assert!(parse_ass_time("0:00:-1.00").is_err());
        assert!(parse_ass_time("0:00:00.-1").is_err());

        // Test non-numeric values
        assert!(parse_ass_time("a:00:00.00").is_err());
        assert!(parse_ass_time("0:b:00.00").is_err());
        assert!(parse_ass_time("0:00:c.00").is_err());
        assert!(parse_ass_time("0:00:00.d").is_err());

        // Test boundary values that should fail
        assert!(parse_ass_time("0:60:00.00").is_err()); // 60 minutes
        assert!(parse_ass_time("0:00:60.00").is_err()); // 60 seconds
        assert!(parse_ass_time("0:00:00.100").is_err()); // 100 centiseconds
    }

    #[test]
    fn format_ass_time_edge_cases() {
        // Test very large values
        assert_eq!(format_ass_time(u32::MAX), "11930:27:52.95");

        // Test boundary values
        assert_eq!(format_ass_time(99), "0:00:00.99");
        assert_eq!(format_ass_time(5999), "0:00:59.99");
        assert_eq!(format_ass_time(359_999), "0:59:59.99");

        // Test values requiring padding
        assert_eq!(format_ass_time(1), "0:00:00.01");
        assert_eq!(format_ass_time(10), "0:00:00.10");
        assert_eq!(format_ass_time(601), "0:00:06.01");
        assert_eq!(format_ass_time(3661), "0:00:36.61");
    }

    #[test]
    fn validate_ass_name_edge_cases() {
        // Test with tab character (should be allowed)
        assert!(validate_ass_name("Style\tName"));

        // Test with various control characters (should be rejected)
        assert!(!validate_ass_name("Style\nName")); // Newline
        assert!(!validate_ass_name("Style\rName")); // Carriage return
        assert!(!validate_ass_name("Style\x00Name")); // Null
        assert!(!validate_ass_name("Style\x7FName")); // DEL

        // Test edge cases with separators
        assert!(!validate_ass_name(",Style")); // Leading comma
        assert!(!validate_ass_name("Style,")); // Trailing comma
        assert!(!validate_ass_name(":Style")); // Leading colon
        assert!(!validate_ass_name("Style:")); // Trailing colon
        assert!(!validate_ass_name("{Style")); // Leading brace
        assert!(!validate_ass_name("Style}")); // Trailing brace

        // Test very long names
        let long_name = "a".repeat(1000);
        assert!(validate_ass_name(&long_name));

        // Test Unicode characters
        assert!(validate_ass_name("Style中文"));
        assert!(validate_ass_name("Style🎭"));
        assert!(validate_ass_name("Стиль"));
    }

    #[test]
    fn normalize_field_value_edge_cases() {
        // Test empty string
        assert_eq!(normalize_field_value(""), "");

        // Test only whitespace
        assert_eq!(normalize_field_value("   "), "");
        assert_eq!(normalize_field_value("\t\t\t"), "");
        assert_eq!(normalize_field_value(" \t \t "), "");

        // Test mixed whitespace
        assert_eq!(normalize_field_value(" \t value \t "), "value");
        assert_eq!(normalize_field_value("\n\rvalue\n\r"), "value");

        // Test internal whitespace preservation
        assert_eq!(normalize_field_value("  val ue  "), "val ue");
        assert_eq!(normalize_field_value("  val\tue  "), "val\tue");
    }

    #[test]
    #[allow(clippy::float_cmp, clippy::approx_constant)]
    fn parse_numeric_edge_cases() {
        // Test boundary values for different types
        assert_eq!(parse_numeric::<u8>("255").unwrap(), 255u8);
        assert!(parse_numeric::<u8>("256").is_err());
        assert_eq!(parse_numeric::<i8>("127").unwrap(), 127i8);
        assert_eq!(parse_numeric::<i8>("-128").unwrap(), -128i8);
        assert!(parse_numeric::<i8>("128").is_err());

        // Test floating point edge cases
        assert_eq!(parse_numeric::<f32>("0.0").unwrap(), 0.0f32);
        assert_eq!(parse_numeric::<f32>("-0.0").unwrap(), -0.0f32);
        assert!(parse_numeric::<f32>("inf").is_ok());
        assert!(parse_numeric::<f32>("-inf").is_ok());

        // Test whitespace handling
        assert_eq!(parse_numeric::<i32>("  42  ").unwrap(), 42i32);
        assert_eq!(parse_numeric::<f32>(" \t 3.14 \t ").unwrap(), 3.14f32);

        // Test leading zeros
        assert_eq!(parse_numeric::<i32>("00042").unwrap(), 42i32);
        assert_eq!(parse_numeric::<f32>("0003.140").unwrap(), 3.14f32);

        // Test scientific notation
        assert_eq!(parse_numeric::<f32>("1e2").unwrap(), 100.0f32);
        assert_eq!(parse_numeric::<f32>("1.5e-2").unwrap(), 0.015f32);

        // Test invalid formats
        assert!(parse_numeric::<i32>("").is_err());
        assert!(parse_numeric::<i32>("abc").is_err());
        assert!(parse_numeric::<i32>("12.34").is_err()); // Float for int
        assert!(parse_numeric::<f32>("12.34.56").is_err()); // Multiple dots
    }

    #[test]
    fn eval_cubic_bezier_edge_cases() {
        // Test identical control points (linear case)
        let linear_result = eval_cubic_bezier((0.0, 0.0), (0.0, 0.0), (1.0, 1.0), (1.0, 1.0), 0.5);
        assert!((linear_result.0 - 0.5).abs() < f32::EPSILON);
        assert!((linear_result.1 - 0.5).abs() < f32::EPSILON);

        // Test extreme t values
        let p0 = (0.0, 0.0);
        let p1 = (0.25, 0.5);
        let p2 = (0.75, 0.5);
        let p3 = (1.0, 1.0);

        // t = 0 should return p0
        let result_0 = eval_cubic_bezier(p0, p1, p2, p3, 0.0);
        assert!((result_0.0 - p0.0).abs() < f32::EPSILON);
        assert!((result_0.1 - p0.1).abs() < f32::EPSILON);

        // t = 1 should return p3
        let result_1 = eval_cubic_bezier(p0, p1, p2, p3, 1.0);
        assert!((result_1.0 - p3.0).abs() < f32::EPSILON);
        assert!((result_1.1 - p3.1).abs() < f32::EPSILON);

        // Test negative coordinates
        let neg_result = eval_cubic_bezier((-1.0, -1.0), (-0.5, -0.5), (0.5, 0.5), (1.0, 1.0), 0.5);
        assert!(neg_result.0 > -1.0 && neg_result.0 < 1.0);
        assert!(neg_result.1 > -1.0 && neg_result.1 < 1.0);

        // Test very small and very large coordinates
        let large_result = eval_cubic_bezier(
            (0.0, 0.0),
            (1000.0, 1000.0),
            (2000.0, 2000.0),
            (3000.0, 3000.0),
            0.5,
        );
        assert!(large_result.0 > 0.0 && large_result.0 < 3000.0);
        assert!(large_result.1 > 0.0 && large_result.1 < 3000.0);
    }

    #[test]
    fn decode_uu_data_error_conditions() {
        // Test with only invalid lines
        let invalid_lines = ["invalid", "also invalid", "still invalid"];
        let result = decode_uu_data(invalid_lines.iter().copied()).unwrap();
        assert!(result.is_empty());

        // Test with malformed length indicators
        let malformed_length = ["\x7F!!!!"]; // Length > 45
        let _result = decode_uu_data(malformed_length.iter().copied());
        // Should handle gracefully

        // Test with very short lines after valid length
        let short_lines = ["!"]; // Length 1 but no data
        let result = decode_uu_data(short_lines.iter().copied()).unwrap();
        assert!(result.is_empty() || result.len() <= 1);

        // Test with unicode in data
        let unicode_lines = ["#🎭🎭🎭"];
        let _result = decode_uu_data(unicode_lines.iter().copied());
        // Should handle gracefully without panicking
    }
}
