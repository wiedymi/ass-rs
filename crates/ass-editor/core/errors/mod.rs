//! Error types for the ass-editor crate
//!
//! Provides the main `EditorError` enum that wraps `CoreError` from ass-core
//! and adds editor-specific error cases. Follows the same philosophy as core:
//! - Use thiserror for structured error handling (no anyhow)
//! - Provide detailed context for debugging
//! - Support error chains with source information
//! - Maintain zero-cost error handling where possible

mod display;
mod methods;
mod types;

#[cfg(test)]
mod tests;

pub use types::{EditorError, Result};
