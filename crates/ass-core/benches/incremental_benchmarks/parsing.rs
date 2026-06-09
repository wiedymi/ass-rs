//! Core incremental parsing benchmark groups.
//!
//! Covers raw incremental-change application as well as the comparison between
//! incremental updates and a full reparse.

#[cfg(not(feature = "std"))]
use alloc::format;
use ass_core::{parser::Script, utils::ScriptGenerator};
use criterion::{black_box, BenchmarkId, Criterion, Throughput};
use std::{hint::black_box as std_black_box, time::Instant};

use crate::helpers::{
    apply_incremental_change, apply_text_change, create_cross_section_change,
    create_section_change, is_quick_bench,
};

/// Benchmark core incremental parsing operations
pub fn bench_incremental_parsing(c: &mut Criterion) {
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
pub fn bench_incremental_vs_full(c: &mut Criterion) {
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
