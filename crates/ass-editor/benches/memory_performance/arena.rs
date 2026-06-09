//! Benchmarks for arena allocator efficiency and repeated-edit memory reuse.

use crate::common::generate_large_script;
use ass_editor::core::{EditorDocument, Position, Range};
use criterion::{black_box, Criterion};

/// Benchmark arena allocator efficiency
#[cfg(feature = "arena")]
pub fn bench_arena_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("arena_operations");

    // Test arena reset after many operations
    group.bench_function("arena_reset_efficiency", |b| {
        b.iter_batched(
            || {
                let mut doc =
                    EditorDocument::from_content(&generate_large_script(500, 20)).unwrap();

                // Perform many operations to fill arena
                for i in 0..100 {
                    let pos = Position::new(1000 + i * 5);
                    doc.insert(pos, "TEST").unwrap();
                }

                // Undo half
                for _ in 0..50 {
                    doc.undo().unwrap();
                }

                doc
            },
            |mut doc| {
                // Perform some operation that might trigger cleanup
                doc.undo().ok();
                black_box(())
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // Test memory efficiency with repeated operations
    group.bench_function("repeated_ops_memory", |b| {
        b.iter_batched(
            || EditorDocument::from_content(&generate_large_script(200, 10)).unwrap(),
            |mut doc| {
                // Repeatedly modify same location
                let pos = Position::new(1000);
                for i in 0..20 {
                    let range = Range::new(pos, Position::new(pos.offset + 4));
                    doc.replace(range, &format!("NEW{i}")).unwrap();
                }
                black_box(())
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}
