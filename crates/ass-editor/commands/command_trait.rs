//! The core `EditorCommand` trait implemented by all editor commands.

use crate::core::{EditorDocument, Result};

use super::CommandResult;

/// Trait for editor commands that can be executed and undone
///
/// All commands implement this trait to provide a consistent interface
/// for execution, undo/redo, and introspection.
///
/// # Examples
///
/// Creating a custom command:
///
/// ```
/// use ass_editor::{EditorCommand, EditorDocument, CommandResult, Result, Position, Range};
///
/// #[derive(Debug)]
/// struct UppercaseCommand {
///     description: String,
/// }
///
/// impl UppercaseCommand {
///     fn new() -> Self {
///         Self {
///             description: "Convert to uppercase".to_string(),
///         }
///     }
/// }
///
/// impl EditorCommand for UppercaseCommand {
///     fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
///         let text = document.text().to_uppercase();
///         let range = Range::new(Position::new(0), Position::new(document.len()));
///         document.replace(range, &text)?;
///         Ok(CommandResult::success().with_message("Text converted to uppercase".to_string()))
///     }
///
///     fn description(&self) -> &str {
///         &self.description
///     }
/// }
/// ```
pub trait EditorCommand: core::fmt::Debug + Send + Sync {
    /// Execute the command on the given document
    ///
    /// Returns a result indicating success/failure and metadata about
    /// the operation for undo/redo tracking.
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult>;

    /// Get a human-readable description of the command
    fn description(&self) -> &str;

    /// Check if this command modifies document content
    ///
    /// Used to determine if the document should be marked as modified
    /// and whether to save undo state.
    fn modifies_content(&self) -> bool {
        true
    }

    /// Get the estimated memory usage of this command
    ///
    /// Used for memory management in undo stacks with limited capacity.
    /// Default implementation provides a conservative estimate.
    fn memory_usage(&self) -> usize {
        64 // Conservative default estimate for command overhead
    }
}
