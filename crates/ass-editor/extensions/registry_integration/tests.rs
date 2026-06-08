//! Tests for the registry integration glue between ass-editor extensions and
//! ass-core's plugin system.

use super::*;
#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, string::ToString};

#[test]
fn test_registry_integration_creation() {
    let integration = RegistryIntegration::new();
    assert!(integration.tag_providers.is_empty());
    assert!(integration.section_providers.is_empty());
}

#[test]
fn test_register_builtin_handlers() {
    let mut integration = RegistryIntegration::new();

    // Should successfully register all built-in handlers
    assert!(integration.register_builtin_handlers().is_ok());
}

#[test]
fn test_register_builtin_sections() {
    let mut integration = RegistryIntegration::new();

    // Should successfully register built-in section processors
    assert!(integration.register_builtin_sections().is_ok());
}

#[test]
fn test_custom_tag_handler_registration() {
    use ass_core::plugin::{TagHandler, TagResult};

    struct TestTagHandler;
    impl TagHandler for TestTagHandler {
        fn name(&self) -> &'static str {
            "test"
        }

        fn process(&self, _args: &str) -> TagResult {
            TagResult::Processed
        }

        fn validate(&self, _args: &str) -> bool {
            true
        }
    }

    let mut integration = RegistryIntegration::new();
    let handler = Box::new(TestTagHandler);

    assert!(integration
        .register_custom_tag_handler("test-extension".to_string(), handler)
        .is_ok());
}

#[test]
fn test_custom_section_processor_registration() {
    use ass_core::plugin::{SectionProcessor, SectionResult};

    struct TestSectionProcessor;
    impl SectionProcessor for TestSectionProcessor {
        fn name(&self) -> &'static str {
            "TestSection"
        }

        fn process(&self, _header: &str, _lines: &[&str]) -> SectionResult {
            SectionResult::Processed
        }

        fn validate(&self, _header: &str, _lines: &[&str]) -> bool {
            true
        }
    }

    let mut integration = RegistryIntegration::new();
    let processor = Box::new(TestSectionProcessor);

    assert!(integration
        .register_custom_section_processor("test-extension".to_string(), processor)
        .is_ok());
}

#[test]
fn test_registry_access() {
    let integration = RegistryIntegration::new();

    // Should be able to access the registry
    let _registry = integration.registry();

    // Mutable access
    let mut integration = RegistryIntegration::new();
    let _registry_mut = integration.registry_mut();
}

#[test]
fn test_full_integration() {
    let mut integration = RegistryIntegration::new();

    // Register all built-ins
    assert!(integration.register_builtin_handlers().is_ok());
    assert!(integration.register_builtin_sections().is_ok());

    // The registry should now have many handlers registered
    // We can't easily test the exact count, but we can verify it worked
    let registry = integration.registry();

    // Use the registry to parse some ASS content with tags
    let test_content = "[Script Info]\nTitle: Test\n\n[Events]\nDialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,{\\b1}Bold{\\b0} text";

    // Parse with the registry
    let result = ass_core::parser::Script::builder()
        .with_registry(registry)
        .parse(test_content);

    assert!(result.is_ok());
}
