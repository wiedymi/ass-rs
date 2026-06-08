//! Animation and effect tag handlers for ASS override tags
//!
//! Implements handlers for animation and transition effects including
//! transforms, fading, and alpha animations. These are some of the most
//! complex tags in the ASS format.
//!
//! # Supported Tags
//!
//! - `t`: Animated style transformations
//! - `fade`: Complex alpha animation
//! - `fad`: Simple fade in/out
//!
//! # Performance
//!
//! - Efficient argument parsing
//! - Complex validation for multi-parameter tags
//! - Minimal memory footprint per handler

mod fade_handlers;
mod transform_handlers;

#[cfg(test)]
mod fade_tests;
#[cfg(test)]
mod transform_tests;

pub use fade_handlers::{FadeTagHandler, SimpleFadeTagHandler};
pub use transform_handlers::TransformTagHandler;

use crate::plugin::TagHandler;

/// Create all animation tag handlers
///
/// Returns a vector of boxed tag handlers for animation operations.
///
/// # Example
///
/// ```rust
/// use ass_core::plugin::{ExtensionRegistry, tags::animation::create_animation_handlers};
///
/// let mut registry = ExtensionRegistry::new();
/// for handler in create_animation_handlers() {
///     registry.register_tag_handler(handler).unwrap();
/// }
/// ```
#[must_use]
pub fn create_animation_handlers() -> alloc::vec::Vec<alloc::boxed::Box<dyn TagHandler>> {
    alloc::vec![
        alloc::boxed::Box::new(TransformTagHandler),
        alloc::boxed::Box::new(FadeTagHandler),
        alloc::boxed::Box::new(SimpleFadeTagHandler),
    ]
}
