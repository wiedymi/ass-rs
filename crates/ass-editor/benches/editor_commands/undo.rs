//! Benchmarks for command execution with undo/redo.

use ass_editor::commands::{CreateStyleCommand, InsertTextCommand};
use ass_editor::core::{EditorDocument, Position, StyleBuilder};
use ass_editor::EditorCommand;
use criterion::{black_box, Criterion};

use crate::common::generate_complex_script;

/// Benchmark command with undo
pub fn bench_command_with_undo(c: &mut Criterion) {
    let mut group = c.benchmark_group("command_with_undo");

    group.bench_function("execute_and_undo", |b| {
        b.iter_batched(
            || EditorDocument::from_content(&generate_complex_script(5, 50)).unwrap(),
            |mut doc| {
                // Execute command
                let style_builder = StyleBuilder::default().font("Arial").size(24);
                let command = CreateStyleCommand::new("TestStyle".to_string(), style_builder);
                command.execute(&mut doc).unwrap();

                // Undo
                black_box(doc.undo().unwrap())
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.bench_function("execute_undo_redo", |b| {
        b.iter_batched(
            || EditorDocument::from_content(&generate_complex_script(5, 50)).unwrap(),
            |mut doc| {
                // Execute command
                let pos = Position::new(1000);
                let command = InsertTextCommand::new(pos, "Test".to_string());
                command.execute(&mut doc).unwrap();

                // Undo then redo
                doc.undo().unwrap();
                black_box(doc.redo().unwrap())
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}
