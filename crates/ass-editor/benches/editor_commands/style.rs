//! Benchmarks for style command operations.

use ass_editor::commands::{
    ApplyStyleCommand, CloneStyleCommand, CreateStyleCommand, EditStyleCommand,
};
use ass_editor::core::{EditorDocument, StyleBuilder};
use ass_editor::EditorCommand;
use criterion::{black_box, Criterion};

use crate::common::generate_complex_script;

/// Benchmark style command operations
pub fn bench_style_commands(c: &mut Criterion) {
    let mut group = c.benchmark_group("style_commands");

    // Create style command
    group.bench_function("create_style", |b| {
        b.iter_batched(
            || EditorDocument::from_content(&generate_complex_script(5, 20)).unwrap(),
            |mut doc| {
                let style_builder = StyleBuilder::default()
                    .font("Impact")
                    .size(32)
                    .color("&H00FF00FF")
                    .secondary_color("&H00000000");
                let command = CreateStyleCommand::new("NewStyle".to_string(), style_builder);

                black_box(command.execute(&mut doc).unwrap())
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // Edit style command
    group.bench_function("edit_style", |b| {
        b.iter_batched(
            || EditorDocument::from_content(&generate_complex_script(5, 20)).unwrap(),
            |mut doc| {
                let command = EditStyleCommand::new("Style1".to_string())
                    .set_font("Helvetica")
                    .set_size(28)
                    .set_bold(true);

                black_box(command.execute(&mut doc).unwrap())
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // Clone style command
    group.bench_function("clone_style", |b| {
        b.iter_batched(
            || EditorDocument::from_content(&generate_complex_script(5, 20)).unwrap(),
            |mut doc| {
                let command =
                    CloneStyleCommand::new("Default".to_string(), "ClonedStyle".to_string());
                black_box(command.execute(&mut doc).unwrap())
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // Apply style to events
    group.bench_function("apply_style", |b| {
        b.iter_batched(
            || EditorDocument::from_content(&generate_complex_script(5, 50)).unwrap(),
            |mut doc| {
                let command = ApplyStyleCommand::new("Default".to_string(), "Style1".to_string());
                black_box(command.execute(&mut doc).unwrap())
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}
