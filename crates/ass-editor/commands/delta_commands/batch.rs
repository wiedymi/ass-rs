//! Batch command for executing multiple delta commands efficiently

use super::DeltaCommand;
use crate::core::{EditorDocument, Result};

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, vec::Vec};

/// Batch command that executes multiple delta commands efficiently
pub struct DeltaBatchCommand {
    commands: Vec<Box<dyn DeltaCommand>>,
}

impl DeltaBatchCommand {
    /// Create a new batch command
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    /// Add a command to the batch
    pub fn add_command<T: DeltaCommand + 'static>(mut self, command: T) -> Self {
        self.commands.push(Box::new(command));
        self
    }

    /// Execute all commands in the batch
    pub fn execute_batch(
        &self,
        document: &mut EditorDocument,
    ) -> Result<Vec<super::CommandResult>> {
        let mut results = Vec::new();

        for command in &self.commands {
            let result = command.execute_with_delta(document)?;
            results.push(result);
        }

        Ok(results)
    }
}

impl Default for DeltaBatchCommand {
    fn default() -> Self {
        Self::new()
    }
}
