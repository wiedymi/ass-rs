//! Targeted tests for uncovered code paths in utils/mod.rs
//!
//! These tests specifically target the uncovered lines identified in coverage analysis
//! to improve test coverage for utility functions.

#[path = "utils_targeted_coverage/color.rs"]
mod color;
#[path = "utils_targeted_coverage/error.rs"]
mod error;
#[path = "utils_targeted_coverage/spans.rs"]
mod spans;
#[path = "utils_targeted_coverage/time.rs"]
mod time;
#[path = "utils_targeted_coverage/uu.rs"]
mod uu;
