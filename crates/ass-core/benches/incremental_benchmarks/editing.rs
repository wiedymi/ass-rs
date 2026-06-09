//! Editor-simulation and memory-usage benchmark groups.
//!
//! Replays typing/backspace/paste sequences and measures memory-efficiency
//! characteristics of incremental changes.

#[cfg(not(feature = "std"))]
use alloc::string::ToString;
use ass_core::utils::ScriptGenerator;
use criterion::{black_box, BenchmarkId, Criterion};
use std::{hint::black_box as std_black_box, time::Instant};

use crate::{
    helpers::{apply_incremental_change, apply_text_change, create_section_change},
    simulations::{create_backspace_simulation, create_paste_simulation, create_typing_simulation},
};

/// Benchmark editor simulation scenarios
pub fn bench_editor_simulation(c: &mut Criterion) {
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
pub fn bench_memory_usage(c: &mut Criterion) {
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
