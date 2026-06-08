//! Benchmark utilities for generating synthetic ASS scripts
//!
//! This module provides generators for creating test ASS scripts with varying
//! complexity levels, used primarily for benchmarking parser performance.
//! All generators produce valid ASS format strings that can be parsed by
//! the core parser.

mod dialogue;
mod generator;
mod script_sections;
mod scripts;

#[cfg(test)]
mod generator_tests;
#[cfg(test)]
mod scripts_tests;

pub use generator::{ComplexityLevel, ScriptGenerator};
pub use scripts::{create_test_event, generate_overlapping_script, generate_script_with_issues};
