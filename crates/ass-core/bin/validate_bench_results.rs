//! Benchmark validation tool for performance targets
//!
//! Validates that benchmark results meet the project's performance targets:
//! - <5ms incremental parsing for typical operations
//! - <1.1x input memory ratio
//! - Consistent performance across different subtitle types

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::format;
use std::{env, process};

#[path = "validate_bench_results/loader.rs"]
mod loader;
#[path = "validate_bench_results/types.rs"]
mod types;
#[path = "validate_bench_results/validate.rs"]
mod validate;

use loader::load_benchmark_results;
use types::{PerformanceTargets, ValidationResults};
use validate::{validate_incremental_performance, validate_parser_performance};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!(
            "Usage: {} <parser_results.json> <incremental_results.json>",
            args[0]
        );
        process::exit(1);
    }

    let parser_results_path = &args[1];
    let incremental_results_path = &args[2];

    let targets = PerformanceTargets::default();
    let mut results = ValidationResults::new();

    // Validate incremental parsing performance
    if let Ok(incremental_results) = load_benchmark_results(incremental_results_path) {
        validate_incremental_performance(&incremental_results, &targets, &mut results);
    } else {
        results.add_fail(format!(
            "Failed to load incremental results from {incremental_results_path}"
        ));
    }

    // Validate parser performance
    if let Ok(parser_results) = load_benchmark_results(parser_results_path) {
        validate_parser_performance(&parser_results, &targets, &mut results);
    } else {
        results.add_fail(format!(
            "Failed to load parser results from {parser_results_path}"
        ));
    }

    // Print summary
    results.print_summary();

    if results.is_success() {
        println!("\\n🎉 All performance targets met!");
    } else {
        println!("\\n💥 Performance validation failed!");
        process::exit(1);
    }
}
