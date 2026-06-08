//! State management for streaming ASS parser
//!
//! Provides state machine components for incremental parsing with
//! proper section tracking and context management.

mod context;
mod parser_state;
mod section_kind;

#[cfg(test)]
mod context_tests;
#[cfg(test)]
mod parser_state_tests;
#[cfg(test)]
mod section_kind_tests;

pub use context::StreamingContext;
pub use parser_state::ParserState;
pub use section_kind::SectionKind;
