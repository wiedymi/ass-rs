//! Memory pool configuration for editor sessions.
//!
//! Defines [`MemoryPoolConfig`], the tunable thresholds that drive arena
//! reset cadence, garbage-collection timing, and memory-pressure detection
//! in [`MemoryPool`](super::MemoryPool).

#[cfg(feature = "std")]
use std::time::Duration;

/// Memory pool configuration
#[derive(Debug, Clone)]
pub struct MemoryPoolConfig {
    /// Maximum memory per arena before forcing reset
    pub max_arena_size: usize,

    /// Number of operations before considering arena reset
    pub reset_threshold: usize,

    /// Memory pressure threshold (0.0 - 1.0)
    pub pressure_threshold: f64,

    /// Enable automatic garbage collection
    pub auto_gc: bool,

    /// Minimum time between GC cycles
    #[cfg(feature = "std")]
    pub min_gc_interval: Duration,

    /// Memory growth rate that triggers GC
    pub growth_rate_threshold: f64,
}

impl Default for MemoryPoolConfig {
    fn default() -> Self {
        Self {
            max_arena_size: 64 * 1024 * 1024, // 64MB per arena
            reset_threshold: 1000,
            pressure_threshold: 0.8, // 80% memory pressure
            auto_gc: true,
            #[cfg(feature = "std")]
            min_gc_interval: Duration::from_secs(30),
            growth_rate_threshold: 2.0, // 200% growth triggers GC
        }
    }
}
