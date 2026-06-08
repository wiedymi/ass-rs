//! Unit tests for extension metadata, state, command, and result types.

use super::*;

#[cfg(not(feature = "std"))]
use alloc::string::ToString;

#[test]
fn extension_info_creation() {
    let info = ExtensionInfo::new(
        "test-extension".to_string(),
        "1.0.0".to_string(),
        "Test Author".to_string(),
        "A test extension".to_string(),
    )
    .with_capability(ExtensionCapability::TextProcessing)
    .with_dependency("core-extension".to_string())
    .with_homepage("https://example.com".to_string())
    .with_license("MIT".to_string());

    assert_eq!(info.name, "test-extension");
    assert_eq!(info.version, "1.0.0");
    assert!(info.has_capability(&ExtensionCapability::TextProcessing));
    assert_eq!(info.dependencies.len(), 1);
    assert_eq!(info.homepage, Some("https://example.com".to_string()));
    assert_eq!(info.license, Some("MIT".to_string()));
}

#[test]
fn extension_capability_description() {
    let capability = ExtensionCapability::TextProcessing;
    assert_eq!(
        capability.description(),
        "Text processing and transformation"
    );
}

#[test]
fn extension_state_checks() {
    let state = ExtensionState::Active;
    assert!(state.is_active());
    assert!(state.is_usable());
    assert!(!state.is_error());

    let error_state = ExtensionState::Error;
    assert!(!error_state.is_active());
    assert!(!error_state.is_usable());
    assert!(error_state.is_error());
}

#[test]
fn extension_command_creation() {
    let command = ExtensionCommand::new(
        "test-command".to_string(),
        "Test Command".to_string(),
        "A test command".to_string(),
    )
    .with_shortcut("Ctrl+T".to_string())
    .with_category("Testing".to_string())
    .requires_document(false);

    assert_eq!(command.id, "test-command");
    assert_eq!(command.shortcut, Some("Ctrl+T".to_string()));
    assert_eq!(command.category, "Testing");
    assert!(!command.requires_document);
}

#[test]
fn extension_result_creation() {
    let success = ExtensionResult::success_with_message("Success!".to_string())
        .with_data("key".to_string(), "value".to_string());

    assert!(success.success);
    assert_eq!(success.message, Some("Success!".to_string()));
    assert_eq!(success.data.get("key"), Some(&"value".to_string()));

    let failure = ExtensionResult::failure("Failed!".to_string());
    assert!(!failure.success);
    assert_eq!(failure.message, Some("Failed!".to_string()));
}
