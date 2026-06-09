//! Extension manager edge cases for ass-editor.
//!
//! Config get/set behaviour and queries for missing extensions.

use ass_editor::ExtensionManager;

#[test]
fn test_extension_manager_edge_cases() {
    let mut manager = ExtensionManager::new();

    // Double registration should fail or be idempotent
    let _ext_count_before = manager.list_extensions().len();

    // Getting state of non-existent extension
    assert_eq!(manager.get_extension_state("non-existent"), None);

    // Setting and getting config
    manager.set_config("key1".to_string(), "value1".to_string());
    assert_eq!(manager.get_config("key1").as_deref(), Some("value1"));

    // Overwriting config
    manager.set_config("key1".to_string(), "value2".to_string());
    assert_eq!(manager.get_config("key1").as_deref(), Some("value2"));

    // Empty key/value
    manager.set_config("".to_string(), "empty_key".to_string());
    assert_eq!(manager.get_config("").as_deref(), Some("empty_key"));

    manager.set_config("empty_value".to_string(), "".to_string());
    assert_eq!(manager.get_config("empty_value").as_deref(), Some(""));
}
