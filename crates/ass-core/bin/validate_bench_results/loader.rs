//! Benchmark result loading and lightweight JSON extraction
//!
//! Reads criterion-style JSON output line by line and extracts the minimal
//! fields needed for validation without pulling in a full JSON dependency.

#[cfg(not(feature = "std"))]
use alloc::{format, string::ToString};
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

use crate::types::BenchmarkResult;

/// Load benchmark results from criterion JSON output
pub fn load_benchmark_results(path: &str) -> io::Result<Vec<BenchmarkResult>> {
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
