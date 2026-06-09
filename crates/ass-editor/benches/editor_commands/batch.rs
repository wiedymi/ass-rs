//! Benchmarks for batch command execution.

use ass_editor::commands::{
    BatchCommand, EditStyleCommand, InsertTagCommand, InsertTextCommand, TimingAdjustCommand,
};
use ass_editor::core::{EditorDocument, Position};
use ass_editor::EditorCommand;
use criterion::{black_box, BenchmarkId, Criterion};

use crate::common::generate_complex_script;

/// Benchmark batch command execution
pub fn bench_batch_commands(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_commands");

    for batch_size in [5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::new("mixed_commands", batch_size),
            batch_size,
            |b, &size| {
                b.iter_batched(
                    || EditorDocument::from_content(&generate_complex_script(10, 100)).unwrap(),
                    |mut doc| {
                        let mut batch = BatchCommand::new("Complex batch operation".to_string());

                        // Add various commands
                        for i in 0..size {
                            match i % 4 {
                                0 => {
                                    batch = batch.add_command(Box::new(InsertTagCommand::new(
                                        Position::new(i * 100),
                                        "\\fade(255,0)".to_string(),
                                    )));
                                }
                                1 => {
                                    batch = batch.add_command(Box::new(
                                        EditStyleCommand::new("Default".to_string())
                                            .set_size((22 + i) as u32),
                                    ));
                                }
                                2 => {
                                    batch = batch.add_command(Box::new(TimingAdjustCommand::new(
                                        vec![i],
                                        100,
                                        100,
                                    )));
                                }
                                _ => {
                                    let pos = Position::new(1000 + i * 10);
                                    batch = batch.add_command(Box::new(InsertTextCommand::new(
                                        pos,
                                        "X".to_string(),
                                    )));
                                }
                            }
                        }

                        black_box(batch.execute(&mut doc).unwrap())
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}
