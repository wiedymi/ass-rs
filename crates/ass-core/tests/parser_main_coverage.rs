//! Comprehensive coverage tests for parser/main.rs
//!
//! This module contains tests specifically designed to achieve complete coverage
//! of the main parser functionality, focusing on error paths and edge cases.

#[path = "parser_main_coverage/content_handling.rs"]
mod content_handling;
#[path = "parser_main_coverage/edge_cases.rs"]
mod edge_cases;
#[path = "parser_main_coverage/size_and_bom.rs"]
mod size_and_bom;
