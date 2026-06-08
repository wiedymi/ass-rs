//! Karaoke effect tag handlers for ASS override tags
//!
//! Implements handlers for karaoke timing tags like `\k`, `\kf`, `\ko`,
//! and the v4++ `\kt` tag for absolute timing.
//!
//! # Karaoke Tags
//!
//! - `\k` - Basic karaoke highlight (hundredths of seconds)
//! - `\kf` - Fill karaoke highlight
//! - `\ko` - Outline karaoke highlight
//! - `\kt` - Absolute karaoke timing (v4++ only, centiseconds)
//!
//! # Examples
//!
//! ```rust
//! use ass_core::plugin::{ExtensionRegistry, tags::karaoke::create_karaoke_handlers};
//!
//! let mut registry = ExtensionRegistry::new();
//! for handler in create_karaoke_handlers() {
//!     registry.register_tag_handler(handler).unwrap();
//! }
//! ```

mod handlers;

#[cfg(test)]
mod tests;

pub use handlers::{
    BasicKaraokeTagHandler, FillKaraokeTagHandler, KaraokeTimingTagHandler,
    OutlineKaraokeTagHandler,
};

use crate::plugin::TagHandler;
use alloc::{boxed::Box, vec, vec::Vec};

/// Create all karaoke tag handlers
///
/// Returns a vector of boxed tag handlers for all supported karaoke tags.
/// Includes `\k`, `\kf`, `\ko`, and the v4++ `\kt` tag handlers.
///
/// # Examples
///
/// ```rust
/// use ass_core::plugin::{ExtensionRegistry, tags::karaoke::create_karaoke_handlers};
///
/// let mut registry = ExtensionRegistry::new();
/// for handler in create_karaoke_handlers() {
///     registry.register_tag_handler(handler).unwrap();
/// }
///
/// // Test that all handlers are registered
/// assert!(registry.has_tag_handler("k"));
/// assert!(registry.has_tag_handler("kf"));
/// assert!(registry.has_tag_handler("ko"));
/// assert!(registry.has_tag_handler("kt"));
/// ```
#[must_use]
pub fn create_karaoke_handlers() -> Vec<Box<dyn TagHandler>> {
    vec![
        Box::new(BasicKaraokeTagHandler),
        Box::new(FillKaraokeTagHandler),
        Box::new(OutlineKaraokeTagHandler),
        Box::new(KaraokeTimingTagHandler),
    ]
}
