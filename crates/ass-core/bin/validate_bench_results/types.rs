//! Shared data types for benchmark result validation
//!
//! Defines the performance targets, parsed benchmark records, and the
//! accumulated validation results used to report pass/fail/warning state.

/// Performance targets for validation
#[derive(Debug)]
pub struct PerformanceTargets {
    /// Maximum time for incremental parsing operations (milliseconds)
    pub max_incremental_time_ms: f64,
    /// Maximum memory ratio compared to input size
    pub max_memory_ratio: f64,
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
pub struct BenchmarkResult {
    /// Benchmark name
    pub name: String,
    /// Time in nanoseconds
    pub time_ns: f64,
    /// Throughput in bytes per second
    pub throughput_bytes_per_sec: Option<f64>,
    /// Memory usage in bytes
    pub memory_usage: Option<usize>,
}

/// Performance validation results
#[derive(Debug)]
pub struct ValidationResults {
    /// Tests that passed validation
    passed: Vec<String>,
    /// Tests that failed validation
    failed: Vec<String>,
    /// Warning messages
    warnings: Vec<String>,
}

impl ValidationResults {
    /// Create a new validation results struct
    pub const fn new() -> Self {
        Self {
            passed: Vec::new(),
            failed: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Add a passing test
    pub fn add_pass(&mut self, test: String) {
        self.passed.push(test);
    }

    /// Add a failing test
    pub fn add_fail(&mut self, test: String) {
        self.failed.push(test);
    }

    /// Add a warning
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    /// Print validation summary
    pub fn print_summary(&self) {
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

    /// Check if all validations passed
    pub fn is_success(&self) -> bool {
        self.failed.is_empty()
    }
}
