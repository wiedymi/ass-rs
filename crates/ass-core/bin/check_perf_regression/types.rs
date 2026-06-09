//! Benchmark result and regression-analysis data types.
//!
//! Holds the parsed criterion timing for a single benchmark and the
//! accumulated regression/improvement/threshold/target-violation state used
//! to report the overall performance verdict.

/// Benchmark result data
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// Benchmark name
    pub _name: String,
    /// Time in nanoseconds
    pub time_ns: f64,
    /// Confidence interval (not used)
    pub _confidence_interval: Option<(f64, f64)>,
}

impl BenchmarkResult {
    /// Convert time to milliseconds
    pub fn time_ms(&self) -> f64 {
        self.time_ns / 1_000_000.0
    }
}

/// Performance regression analysis results
#[derive(Debug)]
pub struct RegressionAnalysis {
    /// Performance regressions found
    pub regressions: Vec<String>,
    /// Performance improvements found
    pub improvements: Vec<String>,
    /// Results within acceptable threshold
    pub within_threshold: Vec<String>,
    /// Violations of absolute performance targets
    pub target_violations: Vec<String>,
}

impl RegressionAnalysis {
    /// Create new analysis instance
    pub const fn new() -> Self {
        Self {
            regressions: Vec::new(),
            improvements: Vec::new(),
            within_threshold: Vec::new(),
            target_violations: Vec::new(),
        }
    }

    /// Check if analysis found any failures
    pub fn has_failures(&self) -> bool {
        !self.regressions.is_empty() || !self.target_violations.is_empty()
    }

    /// Print summary of analysis results
    pub fn print_summary(&self) {
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
