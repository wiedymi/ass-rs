//! Benchmarks for undo/redo stack push and undo operations.

use crate::common::generate_large_script;
use ass_editor::core::{EditorDocument, Position, UndoStackConfig};
use criterion::{black_box, BenchmarkId, Criterion};

/// Benchmark undo/redo stack operations
pub fn bench_undo_redo_stack(c: &mut Criterion) {
    let mut group = c.benchmark_group("undo_redo_stack");

    // Test with different stack depths
    for max_entries in [50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("push_operations", max_entries),
            max_entries,
            |b, &max| {
                b.iter_batched(
                    || {
                        let mut doc =
                            EditorDocument::from_content(&generate_large_script(100, 10)).unwrap();
                        let config = UndoStackConfig {
                            max_entries: max,
                            max_memory: 100 * 1024 * 1024, // 100MB
                            ..Default::default()
                        };
                        doc.undo_manager_mut().set_config(config);
                        doc
                    },
                    |mut doc| {
                        // Perform many small edits
                        for i in 0..20 {
                            let pos = Position::new(1000 + i * 10);
                            doc.insert(pos, "X").unwrap();
                        }
                        black_box(())
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );

        group.bench_with_input(
            BenchmarkId::new("undo_operations", max_entries),
            max_entries,
            |b, &max| {
                b.iter_batched(
                    || {
                        let mut doc =
                            EditorDocument::from_content(&generate_large_script(100, 10)).unwrap();
                        let config = UndoStackConfig {
                            max_entries: max,
                            max_memory: 100 * 1024 * 1024,
                            ..Default::default()
                        };
                        doc.undo_manager_mut().set_config(config);

                        // Fill with operations
                        for i in 0..30 {
                            let pos = Position::new(1000 + i * 10);
                            doc.insert(pos, "X").unwrap();
                        }
                        doc
                    },
                    |mut doc| {
                        // Undo multiple operations
                        for _ in 0..10 {
                            doc.undo().unwrap();
                        }
                        black_box(())
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}
