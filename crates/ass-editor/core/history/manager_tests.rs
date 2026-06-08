//! Tests for [`UndoManager`] coordination and [`HistoryStats`].

use super::*;
use crate::commands::CommandResult;
use crate::core::position::Position;
#[cfg(not(feature = "std"))]
use alloc::format;
#[cfg(not(feature = "std"))]
use alloc::string::ToString;

#[test]
fn undo_manager_operations() {
    let mut manager = UndoManager::new();

    // Simulate an insert operation
    let operation = Operation::Insert {
        position: Position::new(0),
        text: "Hello".to_string(),
    };
    let mut result = CommandResult::success();
    result.content_changed = true;

    // Record the operation
    manager.record_operation(operation, "Insert text".to_string(), &result);

    assert!(manager.can_undo());
    assert_eq!(manager.next_undo_description(), Some("Insert text"));
}

#[test]
fn history_stats() {
    let manager = UndoManager::new();
    let stats = manager.stats();

    assert_eq!(stats.undo_count, 0);
    assert_eq!(stats.redo_count, 0);
    assert_eq!(stats.memory_usage, 0);
}

#[test]
fn programmatic_undo_limit_configuration() {
    // Test custom undo limit configuration
    let custom_config = UndoStackConfig {
        max_entries: 3,
        max_memory: 1000,
        enable_compression: false,
        arena_reset_interval: 0,
    };

    let mut manager = UndoManager::with_config(custom_config);

    // Add more operations than the limit
    for i in 0..5 {
        let operation = Operation::Insert {
            position: Position::new(i * 10),
            text: format!("test{i}"),
        };
        let mut result = CommandResult::success();
        result.content_changed = true;
        manager.record_operation(operation, format!("Insert {i}"), &result);
    }

    // Should only keep the last 3 operations due to max_entries limit
    let stats = manager.stats();
    assert_eq!(stats.undo_count, 3);

    // Check that the correct operations are kept (most recent ones)
    assert_eq!(manager.next_undo_description(), Some("Insert 4"));

    // Test undo operations respect the limit
    manager.pop_undo_entry();
    assert_eq!(manager.next_undo_description(), Some("Insert 3"));

    manager.pop_undo_entry();
    assert_eq!(manager.next_undo_description(), Some("Insert 2"));

    manager.pop_undo_entry();
    assert_eq!(manager.next_undo_description(), None);
}

#[test]
fn undo_manager_config_update() {
    // Test that UndoManager can have its configuration updated
    let mut manager = UndoManager::new();

    // Add an operation with default config
    let operation = Operation::Insert {
        position: Position::new(0),
        text: "test".to_string(),
    };
    let mut result = CommandResult::success();
    result.content_changed = true;
    manager.record_operation(operation, "Initial".to_string(), &result);

    assert_eq!(manager.stats().undo_count, 1);

    // Update to a more restrictive config
    let restrictive_config = UndoStackConfig {
        max_entries: 0, // No undo history allowed
        max_memory: 0,
        enable_compression: false,
        arena_reset_interval: 0,
    };

    manager.set_config(restrictive_config);

    // The stack should be recreated, so previous operations should be gone
    assert_eq!(manager.stats().undo_count, 0);

    // New operations should not be recorded due to 0 limit
    let operation = Operation::Insert {
        position: Position::new(5),
        text: "test2".to_string(),
    };
    manager.record_operation(operation, "Should not record".to_string(), &result);
    assert_eq!(manager.stats().undo_count, 0);
}

#[test]
fn memory_limit_enforcement() {
    // Test that memory limits are enforced
    let memory_limited_config = UndoStackConfig {
        max_entries: 100, // High entry limit
        max_memory: 50,   // Very low memory limit (50 bytes)
        enable_compression: false,
        arena_reset_interval: 0,
    };

    let mut manager = UndoManager::with_config(memory_limited_config);

    // Add operations that exceed memory limit
    for i in 0..5 {
        let operation = Operation::Insert {
            position: Position::new(i * 10),
            text: format!(
                "This is a long text string for operation {i} that should consume memory"
            ),
        };
        let mut result = CommandResult::success();
        result.content_changed = true;
        manager.record_operation(operation, format!("Long operation {i}"), &result);
    }

    // Should have fewer operations due to memory constraint
    let stats = manager.stats();
    assert!(stats.undo_count < 5);
    assert!(stats.memory_usage <= 50 || stats.undo_count == 0);
}
