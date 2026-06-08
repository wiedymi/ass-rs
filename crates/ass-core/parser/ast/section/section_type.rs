//! Section type discriminant and its metadata helpers.
//!
//! Defines the lightweight [`SectionType`] enum used to identify ASS
//! sections without borrowing their content, plus helpers for canonical
//! header names and section classification.

/// Section type discriminant for efficient lookup and filtering
///
/// Provides a lightweight way to identify section types without
/// borrowing section content. Useful for filtering, routing, and
/// type-based operations on collections of sections.
///
/// # Examples
///
/// ```rust
/// use ass_core::parser::ast::SectionType;
///
/// let section_types = vec![SectionType::ScriptInfo, SectionType::Events];
/// assert!(section_types.contains(&SectionType::ScriptInfo));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum SectionType {
    /// [Script Info] section identifier
    ScriptInfo,
    /// [V4+ Styles] section identifier
    Styles,
    /// `[Events\]` section identifier
    Events,
    /// `[Fonts\]` section identifier
    Fonts,
    /// `[Graphics\]` section identifier
    Graphics,
}

impl SectionType {
    /// Get the canonical section header name
    ///
    /// Returns the exact header name as it appears in ASS files,
    /// useful for serialization and error reporting.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ass_core::parser::ast::SectionType;
    /// assert_eq!(SectionType::ScriptInfo.header_name(), "Script Info");
    /// assert_eq!(SectionType::Styles.header_name(), "V4+ Styles");
    /// ```
    #[must_use]
    pub const fn header_name(self) -> &'static str {
        match self {
            Self::ScriptInfo => "Script Info",
            Self::Styles => "V4+ Styles",
            Self::Events => "Events",
            Self::Fonts => "Fonts",
            Self::Graphics => "Graphics",
        }
    }

    /// Check if this section type is required in valid ASS files
    ///
    /// Returns `true` for sections that must be present for a valid
    /// ASS file (Script Info and Events), `false` for optional sections.
    #[must_use]
    pub const fn is_required(self) -> bool {
        matches!(self, Self::ScriptInfo | Self::Events)
    }

    /// Check if this section type contains timed content
    ///
    /// Returns `true` for sections with time-based content that affects
    /// subtitle timing and playback.
    #[must_use]
    pub const fn is_timed(self) -> bool {
        matches!(self, Self::Events)
    }
}
