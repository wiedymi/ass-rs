//! Targeted tests for specific uncovered tokenizer code paths
//!
//! These tests target the exact uncovered lines identified in coverage analysis
//! to ensure all code paths in the tokenizer are exercised.

#[path = "tokenizer_edge_paths/delimiters.rs"]
mod delimiters;
#[path = "tokenizer_edge_paths/edge_cases.rs"]
mod edge_cases;
#[path = "tokenizer_edge_paths/newlines_comments.rs"]
mod newlines_comments;
#[path = "tokenizer_edge_paths/spans.rs"]
mod spans;
