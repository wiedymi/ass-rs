//! Encoding detection routines for ASS subtitle content.
//!
//! Analyzes raw byte content to determine the most likely text encoding,
//! combining BOM detection, UTF-8 validation, and ASS-specific content
//! heuristics with confidence scoring.

use super::super::bom::detect_bom;
use super::EncodingInfo;
use alloc::string::ToString;
use core::str;

/// Detect text encoding with confidence scoring
///
/// Analyzes byte content to determine the most likely encoding.
/// Focuses on encodings commonly used in ASS subtitle files and
/// provides confidence scoring based on content analysis.
///
/// # Arguments
///
/// * `bytes` - Byte sequence to analyze
///
/// # Returns
///
/// `EncodingInfo` with detected encoding and confidence level
///
/// # Examples
///
/// ```rust
/// # use ass_core::utils::utf8::detect_encoding;
/// let text = "[Script Info]\nTitle: Test";
/// let encoding = detect_encoding(text.as_bytes());
/// assert_eq!(encoding.encoding, "UTF-8");
/// assert!(encoding.confidence > 0.8);
/// ```
#[must_use]
pub fn detect_encoding(bytes: &[u8]) -> EncodingInfo {
    // Check for BOM first - gives us certainty about encoding
    if let Some((bom_type, _)) = detect_bom(bytes) {
        return EncodingInfo::with_bom(
            bom_type.encoding_name().to_string(),
            1.0, // BOM gives us certainty
            bom_type,
        );
    }

    // Try UTF-8 validation
    str::from_utf8(bytes).map_or_else(
        |_| detect_non_utf8_encoding(bytes),
        |text| {
            let confidence = if is_likely_ass_content(text) {
                0.95 // High confidence for ASS-like content
            } else {
                0.8 // Still likely UTF-8 but less certain
            };
            EncodingInfo::new("UTF-8".to_string(), confidence)
        },
    )
}

/// Check if text content contains patterns typical of ASS subtitle files
///
/// Analyzes text for ASS-specific patterns like section headers,
/// field names, and content structure to increase confidence
/// in encoding detection.
///
/// # Arguments
///
/// * `text` - Text content to analyze
///
/// # Returns
///
/// `true` if content appears to be ASS subtitle format
#[must_use]
pub fn is_likely_ass_content(text: &str) -> bool {
    // Check for ASS section headers
    if text.contains("[Script Info]")
        || text.contains("[V4+ Styles]")
        || text.contains("[Events]")
        || text.contains("[Fonts]")
        || text.contains("[Graphics]")
    {
        return true;
    }

    // Check for common ASS field names
    if text.contains("Dialogue:")
        || text.contains("Comment:")
        || text.contains("ScriptType:")
        || text.contains("PlayRes")
        || text.contains("Style:")
    {
        return true;
    }

    false
}

/// Attempt to detect non-UTF-8 encodings commonly used in older ASS files
///
/// Provides fallback detection for files that aren't valid UTF-8,
/// focusing on legacy encodings commonly used in subtitle files.
///
/// # Arguments
///
/// * `bytes` - Byte sequence that failed UTF-8 validation
///
/// # Returns
///
/// `EncodingInfo` with best guess for the encoding
fn detect_non_utf8_encoding(bytes: &[u8]) -> EncodingInfo {
    let has_extended_ascii = bytes.iter().any(|&b| b >= 0x80);

    if has_extended_ascii {
        // Common legacy encoding for subtitle files
        EncodingInfo::new("Windows-1252".to_string(), 0.6)
    } else {
        // Pure ASCII is safe to assume
        EncodingInfo::new("ASCII".to_string(), 0.9)
    }
}
