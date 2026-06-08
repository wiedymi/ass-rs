//! Script Info AST node for ASS script metadata
//!
//! Contains the `ScriptInfo` struct representing the [Script Info] section
//! of ASS files with zero-copy design and convenient accessor methods
//! for common metadata fields.

use super::Span;

mod info;

#[cfg(test)]
mod tests;

pub use info::ScriptInfo;
