//! Comprehensive tokenizer coverage tests
//!
//! Tests targeting specific uncovered code paths in the tokenizer module
//! to improve test coverage and ensure robust tokenization.

#[path = "tokenizer_coverage_tests/context_transitions.rs"]
mod context_transitions;
#[path = "tokenizer_coverage_tests/delimiters_and_content.rs"]
mod delimiters_and_content;
#[path = "tokenizer_coverage_tests/tokenizer_api.rs"]
mod tokenizer_api;
