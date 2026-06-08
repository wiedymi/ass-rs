//! Tests for the format facade types and registry.

use super::*;
#[cfg(not(feature = "std"))]
use alloc::string::ToString;
#[cfg(not(feature = "std"))]
use alloc::{format, string::String, vec};

#[test]
fn test_format_info_creation() {
    let info = FormatInfo {
        name: "Test Format".to_string(),
        extensions: vec!["test".to_string(), "tst".to_string()],
        mime_type: "text/test".to_string(),
        description: "A test format".to_string(),
        supports_styling: true,
        supports_positioning: false,
    };

    assert_eq!(info.name, "Test Format");
    assert_eq!(info.extensions.len(), 2);
    assert!(info.supports_styling);
    assert!(!info.supports_positioning);
}

#[test]
fn test_format_options_default() {
    let options = FormatOptions::default();
    assert_eq!(options.encoding, "UTF-8");
    assert!(options.preserve_formatting);
    assert!(options.custom_options.is_empty());
}

#[test]
fn test_format_result_creation() {
    let result = FormatResult::success(42)
        .with_warnings(vec!["Warning 1".to_string()])
        .with_metadata("key".to_string(), "value".to_string());

    assert!(result.success);
    assert_eq!(result.lines_processed, 42);
    assert_eq!(result.warnings.len(), 1);
    assert_eq!(result.metadata.get("key"), Some(&"value".to_string()));
}

#[test]
fn test_format_registry_creation() {
    let registry = FormatRegistry::new();
    assert!(registry.formats.is_empty());
    assert!(registry.importers.is_empty());
    assert!(registry.exporters.is_empty());
}

#[test]
fn test_format_registry_extensions() {
    let registry = FormatRegistry::new();
    let import_exts = registry.supported_import_extensions();
    let export_exts = registry.supported_export_extensions();

    assert!(import_exts.is_empty());
    assert!(export_exts.is_empty());
}
