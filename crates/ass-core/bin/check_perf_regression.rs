//! Performance regression checker for criterion benchmark results
//!
//! Analyzes criterion benchmark results to detect performance regressions
//! and ensure performance targets are maintained.

#[cfg(not(feature = "std"))]
extern crate alloc;
use std::{env, process};

#[path = "check_perf_regression/analysis.rs"]
mod analysis;
#[path = "check_perf_regression/loading.rs"]
mod loading;
#[path = "check_perf_regression/types.rs"]
mod types;

use analysis::{analyze_performance_targets, analyze_regressions};
use loading::{load_comparison_data, load_criterion_results};
use types::RegressionAnalysis;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <criterion_output_dir>", args[0]);
        process::exit(1);
    }

    let criterion_dir = &args[1];
    let mut analysis = RegressionAnalysis::new();

    // Load and analyze benchmark results
    if let Ok(results) = load_criterion_results(criterion_dir) {
        analyze_performance_targets(&results, &mut analysis);

        // Look for comparison data if available
        if let Ok(comparisons) = load_comparison_data(criterion_dir) {
            analyze_regressions(&comparisons, &mut analysis);
        }
    } else {
        eprintln!("Failed to load benchmark results from {criterion_dir}");
        process::exit(1);
    }

    // Print analysis results
    analysis.print_summary();

    if analysis.has_failures() {
        println!("\\n💥 Performance validation failed!");
        process::exit(1);
    } else {
        println!("\\n🎉 All performance checks passed!");
    }
}
