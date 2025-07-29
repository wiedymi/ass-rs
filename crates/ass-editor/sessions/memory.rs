//! Memory management for editor sessions
//!
//! Provides arena reset logic, memory pooling, and garbage collection
//! strategies to prevent memory accumulation and ensure efficient
//! resource utilization across multiple editing sessions.

#[cfg(feature = "arena")]
use bumpalo::Bump;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

#[cfg(feature = "std")]
use std::time::{Duration, Instant};

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

/// Memory pool manager for efficient allocation and cleanup
///
/// Manages multiple arenas with intelligent reset strategies to minimize
/// memory fragmentation and ensure consistent performance across long
/// editing sessions.
#[derive(Debug)]
pub struct MemoryPool {
    /// Configuration for this pool
    config: MemoryPoolConfig,
    
    /// Primary arena for active allocations
    #[cfg(feature = "arena")]
    primary_arena: Bump,
    
    /// Secondary arena for temporary allocations
    #[cfg(feature = "arena")]
    temp_arena: Bump,
    
    /// Memory usage statistics
    stats: MemoryStats,
    
    /// Operations since last reset
    ops_since_reset: usize,
    
    /// Last garbage collection time
    #[cfg(feature = "std")]
    last_gc: Option<Instant>,
    
    /// Memory usage at last GC
    memory_at_last_gc: usize,
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
    
    /// Check if arena reset is needed based on configured conditions
    fn check_reset_conditions(&mut self) {
        let should_reset = self.ops_since_reset >= self.config.reset_threshold
            || self.is_under_memory_pressure()
            || self.should_perform_gc();
        
        if should_reset {
            #[cfg(feature = "arena")]
            self.reset_primary_arena();
        }
    }
    
    /// Check if system is under memory pressure
    fn is_under_memory_pressure(&self) -> bool {
        if self.config.max_arena_size == 0 {
            return false;
        }
        
        let pressure_ratio = self.stats.memory_in_use as f64 / self.config.max_arena_size as f64;
        pressure_ratio > self.config.pressure_threshold
    }
    
    /// Check if garbage collection should be performed
    fn should_perform_gc(&self) -> bool {
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
    fn update_memory_stats(&mut self) {
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn memory_pool_creation() {
        let pool = MemoryPool::new();
        assert_eq!(pool.stats().memory_in_use, 0);
        assert_eq!(pool.stats().arena_resets, 0);
    }
    
    #[test]
    fn memory_pool_configuration() {
        let config = MemoryPoolConfig {
            max_arena_size: 1024,
            reset_threshold: 10,
            ..Default::default()
        };
        
        let pool = MemoryPool::with_config(config);
        assert_eq!(pool.config().max_arena_size, 1024);
        assert_eq!(pool.config().reset_threshold, 10);
    }
    
    #[test]
    #[cfg(feature = "arena")]
    fn arena_reset_functionality() {
        let mut pool = MemoryPool::new();
        
        // Trigger operations to exceed threshold
        for _ in 0..1001 {
            let _ = pool.primary_arena_mut();
        }
        
        // Should have triggered at least one reset
        assert!(pool.stats().arena_resets > 0);
    }
    
    #[test]
    fn garbage_collection() {
        let mut pool = MemoryPool::new();
        
        // Force garbage collection
        let reclaimed = pool.collect_garbage();
        assert_eq!(pool.stats().gc_cycles, 1);
    }
    
    #[test]
    fn smart_arena_manager() {
        let mut manager = SmartArenaManager::new(ResetStrategy::OperationCount(100));
        
        // Should not reset initially
        assert!(!manager.evaluate_reset_strategy());
        
        // Change to manual strategy
        manager.set_strategy(ResetStrategy::Manual);
        assert_eq!(manager.strategy(), ResetStrategy::Manual);
    }
    
    #[test]
    fn memory_stats_tracking() {
        let pool = MemoryPool::new();
        let stats = pool.stats();
        
        assert_eq!(stats.total_allocated, 0);
        assert_eq!(stats.memory_in_use, 0);
        assert_eq!(stats.peak_memory, 0);
    }
    
    #[test]
    fn memory_pressure_detection() {
        let config = MemoryPoolConfig {
            max_arena_size: 1024,
            pressure_threshold: 0.5, // 50%
            ..Default::default()
        };
        
        let pool = MemoryPool::with_config(config);
        
        // Initially no pressure
        assert!(!pool.is_under_memory_pressure());
        
        // Test acceptable memory usage
        assert!(pool.is_memory_usage_acceptable());
    }
}