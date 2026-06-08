//! Invalid color format detection rule for ASS script linting.
//!
//! Detects invalid color formats in both style definitions and override tags
//! that would cause rendering errors or unexpected visual results.

mod rule;

#[cfg(test)]
mod tests;

pub use rule::InvalidColorRule;
