//! Style AST node for ASS style definitions
//!
//! Contains the Style struct representing style definitions from the
//! [V4+ Styles] section with zero-copy design and style property accessors.

mod conversion;
mod definition;

#[cfg(test)]
mod construction_tests;
#[cfg(test)]
mod conversion_tests;
#[cfg(test)]
mod equality_combinations_tests;
#[cfg(test)]
mod equality_tests;
#[cfg(test)]
mod field_tests;
#[cfg(all(test, debug_assertions))]
mod validation_tests;

pub use definition::Style;
