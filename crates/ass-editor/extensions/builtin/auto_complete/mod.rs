//! Built-in auto-completion extension for ASS/SSA files
//!
//! Provides intelligent auto-completion for:
//! - Section names
//! - Field names based on current section
//! - Style names when referenced in events
//! - Override tags and their parameters
//! - Color codes and common values

mod completions;
mod context;
mod editor_ext;
mod extension;
mod tags;
mod types;

#[cfg(test)]
mod tests;

// Include extended tests
#[cfg(test)]
#[path = "../auto_complete_tests.rs"]
mod extended_tests;

pub use extension::AutoCompleteExtension;
pub use types::{AutoCompleteConfig, CompletionContext, CompletionItem, CompletionType};
