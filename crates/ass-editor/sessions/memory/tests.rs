//! Unit tests for the memory pool, garbage collection, and arena strategies.

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
