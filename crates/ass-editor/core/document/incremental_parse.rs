//! Safe edit fallback and delta-tracking parse helpers
//!
//! `edit_safe` always succeeds by falling back to a plain replace, while the
//! stream-gated helpers expose incremental event editing and delta-aware
//! parsing for command-system integration.

use super::EditorDocument;
use crate::core::errors::Result;
use crate::core::position::Range;

#[cfg(feature = "stream")]
use crate::core::errors::EditorError;
#[cfg(feature = "stream")]
use crate::core::position::Position;
#[cfg(feature = "stream")]
use ass_core::parser::script::ScriptDeltaOwned;
#[cfg(feature = "stream")]
use ass_core::parser::Script;
#[cfg(feature = "stream")]
use core::ops::Range as StdRange;

#[cfg(all(feature = "stream", not(feature = "std")))]
use alloc::format;

impl EditorDocument {
    /// Safe edit with automatic fallback to regular replace on error
    ///
    /// This method tries incremental parsing first for performance,
    /// but falls back to regular replace if incremental parsing is unavailable
    /// or fails. This ensures edits always succeed.
    pub fn edit_safe(&mut self, range: Range, new_text: &str) -> Result<()> {
        #[cfg(feature = "stream")]
        {
            // Try incremental parsing first
            match self.edit_incremental(range, new_text) {
                Ok(_) => return Ok(()),
                Err(_e) => {
                    #[cfg(feature = "std")]
                    eprintln!("Incremental edit failed, falling back to regular replace: {_e}");
                }
            }
        }

        // Fallback to regular replace
        self.replace(range, new_text)
    }

    /// Edit event using incremental parsing for performance
    #[cfg(feature = "stream")]
    pub fn edit_event_incremental(
        &mut self,
        event_text: &str,
        new_text: &str,
    ) -> Result<ScriptDeltaOwned> {
        let content = self.text();
        if let Some(pos) = content.find(event_text) {
            let range = Range::new(Position::new(pos), Position::new(pos + event_text.len()));
            self.edit_incremental(range, new_text)
        } else {
            Err(EditorError::ValidationError {
                message: format!("Event text not found: {event_text}"),
            })
        }
    }

    /// Parse with delta tracking for command system integration
    #[cfg(feature = "stream")]
    pub fn parse_with_delta_tracking<F, R>(
        &self,
        range: Option<StdRange<usize>>,
        new_text: Option<&str>,
        f: F,
    ) -> Result<R>
    where
        F: FnOnce(&Script, Option<&ScriptDeltaOwned>) -> R,
    {
        let content = self.text();
        let script = Script::parse(&content).map_err(EditorError::from)?;

        if let (Some(range), Some(text)) = (range, new_text) {
            // Get delta for the change
            match script.parse_partial(range, text) {
                Ok(delta) => Ok(f(&script, Some(&delta))),
                Err(_) => {
                    // Fallback to full re-parse if incremental fails
                    Ok(f(&script, None))
                }
            }
        } else {
            Ok(f(&script, None))
        }
    }
}
