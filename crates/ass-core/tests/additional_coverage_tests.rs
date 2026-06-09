//! Additional coverage tests for ASS tokenizer edge cases.
//!
//! This module focuses on improving test coverage for previously untested
//! code paths in the tokenizer, particularly error conditions and edge cases
//! that are difficult to trigger through normal usage.

#[cfg(not(feature = "std"))]
extern crate alloc;

#[path = "additional_coverage_tests/scanner.rs"]
mod scanner;
#[path = "additional_coverage_tests/tokenizer_content.rs"]
mod tokenizer_content;
#[path = "additional_coverage_tests/tokenizer_context.rs"]
mod tokenizer_context;
#[path = "additional_coverage_tests/tokenizer_core.rs"]
mod tokenizer_core;
