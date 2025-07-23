//! ASS section processors for the plugin system
//!
//! Provides implementations of the `SectionProcessor` trait for handling
//! non-standard ASS sections. Processors validate section content and
//! handle extended functionality beyond the core ASS specification.
//!
//! # Modules
//!
//! - [`aegisub`] - Aegisub-specific sections (project metadata, extradata)
//!
//! # Usage
//!
//! ```rust
//! use ass_core::plugin::{ExtensionRegistry, sections::aegisub::create_aegisub_processors};
//!
//! let mut registry = ExtensionRegistry::new();
//! for processor in create_aegisub_processors() {
//!     registry.register_section_processor(processor).unwrap();
//! }
//! ```

pub mod aegisub;

pub use aegisub::{create_aegisub_processors, AegisubExtradataProcessor, AegisubProjectProcessor};
