//! SIMD-accelerated hexadecimal parsing utilities
//!
//! Provides vectorized validation and scalar parsing of hexadecimal strings
//! commonly found in ASS files, such as color values and embedded data, with
//! automatic fallback to the scalar path when SIMD is unavailable.

use wide::u8x16;

/// Parse hexadecimal string to u32 using SIMD when available
///
/// Optimized parsing of hex values commonly found in ASS files
/// such as color values (&H00FF00FF&) and embedded data.
///
/// # Arguments
///
/// * `hex_str` - Hexadecimal string (without 0x or &H prefix)
///
/// # Returns
///
/// Parsed u32 value or None if invalid hex format.
///
/// # Example
///
/// ```rust
/// use ass_core::tokenizer::simd::parse_hex_u32;
///
/// let value = parse_hex_u32("00FF00FF").unwrap();
/// assert_eq!(value, 0x00FF00FF);
/// ```
#[must_use]
pub fn parse_hex_u32(hex_str: &str) -> Option<u32> {
    if hex_str.is_empty() || hex_str.len() > 8 {
        return None;
    }

    #[cfg(feature = "simd")]
    {
        if hex_str.len() >= 4 {
            return parse_hex_simd_impl(hex_str);
        }
    }

    parse_hex_scalar(hex_str)
}

/// SIMD implementation for hex parsing
#[cfg(feature = "simd")]
fn parse_hex_simd_impl(hex_str: &str) -> Option<u32> {
    let bytes = hex_str.as_bytes();

    // For hex strings <= 8 chars, we can process them directly with scalar
    if bytes.len() <= 8 {
        return parse_hex_scalar_direct(bytes);
    }

    // For longer strings, validate with SIMD then fall back to scalar
    let chunks = bytes.chunks_exact(16);
    let remainder = chunks.remainder();

    for chunk in chunks {
        let chunk_array: [u8; 16] = chunk.try_into().unwrap();
        let simd_chunk = u8x16::from(chunk_array);

        if !validate_hex_chars_simd(simd_chunk) {
            return None;
        }
    }

    for &byte in remainder {
        if !byte.is_ascii_hexdigit() {
            return None;
        }
    }

    parse_hex_scalar(hex_str)
}

/// Validate hex characters using SIMD
#[cfg(feature = "simd")]
fn validate_hex_chars_simd(simd_chunk: u8x16) -> bool {
    let mut valid_mask = u8x16::splat(0);

    // Check for digits 0-9
    for digit in b'0'..=b'9' {
        valid_mask |= simd_chunk.cmp_eq(u8x16::splat(digit));
    }

    // Check for uppercase A-F
    for hex_char in b'A'..=b'F' {
        valid_mask |= simd_chunk.cmp_eq(u8x16::splat(hex_char));
    }

    // Check for lowercase a-f
    for hex_char in b'a'..=b'f' {
        valid_mask |= simd_chunk.cmp_eq(u8x16::splat(hex_char));
    }

    valid_mask.move_mask() == 0xFFFF
}

/// Direct scalar hex parsing for strings <= 8 characters
#[cfg(feature = "simd")]
fn parse_hex_scalar_direct(bytes: &[u8]) -> Option<u32> {
    if bytes.is_empty() || bytes.len() > 8 {
        return None;
    }

    let mut result: u32 = 0;

    for &byte in bytes {
        let digit_value = match byte {
            b'0'..=b'9' => byte - b'0',
            b'A'..=b'F' => byte - b'A' + 10,
            b'a'..=b'f' => byte - b'a' + 10,
            _ => return None,
        };

        result = result
            .checked_mul(16)?
            .checked_add(u32::from(digit_value))?;
    }

    Some(result)
}

/// Scalar hex parsing implementation
fn parse_hex_scalar(hex_str: &str) -> Option<u32> {
    u32::from_str_radix(hex_str, 16).ok()
}
