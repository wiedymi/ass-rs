//! Main parser coordination and dispatch logic
//!
//! Contains the core `Parser` struct that orchestrates parsing of different
//! ASS script sections and handles error recovery.

mod helpers;
mod parse;
mod registry;
mod section;
mod state;

#[cfg(test)]
mod tests_basic;
#[cfg(test)]
mod tests_edge;
#[cfg(test)]
mod tests_errors;
#[cfg(test)]
mod tests_recovery;

pub(super) use state::Parser;

#[cfg(test)]
use crate::{parser::errors::IssueSeverity, ScriptVersion};
#[cfg(test)]
use alloc::{format, string::String};
