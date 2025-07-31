//! Benchmarks for editor commands performance
//!
//! Tests the performance of various command operations including:
//! - Style commands (create, edit, delete, clone)
//! - Event commands (split, merge, timing adjustments)
//! - Tag commands (insert, remove, replace)
//! - Batch command execution

use ass_editor::{
    commands::*,
    core::{EditorDocument, Position, Range, StyleBuilder},
};
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

/// Generate a test script with styles and events
fn generate_complex_script(styles: usize, events: usize) -> String {
    let mut script = String::from(
        r#"[Script Info]
Title: Command Benchmark Script
ScriptType: v4.00+
WrapStyle: 0
ScaledBorderAndShadow: yes
YCbCr Matrix: None
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
"#,
    );

    // Add additional styles
    for i in 1..styles {
        script.push_str(&format!(
            "Style: Style{i},Arial,24,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1\n"
        ));
    }

    script.push_str("\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");

    // Add events
    for i in 0..events {
        let style = if i % 3 == 0 { "Default" } else { &format!("Style{}", (i % styles).max(1)) };
        let start_time = format!("0:{:02}:{:02}.00", i / 60, i % 60);
        let end_time = format!("0:{:02}:{:02}.00", (i + 5) / 60, (i + 5) % 60);
        script.push_str(&format!(
            "Dialogue: 0,{start_time},{end_time},{style},,0,0,0,,Event {i} with {{\\pos(960,540)}}some {{\\b1}}bold{{\\b0}} text\n"
        ));
    }

    script
}

/// Benchmark style command operations
fn bench_style_commands(c: &mut Criterion) {
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
                let command = CloneStyleCommand::new("Default".to_string(), "ClonedStyle".to_string());
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

/// Benchmark event command operations
fn bench_event_commands(c: &mut Criterion) {
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
                let command = TimingAdjustCommand::new(vec![], 500, 500);  // 500ms offset for all events
                black_box(command.execute(&mut doc).unwrap())
            },
            criterion::BatchSize::SmallInput,
        );
    });
    
    group.finish();
}

/// Benchmark tag command operations
fn bench_tag_commands(c: &mut Criterion) {
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
                let command = ReplaceTagCommand::new(range, "\\pos(960,540)".to_string(), "\\pos(640,360)".to_string());
                black_box(command.execute(&mut doc).unwrap())
            },
            criterion::BatchSize::SmallInput,
        );
    });
    
    group.finish();
}

/// Benchmark batch command execution
fn bench_batch_commands(c: &mut Criterion) {
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
                                    batch = batch.add_command(Box::new(
                                        InsertTagCommand::new(Position::new(i * 100), "\\fade(255,0)".to_string())
                                    ));
                                }
                                1 => {
                                    batch = batch.add_command(Box::new(
                                        EditStyleCommand::new("Default".to_string())
                                            .set_size((22 + i) as u32)
                                    ));
                                }
                                2 => {
                                    batch = batch.add_command(Box::new(
                                        TimingAdjustCommand::new(vec![i], 100, 100)
                                    ));
                                }
                                _ => {
                                    let pos = Position::new(1000 + i * 10);
                                    batch = batch.add_command(Box::new(
                                        InsertTextCommand::new(pos, "X".to_string())
                                    ));
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

/// Benchmark command with undo
fn bench_command_with_undo(c: &mut Criterion) {
    let mut group = c.benchmark_group("command_with_undo");
    
    group.bench_function("execute_and_undo", |b| {
        b.iter_batched(
            || EditorDocument::from_content(&generate_complex_script(5, 50)).unwrap(),
            |mut doc| {
                // Execute command
                let style_builder = StyleBuilder::default()
                    .font("Arial")
                    .size(24);
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

criterion_group!(
    benches,
    bench_style_commands,
    bench_event_commands,
    bench_tag_commands,
    bench_batch_commands,
    bench_command_with_undo
);
criterion_main!(benches);