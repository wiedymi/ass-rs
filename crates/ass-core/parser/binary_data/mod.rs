//! Binary data parsing for `[Fonts]` and `[Graphics]` sections
//!
//! Handles UU-encoded font and graphic data embedded in ASS scripts.
//! Both sections use similar structure: filename declaration followed by
//! base64/UU-encoded data lines.

mod binary_parser;
mod section_parsers;

#[cfg(test)]
mod fonts_edge_tests;
#[cfg(test)]
mod fonts_tests;
#[cfg(test)]
mod graphics_edge_tests;
#[cfg(test)]
mod graphics_tests;

pub(in crate::parser) use section_parsers::{FontsParser, GraphicsParser};
