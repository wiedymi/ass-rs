//! Font tag handlers for ASS override tags
//!
//! Implements handlers for font-related commands including font name,
//! size, and encoding. These handlers validate arguments according to
//! ASS specifications.
//!
//! # Supported Tags
//!
//! - `fn`: Font name
//! - `fs`: Font size
//! - `fe`: Font encoding (character set)
//!
//! # Performance
//!
//! - Zero allocations for validation
//! - Fast numeric validation for size
//! - Minimal memory footprint per handler

mod font_handlers;

#[cfg(test)]
mod handler_tests;

pub use font_handlers::{FontEncodingTagHandler, FontNameTagHandler, FontSizeTagHandler};

use crate::plugin::TagHandler;

/// Create all font tag handlers
///
/// Returns a vector of boxed tag handlers for font-related operations.
///
/// # Example
///
/// ```rust
/// use ass_core::plugin::{ExtensionRegistry, tags::font::create_font_handlers};
///
/// let mut registry = ExtensionRegistry::new();
/// for handler in create_font_handlers() {
///     registry.register_tag_handler(handler).unwrap();
/// }
/// ```
#[must_use]
pub fn create_font_handlers() -> alloc::vec::Vec<alloc::boxed::Box<dyn TagHandler>> {
    alloc::vec![
        alloc::boxed::Box::new(FontNameTagHandler),
        alloc::boxed::Box::new(FontSizeTagHandler),
        alloc::boxed::Box::new(FontEncodingTagHandler),
    ]
}
