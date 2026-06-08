//! Reset strategies and the smart arena manager for [`MemoryPool`].
//!
//! Defines [`ResetStrategy`], the set of policies for reclaiming arena
//! memory, and [`SmartArenaManager`], which evaluates and applies a chosen
//! strategy against an owned [`MemoryPool`](super::MemoryPool).

#[cfg(feature = "std")]
use std::time::{Duration, Instant};

use super::pool::MemoryPool;

/// Arena reset strategy for different usage patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResetStrategy {
    /// Reset after a fixed number of operations
    OperationCount(usize),

    /// Reset when memory usage exceeds threshold
    MemoryThreshold(usize),

    /// Reset based on time intervals
    #[cfg(feature = "std")]
    TimeInterval(Duration),

    /// Reset when memory pressure exceeds ratio
    PressureRatio(f64),

    /// Hybrid strategy combining multiple factors
    Hybrid,

    /// Never reset automatically
    Manual,
}

impl Default for ResetStrategy {
    fn default() -> Self {
        Self::Hybrid
    }
}

/// Smart arena manager that applies different reset strategies
#[derive(Debug)]
pub struct SmartArenaManager {
    /// The underlying memory pool
    pool: MemoryPool,

    /// Reset strategy to use
    strategy: ResetStrategy,

    /// Last strategy evaluation time
    #[cfg(feature = "std")]
    last_evaluation: Instant,
}

impl SmartArenaManager {
    /// Create a new smart arena manager
    pub fn new(strategy: ResetStrategy) -> Self {
        Self {
            pool: MemoryPool::new(),
            strategy,
            #[cfg(feature = "std")]
            last_evaluation: Instant::now(),
        }
    }

    /// Evaluate and potentially apply reset strategy
    pub fn evaluate_reset_strategy(&mut self) -> bool {
        match self.strategy {
            ResetStrategy::OperationCount(threshold) => {
                if self.pool.ops_since_reset >= threshold {
                    self.pool.force_cleanup();
                    true
                } else {
                    false
                }
            }

            ResetStrategy::MemoryThreshold(threshold) => {
                if self.pool.stats.memory_in_use >= threshold {
                    self.pool.force_cleanup();
                    true
                } else {
                    false
                }
            }

            #[cfg(feature = "std")]
            ResetStrategy::TimeInterval(interval) => {
                if self.last_evaluation.elapsed() >= interval {
                    self.pool.force_cleanup();
                    self.last_evaluation = Instant::now();
                    true
                } else {
                    false
                }
            }

            ResetStrategy::PressureRatio(ratio) => {
                if self.pool.is_under_memory_pressure() {
                    let current_ratio = self.pool.stats.memory_in_use as f64
                        / self.pool.config.max_arena_size as f64;
                    if current_ratio >= ratio {
                        self.pool.force_cleanup();
                        return true;
                    }
                }
                false
            }

            ResetStrategy::Hybrid => {
                // Combine multiple strategies
                let should_reset = self.pool.ops_since_reset >= 1000
                    || self.pool.is_under_memory_pressure()
                    || self.pool.should_perform_gc();

                if should_reset {
                    self.pool.collect_garbage();
                    true
                } else {
                    false
                }
            }

            ResetStrategy::Manual => false, // Never reset automatically
        }
    }

    /// Get reference to underlying memory pool
    #[must_use]
    pub fn pool(&self) -> &MemoryPool {
        &self.pool
    }

    /// Get mutable reference to underlying memory pool
    pub fn pool_mut(&mut self) -> &mut MemoryPool {
        self.evaluate_reset_strategy();
        &mut self.pool
    }

    /// Change reset strategy
    pub fn set_strategy(&mut self, strategy: ResetStrategy) {
        self.strategy = strategy;
    }

    /// Get current strategy
    #[must_use]
    pub const fn strategy(&self) -> ResetStrategy {
        self.strategy
    }
}
