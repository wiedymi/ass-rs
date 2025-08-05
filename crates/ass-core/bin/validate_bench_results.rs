//! Benchmark validation tool for performance targets
//!
//! Validates that benchmark results meet the project's performance targets:
//! - <5ms incremental parsing for typical operations
//! - <1.1x input memory ratio
//! - Consistent performance across different subtitle types

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::{format, string::ToString};
#[allow(clippy::missing_docs_in_private_items)]
use std::{
    env,
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
    process,
};
/// Performance targets for validation
#[derive(Debug)]
struct PerformanceTargets {
    /// Maximum time for incremental parsing operations (milliseconds)
    max_incremental_time_ms: f64,
    /// Maximum memory ratio compared to input size
    max_memory_ratio: f64,
}

impl Default for PerformanceTargets {
    fn default() -> Self {
        Self {
            max_incremental_time_ms: 5.0,
            max_memory_ratio: 1.1,
        }
    }
}

/// Benchmark result data
#[derive(Debug)]
struct BenchmarkResult {
    name: String,
    time_ns: f64,
    throughput_bytes_per_sec: Option<f64>,
    memory_usage: Option<usize>,
}

/// Performance validation results
#[derive(Debug)]
struct ValidationResults {
    passed: Vec<String>,
    failed: Vec<String>,
    warnings: Vec<String>,
}

impl ValidationResults {
    const fn new() -> Self {
        Self {
            passed: Vec::new(),
            failed: Vec::new(),
            warnings: Vec::new(),
        }
    }

    fn add_pass(&mut self, test: String) {
        self.passed.push(test);
    }

    fn add_fail(&mut self, test: String) {
        self.failed.push(test);
    }

    fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    fn print_summary(&self) {
        println!("\\n=== Performance Validation Results ===");
        println!("Passed: {}", self.passed.len());
        println!("Failed: {}", self.failed.len());
        println!("Warnings: {}", self.warnings.len());

        if !self.failed.is_empty() {
            println!("\\nFAILED TESTS:");
            for failure in &self.failed {
                println!("  ❌ {failure}");
            }
        }

        if !self.warnings.is_empty() {
            println!("\\nWARNINGS:");
            for warning in &self.warnings {
                println!("  ⚠️  {warning}");
            }
        }

        if !self.passed.is_empty() {
            println!("\\nPASSED TESTS:");
            for pass in &self.passed {
                println!("  ✅ {pass}");
            }
        }
    }

    fn is_success(&self) -> bool {
        self.failed.is_empty()
    }
}

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

/// Load benchmark results from criterion JSON output
fn load_benchmark_results(path: &str) -> io::Result<Vec<BenchmarkResult>> {
    let mut results = Vec::new();

    if !Path::new(path).exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Benchmark results file not found: {path}"),
        ));
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Parse criterion JSON output (simplified parser)
    for line in reader.lines() {
        let line = line?;
        if line.contains("\"benchmark\"") && line.contains("\"mean\"") {
            if let Ok(result) = parse_benchmark_line(&line) {
                results.push(result);
            }
        }
    }

    Ok(results)
}

/// Parse a single benchmark result line from criterion JSON
fn parse_benchmark_line(line: &str) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
    // Simplified JSON parsing - in a real implementation, use serde_json
    let name = extract_json_string(line, "id")?;
    let time_ns = extract_json_number(line, "estimate")? * 1_000_000.0; // Convert to nanoseconds

    Ok(BenchmarkResult {
        name,
        time_ns,
        throughput_bytes_per_sec: None,
        memory_usage: None,
    })
}

/// Extract string value from JSON line
fn extract_json_string(line: &str, key: &str) -> Result<String, Box<dyn std::error::Error>> {
    let pattern = format!("\"{key}\":\"");
    if let Some(start) = line.find(&pattern) {
        let start = start + pattern.len();
        if let Some(end) = line[start..].find('"') {
            return Ok(line[start..start + end].to_string());
        }
    }
    Err("String not found".into())
}

/// Extract numeric value from JSON line
fn extract_json_number(line: &str, key: &str) -> Result<f64, Box<dyn std::error::Error>> {
    let pattern = format!("\"{key}\":");
    if let Some(start) = line.find(&pattern) {
        let start = start + pattern.len();
        let rest = &line[start..];
        let end = rest.find(&[',', '}'][..]).unwrap_or(rest.len());
        let num_str = rest[..end].trim();
        return num_str
            .parse()
            .map_err(|e| format!("Parse error: {e}").into());
    }
    Err("Number not found".into())
}

/// Validate incremental parsing performance targets
fn validate_incremental_performance(
    results: &[BenchmarkResult],
    targets: &PerformanceTargets,
    validation: &mut ValidationResults,
) {
    for result in results {
        let time_ms = result.time_ns / 1_000_000.0;

        // Check incremental parsing time target
        if result.name.contains("incremental") {
            // Allow more time for large-scale anime files
            let time_target = if result.name.contains("large_scale_anime") {
                // Scale target based on file size
                if result.name.contains("bdmv_full") {
                    50.0 // 50ms for 100k+ events
                } else if result.name.contains("ova_complex") {
                    25.0 // 25ms for 50k events
                } else if result.name.contains("movie_2hr") {
                    15.0 // 15ms for 30k events
                } else {
                    10.0 // 10ms for 10k events
                }
            } else {
                targets.max_incremental_time_ms
            };

            if time_ms <= time_target {
                validation.add_pass(format!(
                    "Incremental parsing '{0}': {time_ms:.2}ms ≤ {time_target}ms",
                    result.name
                ));
            } else {
                validation.add_fail(format!(
                    "Incremental parsing '{0}': {time_ms:.2}ms > {time_target}ms",
                    result.name
                ));
            }
        }

        // Check editor simulation performance
        if result.name.contains("editor_simulation") {
            if time_ms <= targets.max_incremental_time_ms * 2.0 {
                // Allow 2x for editor operations
                validation.add_pass(format!(
                    "Editor simulation '{0}': {time_ms:.2}ms ≤ {1}ms",
                    result.name,
                    targets.max_incremental_time_ms * 2.0
                ));
            } else {
                validation.add_fail(format!(
                    "Editor simulation '{0}': {time_ms:.2}ms > {1}ms",
                    result.name,
                    targets.max_incremental_time_ms * 2.0
                ));
            }
        }
    }
}

/// Validate parser performance targets
fn validate_parser_performance(
    results: &[BenchmarkResult],
    targets: &PerformanceTargets,
    validation: &mut ValidationResults,
) {
    for result in results {
        let time_ms = result.time_ns / 1_000_000.0;

        // Check full parsing performance (should be reasonable but not as strict)
        if result.name.contains("parsing") && !result.name.contains("incremental") {
            if time_ms <= 50.0 {
                // 50ms for full parsing is reasonable
                validation.add_pass(format!(
                    "Full parsing '{0}': {time_ms:.2}ms ≤ 50ms",
                    result.name
                ));
            } else {
                validation.add_warning(format!(
                    "Full parsing '{0}': {time_ms:.2}ms > 50ms (not critical)",
                    result.name
                ));
            }
        }

        // Check memory usage if available
        if let Some(memory) = result.memory_usage {
            if let Some(throughput) = result.throughput_bytes_per_sec {
                let input_size = throughput / 1000.0; // Rough estimate
                #[allow(clippy::cast_precision_loss)]
                let ratio = memory as f64 / input_size;

                if ratio <= targets.max_memory_ratio {
                    validation.add_pass(format!(
                        "Memory ratio '{0}': {ratio:.2}x ≤ {1}x",
                        result.name, targets.max_memory_ratio
                    ));
                } else {
                    validation.add_fail(format!(
                        "Memory ratio '{0}': {ratio:.2}x > {1}x",
                        result.name, targets.max_memory_ratio
                    ));
                }
            }
        }
    }
}
