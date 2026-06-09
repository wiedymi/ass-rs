//! Extended tests for the auto-completion extension.
//!
//! The suite is split into focused submodules so each file stays small:
//! - [`section_field`]: section and field completion tests
//! - [`tags`]: override-tag completion tests
//! - [`workflow`]: document-driven completion workflow tests
//! - [`lifecycle`]: extension lifecycle and command tests

#[path = "auto_complete_tests/lifecycle.rs"]
mod lifecycle;
#[path = "auto_complete_tests/section_field.rs"]
mod section_field;
#[path = "auto_complete_tests/tags.rs"]
mod tags;
#[path = "auto_complete_tests/workflow.rs"]
mod workflow;
