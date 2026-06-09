//! Loading and lightweight JSON extraction for criterion benchmark output.
//!
//! Walks a criterion output directory, reading the `base`/`change`
//! `estimates.json` files line by line and extracting the minimal point
//! estimates needed for regression analysis without a full JSON dependency.

#[cfg(not(feature = "std"))]
use alloc::{format, string::ToString};
use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

use crate::types::BenchmarkResult;

/// Load benchmark results from criterion output directory
pub fn load_criterion_results(criterion_dir: &str) -> io::Result<HashMap<String, BenchmarkResult>> {
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
            let estimate_end = estimate_section
                .find(&[',', '}'][..])
                .unwrap_or(estimate_section.len());
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
pub fn load_comparison_data(criterion_dir: &str) -> io::Result<HashMap<String, f64>> {
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
            let estimate_end = estimate_section
                .find(&[',', '}'][..])
                .unwrap_or(estimate_section.len());
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
