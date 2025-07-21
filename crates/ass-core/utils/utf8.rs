//! UTF-8 and text encoding utilities for ASS script processing
//!
//! Provides BOM handling, encoding detection, and UTF-8 validation utilities
//! optimized for ASS subtitle script processing with zero-copy design.
//!
//! # Features
//!
//! - BOM detection and stripping for common encodings
//! - UTF-8 validation with detailed error reporting
//! - Encoding detection for legacy ASS files
//! - no_std compatible implementation
//! - Zero-copy operations where possible
//!
//! # Example
//!
//! ```rust
//! use ass_core::utils::{strip_bom, detect_encoding};
//!
//! let input = "\u{FEFF}[Script Info]\nTitle: Test";
//! let (stripped, had_bom) = strip_bom(input);
//! assert_eq!(stripped, "[Script Info]\nTitle: Test");
//! assert!(had_bom);
//! ```

use crate::utils::CoreError;
use alloc::{format, string::String, string::ToString};
use core::str;

/// Byte Order Mark (BOM) signatures for common encodings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BomType {
    /// UTF-8 BOM (EF BB BF)
    Utf8,
    /// UTF-16 Little Endian (FF FE)
    Utf16Le,
    /// UTF-16 Big Endian (FE FF)
    Utf16Be,
    /// UTF-32 Little Endian (FF FE 00 00)
    Utf32Le,
    /// UTF-32 Big Endian (00 00 FE FF)
    Utf32Be,
}

impl BomType {
    /// Get byte signature for this BOM type
    pub fn signature(self) -> &'static [u8] {
        match self {
            BomType::Utf8 => &[0xEF, 0xBB, 0xBF],
            BomType::Utf16Le => &[0xFF, 0xFE],
            BomType::Utf16Be => &[0xFE, 0xFF],
            BomType::Utf32Le => &[0xFF, 0xFE, 0x00, 0x00],
            BomType::Utf32Be => &[0x00, 0x00, 0xFE, 0xFF],
        }
    }

    /// Get length of this BOM in bytes
    pub fn len(self) -> usize {
        self.signature().len()
    }

    /// Check if BOM is empty (never true for valid BOMs)
    pub fn is_empty(self) -> bool {
        false
    }

    /// Get encoding name for this BOM
    pub fn encoding_name(self) -> &'static str {
        match self {
            BomType::Utf8 => "UTF-8",
            BomType::Utf16Le => "UTF-16LE",
            BomType::Utf16Be => "UTF-16BE",
            BomType::Utf32Le => "UTF-32LE",
            BomType::Utf32Be => "UTF-32BE",
        }
    }
}

/// Detected text encoding information
#[derive(Debug, Clone, PartialEq)]
pub struct EncodingInfo {
    /// Detected encoding name
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
    /// Create new encoding info
    pub fn new(encoding: String, confidence: f32) -> Self {
        Self {
            encoding,
            confidence,
            has_bom: false,
            bom_type: None,
            is_valid: true,
        }
    }

    /// Create encoding info with BOM
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

/// Detect and strip BOM from text input
///
/// Returns the text without BOM and information about what was stripped.
/// This is a zero-copy operation that returns a slice into the original text.
///
/// # Example
///
/// ```rust
/// use ass_core::utils::strip_bom;
///
/// let input = "\u{FEFF}Hello World";
/// let (stripped, had_bom) = strip_bom(input);
/// assert_eq!(stripped, "Hello World");
/// assert!(had_bom);
/// ```
pub fn strip_bom(text: &str) -> (&str, bool) {
    let bytes = text.as_bytes();

    // Check for UTF-8 BOM first (most common for ASS files)
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return (&text[3..], true);
    }

    // Check for UTF-16/32 BOMs (less common but possible)
    if bytes.starts_with(&[0xFF, 0xFE, 0x00, 0x00]) {
        // UTF-32LE - but we can't handle this as &str, return as-is
        return (text, false);
    }

    if bytes.starts_with(&[0x00, 0x00, 0xFE, 0xFF]) {
        // UTF-32BE - but we can't handle this as &str, return as-is
        return (text, false);
    }

    if bytes.starts_with(&[0xFF, 0xFE]) {
        // UTF-16LE - but we can't handle this as &str, return as-is
        return (text, false);
    }

    if bytes.starts_with(&[0xFE, 0xFF]) {
        // UTF-16BE - but we can't handle this as &str, return as-is
        return (text, false);
    }

    // No BOM detected
    (text, false)
}

/// Detect BOM type from byte sequence
///
/// Returns the detected BOM type and the number of bytes to skip.
/// Returns None if no BOM is detected.
pub fn detect_bom(bytes: &[u8]) -> Option<(BomType, usize)> {
    // Check longer BOMs first to avoid false matches
    if bytes.starts_with(&[0xFF, 0xFE, 0x00, 0x00]) {
        Some((BomType::Utf32Le, 4))
    } else if bytes.starts_with(&[0x00, 0x00, 0xFE, 0xFF]) {
        Some((BomType::Utf32Be, 4))
    } else if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        Some((BomType::Utf8, 3))
    } else if bytes.starts_with(&[0xFF, 0xFE]) {
        Some((BomType::Utf16Le, 2))
    } else if bytes.starts_with(&[0xFE, 0xFF]) {
        Some((BomType::Utf16Be, 2))
    } else {
        None
    }
}

/// Detect text encoding with confidence scoring
///
/// Analyzes text content to determine the most likely encoding.
/// Focuses on encodings commonly used in ASS subtitle files.
///
/// # Example
///
/// ```rust
/// use ass_core::utils::detect_encoding;
///
/// let text = "[Script Info]\nTitle: Test";
/// let encoding = detect_encoding(text.as_bytes());
/// assert_eq!(encoding.encoding, "UTF-8");
/// assert!(encoding.confidence > 0.8);
/// ```
pub fn detect_encoding(bytes: &[u8]) -> EncodingInfo {
    // Check for BOM first
    if let Some((bom_type, _)) = detect_bom(bytes) {
        return EncodingInfo::with_bom(
            bom_type.encoding_name().to_string(),
            1.0, // BOM gives us certainty
            bom_type,
        );
    }

    // Try UTF-8 validation
    match str::from_utf8(bytes) {
        Ok(_) => {
            // Valid UTF-8, check for ASS-specific patterns to increase confidence
            let confidence = if is_likely_ass_content(bytes) {
                0.95
            } else {
                0.8
            };
            EncodingInfo::new("UTF-8".to_string(), confidence)
        }
        Err(_) => {
            // Not valid UTF-8, try to detect other common encodings
            detect_non_utf8_encoding(bytes)
        }
    }
}

/// Check if bytes contain patterns typical of ASS subtitle files
fn is_likely_ass_content(bytes: &[u8]) -> bool {
    let text = match str::from_utf8(bytes) {
        Ok(t) => t,
        Err(_) => return false,
    };

    // Look for ASS-specific patterns
    text.contains("[Script Info]")
        || text.contains("[V4+ Styles]")
        || text.contains("[Events]")
        || text.contains("Dialogue:")
        || text.contains("Comment:")
        || text.contains("ScriptType:")
}

/// Attempt to detect non-UTF-8 encodings commonly used in older ASS files
fn detect_non_utf8_encoding(bytes: &[u8]) -> EncodingInfo {
    // Check for common Windows codepages used in subtitle files

    // Look for high-bit characters that might indicate Windows-1252 or similar
    let has_extended_ascii = bytes.iter().any(|&b| b >= 0x80);

    if has_extended_ascii {
        // Could be Windows-1252, ISO-8859-1, or similar
        // Without proper decoding libraries, we make a conservative guess
        EncodingInfo::new("Windows-1252".to_string(), 0.6)
    } else {
        // Pure ASCII - safe to treat as UTF-8
        EncodingInfo::new("ASCII".to_string(), 0.9)
    }
}

/// Validate UTF-8 with detailed error information
///
/// Provides more detailed error reporting than standard UTF-8 validation,
/// including the position and nature of encoding errors.
///
/// # Example
///
/// ```rust
/// use ass_core::utils::utf8::validate_utf8;
///
/// let valid_text = "Hello, 世界!";
/// assert!(validate_utf8(valid_text.as_bytes()).is_ok());
/// ```
pub fn validate_utf8(bytes: &[u8]) -> Result<(), CoreError> {
    match str::from_utf8(bytes) {
        Ok(_) => Ok(()),
        Err(err) => {
            let position = err.valid_up_to();
            let _error_len = err.error_len().unwrap_or(1);

            let message = if let Some(len) = err.error_len() {
                format!(
                    "Invalid UTF-8 sequence of {} bytes at position {}",
                    len, position
                )
            } else {
                format!("Incomplete UTF-8 sequence at position {}", position)
            };

            Err(CoreError::utf8_error(position, message))
        }
    }
}

/// Attempt to recover from UTF-8 errors by replacing invalid sequences
///
/// Returns valid UTF-8 text with invalid sequences replaced by the Unicode
/// replacement character (�). Also returns the number of replacements made.
///
/// # Example
///
/// ```rust
/// use ass_core::utils::utf8::recover_utf8;
///
/// let valid_text = "Hello, World!";
/// let (recovered, replacements) = recover_utf8(valid_text.as_bytes());
/// assert_eq!(recovered, "Hello, World!");
/// assert_eq!(replacements, 0);
/// ```
pub fn recover_utf8(bytes: &[u8]) -> (String, usize) {
    match str::from_utf8(bytes) {
        Ok(s) => (s.to_string(), 0),
        Err(_) => {
            // Use lossy conversion which replaces invalid sequences
            let recovered = String::from_utf8_lossy(bytes);
            let replacements = recovered.matches('\u{FFFD}').count();
            (recovered.into_owned(), replacements)
        }
    }
}

/// Normalize line endings to Unix style (\n)
///
/// Converts Windows (\r\n) and classic Mac (\r) line endings to Unix (\n).
/// Returns a new string with normalized line endings.
///
/// # Example
///
/// ```rust
/// use ass_core::utils::utf8::normalize_line_endings;
///
/// let input = "Line 1\r\nLine 2\rLine 3\n";
/// let normalized = normalize_line_endings(input);
/// assert_eq!(normalized, "Line 1\nLine 2\nLine 3\n");
/// ```
pub fn normalize_line_endings(text: &str) -> String {
    // Replace \r\n first, then \r to avoid double conversion
    text.replace("\r\n", "\n").replace('\r', "\n")
}

/// Check if text contains only valid ASS characters
///
/// ASS files should generally contain only printable characters plus
/// specific control characters like tabs and newlines.
pub fn is_valid_ass_text(text: &str) -> bool {
    text.chars().all(|c| {
        c.is_ascii_graphic()  // Printable ASCII
            || c == ' '       // Space
            || c == '\t'      // Tab
            || c == '\n'      // Newline
            || c == '\r'      // Carriage return
            || (!c.is_ascii() && !c.is_control()) // Non-ASCII printable (Unicode)
    })
}

/// Truncate text at UTF-8 character boundary
///
/// Safely truncates text to the specified byte length without breaking
/// UTF-8 character sequences. Returns the truncated text and whether
/// truncation occurred.
pub fn truncate_at_char_boundary(text: &str, max_bytes: usize) -> (&str, bool) {
    if text.len() <= max_bytes {
        return (text, false);
    }

    // Find the largest valid UTF-8 boundary <= max_bytes
    let mut boundary = max_bytes;
    while boundary > 0 && !text.is_char_boundary(boundary) {
        boundary -= 1;
    }

    (&text[..boundary], true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_utf8_bom() {
        let text_with_bom = "\u{FEFF}Hello World";
        let (stripped, had_bom) = strip_bom(text_with_bom);
        assert_eq!(stripped, "Hello World");
        assert!(had_bom);
    }

    #[test]
    fn strip_no_bom() {
        let text_without_bom = "Hello World";
        let (stripped, had_bom) = strip_bom(text_without_bom);
        assert_eq!(stripped, "Hello World");
        assert!(!had_bom);
    }

    #[test]
    fn detect_utf8_bom() {
        let bytes = &[0xEF, 0xBB, 0xBF, b'H', b'i'];
        let (bom_type, skip) = detect_bom(bytes).unwrap();
        assert_eq!(bom_type, BomType::Utf8);
        assert_eq!(skip, 3);
    }

    #[test]
    fn detect_utf16le_bom() {
        let bytes = &[0xFF, 0xFE, b'H', 0x00, b'i', 0x00];
        let (bom_type, skip) = detect_bom(bytes).unwrap();
        assert_eq!(bom_type, BomType::Utf16Le);
        assert_eq!(skip, 2);
    }

    #[test]
    fn detect_no_bom() {
        let bytes = b"Hello World";
        assert!(detect_bom(bytes).is_none());
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
    fn validate_valid_utf8() {
        let text = "Hello, 世界! 🎵";
        assert!(validate_utf8(text.as_bytes()).is_ok());
    }

    #[test]
    fn validate_invalid_utf8() {
        let invalid_bytes = &[0xFF, 0xFE, 0x80]; // Invalid UTF-8
        assert!(validate_utf8(invalid_bytes).is_err());
    }

    #[test]
    fn recover_valid_utf8() {
        let text = "Hello, World!";
        let (recovered, replacements) = recover_utf8(text.as_bytes());
        assert_eq!(recovered, "Hello, World!");
        assert_eq!(replacements, 0);
    }

    #[test]
    fn recover_invalid_utf8() {
        let invalid_bytes = &[b'H', b'i', 0xFF, b'!'];
        let (recovered, replacements) = recover_utf8(invalid_bytes);
        assert_eq!(recovered, "Hi�!");
        assert_eq!(replacements, 1);
    }

    #[test]
    fn normalize_line_endings_windows() {
        let input = "Line 1\r\nLine 2\r\nLine 3";
        let normalized = normalize_line_endings(input);
        assert_eq!(normalized, "Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn normalize_line_endings_mac() {
        let input = "Line 1\rLine 2\rLine 3";
        let normalized = normalize_line_endings(input);
        assert_eq!(normalized, "Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn normalize_line_endings_mixed() {
        let input = "Line 1\r\nLine 2\rLine 3\n";
        let normalized = normalize_line_endings(input);
        assert_eq!(normalized, "Line 1\nLine 2\nLine 3\n");
    }

    #[test]
    fn valid_ass_text() {
        assert!(is_valid_ass_text("Hello World"));
        assert!(is_valid_ass_text("Hello\tWorld\n"));
        assert!(is_valid_ass_text("Hello 世界"));
        assert!(!is_valid_ass_text("Hello\x00World")); // Null character
        assert!(!is_valid_ass_text("Hello\x1FWorld")); // Control character
    }

    #[test]
    fn truncate_ascii() {
        let text = "Hello World";
        let (truncated, was_truncated) = truncate_at_char_boundary(text, 5);
        assert_eq!(truncated, "Hello");
        assert!(was_truncated);
    }

    #[test]
    fn truncate_unicode() {
        let text = "Hello 世界";
        let (truncated, was_truncated) = truncate_at_char_boundary(text, 8);
        assert_eq!(truncated, "Hello "); // Stops before the Unicode character
        assert!(was_truncated);
    }

    #[test]
    fn truncate_no_change() {
        let text = "Hello";
        let (truncated, was_truncated) = truncate_at_char_boundary(text, 10);
        assert_eq!(truncated, "Hello");
        assert!(!was_truncated);
    }

    #[test]
    fn bom_type_properties() {
        assert_eq!(BomType::Utf8.signature(), &[0xEF, 0xBB, 0xBF]);
        assert_eq!(BomType::Utf8.len(), 3);
        assert!(!BomType::Utf8.is_empty());
        assert_eq!(BomType::Utf8.encoding_name(), "UTF-8");
    }

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
    fn is_likely_ass_content_detection() {
        assert!(is_likely_ass_content(b"[Script Info]\nTitle: Test"));
        assert!(is_likely_ass_content(b"[V4+ Styles]\nFormat: Name"));
        assert!(is_likely_ass_content(b"Dialogue: 0,0:00:00.00"));
        assert!(!is_likely_ass_content(b"This is just regular text"));
    }
}
