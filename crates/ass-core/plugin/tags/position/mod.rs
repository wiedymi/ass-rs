//! Position and movement tag handlers for ASS override tags
//!
//! Implements handlers for positioning and movement commands including
//! absolute positioning and animated movement. These handlers validate
//! numeric arguments and coordinate pairs according to ASS specifications.
//!
//! # Supported Tags
//!
//! - `pos`: Absolute positioning (x,y)
//! - `move`: Animated movement (x1,y1,x2,y2\[,t1,t2\])
//!
//! # Performance
//!
//! - Zero allocations for argument parsing
//! - Fast numeric validation
//! - Minimal memory footprint per handler

mod move_handler;
mod position_handler;
mod validation;

#[cfg(test)]
mod handler_tests;

pub use move_handler::MoveTagHandler;
pub use position_handler::PositionTagHandler;

use crate::plugin::TagHandler;

/// Create all position and movement tag handlers
///
/// Returns a vector of boxed tag handlers for position-related operations.
/// Useful for bulk registration with the extension registry.
///
/// # Example
///
/// ```rust
/// use ass_core::plugin::{ExtensionRegistry, tags::position::create_position_handlers};
///
/// let mut registry = ExtensionRegistry::new();
/// for handler in create_position_handlers() {
///     registry.register_tag_handler(handler).unwrap();
/// }
/// ```
#[must_use]
pub fn create_position_handlers() -> alloc::vec::Vec<alloc::boxed::Box<dyn TagHandler>> {
    alloc::vec![
        alloc::boxed::Box::new(PositionTagHandler),
        alloc::boxed::Box::new(MoveTagHandler),
    ]
}
