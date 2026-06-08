//! Style management commands for ASS documents
//!
//! Provides commands for creating, editing, deleting, cloning, and applying styles
//! with proper validation and delta tracking.

mod apply;
mod clone;
mod create;
mod delete;
mod edit;

#[cfg(test)]
mod tests;

pub use apply::ApplyStyleCommand;
pub use clone::CloneStyleCommand;
pub use create::CreateStyleCommand;
pub use delete::DeleteStyleCommand;
pub use edit::EditStyleCommand;
