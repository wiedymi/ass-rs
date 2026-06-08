//! Text encoding error utilities for ASS-RS
//!
//! Provides specialized error creation and validation functions for text
//! encoding issues including UTF-8 validation, encoding detection, and
//! character conversion errors. Focuses on providing detailed context.

mod creation;
mod validation;

/// Check for common encoding issues in ASS content
///
/// Performs heuristic checks for common encoding problems that can occur
/// when ASS files are saved with incorrect encoding settings.
///
/// # Arguments
///
/// * `text` - Text content to analyze
#[cfg(test)]
mod tests;

pub use creation::{utf8_error, validation_error};
pub use validation::{validate_ass_text_content, validate_bom_handling, validate_utf8_detailed};
