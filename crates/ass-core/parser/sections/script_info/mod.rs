//! Script Info section parser for ASS scripts.
//!
//! Handles parsing of the [Script Info] section which contains metadata
//! and configuration parameters for the subtitle script.

mod parser;

#[cfg(test)]
mod tests;

pub use parser::ScriptInfoParser;
