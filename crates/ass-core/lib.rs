//! # ASS-RS Core
//!
//! High-performance, memory-efficient ASS (Advanced `SubStation` Alpha) subtitle format parser,
//! analyzer, and manipulator. Surpasses libass in modularity, reusability, and efficiency
//! through zero-copy parsing, trait-based extensibility, and strict memory management.
//!
//! ## Features
//!
//! - **Zero-copy parsing**: Uses `&str` spans to avoid allocations
//! - **Incremental updates**: Partial re-parsing for <2ms edits
//! - **SIMD optimization**: Feature-gated performance improvements
//! - **Extensible plugins**: Runtime tag and section handlers
//! - **Strict compliance**: Full ASS v4+, SSA v4, and libass 0.17.4+ support
//! - **Thread-safe**: Immutable `Script` design with `Send + Sync`
//!
//! ## Quick Start
//!
//! ```rust
//! use ass_core::{Script, analysis::ScriptAnalysis};
//!
//! let script_text = r#"
//! [Script Info]
//! Title: Example
//! ScriptType: v4.00+
//!
//! [V4+ Styles]
//! Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
//! Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
//!
//! [Events]
//! Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
//! Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello World!
//! "#;
//!
//! let script = Script::parse(script_text)?;
//! let analysis = ScriptAnalysis::analyze(&script)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Performance Targets
//!
//! - Parse: <5ms for 1KB scripts
//! - Analysis: <2ms for typical content
//! - Memory: ~1.1x input size via zero-copy spans
//! - Incremental: <2ms for single-event updates

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(clippy::all)]
#![deny(unsafe_code)]

extern crate alloc;

pub mod parser;
pub mod tokenizer;

#[cfg(feature = "analysis")]
#[cfg_attr(docsrs, doc(cfg(feature = "analysis")))]
pub mod analysis;

#[cfg(feature = "plugins")]
#[cfg_attr(docsrs, doc(cfg(feature = "plugins")))]
pub mod plugin;

pub mod utils;

pub use parser::{ParseError, Script, Section};
pub use tokenizer::{AssTokenizer, Token};

#[cfg(feature = "analysis")]
pub use analysis::ScriptAnalysis;

#[cfg(feature = "plugins")]
pub use plugin::ExtensionRegistry;

pub use utils::{CoreError, Spans};

/// Crate version for runtime compatibility checks
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Supported ASS script versions for compatibility and feature detection.
///
/// ASS scripts can declare different versions that affect parsing behavior
/// and available features. This enum helps determine which parsing mode
/// to use and which features are available.
///
/// # Examples
///
/// ```rust
/// use ass_core::ScriptVersion;
///
/// // Parse from header
/// let version = ScriptVersion::from_header("v4.00+").unwrap();
/// assert_eq!(version, ScriptVersion::AssV4);
///
/// // Check feature support
/// assert!(!ScriptVersion::SsaV4.supports_extensions());
/// assert!(ScriptVersion::AssV4Plus.supports_extensions());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptVersion {
    /// SSA v4.00 (SubStation Alpha legacy format).
    ///
    /// Provides compatibility with legacy SSA files. Limited feature set
    /// compared to modern ASS versions.
    SsaV4,
    /// ASS v4.00+ (Advanced SubStation Alpha standard).
    ///
    /// The most common format used by modern subtitle tools. Supports
    /// all standard ASS features and tags.
    AssV4,
    /// ASS v4.00+ with extensions (libass 0.17.4+ compatibility).
    ///
    /// Extended format supporting newer features like `\kt` karaoke tags,
    /// Unicode line wrapping, and other libass extensions.
    AssV4Plus,
}

impl ScriptVersion {
    /// Parse script version from a `ScriptType` header value.
    ///
    /// Converts header strings commonly found in `[Script Info]` sections
    /// to the appropriate script version enum. Handles various formats
    /// including extended versions.
    ///
    /// # Arguments
    ///
    /// * `header` - The header value string (usually from `ScriptType` field)
    ///
    /// # Returns
    ///
    /// Returns `Some(ScriptVersion)` if the header is recognized, or `None`
    /// if the version string is invalid or unsupported.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ass_core::ScriptVersion;
    ///
    /// assert_eq!(ScriptVersion::from_header("v4.00"), Some(ScriptVersion::SsaV4));
    /// assert_eq!(ScriptVersion::from_header("v4.00+"), Some(ScriptVersion::AssV4));
    /// assert_eq!(ScriptVersion::from_header("v4.00++"), Some(ScriptVersion::AssV4Plus));
    /// assert_eq!(ScriptVersion::from_header("invalid"), None);
    /// ```
    pub fn from_header(header: &str) -> Option<Self> {
        match header.trim() {
            "v4.00" => Some(Self::SsaV4),
            "v4.00+" => Some(Self::AssV4),
            "v4.00++" | "v4.00+ extended" => Some(Self::AssV4Plus),
            _ => None,
        }
    }

    /// Check if the script version supports modern ASS extensions.
    ///
    /// Modern ASS extensions include features like:
    /// - `\kt` karaoke timing tags
    /// - Unicode line wrapping
    /// - Extended color formats
    /// - Advanced animation features
    ///
    /// Only `AssV4Plus` currently supports these extensions, as they
    /// require libass 0.17.4+ compatibility.
    ///
    /// # Returns
    ///
    /// Returns `true` if the version supports extensions, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ass_core::ScriptVersion;
    ///
    /// assert!(!ScriptVersion::SsaV4.supports_extensions());
    /// assert!(!ScriptVersion::AssV4.supports_extensions());
    /// assert!(ScriptVersion::AssV4Plus.supports_extensions());
    /// ```
    pub fn supports_extensions(self) -> bool {
        matches!(self, Self::AssV4Plus)
    }
}

/// Result type for core operations, using the crate's unified `CoreError`.
///
/// This type alias provides a convenient way to return results from core operations
/// without having to specify the error type explicitly in every function signature.
///
/// # Examples
///
/// ```rust
/// use ass_core::{Result, Script};
///
/// fn parse_script_safely(input: &str) -> Result<Script> {
///     Script::parse(input)
/// }
/// ```
pub type Result<T> = core::result::Result<T, CoreError>;

#[cfg(test)]
mod integration_tests {
    use super::*;
    #[cfg(feature = "analysis")]
    use crate::analysis::{
        linting::{lint_script, LintConfig},
        ScriptAnalysis,
    };

    /// Comprehensive integration test verifying core functionality works correctly
    #[test]
    fn test_core_functionality_integration() {
        let script_text = r"
[Script Info]
Title: Test Script
ScriptType: v4.00+

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: Large,Arial,80,&H00FF0000,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello World!
Dialogue: 0,0:00:02.00,0:00:07.00,Default,,0,0,0,,Overlapping dialogue
Dialogue: 0,0:00:10.00,0:00:15.00,Large,,0,0,0,,{\t(0,1000,\fscx200\fscy200)}Large animated text
Comment: 0,0:00:30.00,0:00:35.00,Default,,0,0,0,,This is a comment
";

        let script = Script::parse(script_text).expect("Should parse script successfully");
        assert!(
            script.sections().len() >= 2,
            "Should have parsed multiple sections"
        );

        let version = ScriptVersion::from_header("v4.00+").expect("Should detect script version");
        assert_eq!(version, ScriptVersion::AssV4);
        assert!(!version.supports_extensions());

        let version_plus =
            ScriptVersion::from_header("v4.00++").expect("Should detect extended version");
        assert_eq!(version_plus, ScriptVersion::AssV4Plus);
        assert!(version_plus.supports_extensions());

        #[cfg(feature = "analysis")]
        {
            let analysis =
                ScriptAnalysis::analyze(&script).expect("Should analyze script successfully");

            assert!(
                !analysis.resolved_styles().is_empty(),
                "Should resolve styles"
            );

            assert!(
                !analysis.dialogue_info().is_empty(),
                "Should analyze dialogue events"
            );

            let perf = analysis.performance_summary();
            assert!(
                perf.performance_score <= 100,
                "Performance score should be valid"
            );

            let default_style = analysis.resolve_style("Default");
            assert!(default_style.is_some(), "Should find Default style");

            let lint_config = LintConfig::default();
            let issues =
                lint_script(&script, &lint_config).expect("Should run linting successfully");

            assert!(!issues.is_empty(), "Should detect some lint issues");
        }
    }

    #[test]
    fn test_script_version_functionality() {
        assert_eq!(
            ScriptVersion::from_header("v4.00"),
            Some(ScriptVersion::SsaV4)
        );
        assert_eq!(
            ScriptVersion::from_header("v4.00+"),
            Some(ScriptVersion::AssV4)
        );
        assert_eq!(
            ScriptVersion::from_header("v4.00++"),
            Some(ScriptVersion::AssV4Plus)
        );
        assert_eq!(
            ScriptVersion::from_header("v4.00+ extended"),
            Some(ScriptVersion::AssV4Plus)
        );
        assert_eq!(ScriptVersion::from_header("invalid"), None);

        assert!(!ScriptVersion::SsaV4.supports_extensions());
        assert!(!ScriptVersion::AssV4.supports_extensions());
        assert!(ScriptVersion::AssV4Plus.supports_extensions());
    }

    #[test]
    fn test_error_handling() {
        let invalid_script = "This is not a valid ASS script";
        let result = Script::parse(invalid_script);

        if let Ok(script) = result {
            assert!(
                !script.issues().is_empty(),
                "Invalid script should have parse issues"
            );
        }
    }

    #[test]
    fn test_empty_script_handling() {
        let empty_script = "";
        let result = Script::parse(empty_script);

        assert!(result.is_ok(), "Should handle empty script gracefully");

        let script = result.unwrap();
        assert_eq!(
            script.sections().len(),
            0,
            "Empty script should have no sections"
        );
    }
}
