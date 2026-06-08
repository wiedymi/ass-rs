//! Script Info management commands for ASS documents
//!
//! Provides commands for managing script metadata like Title, Author, Resolution,
//! and other properties in the `[Script Info]` section.

mod delete;
mod get;
mod set;

#[cfg(test)]
mod tests;

pub use delete::DeleteScriptInfoCommand;
pub use get::{GetAllScriptInfoCommand, GetScriptInfoCommand};
pub use set::SetScriptInfoCommand;
