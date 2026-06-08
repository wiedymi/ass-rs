//! Fluent API builders for toggling event types and modifying effects.

use crate::commands::{EditorCommand, EffectOperation, EventEffectCommand, ToggleEventTypeCommand};
use crate::core::{EditorDocument, Result};

#[cfg(not(feature = "std"))]
use alloc::{string::ToString, vec, vec::Vec};

/// Fluent API builder for toggling event types
pub struct EventToggler<'a> {
    document: &'a mut EditorDocument,
    event_indices: Vec<usize>,
}

impl<'a> EventToggler<'a> {
    /// Create a new event toggler
    pub(crate) fn new(document: &'a mut EditorDocument) -> Self {
        Self {
            document,
            event_indices: Vec::new(), // Default to all events
        }
    }

    /// Specify which events to toggle
    pub fn events(mut self, indices: Vec<usize>) -> Self {
        self.event_indices = indices;
        self
    }

    /// Toggle a single event
    pub fn event(mut self, index: usize) -> Self {
        self.event_indices = vec![index];
        self
    }

    /// Execute the toggle
    pub fn apply(self) -> Result<&'a mut EditorDocument> {
        let command = ToggleEventTypeCommand::new(self.event_indices);
        command.execute(self.document)?;
        Ok(self.document)
    }
}

/// Fluent API builder for event effects
pub struct EventEffector<'a> {
    document: &'a mut EditorDocument,
    event_indices: Vec<usize>,
}

impl<'a> EventEffector<'a> {
    /// Create a new event effector
    pub(crate) fn new(document: &'a mut EditorDocument) -> Self {
        Self {
            document,
            event_indices: Vec::new(), // Default to all events
        }
    }

    /// Specify which events to modify
    pub fn events(mut self, indices: Vec<usize>) -> Self {
        self.event_indices = indices;
        self
    }

    /// Modify a single event
    pub fn event(mut self, index: usize) -> Self {
        self.event_indices = vec![index];
        self
    }

    /// Set the effect (replace existing)
    pub fn set(self, effect: &str) -> Result<&'a mut EditorDocument> {
        let command = EventEffectCommand::set_effect(self.event_indices, effect.to_string());
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Clear all effects
    pub fn clear(self) -> Result<&'a mut EditorDocument> {
        let command = EventEffectCommand::clear_effect(self.event_indices);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Append to existing effect
    pub fn append(self, effect: &str) -> Result<&'a mut EditorDocument> {
        let command = EventEffectCommand::append_effect(self.event_indices, effect.to_string());
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Prepend to existing effect
    pub fn prepend(self, effect: &str) -> Result<&'a mut EditorDocument> {
        let command = EventEffectCommand::new(
            self.event_indices,
            effect.to_string(),
            EffectOperation::Prepend,
        );
        command.execute(self.document)?;
        Ok(self.document)
    }
}
