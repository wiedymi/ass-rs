//! SIMD-accelerated UTF-8 validation utilities
//!
//! Provides batch validation of UTF-8 byte sequences using vectorized
//! operations, with automatic fallback to the scalar path when SIMD is
//! unavailable.

use crate::utils::CoreError;
use wide::u8x16;

/// Batch validate UTF-8 sequences using SIMD
///
/// Validates multiple bytes at once for UTF-8 compliance.
/// Provides faster validation for large text blocks.
/// Validate UTF-8 encoding of byte slice using batch processing
///
/// # Errors
///
/// Returns an error if the byte slice contains invalid UTF-8 sequences.
pub fn validate_utf8_batch(bytes: &[u8]) -> Result<(), CoreError> {
    #[cfg(feature = "simd")]
    {
        if bytes.len() >= 16 {
            return validate_utf8_simd_impl(bytes);
        }
    }

    validate_utf8_scalar(bytes)
}

/// SIMD implementation for UTF-8 validation
#[cfg(feature = "simd")]
fn validate_utf8_simd_impl(bytes: &[u8]) -> Result<(), CoreError> {
    let chunks = bytes.chunks_exact(16);
    let remainder = chunks.remainder();

    for chunk in chunks {
        let chunk_array: [u8; 16] = chunk.try_into().unwrap();
        let simd_chunk = u8x16::from(chunk_array);
        let ascii_mask = u8x16::splat(0x80);

        let has_non_ascii = (simd_chunk & ascii_mask).move_mask();
        if has_non_ascii != 0 {
            return validate_utf8_scalar(bytes);
        }
    }

    validate_utf8_scalar(remainder)
}

/// Scalar UTF-8 validation implementation
fn validate_utf8_scalar(bytes: &[u8]) -> Result<(), CoreError> {
    core::str::from_utf8(bytes)
        .map(|_| ())
        .map_err(|e| CoreError::utf8_error(e.valid_up_to(), format!("{e}")))
}
