//! Transform and rotation tag handlers for ASS override tags
//!
//! Implements handlers for transformation commands including rotation,
//! scaling, shearing, and spacing. These handlers validate numeric
//! arguments and handle both integer and decimal values.
//!
//! # Supported Tags
//!
//! - `frz`: Z-axis rotation (degrees)
//! - `frx`: X-axis rotation (degrees)
//! - `fry`: Y-axis rotation (degrees)
//! - `fscx`: X-axis scale (percent)
//! - `fscy`: Y-axis scale (percent)
//! - `fax`: X-axis shear factor
//! - `fay`: Y-axis shear factor
//! - `fsp`: Letter spacing (pixels)
//!
//! # Performance
//!
//! - Zero allocations for argument parsing
//! - Fast numeric validation
//! - Minimal memory footprint per handler

mod rotation_handlers;
mod scale_handlers;
mod shear_handlers;
mod spacing_handlers;
mod validation;

#[cfg(test)]
mod handler_tests;

#[cfg(test)]
mod integration_tests;

pub use rotation_handlers::{RotationXTagHandler, RotationYTagHandler, RotationZTagHandler};
pub use scale_handlers::{ScaleXTagHandler, ScaleYTagHandler};
pub use shear_handlers::{ShearXTagHandler, ShearYTagHandler};
pub use spacing_handlers::SpacingTagHandler;

use crate::plugin::TagHandler;

/// Create all transform and rotation tag handlers
///
/// Returns a vector of boxed tag handlers for transformation operations.
/// Useful for bulk registration with the extension registry.
///
/// # Example
///
/// ```rust
/// use ass_core::plugin::{ExtensionRegistry, tags::transform::create_transform_handlers};
///
/// let mut registry = ExtensionRegistry::new();
/// for handler in create_transform_handlers() {
///     registry.register_tag_handler(handler).unwrap();
/// }
/// ```
#[must_use]
pub fn create_transform_handlers() -> alloc::vec::Vec<alloc::boxed::Box<dyn TagHandler>> {
    alloc::vec![
        alloc::boxed::Box::new(RotationZTagHandler),
        alloc::boxed::Box::new(RotationXTagHandler),
        alloc::boxed::Box::new(RotationYTagHandler),
        alloc::boxed::Box::new(ScaleXTagHandler),
        alloc::boxed::Box::new(ScaleYTagHandler),
        alloc::boxed::Box::new(ShearXTagHandler),
        alloc::boxed::Box::new(ShearYTagHandler),
        alloc::boxed::Box::new(SpacingTagHandler),
    ]
}
