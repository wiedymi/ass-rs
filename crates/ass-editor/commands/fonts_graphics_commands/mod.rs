//! Fonts and Graphics management commands for ASS documents
//!
//! Provides commands for managing embedded fonts and graphics in the
//! `[Fonts]` and `[Graphics]` sections using UU-encoding.

mod add;
mod clear;
mod encoding;
mod list;
mod remove;

#[cfg(test)]
mod tests;

pub use add::{AddFontCommand, AddGraphicCommand};
pub use clear::{ClearFontsCommand, ClearGraphicsCommand};
pub use list::{ListFontsCommand, ListGraphicsCommand};
pub use remove::{RemoveFontCommand, RemoveGraphicCommand};

use encoding::uuencode_data;
