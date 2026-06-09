//! Comprehensive tests for tokenizer functionality.
//!
//! Split into focused submodules so each file stays small and cohesive while
//! covering the full tokenizer surface: high-level [`AssTokenizer`] behaviour
//! and the lower-level scanner, state, and token component units.

mod advanced_state;
mod basic;
mod component_issues;
mod component_state;
mod component_tokens;
mod content_unicode;
mod context_flow;
mod context_scanning;
mod delimiter_context;
mod edge_cases;
mod fields_delimiters;
mod state_position;
mod unicode_long;
