//! Benchmarks for event command operations.

use ass_editor::commands::{MergeEventsCommand, SplitEventCommand, TimingAdjustCommand};
use ass_editor::core::EditorDocument;
use ass_editor::EditorCommand;
use criterion::{black_box, Criterion};

use crate::common::generate_complex_script;

/// Benchmark event command operations
pub fn bench_event_commands(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_commands");

    // Split event command
    group.bench_function("split_event", |b| {
        b.iter_batched(
            || EditorDocument::from_content(&generate_complex_script(5, 50)).unwrap(),
            |mut doc| {
                let command = SplitEventCommand::new(5, "0:00:02.50".to_string());
                black_box(command.execute(&mut doc).unwrap())
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // Merge events command
    group.bench_function("merge_events", |b| {
        b.iter_batched(
            || EditorDocument::from_content(&generate_complex_script(5, 50)).unwrap(),
            |mut doc| {
                let command = MergeEventsCommand::new(10, 11);
                black_box(command.execute(&mut doc).unwrap())
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // Timing adjust command
    group.bench_function("timing_adjust", |b| {
        b.iter_batched(
            || EditorDocument::from_content(&generate_complex_script(5, 100)).unwrap(),
            |mut doc| {
                let command = TimingAdjustCommand::new(vec![], 500, 500); // 500ms offset for all events
                black_box(command.execute(&mut doc).unwrap())
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}
