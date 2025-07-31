//! Benchmarks for incremental parsing performance
//!
//! Verifies that we meet the performance targets:
//! - <1ms for single edit operations
//! - <5ms for reparse after multiple edits

use ass_editor::core::{EditorDocument, Position, Range};
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

/// Generate a realistic ASS script for benchmarking
fn generate_test_script(events: usize) -> String {
    let mut script = String::from(
        r#"[Script Info]
Title: Benchmark Script
ScriptType: v4.00+
WrapStyle: 0
ScaledBorderAndShadow: yes
YCbCr Matrix: None
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
"#,
    );

    // Add events
    for i in 0..events {
        let start_time = format!("0:{:02}:{:02}.00", i / 60, i % 60);
        let end_time = format!("0:{:02}:{:02}.00", (i + 5) / 60, (i + 5) % 60);
        script.push_str(&format!(
            "Dialogue: 0,{start_time},{end_time},Default,,0,0,0,,Event {i} - Some dialogue text with {{\\pos(960,540)}}positioning\n"
        ));
    }

    script
}

/// Benchmark single character insertion
fn bench_single_edit(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_edit");
    
    for size in [10, 100, 500].iter() {
        let script = generate_test_script(*size);
        
        group.bench_with_input(
            BenchmarkId::new("incremental", size),
            size,
            |b, _| {
                b.iter_batched(
                    || {
                        EditorDocument::from_content(&script).unwrap()
                    },
                    |mut doc| {
                        // Edit in the middle of an event
                        let pos = Position::new(script.len() / 2);
                        let range = Range::new(pos, pos);
                        black_box(doc.edit_incremental(range, "X").unwrap())
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
        
        // Compare with regular edit
        group.bench_with_input(
            BenchmarkId::new("regular", size),
            size,
            |b, _| {
                b.iter_batched(
                    || EditorDocument::from_content(&script).unwrap(),
                    |mut doc| {
                        let pos = Position::new(script.len() / 2);
                        let range = Range::new(pos, pos);
                        doc.replace(range, "X").unwrap();
                        black_box(())
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }
    
    group.finish();
}

/// Benchmark word replacement
fn bench_word_replace(c: &mut Criterion) {
    let mut group = c.benchmark_group("word_replace");
    
    let sizes = [10, 100, 500];
    for size in sizes.iter() {
        let script = generate_test_script(*size);
        
        group.bench_with_input(
            BenchmarkId::new("incremental", size),
            size,
            |b, _| {
                b.iter_batched(
                    || EditorDocument::from_content(&script).unwrap(),
                    |mut doc| {
                        // Replace "Event" with "Scene" in middle of document
                        if let Some(pos) = doc.text().find("Event") {
                            let start = Position::new(pos);
                            let end = Position::new(pos + 5);
                            let range = Range::new(start, end);
                            black_box(doc.edit_incremental(range, "Scene").unwrap())
                        } else {
                            black_box(doc.edit_incremental(Range::new(Position::new(0), Position::new(0)), "").unwrap())
                        }
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }
    
    group.finish();
}

/// Benchmark full reparse after threshold
fn bench_reparse_threshold(c: &mut Criterion) {
    let mut group = c.benchmark_group("reparse_threshold");
    
    let script = generate_test_script(100);
    
    group.bench_function("after_many_edits", |b| {
        b.iter_batched(
            || {
                let mut doc = EditorDocument::from_content(&script).unwrap();
                
                // Make several small edits to accumulate changes
                for i in 0..20 {
                    let pos = Position::new((i * 100).min(script.len() - 1));
                    let range = Range::new(pos, pos);
                    let _ = doc.edit_incremental(range, "x");
                }
                
                doc
            },
            |mut doc| {
                // This edit should trigger full reparse due to accumulated changes
                let pos = Position::new(1000.min(script.len() - 1));
                let range = Range::new(pos, pos);
                black_box(doc.edit_incremental(range, "TRIGGER").unwrap())
            },
            criterion::BatchSize::SmallInput,
        );
    });
    
    group.finish();
}

/// Benchmark line insertion (common operation)
fn bench_line_insertion(c: &mut Criterion) {
    let mut group = c.benchmark_group("line_insertion");
    
    let script = generate_test_script(100);
    let new_event = "\nDialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,New inserted line";
    
    group.bench_function("incremental", |b| {
        b.iter_batched(
            || {
                EditorDocument::from_content(&script).unwrap()
            },
            |mut doc| {
                // Insert at end of events section
                let pos = Position::new(doc.text().len() - 1);
                let range = Range::new(pos, pos);
                black_box(doc.edit_incremental(range, new_event).unwrap())
            },
            criterion::BatchSize::SmallInput,
        );
    });
    
    group.finish();
}

/// Benchmark safe edit with fallback
fn bench_safe_edit(c: &mut Criterion) {
    let mut group = c.benchmark_group("safe_edit");
    
    let script = generate_test_script(50);
    
    group.bench_function("with_fallback", |b| {
        b.iter_batched(
            || EditorDocument::from_content(&script).unwrap(),
            |mut doc| {
                let pos = Position::new(script.len() / 2);
                let range = Range::new(pos, pos);
                doc.edit_safe(range, "SAFE").unwrap();
                black_box(())
            },
            criterion::BatchSize::SmallInput,
        );
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_single_edit,
    bench_word_replace,
    bench_reparse_threshold,
    bench_line_insertion,
    bench_safe_edit
);
criterion_main!(benches);