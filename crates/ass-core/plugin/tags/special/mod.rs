//! Special character tag handlers for ASS override tags
//!
//! Implements handlers for special characters like line breaks and
//! non-breaking spaces. These tags don't take arguments but affect
//! text layout and rendering.
//!
//! # Supported Tags
//!
//! - `n`: Soft line break (ignored with smart wrapping)
//! - `N`: Hard line break (always creates new line)
//! - `h`: Hard space (non-breaking space)
//!
//! # Performance
//!
//! - Zero allocations
//! - O(1) validation and processing
//! - Minimal memory footprint per handler

mod handlers;

#[cfg(test)]
mod tests;

pub use handlers::{HardLineBreakTagHandler, HardSpaceTagHandler, SoftLineBreakTagHandler};

use crate::plugin::TagHandler;

/// Create all special character tag handlers
///
/// Returns a vector of boxed tag handlers for special character operations.
///
/// # Example
///
/// ```rust
/// use ass_core::plugin::{ExtensionRegistry, tags::special::create_special_handlers};
///
/// let mut registry = ExtensionRegistry::new();
/// for handler in create_special_handlers() {
///     registry.register_tag_handler(handler).unwrap();
/// }
/// ```
#[must_use]
pub fn create_special_handlers() -> alloc::vec::Vec<alloc::boxed::Box<dyn TagHandler>> {
    alloc::vec![
        alloc::boxed::Box::new(SoftLineBreakTagHandler),
        alloc::boxed::Box::new(HardLineBreakTagHandler),
        alloc::boxed::Box::new(HardSpaceTagHandler),
    ]
}
