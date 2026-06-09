//! Integration tests for ass-editor components
//!
//! These tests are designed to work regardless of features enabled.
//! Feature-specific functionality is tested only when the feature is available.

#[path = "integration_test/basic.rs"]
mod basic;

#[path = "integration_test/advanced.rs"]
mod advanced;

#[path = "integration_test/events_info.rs"]
mod events_info;

#[path = "integration_test/media.rs"]
mod media;

#[path = "integration_test/workflow.rs"]
mod workflow;
