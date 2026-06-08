//! Command system for editor operations
//!
//! Provides a trait-based command system with fluent APIs for text editing,
//! undo/redo support, and extensible command types. All commands are atomic
//! and can be undone/redone efficiently.

pub mod delta_commands;
pub mod event_commands;
pub mod fonts_graphics_commands;
pub mod karaoke_commands;
pub mod macros;
pub mod script_info_commands;
pub mod style_commands;
pub mod tag_commands;

mod batch_command;
mod command_result;
mod command_trait;
mod fluent_builder;
mod text_commands;

#[cfg(test)]
mod commands_tests;

// Re-export all command modules
pub use delta_commands::*;
pub use event_commands::*;
pub use fonts_graphics_commands::*;
pub use karaoke_commands::*;
pub use script_info_commands::*;
pub use style_commands::*;
pub use tag_commands::*;

pub use batch_command::BatchCommand;
pub use command_result::CommandResult;
pub use command_trait::EditorCommand;
pub use fluent_builder::{DocumentCommandExt, TextCommand};
pub use text_commands::{DeleteTextCommand, InsertTextCommand, ReplaceTextCommand};
