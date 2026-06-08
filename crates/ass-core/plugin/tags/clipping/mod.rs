//! Clipping tag handlers for ASS override tags
//!
//! Implements handlers for clipping masks including rectangular and
//! vector-based shape clipping. These tags control which parts of
//! text are visible.
//!
//! # Supported Tags
//!
//! - `clip`: Rectangular or vector clipping mask
//!
//! # Performance
//!
//! - Efficient argument parsing
//! - Support for both rectangular and vector formats
//! - Minimal memory footprint per handler

mod clip_handler;

#[cfg(test)]
mod tests;

pub use clip_handler::ClipTagHandler;

use crate::plugin::TagHandler;

/// Create all clipping tag handlers
///
/// Returns a vector of boxed tag handlers for clipping operations.
///
/// # Example
///
/// ```rust
/// use ass_core::plugin::{ExtensionRegistry, tags::clipping::create_clipping_handlers};
///
/// let mut registry = ExtensionRegistry::new();
/// for handler in create_clipping_handlers() {
///     registry.register_tag_handler(handler).unwrap();
/// }
/// ```
#[must_use]
pub fn create_clipping_handlers() -> alloc::vec::Vec<alloc::boxed::Box<dyn TagHandler>> {
    alloc::vec![alloc::boxed::Box::new(ClipTagHandler)]
}
