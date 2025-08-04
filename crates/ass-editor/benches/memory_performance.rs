//! Benchmarks for memory usage and large document handling
//!
//! Tests the performance and memory efficiency of:
//! - Large document operations
//! - Undo/redo stack management
//! - Arena allocator efficiency
//! - Memory cleanup and garbage collection

use ass_editor::{
    commands::*,
    core::{EditorDocument, Position, Range, UndoStackConfig},
};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

/// Generate a very large ASS script
fn generate_large_script(events: usize, styles: usize) -> String {
    let mut script = String::from(
        r#"[Script Info]
Title: Large Memory Test Script
ScriptType: v4.00+
WrapStyle: 0
ScaledBorderAndShadow: yes
YCbCr Matrix: TV.601
PlayResX: 1920
PlayResY: 1080
Video Zoom Percent: 1.000000
Video Position: 0

[Aegisub Project Garbage]
Last Style Storage: Default
Video File: test.mp4
Video AR Value: 1.777778
Video Zoom Percent: 0.500000
Scroll Position: 0
Active Line: 0
Video Position: 0

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
"#,
    );

    // Add many styles
    for i in 0..styles {
        let color1 = format!("&H{:02X}FFFFFF", (i * 10) % 256);
        let color2 = format!("&H{:02X}000000", (i * 20) % 256);
        script.push_str(&format!(
            "Style: Style{i},Arial,{},{color1},{color2},&H00000000,&H00000000,{},{},0,0,100,100,0,{},1,2,0,{},10,10,10,1\n",
            20 + (i % 10),
            i % 2,
            (i + 1) % 2,
            i % 360,
            (i % 9) + 1
        ));
    }

    script.push_str("\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");

    // Add many events with complex content
    let complex_texts = [
        "Simple dialogue line with no formatting",
        "{\\pos(960,540)\\fade(255,0,255,0,0,500,1000)}Positioned and fading text",
        "{\\k20}Ka{\\k30}ra{\\k25}o{\\k35}ke {\\k40}ti{\\k30}ming {\\k45}test",
        "{\\move(100,100,1820,980)\\c&HFF0000&\\3c&H0000FF&\\bord3}Moving colored text",
        "{\\be1\\blur5\\fscx120\\fscy120}Blurred and scaled text",
        "{\\clip(100,100,500,500)}Clipped text region example",
        "{\\org(960,540)\\frz45\\t(\\frz0)}Rotating text animation",
        "Multi-line\\Ndialogue\\Nwith\\Nline breaks",
        "{\\an5\\q2}Center aligned wrapped text that should span multiple lines when rendered",
        "{\\p1}m 0 0 l 100 0 100 100 0 100{\\p0} Drawing command",
    ];

    for i in 0..events {
        let layer = i % 3;
        let style = format!("Style{}", i % styles);
        let start_seconds = i * 2;
        let start_time = format!(
            "0:{:02}:{:02}.{:02}",
            start_seconds / 3600,
            (start_seconds % 3600) / 60,
            (start_seconds % 60)
        );
        let end_time = format!(
            "0:{:02}:{:02}.{:02}",
            (start_seconds + 5) / 3600,
            ((start_seconds + 5) % 3600) / 60,
            ((start_seconds + 5) % 60)
        );
        let text = &complex_texts[i % complex_texts.len()];
        let actor = if i % 5 == 0 {
            format!("Actor{}", i % 10)
        } else {
            String::new()
        };

        script.push_str(&format!(
            "Dialogue: {layer},{start_time},{end_time},{style},{actor},0,0,0,,{text} [Event #{i}]\n"
        ));
    }

    script
}

/// Benchmark large document creation and parsing
fn bench_large_document_ops(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_document_ops");

    for size in [1000, 5000, 10000].iter() {
        let script = generate_large_script(*size, 50);
        let script_size = script.len();

        group.throughput(Throughput::Bytes(script_size as u64));
        group.bench_with_input(BenchmarkId::new("parse", size), size, |b, _| {
            b.iter(|| black_box(EditorDocument::from_content(&script).unwrap()));
        });

        group.bench_with_input(BenchmarkId::new("clone", size), size, |b, _| {
            let doc = EditorDocument::from_content(&script).unwrap();
            b.iter(|| black_box(EditorDocument::from_content(&doc.text()).unwrap()));
        });
    }

    group.finish();
}

/// Benchmark undo/redo stack operations
fn bench_undo_redo_stack(c: &mut Criterion) {
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

/// Benchmark arena allocator efficiency
#[cfg(feature = "arena")]
fn bench_arena_operations(c: &mut Criterion) {
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

/// Benchmark batch operations on large documents
fn bench_batch_large_ops(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_large_ops");

    let script = generate_large_script(5000, 50);

    // Batch style changes
    group.bench_function("batch_style_changes", |b| {
        b.iter_batched(
            || EditorDocument::from_content(&script).unwrap(),
            |mut doc| {
                let batch = BatchCommand::new("Batch style update".to_string())
                    .add_command(Box::new(
                        EditStyleCommand::new("Style1".to_string())
                            .set_size(24)
                            .set_bold(true),
                    ))
                    .add_command(Box::new(
                        EditStyleCommand::new("Style5".to_string()).set_font("Helvetica"),
                    ))
                    .add_command(Box::new(
                        EditStyleCommand::new("Style10".to_string()).set_color("&H00FF00FF"),
                    ));

                black_box(batch.execute(&mut doc).unwrap())
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // Batch tag operations
    group.bench_function("batch_tag_ops", |b| {
        b.iter_batched(
            || EditorDocument::from_content(&script).unwrap(),
            |mut doc| {
                let batch = BatchCommand::new("Batch tag update".to_string())
                    .add_command(Box::new(ReplaceTagCommand::new(
                        Range::new(Position::new(0), Position::new(10000)),
                        "\\pos(960,540)".to_string(),
                        "\\pos(640,360)".to_string(),
                    )))
                    .add_command(Box::new(
                        RemoveTagCommand::new(Range::new(Position::new(0), Position::new(10000)))
                            .pattern("\\be1".to_string()),
                    ))
                    .add_command(Box::new(InsertTagCommand::new(
                        Position::new(1000),
                        "\\fade(255,0)".to_string(),
                    )));

                black_box(batch.execute(&mut doc).unwrap())
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

/// Benchmark validation on large documents
fn bench_large_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_validation");

    for size in [1000, 5000, 10000].iter() {
        let script = generate_large_script(*size, 50);

        group.bench_with_input(BenchmarkId::new("validate_basic", size), size, |b, _| {
            b.iter_batched(
                || EditorDocument::from_content(&script).unwrap(),
                |doc| {
                    doc.validate().unwrap();
                    black_box(())
                },
                criterion::BatchSize::SmallInput,
            );
        });

        group.bench_with_input(
            BenchmarkId::new("validate_comprehensive", size),
            size,
            |b, _| {
                b.iter_batched(
                    || EditorDocument::from_content(&script).unwrap(),
                    |mut doc| black_box(doc.validate_comprehensive().unwrap()),
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

#[cfg(feature = "arena")]
criterion_group!(
    benches,
    bench_large_document_ops,
    bench_undo_redo_stack,
    bench_arena_operations,
    bench_batch_large_ops,
    bench_large_validation
);

#[cfg(not(feature = "arena"))]
criterion_group!(
    benches,
    bench_large_document_ops,
    bench_undo_redo_stack,
    bench_batch_large_ops,
    bench_large_validation
);

criterion_main!(benches);
