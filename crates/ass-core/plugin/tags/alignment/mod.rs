//! Alignment and layout tag handlers for ASS override tags
//!
//! Implements handlers for text alignment and wrapping style commands.
//! These handlers validate alignment codes according to ASS specifications.
//!
//! # Supported Tags
//!
//! - `a`: Legacy alignment (1-3 + 4/8 modifiers)
//! - `an`: Numpad-style alignment (1-9)
//! - `q`: Wrapping style (0-3)
//!
//! # Performance
//!
//! - Zero allocations for validation
//! - Fast integer validation
//! - Minimal memory footprint per handler

mod handlers;

#[cfg(test)]
mod tests;

pub use handlers::{AlignmentTagHandler, NumpadAlignmentTagHandler, WrappingStyleTagHandler};

use crate::plugin::TagHandler;

/// Create all alignment tag handlers
///
/// Returns a vector of boxed tag handlers for alignment operations.
///
/// # Example
///
/// ```rust
/// use ass_core::plugin::{ExtensionRegistry, tags::alignment::create_alignment_handlers};
///
/// let mut registry = ExtensionRegistry::new();
/// for handler in create_alignment_handlers() {
///     registry.register_tag_handler(handler).unwrap();
/// }
/// ```
#[must_use]
pub fn create_alignment_handlers() -> alloc::vec::Vec<alloc::boxed::Box<dyn TagHandler>> {
    alloc::vec![
        alloc::boxed::Box::new(AlignmentTagHandler),
        alloc::boxed::Box::new(NumpadAlignmentTagHandler),
        alloc::boxed::Box::new(WrappingStyleTagHandler),
    ]
}
