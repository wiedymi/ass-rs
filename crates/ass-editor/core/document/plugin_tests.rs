//! Tests for the extension-registry integration on documents

use super::*;
use crate::core::position::Position;

#[test]
fn test_registry_integration() {
    let mut doc = EditorDocument::new();

    // Initially no registry
    assert!(doc.registry().is_none());

    // Initialize with registry
    doc.initialize_registry().unwrap();
    assert!(doc.registry().is_some());

    // Parse with extensions
    doc.insert(Position::new(0), "[Script Info]\nTitle: Test\n\n[Events]\nDialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,{\\b1}Bold{\\b0} text").unwrap();

    let section_count = doc
        .parse_with_extensions(|script| script.sections().len())
        .unwrap();
    assert_eq!(section_count, 2);
}

#[test]
fn test_custom_tag_handler() {
    use ass_core::plugin::{TagHandler, TagResult};

    struct CustomHandler;
    impl TagHandler for CustomHandler {
        fn name(&self) -> &'static str {
            "custom"
        }

        fn process(&self, _args: &str) -> TagResult {
            TagResult::Processed
        }

        fn validate(&self, _args: &str) -> bool {
            true
        }
    }

    let mut doc = EditorDocument::new();
    doc.initialize_registry().unwrap();

    // Register custom tag handler
    assert!(doc
        .register_tag_handler("test-extension".to_string(), Box::new(CustomHandler))
        .is_ok());
}

#[test]
fn test_custom_section_processor() {
    use ass_core::plugin::{SectionProcessor, SectionResult};

    struct CustomProcessor;
    impl SectionProcessor for CustomProcessor {
        fn name(&self) -> &'static str {
            "CustomSection"
        }

        fn process(&self, _header: &str, _lines: &[&str]) -> SectionResult {
            SectionResult::Processed
        }

        fn validate(&self, _header: &str, _lines: &[&str]) -> bool {
            true
        }
    }

    let mut doc = EditorDocument::new();
    doc.initialize_registry().unwrap();

    // Register custom section processor
    assert!(doc
        .register_section_processor("test-extension".to_string(), Box::new(CustomProcessor))
        .is_ok());
}
