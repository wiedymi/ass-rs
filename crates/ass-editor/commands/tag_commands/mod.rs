//! Tag management commands for ASS override tags
//!
//! Provides commands for inserting, removing, replacing, wrapping, and parsing
//! ASS override tags like \b1, \i1, \c&H00FF00&, \pos(100,200), etc. with
//! proper validation and nested tag handling.

#![allow(clippy::while_let_on_iterator)]

mod insert;
mod parse;
mod remove;
mod replace;
mod wrap;

#[cfg(test)]
mod tests;

pub use insert::InsertTagCommand;
pub use parse::{ParseTagCommand, ParsedTag};
pub use remove::RemoveTagCommand;
pub use replace::ReplaceTagCommand;
pub use wrap::WrapTagCommand;
