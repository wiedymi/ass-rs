//! Benchmarks for tag command operations.

use ass_editor::commands::{InsertTagCommand, RemoveTagCommand, ReplaceTagCommand};
use ass_editor::core::{EditorDocument, Position, Range};
use ass_editor::EditorCommand;
use criterion::{black_box, Criterion};

use crate::common::generate_complex_script;

/// Benchmark tag command operations
pub fn bench_tag_commands(c: &mut Criterion) {
    let mut group = c.benchmark_group("tag_commands");

    // Insert tag command
    group.bench_function("insert_tag", |b| {
        b.iter_batched(
            || EditorDocument::from_content(&generate_complex_script(5, 50)).unwrap(),
            |mut doc| {
                // Find position at event 25 (roughly)
                let pos = Position::new(2500); // Approximate position
                let command = InsertTagCommand::new(pos, "\\fs32".to_string());
                black_box(command.execute(&mut doc).unwrap())
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // Remove tag command
    group.bench_function("remove_tag", |b| {
        b.iter_batched(
            || EditorDocument::from_content(&generate_complex_script(5, 50)).unwrap(),
            |mut doc| {
                // Remove tags from a specific range
                let range = Range::new(Position::new(1000), Position::new(2000));
                let command = RemoveTagCommand::new(range).pattern("\\b1".to_string());
                black_box(command.execute(&mut doc).unwrap())
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // Replace tag command
    group.bench_function("replace_tag", |b| {
        b.iter_batched(
            || EditorDocument::from_content(&generate_complex_script(5, 50)).unwrap(),
            |mut doc| {
                // Replace tags in a specific range
                let range = Range::new(Position::new(0), Position::new(5000));
                let command = ReplaceTagCommand::new(
                    range,
                    "\\pos(960,540)".to_string(),
                    "\\pos(640,360)".to_string(),
                );
                black_box(command.execute(&mut doc).unwrap())
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}
