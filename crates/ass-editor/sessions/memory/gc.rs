//! Garbage collection, memory-pressure, and cleanup logic for [`MemoryPool`].
//!
//! Houses the reset-condition checks, pressure detection, GC cycle, and
//! statistics-update routines that complement the construction and accessor
//! methods defined in the sibling [`pool`](super::pool) module.

#[cfg(feature = "std")]
use std::time::Instant;

use super::pool::MemoryPool;

impl MemoryPool {
    /// Check if arena reset is needed based on configured conditions
    pub(super) fn check_reset_conditions(&mut self) {
        let should_reset = self.ops_since_reset >= self.config.reset_threshold
            || self.is_under_memory_pressure()
            || self.should_perform_gc();

        if should_reset {
            #[cfg(feature = "arena")]
            self.reset_primary_arena();
        }
    }

    /// Check if system is under memory pressure
    pub(super) fn is_under_memory_pressure(&self) -> bool {
        if self.config.max_arena_size == 0 {
            return false;
        }

        let pressure_ratio = self.stats.memory_in_use as f64 / self.config.max_arena_size as f64;
        pressure_ratio > self.config.pressure_threshold
    }

    /// Check if garbage collection should be performed
    pub(super) fn should_perform_gc(&self) -> bool {
        if !self.config.auto_gc {
            return false;
        }

        #[cfg(feature = "std")]
        {
            // Check time-based GC interval
            if let Some(last_gc) = self.last_gc {
                if last_gc.elapsed() < self.config.min_gc_interval {
                    return false;
                }
            }
        }

        // Check growth-based GC trigger
        if self.memory_at_last_gc > 0 {
            let growth_ratio = self.stats.memory_in_use as f64 / self.memory_at_last_gc as f64;
            growth_ratio > self.config.growth_rate_threshold
        } else {
            self.stats.memory_in_use > self.config.max_arena_size / 2
        }
    }

    /// Perform garbage collection
    pub fn collect_garbage(&mut self) -> usize {
        let memory_before = self.stats.memory_in_use;

        // Reset temp arena (safe to do)
        #[cfg(feature = "arena")]
        self.reset_temp_arena();

        // Consider resetting primary arena if pressure is high
        if self.is_under_memory_pressure() {
            #[cfg(feature = "arena")]
            self.reset_primary_arena();
        }

        self.stats.gc_cycles += 1;
        #[cfg(feature = "std")]
        {
            self.last_gc = Some(Instant::now());
        }
        self.memory_at_last_gc = self.stats.memory_in_use;

        memory_before.saturating_sub(self.stats.memory_in_use)
    }

    /// Update memory statistics
    pub(super) fn update_memory_stats(&mut self) {
        // In a real implementation, we would query the actual arena sizes
        // For now, we'll use estimates based on operations

        #[cfg(feature = "arena")]
        {
            // Estimate memory usage based on arena capacity
            let estimated_primary = self.primary_arena.allocated_bytes();
            let estimated_temp = self.temp_arena.allocated_bytes();

            self.stats.memory_in_use = estimated_primary + estimated_temp;
            self.stats.total_allocated = self.stats.memory_in_use;

            if self.stats.memory_in_use > self.stats.peak_memory {
                self.stats.peak_memory = self.stats.memory_in_use;
            }
        }
    }

    /// Force a complete memory cleanup
    pub fn force_cleanup(&mut self) {
        #[cfg(feature = "arena")]
        self.reset_all_arenas();

        self.ops_since_reset = 0;
        self.memory_at_last_gc = 0;

        #[cfg(feature = "std")]
        {
            self.last_gc = Some(Instant::now());
        }

        self.update_memory_stats();
    }
}
