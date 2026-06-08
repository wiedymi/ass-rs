//! Configuration for undo-stack capacity and memory behavior.
//!
//! [`UndoStackConfig`] controls how many entries the undo stack retains, its
//! memory budget, compression, and arena-reset cadence.

/// Configuration for undo stack behavior
#[derive(Debug, Clone)]
pub struct UndoStackConfig {
    /// Maximum number of undo entries to keep
    pub max_entries: usize,

    /// Maximum memory usage in bytes (0 = unlimited)
    pub max_memory: usize,

    /// Whether to enable compression of old entries
    pub enable_compression: bool,

    /// Interval for arena resets (0 = never reset)
    pub arena_reset_interval: usize,
}

impl Default for UndoStackConfig {
    fn default() -> Self {
        Self {
            max_entries: 50, // Set a sensible default, can be overridden programmatically
            max_memory: 10 * 1024 * 1024, // 10MB default
            enable_compression: true,
            arena_reset_interval: 100, // Reset arena every 100 operations
        }
    }
}
