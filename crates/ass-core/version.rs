//! ASS script version detection and feature support.
//!
//! Defines [`ScriptVersion`], which identifies the SSA/ASS format variant a
//! script declares and whether it supports modern libass 0.17.4+ extensions.

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
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum ScriptVersion {
    /// SSA v4.00 (`SubStation` Alpha legacy format).
    ///
    /// Provides compatibility with legacy SSA files. Limited feature set
    /// compared to modern ASS versions.
    SsaV4,
    /// ASS v4.00+ (Advanced `SubStation` Alpha standard).
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
    #[must_use]
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
    #[must_use]
    pub const fn supports_extensions(self) -> bool {
        matches!(self, Self::AssV4Plus)
    }
}
