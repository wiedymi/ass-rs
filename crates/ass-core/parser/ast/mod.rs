//! AST (Abstract Syntax Tree) definitions for ASS scripts
//!
//! Provides zero-copy AST nodes using lifetime-generic design for maximum performance.
//! All nodes reference spans in the original source text to avoid allocations.
//!
//! # Thread Safety
//!
//! All AST nodes are immutable after construction and implement `Send + Sync`
//! for safe multi-threaded access.
//!
//! # Performance
//!
//! - Zero allocations via `&'a str` spans
//! - Memory usage ~1.1x input size
//! - Validation via pointer arithmetic for span checking
//!
//! # Examples
//!
//! ```rust
//! use ass_core::parser::ast::{Section, ScriptInfo, Event, EventType, Span};
//!
//! // Create script info
//! let info = ScriptInfo { fields: vec![("Title", "Test")], span: Span::new(0, 0, 0, 0) };
//! let section = Section::ScriptInfo(info);
//!
//! // Create dialogue event
//! let event = Event {
//!     event_type: EventType::Dialogue,
//!     start: "0:00:05.00",
//!     end: "0:00:10.00",
//!     text: "Hello World!",
//!     ..Event::default()
//! };
//! ```

#[cfg(not(feature = "std"))]
extern crate alloc;

mod event;
mod media;
mod script_info;
mod section;
mod span;
mod style;

#[cfg(test)]
mod span_tests;

// Re-export all public types to maintain API compatibility
pub use event::{Event, EventType};
pub use media::{Font, Graphic};
pub use script_info::ScriptInfo;
pub use section::{Section, SectionType};
pub use span::Span;
pub use style::Style;
