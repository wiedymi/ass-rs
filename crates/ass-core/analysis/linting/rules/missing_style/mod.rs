//! Missing style reference detection rule for ASS script linting.
//!
//! Detects events that reference undefined styles, which would cause
//! rendering errors or fallback to default styling behavior.

mod rule;

#[cfg(test)]
mod tests;

pub use rule::MissingStyleRule;
