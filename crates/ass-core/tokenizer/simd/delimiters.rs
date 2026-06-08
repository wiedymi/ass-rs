//! SIMD-accelerated delimiter scanning utilities
//!
//! Provides vectorized and scalar implementations for locating common ASS
//! delimiters within input text, with automatic fallback to the scalar path
//! when SIMD is unavailable.

use wide::u8x16;

/// Scan for delimiter characters using SIMD acceleration
///
/// Searches for common ASS delimiters (comma, colon, braces, brackets)
/// in the input text using vectorized operations when available.
///
/// # Arguments
///
/// * `text` - Input text to scan for delimiters
///
/// # Returns
///
/// Byte offset of first delimiter found, or None if no delimiters present.
///
/// # Example
///
/// ```rust
/// use ass_core::tokenizer::simd::scan_delimiters;
///
/// let text = "name: value, next";
/// let offset = scan_delimiters(text).unwrap();
/// assert_eq!(offset, 4); // Position of ':'
/// ```
#[must_use]
pub fn scan_delimiters(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();

    #[cfg(feature = "simd")]
    {
        if bytes.len() >= 16 {
            return scan_delimiters_simd_impl(bytes);
        }
    }

    scan_delimiters_scalar(bytes)
}

/// SIMD implementation for delimiter scanning
#[cfg(feature = "simd")]
fn scan_delimiters_simd_impl(bytes: &[u8]) -> Option<usize> {
    let delim_colon = u8x16::splat(b':');
    let delim_comma = u8x16::splat(b',');
    let delim_open_brace = u8x16::splat(b'{');
    let delim_close_brace = u8x16::splat(b'}');
    let delim_open_bracket = u8x16::splat(b'[');
    let delim_close_bracket = u8x16::splat(b']');
    let delim_newline = u8x16::splat(b'\n');
    let delim_carriage = u8x16::splat(b'\r');

    let chunks = bytes.chunks_exact(16);
    let remainder = chunks.remainder();

    for (chunk_idx, chunk) in chunks.enumerate() {
        let chunk_array: [u8; 16] = chunk.try_into().unwrap();
        let simd_chunk = u8x16::from(chunk_array);

        let mask = simd_chunk.cmp_eq(delim_colon)
            | simd_chunk.cmp_eq(delim_comma)
            | simd_chunk.cmp_eq(delim_open_brace)
            | simd_chunk.cmp_eq(delim_close_brace)
            | simd_chunk.cmp_eq(delim_open_bracket)
            | simd_chunk.cmp_eq(delim_close_bracket)
            | simd_chunk.cmp_eq(delim_newline)
            | simd_chunk.cmp_eq(delim_carriage);

        let mask_bits = mask.move_mask();
        if mask_bits != 0 {
            let first_match = mask_bits.trailing_zeros() as usize;
            return Some(chunk_idx * 16 + first_match);
        }
    }

    if !remainder.is_empty() {
        let remainder_offset = bytes.len() - remainder.len();
        if let Some(pos) = scan_delimiters_scalar(remainder) {
            return Some(remainder_offset + pos);
        }
    }

    None
}

/// Scalar implementation for delimiter scanning
fn scan_delimiters_scalar(bytes: &[u8]) -> Option<usize> {
    for (i, &byte) in bytes.iter().enumerate() {
        match byte {
            b':' | b',' | b'{' | b'}' | b'[' | b']' | b'\n' | b'\r' => return Some(i),
            _ => {}
        }
    }
    None
}
