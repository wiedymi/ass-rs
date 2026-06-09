//! Tests targeting uncovered code paths in parser/main.rs
//!
//! These tests specifically target the uncovered lines identified in coverage analysis
//! to ensure all error handling and edge case paths are properly tested.

#[path = "parser_main_edge_paths/error_recovery.rs"]
mod error_recovery;
#[path = "parser_main_edge_paths/input_validation.rs"]
mod input_validation;
#[path = "parser_main_edge_paths/section_parsing.rs"]
mod section_parsing;
