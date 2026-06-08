//! Dialogue event analysis construction.
//!
//! Implements the analysis entry points that parse timing, run text analysis,
//! and compute complexity scores for a [`DialogueInfo`].

use super::DialogueInfo;
use crate::{
    analysis::events::{
        scoring::{calculate_animation_score, calculate_complexity_score},
        text_analysis::TextAnalysis,
    },
    parser::Event,
    utils::{parse_ass_time, CoreError},
    Result,
};

#[cfg(feature = "plugins")]
use crate::plugin::ExtensionRegistry;

impl<'a> DialogueInfo<'a> {
    /// Analyze a dialogue event comprehensively
    ///
    /// Performs timing parsing, text analysis, and complexity scoring.
    /// Results are cached within the returned `DialogueInfo` instance.
    ///
    /// # Arguments
    ///
    /// * `event` - Dialogue event to analyze
    ///
    /// # Returns
    ///
    /// `DialogueInfo` with complete analysis results, or error if parsing fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use ass_core::analysis::events::dialogue_info::DialogueInfo;
    /// # use ass_core::parser::Event;
    /// let event = Event {
    ///     start: "0:00:00.00",
    ///     end: "0:00:05.00",
    ///     text: "Hello {\\b1}World{\\b0}!",
    ///     ..Default::default()
    /// };
    ///
    /// let info = DialogueInfo::analyze(&event)?;
    /// assert_eq!(info.duration_ms(), 5000);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the event times are invalid or cannot be parsed.
    pub fn analyze(event: &'a Event<'a>) -> Result<Self> {
        #[cfg(feature = "plugins")]
        return Self::analyze_with_registry(event, None);
        #[cfg(not(feature = "plugins"))]
        return Self::analyze_impl(event);
    }

    /// Analyze a dialogue event with extension registry support
    ///
    /// Same as [`analyze`](Self::analyze) but allows custom tag handlers via registry.
    /// Unhandled tags fall back to standard processing.
    ///
    /// # Arguments
    ///
    /// * `event` - Dialogue event to analyze
    /// * `registry` - Optional registry for custom tag handlers
    ///
    /// # Returns
    ///
    /// `DialogueInfo` with complete analysis results, or error if parsing fails.
    ///
    /// # Errors
    ///
    /// Returns an error if the event times are invalid or cannot be parsed.
    #[cfg(feature = "plugins")]
    pub fn analyze_with_registry(
        event: &'a Event<'a>,
        registry: Option<&ExtensionRegistry>,
    ) -> Result<Self> {
        Self::analyze_impl_with_registry(event, registry)
    }

    /// Internal implementation without plugins support
    #[cfg(not(feature = "plugins"))]
    fn analyze_impl(event: &'a Event<'a>) -> Result<Self> {
        Self::analyze_impl_with_registry(event)
    }

    /// Internal implementation that supports optional registry
    fn analyze_impl_with_registry(
        event: &'a Event<'a>,
        #[cfg(feature = "plugins")] registry: Option<&ExtensionRegistry>,
    ) -> Result<Self> {
        let start_cs = parse_ass_time(event.start)?;

        let end_cs = parse_ass_time(event.end)?;

        if start_cs >= end_cs {
            return Err(CoreError::parse("Start time must be before end time"));
        }

        #[cfg(feature = "plugins")]
        let text_info = if let Some(registry) = registry {
            TextAnalysis::analyze_with_registry(event.text, Some(registry))?
        } else {
            TextAnalysis::analyze(event.text)?
        };

        #[cfg(not(feature = "plugins"))]
        let text_info = TextAnalysis::analyze(event.text)?;

        let animation_score = calculate_animation_score(text_info.override_tags());
        let complexity_score = calculate_complexity_score(
            animation_score,
            text_info.char_count(),
            text_info.override_tags().len(),
        );

        Ok(Self {
            event,
            start_cs,
            end_cs,
            animation_score,
            complexity_score,
            text_info,
        })
    }
}
