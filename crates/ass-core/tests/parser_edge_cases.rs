//! Edge case and error handling tests for the ASS parser.
//!
//! This module contains comprehensive tests targeting previously untested code paths
//! in the parser, focusing on error recovery, edge cases, and security limits.

#[cfg(not(feature = "std"))]
extern crate alloc;

#[path = "parser_edge_cases/embedded_sections.rs"]
mod embedded_sections;
#[path = "parser_edge_cases/limits.rs"]
mod limits;
#[path = "parser_edge_cases/section_headers.rs"]
mod section_headers;
#[path = "parser_edge_cases/unknown_sections.rs"]
mod unknown_sections;
