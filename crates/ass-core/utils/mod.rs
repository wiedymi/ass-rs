//! Utility functions and shared types for ASS-RS core
//!
//! Contains common functionality used across parser, tokenizer, and analysis modules.
//! Focuses on zero-allocation helpers, color processing, and UTF-8 handling.
//!
//! # Performance
//!
//! - Zero-copy span utilities for AST references
//! - SIMD-optimized color conversions when available
//! - Minimal allocation math helpers (bezier evaluation)
//!
//! # Example
//!
//! ```rust
//! use ass_core::utils::{Spans, parse_bgr_color};
//!
//! let color_str = "&H00FF00FF&";
//! let rgba = parse_bgr_color(color_str)?;
//! assert_eq!(rgba, [255, 0, 255, 0]); // BGR -> RGBA
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod benchmark_generators;
pub mod errors;
pub mod hashers;
pub mod utf8;

mod color;
mod fields;
mod math;
mod spans;
mod time;
mod uu;

#[cfg(test)]
mod color_tests;
#[cfg(test)]
mod fields_tests;
#[cfg(test)]
mod math_tests;
#[cfg(test)]
mod spans_tests;
#[cfg(test)]
mod time_tests;
#[cfg(test)]
mod uu_tests;

pub use benchmark_generators::{
    create_test_event, generate_overlapping_script, generate_script_with_issues, ComplexityLevel,
    ScriptGenerator,
};
pub use errors::CoreError;
pub use hashers::{create_hash_map, create_hash_map_with_capacity, create_hasher, hash_value};
pub use utf8::{detect_encoding, normalize_line_endings, recover_utf8, strip_bom, validate_utf8};

pub use color::parse_bgr_color;
pub use fields::{normalize_field_value, parse_numeric, validate_ass_name};
pub use math::eval_cubic_bezier;
pub use spans::Spans;
pub use time::{format_ass_time, parse_ass_time};
pub use uu::decode_uu_data;
