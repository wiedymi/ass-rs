//! ASS override tag handlers for the plugin system
//!
//! Provides implementations of the `TagHandler` trait for all standard ASS
//! override tags. Handlers validate tag arguments and process tag operations
//! according to ASS specification compliance.
//!
//! # Modules
//!
//! - [`formatting`] - Basic text formatting tags (bold, italic, underline, strikeout)
//!
//! # Usage
//!
//! ```rust
//! use ass_core::plugin::{ExtensionRegistry, tags::formatting::create_formatting_handlers};
//!
//! let mut registry = ExtensionRegistry::new();
//! for handler in create_formatting_handlers() {
//!     registry.register_tag_handler(handler).unwrap();
//! }
//! ```

pub mod formatting;

pub use formatting::{
    create_formatting_handlers, BoldTagHandler, ItalicTagHandler, StrikeoutTagHandler,
    UnderlineTagHandler,
};
