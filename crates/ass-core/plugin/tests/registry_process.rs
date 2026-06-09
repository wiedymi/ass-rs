//! Tests for tag and section processing dispatch through the registry.

use super::mocks::{MockSectionProcessor, MockTagHandler};
use crate::plugin::{ExtensionRegistry, SectionResult, TagResult};
use alloc::boxed::Box;
#[cfg(not(feature = "std"))]
use alloc::vec;

#[test]
fn process_tag_found() {
    let mut registry = ExtensionRegistry::new();
    let handler = Box::new(MockTagHandler::new("test"));
    registry.register_tag_handler(handler).unwrap();

    let result = registry.process_tag("test", "args");
    assert!(result.is_some());
    assert_eq!(result.unwrap(), TagResult::Processed);
}

#[test]
fn process_tag_not_found() {
    let registry = ExtensionRegistry::new();
    let result = registry.process_tag("nonexistent", "args");
    assert!(result.is_none());
}

#[test]
fn process_tag_failed() {
    let mut registry = ExtensionRegistry::new();
    let handler = Box::new(MockTagHandler::new("test").with_failure(true));
    registry.register_tag_handler(handler).unwrap();

    let result = registry.process_tag("test", "args");
    assert!(result.is_some());
    match result.unwrap() {
        TagResult::Failed(msg) => assert_eq!(msg, "Mock failure"),
        _ => panic!("Expected Failed result"),
    }
}

#[test]
fn process_section_found() {
    let mut registry = ExtensionRegistry::new();
    let processor = Box::new(MockSectionProcessor::new("test"));
    registry.register_section_processor(processor).unwrap();

    let lines = vec!["line1", "line2"];
    let result = registry.process_section("test", "header", &lines);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), SectionResult::Processed);
}

#[test]
fn process_section_not_found() {
    let registry = ExtensionRegistry::new();
    let lines = vec!["line1"];
    let result = registry.process_section("nonexistent", "header", &lines);
    assert!(result.is_none());
}
