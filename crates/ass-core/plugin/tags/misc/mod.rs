//! Miscellaneous tag handlers for ASS override tags
//!
//! Implements handlers for various utility tags including style reset,
//! rotation origin, and short-form rotation.
//!
//! # Supported Tags
//!
//! - `r`: Reset to style or default
//! - `fr`: Short form Z-axis rotation (alias for frz)
//! - `org`: Set rotation/transformation origin point
//!
//! # Performance
//!
//! - Zero allocations for simple tags
//! - Fast validation
//! - Minimal memory footprint per handler

mod handlers;
mod validation;

#[cfg(test)]
mod tests;

pub use handlers::{OriginTagHandler, ResetTagHandler, ShortRotationTagHandler};

use crate::plugin::TagHandler;

/// Create all miscellaneous tag handlers
///
/// Returns a vector of boxed tag handlers for misc operations.
///
/// # Example
///
/// ```rust
/// use ass_core::plugin::{ExtensionRegistry, tags::misc::create_misc_handlers};
///
/// let mut registry = ExtensionRegistry::new();
/// for handler in create_misc_handlers() {
///     registry.register_tag_handler(handler).unwrap();
/// }
/// ```
#[must_use]
pub fn create_misc_handlers() -> alloc::vec::Vec<alloc::boxed::Box<dyn TagHandler>> {
    alloc::vec![
        alloc::boxed::Box::new(ResetTagHandler),
        alloc::boxed::Box::new(ShortRotationTagHandler),
        alloc::boxed::Box::new(OriginTagHandler),
    ]
}
