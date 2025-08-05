//! Comprehensive tests for plugin system functionality

use super::*;
use alloc::boxed::Box;
#[cfg(not(feature = "std"))]
use alloc::{format, vec};

/// Mock tag handler for testing
struct MockTagHandler {
    name: &'static str,
    should_process: bool,
    should_fail: bool,
}

impl MockTagHandler {
    fn new(name: &'static str) -> Self {
        Self {
            name,
            should_process: true,
            should_fail: false,
        }
    }

    fn with_failure(mut self, should_fail: bool) -> Self {
        self.should_fail = should_fail;
        self
    }
}

impl TagHandler for MockTagHandler {
    fn name(&self) -> &'static str {
        self.name
    }

    fn process(&self, _args: &str) -> TagResult {
        if self.should_fail {
            TagResult::Failed("Mock failure".to_string())
        } else if self.should_process {
            TagResult::Processed
        } else {
            TagResult::Ignored
        }
    }

    fn validate(&self, args: &str) -> bool {
        !args.is_empty()
    }
}

/// Mock section processor for testing
struct MockSectionProcessor {
    name: &'static str,
    should_process: bool,
    should_fail: bool,
}

impl MockSectionProcessor {
    fn new(name: &'static str) -> Self {
        Self {
            name,
            should_process: true,
            should_fail: false,
        }
    }
}

impl SectionProcessor for MockSectionProcessor {
    fn name(&self) -> &'static str {
        self.name
    }

    fn process(&self, _header: &str, _lines: &[&str]) -> SectionResult {
        if self.should_fail {
            SectionResult::Failed("Mock failure".to_string())
        } else if self.should_process {
            SectionResult::Processed
        } else {
            SectionResult::Ignored
        }
    }

    fn validate(&self, header: &str, lines: &[&str]) -> bool {
        !header.is_empty() && !lines.is_empty()
    }
}

#[test]
fn tag_result_equality() {
    assert_eq!(TagResult::Processed, TagResult::Processed);
    assert_eq!(TagResult::Ignored, TagResult::Ignored);
    assert_eq!(
        TagResult::Failed("error".to_string()),
        TagResult::Failed("error".to_string())
    );
    assert_ne!(TagResult::Processed, TagResult::Ignored);
}

#[test]
fn section_result_equality() {
    assert_eq!(SectionResult::Processed, SectionResult::Processed);
    assert_eq!(SectionResult::Ignored, SectionResult::Ignored);
    assert_eq!(
        SectionResult::Failed("error".to_string()),
        SectionResult::Failed("error".to_string())
    );
    assert_ne!(SectionResult::Processed, SectionResult::Ignored);
}

#[test]
fn plugin_error_display() {
    let error = PluginError::DuplicateHandler("test".to_string());
    assert_eq!(format!("{error}"), "Handler 'test' already registered");

    let error = PluginError::HandlerNotFound("missing".to_string());
    assert_eq!(format!("{error}"), "Handler 'missing' not found");

    let error = PluginError::ProcessingFailed("failed".to_string());
    assert_eq!(format!("{error}"), "Plugin processing failed: failed");

    let error = PluginError::InvalidConfig("bad config".to_string());
    assert_eq!(
        format!("{error}"),
        "Invalid plugin configuration: bad config"
    );
}

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

#[test]
fn tag_handler_validation() {
    let handler = MockTagHandler::new("test");
    assert!(handler.validate("valid_args"));
    assert!(!handler.validate(""));
}

#[test]
fn section_processor_validation() {
    let processor = MockSectionProcessor::new("test");
    let lines = vec!["line1"];
    assert!(processor.validate("header", &lines));
    assert!(!processor.validate("", &lines));
    assert!(!processor.validate("header", &[]));
}

#[cfg(feature = "std")]
#[test]
fn plugin_error_std_error() {
    use std::error::Error;
    let error = PluginError::ProcessingFailed("test".to_string());
    assert!(error.source().is_none());
}
