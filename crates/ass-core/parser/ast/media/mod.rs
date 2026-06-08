//! Media AST nodes for embedded fonts and graphics
//!
//! Contains Font and Graphic structs representing embedded media from the
//! [Fonts] and [Graphics] sections with zero-copy design and UU-decoding.

mod font;
mod graphic;

#[cfg(test)]
mod font_tests;
#[cfg(test)]
mod graphic_tests;

pub use font::Font;
pub use graphic::Graphic;
