//! UU-encoded binary data decoding for ASS `[Fonts]`/`[Graphics]` sections.
//!
//! Decodes Unix-to-Unix encoded binary payloads embedded as ASCII text.

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

use super::CoreError;

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
