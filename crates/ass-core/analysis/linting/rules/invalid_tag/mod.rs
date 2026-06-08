//! Invalid tag detection rule for ASS script linting.
//!
//! Detects invalid or malformed override tags in event text that would
//! cause parsing errors or unexpected rendering behavior.

mod rule;

#[cfg(test)]
mod tests;

pub use rule::InvalidTagRule;
