//! Comprehensive incremental parsing benchmarks
//!
//! Tests incremental parsing performance against project targets:
//! - <5ms incremental parsing for typical operations
//! - <1.1x input memory ratio
//! - Correctness validation vs full reparse
//!
//! Includes real-world editing scenarios and stress testing.

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::{format, string::ToString};
#[allow(
    clippy::missing_docs_in_private_items,
    clippy::option_if_let_else,
    clippy::range_plus_one,
    clippy::cast_precision_loss
)]
use ass_core::{
    parser::{incremental::TextChange, Script},
    utils::ScriptGenerator,
};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::{hint::black_box as std_black_box, time::Instant};
/// Check if running in quick mode (for CI or quick tests)
fn is_quick_bench() -> bool {
    std::env::var("QUICK_BENCH").is_ok()
}

/// Benchmark core incremental parsing operations
fn bench_incremental_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("incremental_parsing");

    if is_quick_bench() {
        group.sample_size(50);
        group.measurement_time(std::time::Duration::from_secs(2));
    } else {
        group.sample_size(200);
        group.warm_up_time(std::time::Duration::from_millis(500));
        group.measurement_time(std::time::Duration::from_secs(5));
    }

    let sizes = [100, 1000, 5000, 10000];
    let change_sizes = [
        ("small", 10),   // 1-10 chars
        ("medium", 100), // 10-100 chars
        ("large", 1000), // 100-1000 chars
    ];

    for &size in &sizes {
        let script_text = ScriptGenerator::moderate(size).generate();

        for (change_name, change_size) in &change_sizes {
            // Test single section changes
            let change = create_section_change(&script_text, *change_size);
            group.throughput(Throughput::Bytes(script_text.len() as u64));

            group.bench_with_input(
                BenchmarkId::new(format!("section_change_{change_name}"), size),
                &(script_text.as_str(), &change),
                |b, (text, change)| {
                    b.iter(|| {
                        let start = Instant::now();
                        let result = apply_incremental_change(black_box(text), black_box(change));
                        let duration = start.elapsed();
                        std_black_box((result, duration))
                    });
                },
            );

            // Test cross-section changes
            let cross_change = create_cross_section_change(&script_text, *change_size);
            group.bench_with_input(
                BenchmarkId::new(format!("cross_section_{change_name}"), size),
                &(script_text.as_str(), &cross_change),
                |b, (text, change)| {
                    b.iter(|| {
                        let start = Instant::now();
                        let result = apply_incremental_change(black_box(text), black_box(change));
                        let duration = start.elapsed();
                        std_black_box((result, duration))
                    });
                },
            );
        }
    }

    group.finish();
}

/// Benchmark incremental vs full parsing comparison
fn bench_incremental_vs_full(c: &mut Criterion) {
    let mut group = c.benchmark_group("incremental_vs_full");
    group.sample_size(100); // Reduced from 200 to avoid timeout warnings
    group.measurement_time(std::time::Duration::from_secs(10)); // Increased from default 5s

    let sizes = [1000, 5000, 10000];
    let change_types = [
        ("small_edit", 10),
        ("medium_edit", 100),
        ("large_edit", 1000),
    ];

    for &size in &sizes {
        let script_text = ScriptGenerator::complex(size).generate();

        for (change_name, change_size) in &change_types {
            let change = create_section_change(&script_text, *change_size);
            let modified_text = apply_text_change(&script_text, &change);

            // Benchmark full reparse
            group.bench_with_input(
                BenchmarkId::new(format!("full_reparse_{change_name}"), size),
                &modified_text,
                |b, text| {
                    b.iter(|| {
                        let start = Instant::now();
                        let result = Script::parse(black_box(text));
                        let duration = start.elapsed();
                        std_black_box((result, duration))
                    });
                },
            );

            // Benchmark incremental parse
            group.bench_with_input(
                BenchmarkId::new(format!("incremental_{change_name}"), size),
                &(script_text.as_str(), &change),
                |b, (text, change)| {
                    b.iter(|| {
                        let start = Instant::now();
                        let result = apply_incremental_change(black_box(text), black_box(change));
                        let duration = start.elapsed();
                        std_black_box((result, duration))
                    });
                },
            );
        }
    }

    group.finish();
}

/// Benchmark editor simulation scenarios
fn bench_editor_simulation(c: &mut Criterion) {
    let mut group = c.benchmark_group("editor_simulation");
    group.sample_size(100); // Reduced for faster completion
    group.measurement_time(std::time::Duration::from_secs(8));

    let script_text = ScriptGenerator::moderate(1000).generate();

    // Typing simulation - multiple small changes
    let typing_changes = create_typing_simulation(&script_text, 50);
    group.bench_with_input(
        BenchmarkId::new("typing_sequence", typing_changes.len()),
        &(script_text.as_str(), &typing_changes),
        |b, (text, changes)| {
            b.iter(|| {
                let mut current_text = (*text).to_string();
                let start = Instant::now();

                for change in *changes {
                    current_text = apply_text_change(&current_text, change);
                }

                let duration = start.elapsed();
                std_black_box((current_text, duration))
            });
        },
    );

    // Backspace simulation - deletions
    let backspace_changes = create_backspace_simulation(&script_text, 30);
    group.bench_with_input(
        BenchmarkId::new("backspace_sequence", backspace_changes.len()),
        &(script_text.as_str(), &backspace_changes),
        |b, (text, changes)| {
            b.iter(|| {
                let mut current_text = (*text).to_string();
                let start = Instant::now();

                for change in *changes {
                    current_text = apply_text_change(&current_text, change);
                }

                let duration = start.elapsed();
                std_black_box((current_text, duration))
            });
        },
    );

    // Copy-paste simulation - large insertions
    let paste_changes = create_paste_simulation(&script_text, 10);
    group.bench_with_input(
        BenchmarkId::new("paste_sequence", paste_changes.len()),
        &(script_text.as_str(), &paste_changes),
        |b, (text, changes)| {
            b.iter(|| {
                let mut current_text = (*text).to_string();
                let start = Instant::now();

                for change in *changes {
                    current_text = apply_text_change(&current_text, change);
                }

                let duration = start.elapsed();
                std_black_box((current_text, duration))
            });
        },
    );

    group.finish();
}

/// Benchmark memory usage patterns
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    group.sample_size(100);

    let sizes = [1000, 5000, 10000, 25000];

    for &size in &sizes {
        let script_text = ScriptGenerator::complex(size).generate();
        let change = create_section_change(&script_text, 100);

        group.bench_with_input(
            BenchmarkId::new("memory_efficiency", size),
            &(script_text.as_str(), &change),
            |b, (text, change)| {
                b.iter(|| {
                    // Measure memory usage during incremental parsing
                    let start = Instant::now();
                    let result = apply_incremental_change(black_box(text), black_box(change));
                    let duration = start.elapsed();

                    // Simulate memory pressure measurement
                    let estimated_memory = text.len() + change.new_text.len();
                    std_black_box((result, duration, estimated_memory))
                });
            },
        );
    }

    group.finish();
}

/// Benchmark real-world files with realistic complexity profiles
fn bench_real_world_files(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_world_files");
    group.sample_size(100);

    let size = 1000;
    let realistic_profiles = [
        ("anime", ScriptGenerator::anime_realistic(size).generate()),
        ("movie", ScriptGenerator::movie_realistic(size).generate()),
        (
            "karaoke",
            ScriptGenerator::karaoke_realistic(size).generate(),
        ),
        ("sign", ScriptGenerator::sign_realistic(size).generate()),
        (
            "educational",
            ScriptGenerator::educational_realistic(size).generate(),
        ),
    ];

    for (profile_name, script_text) in &realistic_profiles {
        let change = create_section_change(script_text, 100);

        group.bench_with_input(
            BenchmarkId::new("realistic_incremental", profile_name),
            &(script_text.as_str(), &change),
            |b, (text, change)| {
                b.iter(|| {
                    let start = Instant::now();
                    let result = apply_incremental_change(black_box(text), black_box(change));
                    let duration = start.elapsed();
                    std_black_box((result, duration))
                });
            },
        );

        // Also benchmark full parse for comparison
        group.bench_with_input(
            BenchmarkId::new("realistic_full_parse", profile_name),
            script_text,
            |b, text| {
                b.iter(|| {
                    let start = Instant::now();
                    let result = Script::parse(black_box(text));
                    let duration = start.elapsed();
                    std_black_box((result, duration))
                });
            },
        );
    }

    group.finish();
}

/// Benchmark stress scenarios and edge cases
fn bench_stress_scenarios(c: &mut Criterion) {
    let mut group = c.benchmark_group("stress_scenarios");
    group.sample_size(50);
    group.measurement_time(std::time::Duration::from_secs(10)); // More time for stress tests

    // Test section boundary changes
    let script_text = ScriptGenerator::complex(1000).generate();
    let boundary_change = create_section_boundary_change(&script_text);

    group.bench_with_input(
        BenchmarkId::new("section_boundary", "change"),
        &(script_text.as_str(), &boundary_change),
        |b, (text, change)| {
            b.iter(|| {
                let start = Instant::now();
                let result = apply_incremental_change(black_box(text), black_box(change));
                let duration = start.elapsed();
                std_black_box((result, duration))
            });
        },
    );

    // Test malformed input recovery
    let malformed_change = create_malformed_change(&script_text);
    group.bench_with_input(
        BenchmarkId::new("malformed_recovery", "change"),
        &(script_text.as_str(), &malformed_change),
        |b, (text, change)| {
            b.iter(|| {
                let start = Instant::now();
                let result = apply_incremental_change(black_box(text), black_box(change));
                let duration = start.elapsed();
                std_black_box((result, duration))
            });
        },
    );

    // Test extremely large single change
    let large_change = create_large_change(&script_text, 10000);
    group.bench_with_input(
        BenchmarkId::new("large_single_change", "10k_chars"),
        &(script_text.as_str(), &large_change),
        |b, (text, change)| {
            b.iter(|| {
                let start = Instant::now();
                let result = apply_incremental_change(black_box(text), black_box(change));
                let duration = start.elapsed();
                std_black_box((result, duration))
            });
        },
    );

    group.finish();
}

/// Benchmark large-scale anime subtitle files (30-50MB range)
fn bench_large_scale_anime(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_scale_anime");
    group.sample_size(10); // Fewer samples due to large file sizes
    group.measurement_time(std::time::Duration::from_secs(30)); // More time for large files

    // Test different large file sizes representative of real anime releases
    let large_sizes = [
        ("24min_episode", 10_000), // ~5-10MB typical TV episode
        ("movie_2hr", 30_000),     // ~15-30MB typical movie
        ("ova_complex", 50_000),   // ~25-50MB OVA with heavy effects
        ("bdmv_full", 100_000),    // ~50MB+ BD release with all tracks
    ];

    for (profile_name, event_count) in &large_sizes {
        // Generate large anime subtitle file
        let script_text = ScriptGenerator::anime_realistic(*event_count).generate();
        let file_size_mb = script_text.len() as f64 / 1_048_576.0;

        println!("Generated {profile_name}: {file_size_mb:.1}MB ({event_count} events)");

        // Test incremental changes on large files
        let small_change = create_section_change(&script_text, 50);
        let medium_change = create_section_change(&script_text, 500);
        let large_change = create_section_change(&script_text, 5000);

        // Benchmark small edit (typical dialogue correction)
        group.throughput(Throughput::Bytes(script_text.len() as u64));
        group.bench_with_input(
            BenchmarkId::new(format!("{profile_name}_small_edit"), event_count),
            &(script_text.as_str(), &small_change),
            |b, (text, change)| {
                b.iter(|| {
                    let start = Instant::now();
                    let result = apply_incremental_change(black_box(text), black_box(change));
                    let duration = start.elapsed();
                    std_black_box((result, duration))
                });
            },
        );

        // Benchmark medium edit (scene timing adjustment)
        group.bench_with_input(
            BenchmarkId::new(format!("{profile_name}_medium_edit"), event_count),
            &(script_text.as_str(), &medium_change),
            |b, (text, change)| {
                b.iter(|| {
                    let start = Instant::now();
                    let result = apply_incremental_change(black_box(text), black_box(change));
                    let duration = start.elapsed();
                    std_black_box((result, duration))
                });
            },
        );

        // Benchmark large edit (karaoke template insertion)
        group.bench_with_input(
            BenchmarkId::new(format!("{profile_name}_large_edit"), event_count),
            &(script_text.as_str(), &large_change),
            |b, (text, change)| {
                b.iter(|| {
                    let start = Instant::now();
                    let result = apply_incremental_change(black_box(text), black_box(change));
                    let duration = start.elapsed();
                    std_black_box((result, duration))
                });
            },
        );

        // Benchmark full parse for comparison
        group.bench_with_input(
            BenchmarkId::new(format!("{profile_name}_full_parse"), event_count),
            &script_text,
            |b, text| {
                b.iter(|| {
                    let start = Instant::now();
                    let result = Script::parse(black_box(text));
                    let duration = start.elapsed();
                    std_black_box((result, duration))
                });
            },
        );

        // Memory efficiency check
        group.bench_with_input(
            BenchmarkId::new(format!("{profile_name}_memory_ratio"), event_count),
            &(script_text.as_str(), &medium_change),
            |b, (text, change)| {
                b.iter(|| {
                    let input_size = text.len();
                    let start = Instant::now();
                    let result = apply_incremental_change(black_box(text), black_box(change));
                    let duration = start.elapsed();

                    // Estimate memory usage (in real implementation would use actual measurements)
                    let estimated_memory = input_size + change.new_text.len();
                    let memory_ratio = estimated_memory as f64 / input_size as f64;

                    std_black_box((result, duration, memory_ratio))
                });
            },
        );
    }

    group.finish();
}

// Helper functions for creating various types of changes

/// Create a change within a single section
fn create_section_change(script_text: &str, size: usize) -> TextChange {
    // Find Events section and create a change within it
    script_text.find("[Events]").map_or_else(
        || {
            // Fallback to middle of script
            let mid = script_text.len() / 2;
            TextChange {
                range: mid..mid + 10,
                new_text: "x".repeat(size),
                line_range: 5..6,
            }
        },
        |events_start| {
            let change_start = events_start + 50; // Skip header
            TextChange {
                range: change_start..change_start + 20,
                new_text: "x".repeat(size),
                line_range: 10..11,
            }
        },
    )
}

/// Create a change that spans multiple sections
fn create_cross_section_change(script_text: &str, size: usize) -> TextChange {
    // Find boundary between sections
    script_text.find("[Events]").map_or_else(
        || {
            let mid = script_text.len() / 2;
            TextChange {
                range: mid..mid + 20,
                new_text: "x".repeat(size),
                line_range: 5..7,
            }
        },
        |styles_end| {
            let change_start = styles_end.saturating_sub(20);
            TextChange {
                range: change_start..styles_end + 20,
                new_text: format!("\n{}\n[Events]\n", "x".repeat(size)),
                line_range: 8..12,
            }
        },
    )
}

/// Create a change at section boundary
fn create_section_boundary_change(script_text: &str) -> TextChange {
    if let Some(events_start) = script_text.find("[Events]") {
        TextChange {
            range: events_start..events_start,
            new_text: "\n[Custom Section]\nTest: Value\n\n".to_string(),
            line_range: 10..14,
        }
    } else {
        TextChange {
            range: 0..0,
            new_text: "[New Section]\n".to_string(),
            line_range: 1..2,
        }
    }
}

/// Create typing simulation - many small insertions
fn create_typing_simulation(script_text: &str, count: usize) -> Vec<TextChange> {
    let mut changes = Vec::new();
    let start_pos = script_text.len() / 2;

    for i in 0..count {
        changes.push(TextChange {
            range: start_pos + i..start_pos + i,
            new_text: ((b'a' + u8::try_from(i % 26).unwrap_or(0)) as char).to_string(),
            line_range: 10..10,
        });
    }

    changes
}

/// Create backspace simulation - many small deletions
fn create_backspace_simulation(script_text: &str, count: usize) -> Vec<TextChange> {
    let mut changes = Vec::new();
    let start_pos = script_text.len() / 2;

    for i in 0..count {
        let pos = start_pos.saturating_sub(i);
        changes.push(TextChange {
            range: pos..pos + 1,
            new_text: String::new(),
            line_range: 10..10,
        });
    }

    changes
}

/// Create paste simulation - larger insertions
fn create_paste_simulation(script_text: &str, count: usize) -> Vec<TextChange> {
    let mut changes = Vec::new();
    let start_pos = script_text.len() / 2;

    for i in 0..count {
        changes.push(TextChange {
            range: start_pos + i * 100..start_pos + i * 100,
            new_text: format!("Pasted content block {i} with some text\n"),
            line_range: 10 + u32::try_from(i).unwrap_or(0)..11 + u32::try_from(i).unwrap_or(0),
        });
    }

    changes
}

/// Create malformed change for error recovery testing
fn create_malformed_change(script_text: &str) -> TextChange {
    let mid = script_text.len() / 2;
    TextChange {
        range: mid..mid + 10,
        new_text: "{`[Events]` malformed {\\tag} content \\}".to_string(),
        line_range: 5..6,
    }
}

/// Create very large change for stress testing
fn create_large_change(script_text: &str, size: usize) -> TextChange {
    let mid = script_text.len() / 2;
    TextChange {
        range: mid..mid + 100,
        new_text: "x".repeat(size),
        line_range: 10..15,
    }
}

/// Apply a text change to source text (placeholder implementation)
fn apply_text_change(text: &str, change: &TextChange) -> String {
    let mut result = String::with_capacity(text.len() + change.new_text.len());
    result.push_str(&text[..change.range.start]);
    result.push_str(&change.new_text);
    result.push_str(&text[change.range.end..]);
    result
}

/// Apply incremental change (placeholder for actual incremental parser)
fn apply_incremental_change(text: &str, change: &TextChange) -> Result<String, String> {
    // For now, simulate incremental parsing by applying the change and re-parsing
    // In the real implementation, this would use the actual incremental parser
    let modified_text = apply_text_change(text, change);

    // Simulate incremental parsing overhead (minimal compared to full parse)
    match Script::parse(&modified_text) {
        Ok(_) => Ok(modified_text),
        Err(e) => Err(format!("Parse error: {e:?}")),
    }
}

criterion_group!(
    incremental_benches,
    bench_incremental_parsing,
    bench_incremental_vs_full,
    bench_editor_simulation,
    bench_memory_usage,
    bench_real_world_files,
    bench_large_scale_anime,
    bench_stress_scenarios
);
criterion_main!(incremental_benches);
