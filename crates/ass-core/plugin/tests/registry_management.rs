//! Tests for registry removal, clearing, naming, and debug output.

use super::mocks::{MockSectionProcessor, MockTagHandler};
use crate::plugin::ExtensionRegistry;
use alloc::boxed::Box;
#[cfg(not(feature = "std"))]
use alloc::format;

#[test]
fn remove_tag_handler() {
    let mut registry = ExtensionRegistry::new();
    let handler = Box::new(MockTagHandler::new("test"));
    registry.register_tag_handler(handler).unwrap();

    assert!(registry.has_tag_handler("test"));
    let removed = registry.remove_tag_handler("test");
    assert!(removed.is_some());
    assert!(!registry.has_tag_handler("test"));

    let not_found = registry.remove_tag_handler("nonexistent");
    assert!(not_found.is_none());
}

#[test]
fn remove_section_processor() {
    let mut registry = ExtensionRegistry::new();
    let processor = Box::new(MockSectionProcessor::new("test"));
    registry.register_section_processor(processor).unwrap();

    assert!(registry.has_section_processor("test"));
    let removed = registry.remove_section_processor("test");
    assert!(removed.is_some());
    assert!(!registry.has_section_processor("test"));

    let not_found = registry.remove_section_processor("nonexistent");
    assert!(not_found.is_none());
}

#[test]
fn clear_registry() {
    let mut registry = ExtensionRegistry::new();
    let handler = Box::new(MockTagHandler::new("tag"));
    let processor = Box::new(MockSectionProcessor::new("section"));

    registry.register_tag_handler(handler).unwrap();
    registry.register_section_processor(processor).unwrap();
    assert_eq!(registry.extension_count(), 2);

    registry.clear();
    assert_eq!(registry.extension_count(), 0);
    assert!(!registry.has_tag_handler("tag"));
    assert!(!registry.has_section_processor("section"));
}

#[test]
fn tag_handler_names() {
    let mut registry = ExtensionRegistry::new();
    let handler1 = Box::new(MockTagHandler::new("alpha"));
    let handler2 = Box::new(MockTagHandler::new("beta"));

    registry.register_tag_handler(handler1).unwrap();
    registry.register_tag_handler(handler2).unwrap();

    let names = registry.tag_handler_names();
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"alpha"));
    assert!(names.contains(&"beta"));
}

#[test]
fn section_processor_names() {
    let mut registry = ExtensionRegistry::new();
    let processor1 = Box::new(MockSectionProcessor::new("gamma"));
    let processor2 = Box::new(MockSectionProcessor::new("delta"));

    registry.register_section_processor(processor1).unwrap();
    registry.register_section_processor(processor2).unwrap();

    let names = registry.section_processor_names();
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"gamma"));
    assert!(names.contains(&"delta"));
}

#[test]
fn debug_formatting() {
    let mut registry = ExtensionRegistry::new();
    let handler = Box::new(MockTagHandler::new("test_tag"));
    let processor = Box::new(MockSectionProcessor::new("test_section"));

    registry.register_tag_handler(handler).unwrap();
    registry.register_section_processor(processor).unwrap();

    let debug_output = format!("{registry:?}");
    assert!(debug_output.contains("ExtensionRegistry"));
    assert!(debug_output.contains("test_tag"));
    assert!(debug_output.contains("test_section"));
}
