//! Tests for plugin result types and error formatting.

use crate::plugin::{PluginError, SectionResult, TagResult};
#[cfg(not(feature = "std"))]
use alloc::{format, string::ToString};

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

#[cfg(feature = "std")]
#[test]
fn plugin_error_std_error() {
    use std::error::Error;
    let error = PluginError::ProcessingFailed("test".to_string());
    assert!(error.source().is_none());
}
