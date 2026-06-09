//! Edge case and error handling tests for the events parser.
//!
//! This module contains comprehensive tests targeting previously untested code paths
//! in the events parser, focusing on different event types, format handling, and error recovery.
//!
//! The tests are organized into focused submodules under `events_parser_edge_cases/`.

#[cfg(test)]
#[path = "events_parser_edge_cases/boundaries.rs"]
mod boundaries;
#[cfg(test)]
#[path = "events_parser_edge_cases/error_recovery.rs"]
mod error_recovery;
#[cfg(test)]
#[path = "events_parser_edge_cases/event_types.rs"]
mod event_types;
#[cfg(test)]
#[path = "events_parser_edge_cases/formatting.rs"]
mod formatting;
