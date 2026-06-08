//! Command type and constructors for modifying event effects.

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

/// Command to modify event effects
#[derive(Debug, Clone)]
pub struct EventEffectCommand {
    pub event_indices: Vec<usize>,
    pub effect: String,
    pub operation: EffectOperation,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectOperation {
    Set,     // Replace current effect
    Append,  // Add to existing effect
    Prepend, // Add before existing effect
    Clear,   // Remove all effects
}

impl EventEffectCommand {
    /// Create a new effect command
    pub fn new(event_indices: Vec<usize>, effect: String, operation: EffectOperation) -> Self {
        Self {
            event_indices,
            effect,
            operation,
            description: None,
        }
    }

    /// Set effect for specific events
    pub fn set_effect(event_indices: Vec<usize>, effect: String) -> Self {
        Self::new(event_indices, effect, EffectOperation::Set)
    }

    /// Clear effects for specific events
    pub fn clear_effect(event_indices: Vec<usize>) -> Self {
        Self::new(event_indices, String::new(), EffectOperation::Clear)
    }

    /// Append effect to specific events
    pub fn append_effect(event_indices: Vec<usize>, effect: String) -> Self {
        Self::new(event_indices, effect, EffectOperation::Append)
    }

    /// Set a custom description for this command
    #[must_use]
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}
