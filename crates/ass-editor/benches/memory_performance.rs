//! Benchmarks for memory usage and large document handling
//!
//! Tests the performance and memory efficiency of:
//! - Large document operations
//! - Undo/redo stack management
//! - Arena allocator efficiency
//! - Memory cleanup and garbage collection

use criterion::{criterion_group, criterion_main};

#[path = "memory_performance/common.rs"]
mod common;

#[path = "memory_performance/document_ops.rs"]
mod document_ops;

#[path = "memory_performance/undo_redo.rs"]
mod undo_redo;

#[cfg(feature = "arena")]
#[path = "memory_performance/arena.rs"]
mod arena;

#[path = "memory_performance/batch_ops.rs"]
mod batch_ops;

#[path = "memory_performance/validation.rs"]
mod validation;

#[cfg(feature = "arena")]
criterion_group!(
    benches,
    document_ops::bench_large_document_ops,
    undo_redo::bench_undo_redo_stack,
    arena::bench_arena_operations,
    batch_ops::bench_batch_large_ops,
    validation::bench_large_validation
);

#[cfg(not(feature = "arena"))]
criterion_group!(
    benches,
    document_ops::bench_large_document_ops,
    undo_redo::bench_undo_redo_stack,
    batch_ops::bench_batch_large_ops,
    validation::bench_large_validation
);

criterion_main!(benches);
