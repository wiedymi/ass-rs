//! Memory pool manager owning the session arenas.
//!
//! Defines [`MemoryPool`], which holds the primary/temporary arenas and the
//! [`MemoryStats`] counters, exposing construction, arena access, reset, and
//! statistics accessors. Garbage-collection and memory-pressure logic live in
//! the sibling [`gc`](super::gc) module.

#[cfg(feature = "arena")]
use bumpalo::Bump;

#[cfg(not(feature = "std"))]
use alloc::string::String;

#[cfg(feature = "std")]
use std::time::Instant;

use super::{config::MemoryPoolConfig, stats::MemoryStats};

/// Memory pool manager for efficient allocation and cleanup
///
/// Manages multiple arenas with intelligent reset strategies to minimize
/// memory fragmentation and ensure consistent performance across long
/// editing sessions.
#[derive(Debug)]
pub struct MemoryPool {
    /// Configuration for this pool
    pub(super) config: MemoryPoolConfig,

    /// Primary arena for active allocations
    #[cfg(feature = "arena")]
    pub(super) primary_arena: Bump,

    /// Secondary arena for temporary allocations
    #[cfg(feature = "arena")]
    pub(super) temp_arena: Bump,

    /// Memory usage statistics
    pub(super) stats: MemoryStats,

    /// Operations since last reset
    pub(super) ops_since_reset: usize,

    /// Last garbage collection time
    #[cfg(feature = "std")]
    pub(super) last_gc: Option<Instant>,

    /// Memory usage at last GC
    pub(super) memory_at_last_gc: usize,
}

impl MemoryPool {
    /// Create a new memory pool with default configuration
    pub fn new() -> Self {
        Self::with_config(MemoryPoolConfig::default())
    }

    /// Create a new memory pool with custom configuration
    pub fn with_config(config: MemoryPoolConfig) -> Self {
        Self {
            config,
            #[cfg(feature = "arena")]
            primary_arena: Bump::new(),
            #[cfg(feature = "arena")]
            temp_arena: Bump::new(),
            stats: MemoryStats::default(),
            ops_since_reset: 0,
            #[cfg(feature = "std")]
            last_gc: None,
            memory_at_last_gc: 0,
        }
    }

    /// Get the primary arena for long-lived allocations
    #[cfg(feature = "arena")]
    #[must_use]
    pub fn primary_arena(&self) -> &Bump {
        &self.primary_arena
    }

    /// Get mutable reference to primary arena
    #[cfg(feature = "arena")]
    pub fn primary_arena_mut(&mut self) -> &mut Bump {
        self.ops_since_reset += 1;
        self.check_reset_conditions();
        &mut self.primary_arena
    }

    /// Get the temporary arena for short-lived allocations
    #[cfg(feature = "arena")]
    #[must_use]
    pub fn temp_arena(&self) -> &Bump {
        &self.temp_arena
    }

    /// Get mutable reference to temporary arena
    #[cfg(feature = "arena")]
    pub fn temp_arena_mut(&mut self) -> &mut Bump {
        &mut self.temp_arena
    }

    /// Reset the primary arena to reclaim memory
    #[cfg(feature = "arena")]
    pub fn reset_primary_arena(&mut self) {
        self.primary_arena.reset();
        self.stats.arena_resets += 1;
        self.ops_since_reset = 0;
        self.update_memory_stats();
    }

    /// Reset the temporary arena
    #[cfg(feature = "arena")]
    pub fn reset_temp_arena(&mut self) {
        self.temp_arena.reset();
        self.update_memory_stats();
    }

    /// Reset both arenas
    #[cfg(feature = "arena")]
    pub fn reset_all_arenas(&mut self) {
        self.reset_primary_arena();
        self.reset_temp_arena();
    }

    /// Get current memory statistics
    #[must_use]
    pub fn stats(&self) -> &MemoryStats {
        &self.stats
    }

    /// Get memory usage in human-readable format
    #[must_use]
    pub fn memory_usage_string(&self) -> String {
        let mb = self.stats.memory_in_use as f64 / (1024.0 * 1024.0);
        format!("{mb:.2}MB")
    }

    /// Check if memory usage is within acceptable limits
    #[must_use]
    pub fn is_memory_usage_acceptable(&self) -> bool {
        self.stats.memory_in_use <= self.config.max_arena_size
    }

    /// Set memory pool configuration
    pub fn set_config(&mut self, config: MemoryPoolConfig) {
        self.config = config;
    }

    /// Get current configuration
    #[must_use]
    pub fn config(&self) -> &MemoryPoolConfig {
        &self.config
    }
}

impl Default for MemoryPool {
    fn default() -> Self {
        Self::new()
    }
}
