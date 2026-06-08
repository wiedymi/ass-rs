//! Advanced formatting tag handlers for ASS override tags
//!
//! Implements handlers for advanced text formatting including borders,
//! shadows, and edge blur effects. These handlers validate numeric
//! arguments according to ASS specifications.
//!
//! # Supported Tags
//!
//! - `bord`: Border width (pixels)
//! - `shad`: Shadow depth (pixels)
//! - `be`: Blur edges (0/1)
//!
//! # Performance
//!
//! - Zero allocations for validation
//! - Fast numeric validation
//! - Minimal memory footprint per handler

mod handlers;

#[cfg(test)]
mod tests;

pub use handlers::{BlurEdgesTagHandler, BorderTagHandler, ShadowTagHandler};

use crate::plugin::TagHandler;

/// Create all advanced formatting tag handlers
///
/// Returns a vector of boxed tag handlers for advanced formatting operations.
///
/// # Example
///
/// ```rust
/// use ass_core::plugin::{ExtensionRegistry, tags::advanced::create_advanced_handlers};
///
/// let mut registry = ExtensionRegistry::new();
/// for handler in create_advanced_handlers() {
///     registry.register_tag_handler(handler).unwrap();
/// }
/// ```
#[must_use]
pub fn create_advanced_handlers() -> alloc::vec::Vec<alloc::boxed::Box<dyn TagHandler>> {
    alloc::vec![
        alloc::boxed::Box::new(BorderTagHandler),
        alloc::boxed::Box::new(ShadowTagHandler),
        alloc::boxed::Box::new(BlurEdgesTagHandler),
    ]
}
