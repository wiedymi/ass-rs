//! Edge case and error handling tests for the utils module.
//!
//! This module contains comprehensive tests targeting previously untested code paths
//! in the utils module, focusing on span validation, color parsing, and UU decoding.

#[path = "utils_edge_cases/color.rs"]
mod color;
#[path = "utils_edge_cases/error_conditions.rs"]
mod error_conditions;
#[path = "utils_edge_cases/spans.rs"]
mod spans;
#[path = "utils_edge_cases/uu_data.rs"]
mod uu_data;
