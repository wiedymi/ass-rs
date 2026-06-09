//! Extension lifecycle and command tests for the auto-completion extension.

use crate::core::EditorDocument;
use crate::extensions::builtin::auto_complete::AutoCompleteExtension;
use crate::extensions::{EditorExtension, ExtensionManager, ExtensionState};
#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as HashMap;
#[cfg(not(feature = "std"))]
use alloc::string::ToString;
#[cfg(feature = "std")]
use std::collections::HashMap;

#[test]
fn test_extension_lifecycle() {
    let mut ext = AutoCompleteExtension::new();
    let mut manager = ExtensionManager::new();
    let mut doc = EditorDocument::new();
    let mut context = manager
        .create_context("test".to_string(), Some(&mut doc))
        .unwrap();

    // Initialize
    assert_eq!(ext.state(), ExtensionState::Uninitialized);
    ext.initialize(&mut *context).unwrap();
    assert_eq!(ext.state(), ExtensionState::Active);

    // Execute trigger command
    let mut args = HashMap::new();
    args.insert("position".to_string(), "0".to_string());
    let result = ext
        .execute_command("autocomplete.trigger", &args, &mut *context)
        .unwrap();
    assert!(result.success);

    // Shutdown
    ext.shutdown(&mut *context).unwrap();
    assert_eq!(ext.state(), ExtensionState::Shutdown);
}

#[test]
fn test_config_loading() {
    let mut ext = AutoCompleteExtension::new();
    let mut manager = ExtensionManager::new();

    // Set configuration
    manager.set_config(
        "autocomplete.complete_fields".to_string(),
        "false".to_string(),
    );
    manager.set_config("autocomplete.max_suggestions".to_string(), "10".to_string());

    let mut doc = EditorDocument::new();
    let mut context = manager
        .create_context("test".to_string(), Some(&mut doc))
        .unwrap();

    // Initialize should load config
    ext.initialize(&mut *context).unwrap();

    // Config should be loaded
    assert!(!ext.config.complete_fields);
    assert_eq!(ext.config.max_suggestions, 10);
}

#[test]
fn test_update_styles_command() {
    let mut ext = AutoCompleteExtension::new();
    let mut manager = ExtensionManager::new();
    let mut doc = EditorDocument::from_content(
        "[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\nStyle: MyStyle,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,0,0,2,10,10,10,1",
    )
    .unwrap();
    let mut context = manager
        .create_context("test".to_string(), Some(&mut doc))
        .unwrap();

    ext.initialize(&mut *context).unwrap();

    // Execute update styles command
    let result = ext
        .execute_command("autocomplete.update_styles", &HashMap::new(), &mut *context)
        .unwrap();
    assert!(result.success);
    assert!(result.message.unwrap().contains("1 style names"));
}

#[test]
fn test_unknown_command() {
    let mut ext = AutoCompleteExtension::new();
    let mut manager = ExtensionManager::new();
    let mut doc = EditorDocument::new();
    let mut context = manager
        .create_context("test".to_string(), Some(&mut doc))
        .unwrap();

    let result = ext
        .execute_command("unknown.command", &HashMap::new(), &mut *context)
        .unwrap();
    assert!(!result.success);
    assert!(result.message.unwrap().contains("Unknown command"));
}
