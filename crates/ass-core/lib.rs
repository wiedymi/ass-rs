//! # ASS-RS Core
//!
//! High-performance, memory-efficient ASS (Advanced `SubStation` Alpha) subtitle format parser,
//! analyzer, and manipulator. Surpasses libass in modularity, reusability, and efficiency
//! through zero-copy parsing, trait-based extensibility, and strict memory management.
//!
//! ## Features
//!
//! - **Zero-copy parsing**: Uses `&str` spans to avoid allocations
//! - **Incremental updates**: Partial re-parsing for <2ms edits
//! - **SIMD optimization**: Feature-gated performance improvements
//! - **Extensible plugins**: Runtime tag and section handlers
//! - **Strict compliance**: Full ASS v4+, SSA v4, and libass 0.17.4+ support
//! - **Thread-safe**: Immutable `Script` design with `Send + Sync`
//!
//! ## Quick Start
//!
//! ```rust
//! use ass_core::{Script, analysis::ScriptAnalysis};
//!
//! let script_text = r#"
//! [Script Info]
//! Title: Example
//! ScriptType: v4.00+
//!
//! [V4+ Styles]
//! Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
//! Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
//!
//! [Events]
//! Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
//! Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello World!
//! "#;
//!
//! let script = Script::parse(script_text)?;
//! let analysis = ScriptAnalysis::analyze(&script)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Performance Targets
//!
//! - Parse: <5ms for 1KB scripts
//! - Analysis: <2ms for typical content
//! - Memory: ~1.1x input size via zero-copy spans
//! - Incremental: <2ms for single-event updates

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(clippy::all)]
#![deny(unsafe_code)]
#![allow(clippy::negative_feature_names)]

// Always make alloc available, whether in std or no_std mode
extern crate alloc;

pub mod parser;
pub mod tokenizer;

#[cfg(feature = "analysis")]
#[cfg_attr(docsrs, doc(cfg(feature = "analysis")))]
pub mod analysis;

#[cfg(feature = "plugins")]
#[cfg_attr(docsrs, doc(cfg(feature = "plugins")))]
pub mod plugin;

pub mod utils;

mod version;

#[cfg(test)]
mod integration_tests;

pub use parser::{ParseError, Script, Section};
pub use tokenizer::{AssTokenizer, Token};

#[cfg(feature = "analysis")]
pub use analysis::ScriptAnalysis;

#[cfg(feature = "plugins")]
pub use plugin::ExtensionRegistry;

pub use utils::{CoreError, Spans};
pub use version::ScriptVersion;

/// Crate version for runtime compatibility checks
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Result type for core operations, using the crate's unified `CoreError`.
///
/// This type alias provides a convenient way to return results from core operations
/// without having to specify the error type explicitly in every function signature.
///
/// # Examples
///
/// ```rust
/// use ass_core::{Result, Script};
///
/// fn parse_script_safely(input: &str) -> Result<Script> {
///     Script::parse(input)
/// }
/// ```
pub type Result<T> = core::result::Result<T, CoreError>;
