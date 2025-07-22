//! Encoding detection utilities for ASS subtitle files
//!
//! Provides functionality for detecting text encodings, analyzing content
//! patterns, and validating encoding assumptions. Focuses on encodings
//! commonly used in ASS subtitle files with confidence scoring.
//!
//! # Examples
//!
//! ```rust
//! use ass_core::utils::utf8::{detect_encoding, EncodingInfo};
//!
//! let text = "[Script Info]\nTitle: Test";
//! let encoding = detect_encoding(text.as_bytes());
//! assert_eq!(encoding.encoding, "UTF-8");
//! assert!(encoding.confidence > 0.8);
//! ```

use super::bom::{detect_bom, BomType};
use alloc::{string::String, string::ToString};
use core::str;

/// Detected text encoding information with confidence scoring
///
/// Contains the results of encoding detection analysis including
/// the detected encoding name, confidence level, and BOM information.
#[derive(Debug, Clone, PartialEq)]
pub struct EncodingInfo {
    /// Detected encoding name (e.g., "UTF-8", "Windows-1252")
    pub encoding: String,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f32,
    /// Whether a BOM was detected
    pub has_bom: bool,
    /// BOM type if detected
    pub bom_type: Option<BomType>,
    /// Whether the text appears to be valid in this encoding
    pub is_valid: bool,
}

impl EncodingInfo {
    /// Create new encoding info with basic parameters
    ///
    /// # Arguments
    ///
    /// * `encoding` - Name of the detected encoding
    /// * `confidence` - Confidence level (0.0 to 1.0)
    pub fn new(encoding: String, confidence: f32) -> Self {
        Self {
            encoding,
            confidence,
            has_bom: false,
            bom_type: None,
            is_valid: true,
        }
    }

    /// Create encoding info with BOM information
    ///
    /// # Arguments
    ///
    /// * `encoding` - Name of the detected encoding
    /// * `confidence` - Confidence level (0.0 to 1.0)
    /// * `bom_type` - Type of BOM detected
    pub fn with_bom(encoding: String, confidence: f32, bom_type: BomType) -> Self {
        Self {
            encoding,
            confidence,
            has_bom: true,
            bom_type: Some(bom_type),
            is_valid: true,
        }
    }
}

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
/// EncodingInfo with detected encoding and confidence level
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
    match str::from_utf8(bytes) {
        Ok(text) => {
            let confidence = if is_likely_ass_content(text) {
                0.95 // High confidence for ASS-like content
            } else {
                0.8 // Still likely UTF-8 but less certain
            };
            EncodingInfo::new("UTF-8".to_string(), confidence)
        }
        Err(_) => detect_non_utf8_encoding(bytes),
    }
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
/// EncodingInfo with best guess for the encoding
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encoding_info_creation() {
        let info = EncodingInfo::new("UTF-8".to_string(), 0.95);
        assert_eq!(info.encoding, "UTF-8");
        assert_eq!(info.confidence, 0.95);
        assert!(!info.has_bom);
        assert!(info.is_valid);

        let info_with_bom = EncodingInfo::with_bom("UTF-8".to_string(), 1.0, BomType::Utf8);
        assert!(info_with_bom.has_bom);
        assert_eq!(info_with_bom.bom_type, Some(BomType::Utf8));
    }

    #[test]
    fn detect_utf8_encoding() {
        let text = "[Script Info]\nTitle: Test Script";
        let encoding = detect_encoding(text.as_bytes());
        assert_eq!(encoding.encoding, "UTF-8");
        assert!(encoding.confidence > 0.9); // High confidence due to ASS patterns
        assert!(!encoding.has_bom);
    }

    #[test]
    fn detect_encoding_with_bom() {
        let text = "\u{FEFF}[Script Info]";
        let encoding = detect_encoding(text.as_bytes());
        assert_eq!(encoding.encoding, "UTF-8");
        assert_eq!(encoding.confidence, 1.0);
        assert!(encoding.has_bom);
        assert_eq!(encoding.bom_type, Some(BomType::Utf8));
    }

    #[test]
    fn detect_non_utf8_encoding() {
        let invalid_bytes = &[0x80, 0x81, b'H', b'e', b'l', b'l', b'o']; // Invalid UTF-8, no BOM
        let encoding = detect_encoding(invalid_bytes);
        assert_eq!(encoding.encoding, "Windows-1252");
        assert!(encoding.confidence < 1.0);
    }

    #[test]
    fn detect_ascii_encoding() {
        let ascii_bytes = b"Hello World"; // Pure ASCII
        let encoding = detect_encoding(ascii_bytes);
        assert_eq!(encoding.encoding, "UTF-8"); // ASCII is valid UTF-8
        assert!(encoding.confidence > 0.7);
    }

    #[test]
    fn is_likely_ass_content_detection() {
        assert!(is_likely_ass_content("[Script Info]\nTitle: Test"));
        assert!(is_likely_ass_content("[V4+ Styles]\nFormat: Name"));
        assert!(is_likely_ass_content("Dialogue: 0,0:00:00.00"));
        assert!(is_likely_ass_content("ScriptType: v4.00+"));
        assert!(!is_likely_ass_content("This is just regular text"));
        assert!(!is_likely_ass_content("No ASS patterns here"));
    }

    #[test]
    fn encoding_info_equality() {
        let info1 = EncodingInfo::new("UTF-8".to_string(), 0.95);
        let info2 = EncodingInfo::new("UTF-8".to_string(), 0.95);
        let info3 = EncodingInfo::new("ASCII".to_string(), 0.95);

        assert_eq!(info1, info2);
        assert_ne!(info1, info3);
    }

    #[test]
    fn encoding_info_debug() {
        let info = EncodingInfo::new("UTF-8".to_string(), 0.95);
        let debug_str = format!("{:?}", info);
        assert!(debug_str.contains("EncodingInfo"));
        assert!(debug_str.contains("UTF-8"));
    }
}
