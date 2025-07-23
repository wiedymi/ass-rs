//! BOM (Byte Order Mark) detection and handling utilities
//!
//! Provides functionality for detecting, stripping, and validating Byte Order
//! Marks in text input. Focuses on BOM handling for ASS subtitle files which
//! should typically use UTF-8 without BOM for maximum compatibility.
//!
//! # Examples
//!
//! ```rust
//! use ass_core::utils::utf8::{strip_bom, detect_bom, BomType};
//!
//! let input = "\u{FEFF}Hello World";
//! let (stripped, had_bom) = strip_bom(input);
//! assert_eq!(stripped, "Hello World");
//! assert!(had_bom);
//! ```

/// Byte Order Mark (BOM) signatures for common encodings
///
/// Represents the different types of BOMs that can appear at the beginning
/// of text files. Each variant corresponds to a specific text encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    ///
    /// Returns the exact byte sequence that identifies this BOM type
    /// at the beginning of a file or text stream.
    #[must_use] pub const fn signature(self) -> &'static [u8] {
        match self {
            Self::Utf8 => &[0xEF, 0xBB, 0xBF],
            Self::Utf16Le => &[0xFF, 0xFE],
            Self::Utf16Be => &[0xFE, 0xFF],
            Self::Utf32Le => &[0xFF, 0xFE, 0x00, 0x00],
            Self::Utf32Be => &[0x00, 0x00, 0xFE, 0xFF],
        }
    }

    /// Get length of this BOM in bytes
    ///
    /// Returns the number of bytes occupied by this BOM type.
    /// Useful for skipping the BOM when processing text.
    #[must_use] pub const fn len(self) -> usize {
        self.signature().len()
    }

    /// Check if BOM is empty (never true for valid BOMs)
    ///
    /// Provided for completeness with Rust conventions.
    /// Always returns `false` since all BOMs have non-zero length.
    #[must_use] pub const fn is_empty(self) -> bool {
        false
    }

    /// Get encoding name for this BOM
    ///
    /// Returns the canonical name of the text encoding associated
    /// with this BOM type.
    #[must_use] pub const fn encoding_name(self) -> &'static str {
        match self {
            Self::Utf8 => "UTF-8",
            Self::Utf16Le => "UTF-16LE",
            Self::Utf16Be => "UTF-16BE",
            Self::Utf32Le => "UTF-32LE",
            Self::Utf32Be => "UTF-32BE",
        }
    }
}

/// Detect and strip BOM from text input
///
/// Returns the text without BOM and information about what was stripped.
/// This is a zero-copy operation that returns a slice into the original text.
///
/// # Arguments
///
/// * `text` - Input text that may contain a BOM
///
/// # Returns
///
/// Tuple of (`text_without_bom`, `had_bom`)
///
/// # Examples
///
/// ```rust
/// # use ass_core::utils::utf8::strip_bom;
/// let input = "\u{FEFF}Hello World";
/// let (stripped, had_bom) = strip_bom(input);
/// assert_eq!(stripped, "Hello World");
/// assert!(had_bom);
/// ```
#[must_use] pub fn strip_bom(text: &str) -> (&str, bool) {
    let bytes = text.as_bytes();

    // Check for UTF-8 BOM first (most common for ASS files)
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return (&text[3..], true);
    }

    // Other BOMs indicate wrong encoding for ASS files, but we detect them
    if bytes.starts_with(&[0xFF, 0xFE, 0x00, 0x00]) {
        // UTF-32LE - return original text (can't strip from UTF-8 string)
        return (text, false);
    }

    if bytes.starts_with(&[0x00, 0x00, 0xFE, 0xFF]) {
        // UTF-32BE - return original text (can't strip from UTF-8 string)
        return (text, false);
    }

    if bytes.starts_with(&[0xFF, 0xFE]) {
        // UTF-16LE - return original text (can't strip from UTF-8 string)
        return (text, false);
    }

    if bytes.starts_with(&[0xFE, 0xFF]) {
        // UTF-16BE - return original text (can't strip from UTF-8 string)
        return (text, false);
    }

    (text, false)
}

/// Detect BOM type from byte sequence
///
/// Returns the detected BOM type and the number of bytes to skip.
/// Returns None if no BOM is detected. Uses longest-match strategy
/// to handle overlapping BOM signatures correctly.
///
/// # Arguments
///
/// * `bytes` - Byte sequence to analyze for BOM
///
/// # Returns
///
/// Option containing (`BomType`, `bytes_to_skip`) if BOM found
#[must_use] pub fn detect_bom(bytes: &[u8]) -> Option<(BomType, usize)> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bom_type_properties() {
        assert_eq!(BomType::Utf8.signature(), &[0xEF, 0xBB, 0xBF]);
        assert_eq!(BomType::Utf8.len(), 3);
        assert!(!BomType::Utf8.is_empty());
        assert_eq!(BomType::Utf8.encoding_name(), "UTF-8");

        assert_eq!(BomType::Utf16Le.signature(), &[0xFF, 0xFE]);
        assert_eq!(BomType::Utf16Le.len(), 2);
        assert_eq!(BomType::Utf16Le.encoding_name(), "UTF-16LE");
    }

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
    fn detect_utf16be_bom() {
        let bytes = &[0xFE, 0xFF, 0x00, b'H', 0x00, b'i'];
        let (bom_type, skip) = detect_bom(bytes).unwrap();
        assert_eq!(bom_type, BomType::Utf16Be);
        assert_eq!(skip, 2);
    }

    #[test]
    fn detect_utf32le_bom() {
        let bytes = &[0xFF, 0xFE, 0x00, 0x00, b'H', 0x00, 0x00, 0x00];
        let (bom_type, skip) = detect_bom(bytes).unwrap();
        assert_eq!(bom_type, BomType::Utf32Le);
        assert_eq!(skip, 4);
    }

    #[test]
    fn detect_utf32be_bom() {
        let bytes = &[0x00, 0x00, 0xFE, 0xFF, 0x00, 0x00, 0x00, b'H'];
        let (bom_type, skip) = detect_bom(bytes).unwrap();
        assert_eq!(bom_type, BomType::Utf32Be);
        assert_eq!(skip, 4);
    }

    #[test]
    fn detect_no_bom() {
        let bytes = b"Hello World";
        assert!(detect_bom(bytes).is_none());
    }

    #[test]
    fn bom_type_equality() {
        assert_eq!(BomType::Utf8, BomType::Utf8);
        assert_ne!(BomType::Utf8, BomType::Utf16Le);
    }

    #[test]
    fn bom_type_copy_clone() {
        let bom_type = BomType::Utf8;
        let copied = bom_type;
        let cloned = bom_type;

        assert_eq!(bom_type, copied);
        assert_eq!(bom_type, cloned);
    }
}
