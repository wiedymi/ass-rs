//! Extended tests for the syntax highlighting extension.
//!
//! This file is the test module for the syntax highlighting extension; it is
//! split into focused submodules under `syntax_highlight_tests/`.

#[cfg(not(feature = "std"))]
extern crate alloc;

#[path = "syntax_highlight_tests/formatting.rs"]
mod formatting;
#[path = "syntax_highlight_tests/lifecycle.rs"]
mod lifecycle;
#[path = "syntax_highlight_tests/override_tags.rs"]
mod override_tags;
#[path = "syntax_highlight_tests/tokenization.rs"]
mod tokenization;
