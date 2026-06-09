//! Benchmarks for editor commands performance
//!
//! Tests the performance of various command operations including:
//! - Style commands (create, edit, delete, clone)
//! - Event commands (split, merge, timing adjustments)
//! - Tag commands (insert, remove, replace)
//! - Batch command execution

use criterion::{criterion_group, criterion_main};

#[path = "editor_commands/batch.rs"]
mod batch;
#[path = "editor_commands/common.rs"]
mod common;
#[path = "editor_commands/event.rs"]
mod event;
#[path = "editor_commands/style.rs"]
mod style;
#[path = "editor_commands/tag.rs"]
mod tag;
#[path = "editor_commands/undo.rs"]
mod undo;

criterion_group!(
    benches,
    style::bench_style_commands,
    event::bench_event_commands,
    tag::bench_tag_commands,
    batch::bench_batch_commands,
    undo::bench_command_with_undo
);
criterion_main!(benches);
