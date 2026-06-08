//! Memory usage statistics for editor sessions.
//!
//! Defines [`MemoryStats`], the lightweight counter struct used by
//! [`MemoryPool`](super::MemoryPool) to track allocation, reclamation, and
//! garbage-collection activity across long editing sessions.

/// Memory usage statistics for monitoring
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryStats {
    /// Total memory allocated across all arenas
    pub total_allocated: usize,

    /// Memory currently in use
    pub memory_in_use: usize,

    /// Memory that can be reclaimed
    pub reclaimable_memory: usize,

    /// Number of arena resets performed
    pub arena_resets: usize,

    /// Number of garbage collection cycles
    pub gc_cycles: usize,

    /// Peak memory usage
    pub peak_memory: usize,
}

impl Default for MemoryStats {
    fn default() -> Self {
        Self {
            total_allocated: 0,
            memory_in_use: 0,
            reclaimable_memory: 0,
            arena_resets: 0,
            gc_cycles: 0,
            peak_memory: 0,
        }
    }
}
