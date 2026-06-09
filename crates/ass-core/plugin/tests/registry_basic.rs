//! Tests for registry construction and handler/processor registration.

use super::mocks::{MockSectionProcessor, MockTagHandler};
use crate::plugin::{ExtensionRegistry, PluginError};
use alloc::boxed::Box;

#[test]
fn extension_registry_new() {
    let registry = ExtensionRegistry::new();
    assert_eq!(registry.extension_count(), 0);
    assert!(registry.tag_handler_names().is_empty());
    assert!(registry.section_processor_names().is_empty());
}

#[test]
fn extension_registry_default() {
    let registry = ExtensionRegistry::default();
    assert_eq!(registry.extension_count(), 0);
}

#[test]
fn register_tag_handler_success() {
    let mut registry = ExtensionRegistry::new();
    let handler = Box::new(MockTagHandler::new("test"));

    let result = registry.register_tag_handler(handler);
    assert!(result.is_ok());
    assert_eq!(registry.extension_count(), 1);
    assert!(registry.has_tag_handler("test"));
}

#[test]
fn register_tag_handler_duplicate() {
    let mut registry = ExtensionRegistry::new();
    let handler1 = Box::new(MockTagHandler::new("test"));
    let handler2 = Box::new(MockTagHandler::new("test"));

    registry.register_tag_handler(handler1).unwrap();
    let result = registry.register_tag_handler(handler2);

    assert!(result.is_err());
    match result.unwrap_err() {
        PluginError::DuplicateHandler(name) => assert_eq!(name, "test"),
        _ => panic!("Expected DuplicateHandler error"),
    }
}

#[test]
fn register_section_processor_success() {
    let mut registry = ExtensionRegistry::new();
    let processor = Box::new(MockSectionProcessor::new("test"));

    let result = registry.register_section_processor(processor);
    assert!(result.is_ok());
    assert_eq!(registry.extension_count(), 1);
    assert!(registry.has_section_processor("test"));
}

#[test]
fn register_section_processor_duplicate() {
    let mut registry = ExtensionRegistry::new();
    let processor1 = Box::new(MockSectionProcessor::new("test"));
    let processor2 = Box::new(MockSectionProcessor::new("test"));

    registry.register_section_processor(processor1).unwrap();
    let result = registry.register_section_processor(processor2);

    assert!(result.is_err());
    match result.unwrap_err() {
        PluginError::DuplicateHandler(name) => assert_eq!(name, "test"),
        _ => panic!("Expected DuplicateHandler error"),
    }
}
