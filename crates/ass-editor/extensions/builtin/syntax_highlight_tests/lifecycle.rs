//! Extension lifecycle, configuration, caching, and limit tests.
//!
//! Validates cache reuse, initialize/shutdown state transitions, config
//! loading from the manager, and the `max_tokens` limit.

use crate::core::EditorDocument;
use crate::extensions::builtin::syntax_highlight::SyntaxHighlightExtension;
use crate::extensions::{EditorExtension, ExtensionManager, ExtensionState};

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as HashMap;
#[cfg(not(feature = "std"))]
use alloc::string::ToString;
#[cfg(feature = "std")]
use std::collections::HashMap;

#[test]
fn test_cache_functionality() {
    let mut ext = SyntaxHighlightExtension::new();
    let doc = EditorDocument::from_content("[Script Info]\nTitle: Test").unwrap();

    // First tokenization
    let tokens1 = ext.tokenize_document(&doc).unwrap();

    // Second tokenization should use cache
    let tokens2 = ext.tokenize_document(&doc).unwrap();

    assert_eq!(tokens1.len(), tokens2.len());

    // Clear cache
    ext.clear_cache();

    // Should tokenize again after cache clear
    let tokens3 = ext.tokenize_document(&doc).unwrap();
    assert_eq!(tokens1.len(), tokens3.len());
}

#[test]
fn test_extension_lifecycle() {
    let mut ext = SyntaxHighlightExtension::new();
    let mut manager = ExtensionManager::new();
    let mut doc = EditorDocument::new();
    let mut context = manager
        .create_context("test".to_string(), Some(&mut doc))
        .unwrap();

    // Initialize
    assert_eq!(ext.state(), ExtensionState::Uninitialized);
    ext.initialize(&mut *context).unwrap();
    assert_eq!(ext.state(), ExtensionState::Active);

    // Execute command
    let result = ext
        .execute_command("syntax.highlight", &HashMap::new(), &mut *context)
        .unwrap();
    assert!(result.success);

    // Shutdown
    ext.shutdown(&mut *context).unwrap();
    assert_eq!(ext.state(), ExtensionState::Shutdown);
}

#[test]
fn test_config_loading() {
    let mut ext = SyntaxHighlightExtension::new();
    let mut manager = ExtensionManager::new();

    // Set configuration
    manager.set_config("syntax.highlight_tags".to_string(), "false".to_string());
    manager.set_config("syntax.max_tokens".to_string(), "5000".to_string());

    let mut doc = EditorDocument::new();
    let mut context = manager
        .create_context("test".to_string(), Some(&mut doc))
        .unwrap();

    // Initialize should load config
    ext.initialize(&mut *context).unwrap();

    // Config should be loaded
    assert!(!ext.config.highlight_tags);
    assert_eq!(ext.config.max_tokens, 5000);
}

#[test]
fn test_max_tokens_limit() {
    let mut ext = SyntaxHighlightExtension::new();
    ext.config.max_tokens = 3;

    let doc = EditorDocument::from_content(
        "[Script Info]\nTitle: Line 1\nAuthor: Line 2\nVersion: Line 3\nExtra: Line 4",
    )
    .unwrap();

    let tokens = ext.tokenize_document(&doc).unwrap();

    // Should stop at max_tokens
    assert_eq!(tokens.len(), 3);
}
