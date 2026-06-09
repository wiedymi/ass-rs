//! Comprehensive ASS specification coverage integration tests
//!
//! Tests end-to-end parsing and analysis of complex ASS scripts that exercise
//! the full breadth of the ASS specification. Ensures compatibility with
//! libass extensions, Aegisub features, and `TCax` format variations.
//!
//! # Coverage Areas
//!
//! - All event types: Dialogue, Comment, Picture, Sound, Movie, Command
//! - Complete style override tag set including drawing commands
//! - Embedded media sections: [Fonts] and [Graphics] with UU-encoding
//! - Complex animation sequences with timing functions
//! - Unicode text handling and bidirectional content
//! - Performance validation for large scripts
//!
//! The test cases are organized into focused submodules under `spec_coverage/`,
//! with the shared comprehensive script fixture living in `common`.

#[path = "spec_coverage/common.rs"]
mod common;

#[path = "spec_coverage/comprehensive.rs"]
mod comprehensive;
#[path = "spec_coverage/error_recovery.rs"]
mod error_recovery;
#[path = "spec_coverage/parsing.rs"]
mod parsing;
#[path = "spec_coverage/performance.rs"]
mod performance;
#[path = "spec_coverage/stress.rs"]
mod stress;
#[path = "spec_coverage/style_overrides.rs"]
mod style_overrides;
#[path = "spec_coverage/text_analysis.rs"]
mod text_analysis;
