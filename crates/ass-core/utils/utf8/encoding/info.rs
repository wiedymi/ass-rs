//! Encoding information type with confidence scoring.
//!
//! Defines [`EncodingInfo`], the result type produced by encoding detection,
//! carrying the detected encoding name, confidence level, and BOM details.

use super::super::bom::BomType;
use alloc::string::String;

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
    #[must_use]
    pub const fn new(encoding: String, confidence: f32) -> Self {
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
    #[must_use]
    pub const fn with_bom(encoding: String, confidence: f32, bom_type: BomType) -> Self {
        Self {
            encoding,
            confidence,
            has_bom: true,
            bom_type: Some(bom_type),
            is_valid: true,
        }
    }
}
