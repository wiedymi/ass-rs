//! ASS dialogue event analysis and processing
//!
//! Provides comprehensive analysis capabilities for ASS dialogue events including
//! timing validation, overlap detection, text analysis, and performance impact
//! assessment. Designed for zero-copy efficiency with lifetime-generic references.
//!
//! # Features
//!
//! - **Timing Analysis**: Overlap detection using O(n log n) sweep-line algorithm
//! - **Text Processing**: Override tag parsing and Unicode complexity detection
//! - **Performance Scoring**: Animation complexity and rendering impact assessment
//! - **Zero-Copy Design**: Minimal allocations via lifetime-generic spans
//! - **Standards Compliance**: Full ASS v4+ and libass 0.17.4+ compatibility
//!
//! # Performance Targets
//!
//! - Event analysis: <1ms per dialogue event
//! - Overlap detection: <1ms for 1000 events
//! - Text parsing: <0.5ms per event text
//! - Memory usage: ~1.1x input size via zero-copy spans
//!
//! # Quick Start
//!
//! ```rust
//! use ass_core::analysis::events::{DialogueInfo, find_overlapping_events};
//! use ass_core::parser::Event;
//!
//! let event = Event {
//!     start: "0:00:00.00",
//!     end: "0:00:05.00",
//!     text: "Hello {\\b1}world{\\b0}!",
//!     ..Default::default()
//! };
//!
//! let info = DialogueInfo::analyze(&event)?;
//! println!("Duration: {}ms", info.duration_ms());
//! println!("Animation score: {}/10", info.animation_score());
//! println!("Performance impact: {:?}", info.performance_impact());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Module Organization
//!
//! - [`dialogue_info`] - Individual event analysis and timing validation
//! - [`text_analysis`] - Text content parsing and Unicode complexity detection
//! - [`overlap`] - Efficient timing overlap detection algorithms
//! - [`utils`] - Collection operations like sorting and duration calculations

pub mod dialogue_info;
pub mod overlap;
pub mod scoring;
pub mod tags;
pub mod line_breaks;
pub mod text_analysis;
pub mod utils;

pub use dialogue_info::{DialogueInfo, TimingRelation};
pub use overlap::{count_overlapping_events, find_overlapping_event_refs, find_overlapping_events};
pub use scoring::{
    calculate_animation_score, calculate_complexity_score, get_performance_impact,
    PerformanceImpact,
};
pub use tags::{
    calculate_tag_complexity, parse_override_block, DiagnosticKind, OverrideTag, TagDiagnostic,
};
pub use line_breaks::{LineBreakType, LineBreakInfo, TextWithLineBreaks};
pub use text_analysis::TextAnalysis;
pub use utils::{
    calculate_average_duration, calculate_total_duration, count_overlapping_dialogue_events,
    find_events_in_range, find_overlapping_dialogue_events, sort_events_by_time,
};
