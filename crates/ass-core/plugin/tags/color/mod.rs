//! Color and alpha tag handlers for ASS override tags
//!
//! Implements handlers for color and transparency commands including
//! primary, secondary, outline, and shadow colors along with alpha channels.
//! Color values use BGR format (blue-green-red) as per ASS specification.
//!
//! # Supported Tags
//!
//! - `c` or `1c`: Primary color (&Hbbggrr&)
//! - `2c`: Secondary color (&Hbbggrr&)
//! - `3c`: Outline color (&Hbbggrr&)
//! - `4c`: Shadow color (&Hbbggrr&)
//! - `alpha`: All alpha channels (&Haa&)
//! - `1a`: Primary alpha (&Haa&)
//! - `2a`: Secondary alpha (&Haa&)
//! - `3a`: Outline alpha (&Haa&)
//! - `4a`: Shadow alpha (&Haa&)
//!
//! # Performance
//!
//! - Zero allocations for argument parsing
//! - SIMD-optimized hex parsing when available
//! - Fast validation with minimal branching

mod alpha_handlers;
mod color_handlers;
mod validation;

#[cfg(test)]
mod handler_tests;

pub use alpha_handlers::{
    Alpha1TagHandler, Alpha2TagHandler, Alpha3TagHandler, Alpha4TagHandler, AlphaTagHandler,
};
pub use color_handlers::{
    Color1TagHandler, Color2TagHandler, Color3TagHandler, Color4TagHandler, PrimaryColorTagHandler,
};

use crate::plugin::TagHandler;

/// Create all color and alpha tag handlers
///
/// Returns a vector of boxed tag handlers for color-related operations.
/// Useful for bulk registration with the extension registry.
///
/// # Example
///
/// ```rust
/// use ass_core::plugin::{ExtensionRegistry, tags::color::create_color_handlers};
///
/// let mut registry = ExtensionRegistry::new();
/// for handler in create_color_handlers() {
///     registry.register_tag_handler(handler).unwrap();
/// }
/// ```
#[must_use]
pub fn create_color_handlers() -> alloc::vec::Vec<alloc::boxed::Box<dyn TagHandler>> {
    alloc::vec![
        alloc::boxed::Box::new(PrimaryColorTagHandler),
        alloc::boxed::Box::new(Color1TagHandler),
        alloc::boxed::Box::new(Color2TagHandler),
        alloc::boxed::Box::new(Color3TagHandler),
        alloc::boxed::Box::new(Color4TagHandler),
        alloc::boxed::Box::new(AlphaTagHandler),
        alloc::boxed::Box::new(Alpha1TagHandler),
        alloc::boxed::Box::new(Alpha2TagHandler),
        alloc::boxed::Box::new(Alpha3TagHandler),
        alloc::boxed::Box::new(Alpha4TagHandler),
    ]
}
