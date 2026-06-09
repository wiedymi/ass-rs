//! Edge case and error handling tests for the styles parser.
//!
//! This module contains comprehensive tests targeting previously untested code paths
//! in the styles parser, focusing on format handling, field validation, and error recovery.

#[path = "styles_parser_edge_cases/error_recovery.rs"]
mod error_recovery;
#[path = "styles_parser_edge_cases/field_values.rs"]
mod field_values;
#[path = "styles_parser_edge_cases/format_line_handling.rs"]
mod format_line_handling;
