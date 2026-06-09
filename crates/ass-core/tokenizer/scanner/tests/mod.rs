//! Scanner unit tests grouped into cohesive submodules.
//!
//! These tests cover `CharNavigator` position tracking and the
//! `TokenScanner` scanning routines: section headers, style overrides,
//! comments, text, field values, and hex detection.

mod char_navigator_basic;
mod char_navigator_whitespace;
mod scanner_basics;
mod scanner_field_value;
mod scanner_hex;
mod scanner_override_comment;
mod scanner_text;
mod scanner_text_typed;
