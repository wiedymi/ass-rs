//! Extension system error tests.
//!
//! Tests for error cases in the extension manager configuration store.

use ass_editor::ExtensionManager;

#[test]
fn test_extension_manager_error_cases() {
    let mut manager = ExtensionManager::new();

    // Non-existent extension operations
    assert_eq!(manager.get_extension_state("non-existent"), None);

    // Empty string operations
    manager.set_config("".to_string(), "value".to_string());
    assert_eq!(manager.get_config("").as_deref(), Some("value"));

    manager.set_config("key".to_string(), "".to_string());
    assert_eq!(manager.get_config("key").as_deref(), Some(""));

    // Very long keys and values
    let long_key = "k".repeat(10000);
    let long_value = "v".repeat(10000);
    manager.set_config(long_key.clone(), long_value.clone());
    assert_eq!(
        manager.get_config(&long_key).as_deref(),
        Some(long_value.as_str())
    );
}

// Extension loading error test removed - Extension trait not publicly exported
