//! Performance regression checker for criterion benchmark results
//!
//! Analyzes criterion benchmark results to detect performance regressions
//! and ensure performance targets are maintained.

#![allow(clippy::missing_docs_in_private_items)]

use std::{
    collections::HashMap,
    env,
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
    process,
};

/// Performance regression threshold percentage
const REGRESSION_THRESHOLD_PERCENT: f64 = 10.0;
/// Incremental parsing target in milliseconds
const INCREMENTAL_PARSING_TARGET_MS: f64 = 5.0;

/// Benchmark result data
#[derive(Debug, Clone)]
struct BenchmarkResult {
    /// Benchmark name
    _name: String,
    /// Time in nanoseconds
    time_ns: f64,
    /// Confidence interval (not used)
    _confidence_interval: Option<(f64, f64)>,
}

impl BenchmarkResult {
    /// Convert time to milliseconds
    fn time_ms(&self) -> f64 {
        self.time_ns / 1_000_000.0
    }
}

/// Performance regression analysis results
#[derive(Debug)]
struct RegressionAnalysis {
    /// Performance regressions found
    regressions: Vec<String>,
    /// Performance improvements found
    improvements: Vec<String>,
    /// Results within acceptable threshold
    within_threshold: Vec<String>,
    /// Violations of absolute performance targets
    target_violations: Vec<String>,
}

impl RegressionAnalysis {
    /// Create new analysis instance
    const fn new() -> Self {
        Self {
            regressions: Vec::new(),
            improvements: Vec::new(),
            within_threshold: Vec::new(),
            target_violations: Vec::new(),
        }
    }
    
    /// Check if analysis found any failures
    fn has_failures(&self) -> bool {
        !self.regressions.is_empty() || !self.target_violations.is_empty()
    }
    
    /// Print summary of analysis results
    fn print_summary(&self) {
        println!("\\n=== Performance Regression Analysis ===");
        
        if !self.target_violations.is_empty() {
            println!("\\n🚨 PERFORMANCE TARGET VIOLATIONS:");
            for violation in &self.target_violations {
                println!("  {violation}");
            }
        }
        
        if !self.regressions.is_empty() {
            println!("\\n📉 PERFORMANCE REGRESSIONS:");
            for regression in &self.regressions {
                println!("  {regression}");
            }
        }
        
        if !self.improvements.is_empty() {
            println!("\\n📈 PERFORMANCE IMPROVEMENTS:");
            for improvement in &self.improvements {
                println!("  {improvement}");
            }
        }
        
        if !self.within_threshold.is_empty() {
            println!("\\n✅ WITHIN ACCEPTABLE RANGE:");
            for within in &self.within_threshold {
                println!("  {within}");
            }
        }
        
        println!("\\nSummary:");
        println!("  Target violations: {}", self.target_violations.len());
        println!("  Regressions: {}", self.regressions.len());
        println!("  Improvements: {}", self.improvements.len());
        println!("  Within threshold: {}", self.within_threshold.len());
    }
}

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

/// Load benchmark results from criterion output directory
fn load_criterion_results(criterion_dir: &str) -> io::Result<HashMap<String, BenchmarkResult>> {
    let mut results = HashMap::new();
    let criterion_path = Path::new(criterion_dir);
    
    if !criterion_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Criterion directory not found: {criterion_dir}"),
        ));
    }
    
    // Walk through criterion output structure
    for entry in std::fs::read_dir(criterion_path)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let bench_name = entry.file_name().to_string_lossy().to_string();
            let estimates_path = entry.path().join("base").join("estimates.json");
            
            if estimates_path.exists() {
                if let Ok(result) = load_estimates_file(&estimates_path, &bench_name) {
                    results.insert(bench_name, result);
                }
            }
        }
    }
    
    Ok(results)
}

/// Load a single estimates.json file
fn load_estimates_file(path: &Path, bench_name: &str) -> io::Result<BenchmarkResult> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    
    // Parse JSON manually (simplified parser)
    let mut content = String::new();
    for line in reader.lines() {
        content.push_str(&line?);
    }
    
    // Extract mean estimate (simplified JSON parsing)
    let time_ns = extract_mean_estimate(&content)?;
    
    Ok(BenchmarkResult {
        _name: bench_name.to_string(),
        time_ns,
        _confidence_interval: None,
    })
}

/// Extract mean estimate from criterion JSON
fn extract_mean_estimate(json_content: &str) -> io::Result<f64> {
    // Look for "mean":{"confidence_interval":...,"point_estimate":VALUE
    if let Some(mean_start) = json_content.find("\"mean\":{") {
        let mean_section = &json_content[mean_start..];
        if let Some(estimate_start) = mean_section.find("\"point_estimate\":") {
            let estimate_section = &mean_section[estimate_start + 17..];
            let estimate_end = estimate_section.find(&[',', '}'][..]).unwrap_or(estimate_section.len());
            let estimate_str = &estimate_section[..estimate_end];
            
            return estimate_str.parse().map_err(|e| {
                io::Error::new(io::ErrorKind::InvalidData, format!("Parse error: {e}"))
            });
        }
    }
    
    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "Could not find mean estimate in JSON",
    ))
}

/// Load comparison data between baseline and current results
fn load_comparison_data(criterion_dir: &str) -> io::Result<HashMap<String, f64>> {
    let mut comparisons = HashMap::new();
    let criterion_path = Path::new(criterion_dir);
    
    // Look for change estimates
    for entry in std::fs::read_dir(criterion_path)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let bench_name = entry.file_name().to_string_lossy().to_string();
            let change_path = entry.path().join("change").join("estimates.json");
            
            if change_path.exists() {
                if let Ok(change_percent) = load_change_estimate(&change_path) {
                    comparisons.insert(bench_name, change_percent);
                }
            }
        }
    }
    
    Ok(comparisons)
}

/// Load change estimate from comparison JSON
fn load_change_estimate(path: &Path) -> io::Result<f64> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    
    let mut content = String::new();
    for line in reader.lines() {
        content.push_str(&line?);
    }
    
    // Extract difference point estimate
    if let Some(diff_start) = content.find("\"difference\":{") {
        let diff_section = &content[diff_start..];
        if let Some(estimate_start) = diff_section.find("\"point_estimate\":") {
            let estimate_section = &diff_section[estimate_start + 17..];
            let estimate_end = estimate_section.find(&[',', '}'][..]).unwrap_or(estimate_section.len());
            let estimate_str = &estimate_section[..estimate_end];
            
            let ratio: f64 = estimate_str.parse().map_err(|e| {
                io::Error::new(io::ErrorKind::InvalidData, format!("Parse error: {e}"))
            })?;
            
            return Ok(ratio * 100.0); // Convert to percentage
        }
    }
    
    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "Could not find difference estimate in JSON",
    ))
}

/// Analyze performance against absolute targets
fn analyze_performance_targets(
    results: &HashMap<String, BenchmarkResult>,
    analysis: &mut RegressionAnalysis,
) {
    for (name, result) in results {
        let time_ms = result.time_ms();
        
        // Check incremental parsing targets
        if name.contains("incremental") {
            if time_ms > INCREMENTAL_PARSING_TARGET_MS {
                analysis.target_violations.push(format!(
                    "❌ {name}: {time_ms:.2}ms > {INCREMENTAL_PARSING_TARGET_MS}ms target"
                ));
            } else {
                analysis.within_threshold.push(format!(
                    "✅ {name}: {time_ms:.2}ms ≤ {INCREMENTAL_PARSING_TARGET_MS}ms target"
                ));
            }
        }
    }
}

/// Analyze performance regressions against baseline
fn analyze_regressions(
    comparisons: &HashMap<String, f64>,
    analysis: &mut RegressionAnalysis,
) {
    for (name, change_percent) in comparisons {
        let abs_change = change_percent.abs();
        
        if abs_change > REGRESSION_THRESHOLD_PERCENT {
            if *change_percent > 0.0 {
                // Performance got worse
                analysis.regressions.push(format!(
                    "📉 {name}: {change_percent:.1}% slower (>{REGRESSION_THRESHOLD_PERCENT}% threshold)"
                ));
            } else {
                // Performance improved significantly
                analysis.improvements.push(format!(
                    "📈 {name}: {abs_change:.1}% faster"
                ));
            }
        } else {
            analysis.within_threshold.push(format!(
                "✅ {name}: {change_percent:.1}% change (within {REGRESSION_THRESHOLD_PERCENT}% threshold)"
            ));
        }
    }
}