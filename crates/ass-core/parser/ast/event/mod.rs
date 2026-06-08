//! Event AST node for ASS dialogue and commands
//!
//! Contains the Event struct and `EventType` enum representing events from the
//! [Events] section with zero-copy design and time parsing utilities.

mod event_struct;
mod event_type;
mod serialization;

#[cfg(test)]
mod event_tests;
#[cfg(test)]
mod event_type_tests;
#[cfg(test)]
mod serialization_tests;
#[cfg(test)]
mod timing_tests;

pub use event_struct::Event;
pub use event_type::EventType;

use super::Span;
