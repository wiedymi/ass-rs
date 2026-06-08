//! AST section types and validation for ASS scripts
//!
//! Defines the top-level Section enum that represents the main sections
//! of an ASS script ([Script Info], [V4+ Styles], [Events], etc.) with
//! zero-copy design and span validation for debugging.

mod section_enum;
mod section_type;

#[cfg(test)]
mod section_tests;

pub use section_enum::Section;
pub use section_type::SectionType;
