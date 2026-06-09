//! Integration tests for incremental parsing functionality

#![cfg(feature = "stream")]

#[path = "incremental_parsing/delta.rs"]
mod delta;
#[path = "incremental_parsing/incremental_preservation.rs"]
mod incremental_preservation;
#[path = "incremental_parsing/incremental_sections.rs"]
mod incremental_sections;
#[path = "incremental_parsing/partial.rs"]
mod partial;
#[path = "incremental_parsing/performance.rs"]
mod performance;
