//! Visual reporting system for compatibility test results

#[cfg(feature = "libass-compare")]
use crate::debug::{CompatibilityResult, DiffRegion, DiffType};
use crate::utils::RenderError;

#[cfg(feature = "nostd")]
use alloc::{format, string::String, vec::Vec};
#[cfg(not(feature = "nostd"))]
use std::{collections::HashMap, format, fs, path::Path, string::String, vec::Vec};

/// HTML report generator for compatibility test results
pub struct VisualReportGenerator {
    /// Test results to include in report
    results: Vec<CompatibilityResult>,
    /// Output directory for report files
    #[cfg(not(feature = "nostd"))]
    output_dir: String,
    /// Report configuration
    config: ReportConfig,
}

/// Configuration for report generation
#[derive(Debug, Clone)]
pub struct ReportConfig {
    /// Include thumbnail previews in report
    pub include_thumbnails: bool,
    /// Include detailed metrics
    pub include_metrics: bool,
    /// Include performance comparison
    pub include_performance: bool,
    /// Maximum number of results to show per page
    pub max_results_per_page: usize,
    /// Generate separate pages for different test categories
    pub categorize_by_type: bool,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            include_thumbnails: true,
            include_metrics: true,
            include_performance: true,
            max_results_per_page: 50,
            categorize_by_type: true,
        }
    }
}

impl VisualReportGenerator {
    /// Create new report generator
    #[cfg(not(feature = "nostd"))]
    pub fn new(output_dir: String, config: ReportConfig) -> Self {
        Self {
            results: Vec::new(),
            output_dir,
            config,
        }
    }

    /// Add test result to report
    pub fn add_result(&mut self, result: CompatibilityResult) {
        self.results.push(result);
    }

    /// Add multiple test results
    pub fn add_results(&mut self, results: Vec<CompatibilityResult>) {
        self.results.extend(results);
    }

    /// Generate complete HTML report
    #[cfg(not(feature = "nostd"))]
    pub fn generate_report(&self) -> Result<String, RenderError> {
        // Create output directory
        fs::create_dir_all(&self.output_dir)
            .map_err(|e| RenderError::IOError(format!("Failed to create output directory: {e}")))?;

        // Generate main report
        let report_path = Path::new(&self.output_dir).join("compatibility_report.html");
        let html_content = self.generate_html_report()?;

        fs::write(&report_path, html_content)
            .map_err(|e| RenderError::IOError(format!("Failed to write report: {e}")))?;

        // Generate CSS file
        let css_path = Path::new(&self.output_dir).join("report.css");
        fs::write(css_path, Self::generate_css())
            .map_err(|e| RenderError::IOError(format!("Failed to write CSS: {e}")))?;

        // Generate JavaScript file
        let js_path = Path::new(&self.output_dir).join("report.js");
        fs::write(js_path, Self::generate_javascript())
            .map_err(|e| RenderError::IOError(format!("Failed to write JavaScript: {e}")))?;

        // Generate summary JSON
        if self.config.include_metrics {
            let json_path = Path::new(&self.output_dir).join("summary.json");
            let summary = self.generate_summary_json()?;
            fs::write(json_path, summary)
                .map_err(|e| RenderError::IOError(format!("Failed to write summary JSON: {e}")))?;
        }

        Ok(report_path.to_string_lossy().to_string())
    }

    /// Generate HTML report content
    fn generate_html_report(&self) -> Result<String, RenderError> {
        let mut html = String::new();

        // HTML header
        html.push_str(&Self::generate_html_header());

        // Summary section
        html.push_str(&self.generate_summary_section()?);

        // Filter controls
        html.push_str(&Self::generate_filter_controls());

        // Results section
        if self.config.categorize_by_type {
            html.push_str(&self.generate_categorized_results()?);
        } else {
            html.push_str(&self.generate_flat_results()?);
        }

        // Footer
        html.push_str(&Self::generate_html_footer());

        Ok(html)
    }

    /// Generate HTML header
    fn generate_html_header() -> String {
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>ASS Renderer Compatibility Report</title>
    <link rel="stylesheet" href="report.css">
    <script src="report.js"></script>
</head>
<body>
    <div class="container">
        <header>
            <h1>ASS Renderer Compatibility Report</h1>
            <p class="subtitle">Pixel-perfect comparison with libass reference implementation</p>
        </header>
"#
        .to_string()
    }

    /// Generate summary section
    fn generate_summary_section(&self) -> Result<String, RenderError> {
        let total_tests = self.results.len();
        let passed_tests = self.results.iter().filter(|r| r.passed).count();
        let failed_tests = total_tests - passed_tests;

        let avg_diff = if total_tests > 0 {
            self.results
                .iter()
                .map(|r| r.pixel_diff_percentage)
                .sum::<f64>()
                / total_tests as f64
        } else {
            0.0
        };

        let max_diff = self
            .results
            .iter()
            .map(|r| r.pixel_diff_percentage)
            .fold(0.0, f64::max);

        let avg_performance = if self.config.include_performance {
            let performance_results: Vec<f64> = self
                .results
                .iter()
                .filter_map(|r| r.performance_ratio)
                .collect();

            if !performance_results.is_empty() {
                Some(performance_results.iter().sum::<f64>() / performance_results.len() as f64)
            } else {
                None
            }
        } else {
            None
        };

        let mut html = String::new();
        html.push_str(
            r#"<section class="summary">
            <h2>Test Summary</h2>
            <div class="summary-grid">
                <div class="summary-card">"#,
        );

        html.push_str(&format!(
            r#"
                    <h3>Total Tests</h3>
                    <div class="metric-value">{}</div>
                </div>
                <div class="summary-card passed">
                    <h3>Passed</h3>
                    <div class="metric-value">{}</div>
                    <div class="metric-percent">({:.1}%)</div>
                </div>
                <div class="summary-card failed">
                    <h3>Failed</h3>
                    <div class="metric-value">{}</div>
                    <div class="metric-percent">({:.1}%)</div>
                </div>"#,
            total_tests,
            passed_tests,
            (passed_tests as f64 / total_tests as f64) * 100.0,
            failed_tests,
            (failed_tests as f64 / total_tests as f64) * 100.0
        ));

        html.push_str(&format!(
            r#"
                <div class="summary-card">
                    <h3>Average Difference</h3>
                    <div class="metric-value">{:.3}%</div>
                </div>
                <div class="summary-card">
                    <h3>Maximum Difference</h3>
                    <div class="metric-value">{:.3}%</div>
                </div>"#,
            avg_diff * 100.0,
            max_diff * 100.0
        ));

        if let Some(perf) = avg_performance {
            html.push_str(&format!(
                r#"
                <div class="summary-card">
                    <h3>Performance Ratio</h3>
                    <div class="metric-value">{perf:.2}x</div>
                    <div class="metric-help">vs libass</div>
                </div>"#
            ));
        }

        html.push_str(
            r#"
            </div>
        </section>"#,
        );

        Ok(html)
    }

    /// Generate filter controls
    fn generate_filter_controls() -> String {
        r#"<section class="filters">
            <h2>Filters</h2>
            <div class="filter-controls">
                <label>
                    <input type="checkbox" id="show-passed" checked> Show Passed Tests
                </label>
                <label>
                    <input type="checkbox" id="show-failed" checked> Show Failed Tests
                </label>
                <label>
                    Status: 
                    <select id="status-filter">
                        <option value="all">All</option>
                        <option value="passed">Passed Only</option>
                        <option value="failed">Failed Only</option>
                    </select>
                </label>
                <label>
                    Max Difference: 
                    <input type="range" id="diff-threshold" min="0" max="10" step="0.1" value="10">
                    <span id="diff-value">10.0%</span>
                </label>
                <button id="reset-filters">Reset Filters</button>
            </div>
        </section>"#
            .to_string()
    }

    /// Generate categorized results
    fn generate_categorized_results(&self) -> Result<String, RenderError> {
        let mut categories: HashMap<String, Vec<&CompatibilityResult>> = HashMap::new();

        // Categorize results by test name patterns
        for result in &self.results {
            let category = Self::categorize_test(&result.test_name);
            categories.entry(category).or_default().push(result);
        }

        let mut html = String::new();
        html.push_str(
            r#"<section class="results">
            <h2>Test Results by Category</h2>"#,
        );

        for (category, results) in categories {
            html.push_str(&format!(
                r#"
                <div class="category">
                    <h3 class="category-title">{category}</h3>
                    <div class="results-grid">"#
            ));

            for result in results {
                html.push_str(&self.generate_result_card(result)?);
            }

            html.push_str(
                r#"
                    </div>
                </div>"#,
            );
        }

        html.push_str(r#"</section>"#);
        Ok(html)
    }

    /// Generate flat results list
    fn generate_flat_results(&self) -> Result<String, RenderError> {
        let mut html = String::new();
        html.push_str(
            r#"<section class="results">
            <h2>All Test Results</h2>
            <div class="results-grid">"#,
        );

        for result in &self.results {
            html.push_str(&self.generate_result_card(result)?);
        }

        html.push_str(
            r#"
            </div>
        </section>"#,
        );
        Ok(html)
    }

    /// Generate individual result card
    fn generate_result_card(&self, result: &CompatibilityResult) -> Result<String, RenderError> {
        let status_class = if result.passed { "passed" } else { "failed" };
        let diff_percent = result.pixel_diff_percentage * 100.0;

        let mut html = format!(
            r#"
            <div class="result-card {}" data-status="{}" data-diff="{}">
                <h4 class="test-name">{}</h4>
                <div class="test-status">
                    <span class="status-badge">{}</span>
                    <span class="diff-value">{:.3}% diff</span>
                </div>"#,
            status_class,
            if result.passed { "passed" } else { "failed" },
            diff_percent,
            result.test_name,
            if result.passed { "PASS" } else { "FAIL" },
            diff_percent
        );

        // Add metrics
        if self.config.include_metrics {
            html.push_str(&format!(
                r#"
                <div class="metrics">
                    <div class="metric">
                        <span class="label">Max Pixel Diff:</span>
                        <span class="value">{}</span>
                    </div>"#,
                result.max_pixel_diff
            ));

            if let Some(perf) = result.performance_ratio {
                html.push_str(&format!(
                    r#"
                    <div class="metric">
                        <span class="label">Performance:</span>
                        <span class="value">{perf:.2}x</span>
                    </div>"#
                ));
            }

            if !result.diff_regions.is_empty() {
                html.push_str(&format!(
                    r#"
                    <div class="metric">
                        <span class="label">Diff Regions:</span>
                        <span class="value">{}</span>
                    </div>"#,
                    result.diff_regions.len()
                ));
            }

            html.push_str(r#"</div>"#);
        }

        // Add thumbnails if enabled and files exist
        if self.config.include_thumbnails {
            html.push_str(&self.generate_thumbnail_section(&result.test_name)?);
        }

        // Add diff regions details
        if !result.diff_regions.is_empty() {
            html.push_str(&self.generate_diff_regions_section(&result.diff_regions)?);
        }

        html.push_str(r#"</div>"#);
        Ok(html)
    }

    /// Generate thumbnail section for a test
    #[cfg(not(feature = "nostd"))]
    fn generate_thumbnail_section(&self, test_name: &str) -> Result<String, RenderError> {
        let mut html = String::new();

        // Check if thumbnail files exist
        let our_path = Path::new(&self.output_dir).join(format!("{test_name}_ours.png"));
        let libass_path = Path::new(&self.output_dir).join(format!("{test_name}_libass.png"));
        let diff_path = Path::new(&self.output_dir).join(format!("{test_name}_diff.png"));

        if our_path.exists() || libass_path.exists() || diff_path.exists() {
            html.push_str(
                r#"
                <div class="thumbnails">
                    <div class="thumbnail-row">"#,
            );

            if our_path.exists() {
                html.push_str(&format!(
                    r#"
                        <div class="thumbnail">
                            <img src="{test_name}_ours.png" alt="Our Renderer" title="Our Renderer">
                            <label>Ours</label>
                        </div>"#
                ));
            }

            if libass_path.exists() {
                html.push_str(&format!(r#"
                        <div class="thumbnail">
                            <img src="{test_name}_libass.png" alt="libass Reference" title="libass Reference">
                            <label>libass</label>
                        </div>"#));
            }

            if diff_path.exists() {
                html.push_str(&format!(
                    r#"
                        <div class="thumbnail">
                            <img src="{test_name}_diff.png" alt="Difference Map" title="Difference Map">
                            <label>Diff</label>
                        </div>"#
                ));
            }

            html.push_str(
                r#"
                    </div>
                </div>"#,
            );
        }

        Ok(html)
    }

    #[cfg(feature = "nostd")]
    fn generate_thumbnail_section(&self, _test_name: &str) -> Result<String, RenderError> {
        Ok(String::new())
    }

    /// Generate diff regions section
    fn generate_diff_regions_section(&self, regions: &[DiffRegion]) -> Result<String, RenderError> {
        if regions.is_empty() {
            return Ok(String::new());
        }

        let mut html = String::new();
        html.push_str(
            r#"
            <div class="diff-regions">
                <h5>Difference Regions</h5>
                <div class="regions-list">"#,
        );

        for (i, region) in regions.iter().enumerate().take(10) {
            // Limit to first 10 regions
            let region_type = match region.diff_type {
                DiffType::PixelValue => "pixel",
                DiffType::TextPosition => "position",
                DiffType::AnimationTiming => "timing",
                DiffType::FontRendering => "font",
                DiffType::Effects => "effects",
                DiffType::Transform => "transform",
            };

            html.push_str(&format!(
                r#"
                    <div class="region-item {} type-{}">
                        <span class="region-bounds">{}x{} at ({}, {})</span>
                        <span class="region-type">{}</span>
                        <span class="region-diff">avg: {:.1}, max: {}</span>
                    </div>"#,
                if i % 2 == 0 { "even" } else { "odd" },
                region_type,
                region.bounds.2,
                region.bounds.3,
                region.bounds.0,
                region.bounds.1,
                region_type,
                region.avg_diff,
                region.max_diff
            ));
        }

        if regions.len() > 10 {
            html.push_str(&format!(
                r#"
                    <div class="region-item more">
                        ... and {} more regions
                    </div>"#,
                regions.len() - 10
            ));
        }

        html.push_str(
            r#"
                </div>
            </div>"#,
        );

        Ok(html)
    }

    /// Generate CSS styles
    fn generate_css() -> String {
        r#"/* CSS styles for compatibility report */
body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    margin: 0;
    padding: 20px;
    background-color: #f5f5f5;
    color: #333;
}

.container {
    max-width: 1200px;
    margin: 0 auto;
    background: white;
    border-radius: 8px;
    box-shadow: 0 2px 10px rgba(0,0,0,0.1);
    overflow: hidden;
}

header {
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    padding: 30px;
    text-align: center;
}

header h1 {
    margin: 0;
    font-size: 2.5em;
    font-weight: 300;
}

.subtitle {
    margin: 10px 0 0 0;
    opacity: 0.9;
    font-size: 1.1em;
}

.summary {
    padding: 30px;
    border-bottom: 1px solid #eee;
}

.summary-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 20px;
    margin-top: 20px;
}

.summary-card {
    background: #f8f9fa;
    padding: 20px;
    border-radius: 6px;
    text-align: center;
    border-left: 4px solid #007bff;
}

.summary-card.passed {
    border-left-color: #28a745;
    background: #f8fff9;
}

.summary-card.failed {
    border-left-color: #dc3545;
    background: #fff8f8;
}

.summary-card h3 {
    margin: 0 0 10px 0;
    font-size: 0.9em;
    text-transform: uppercase;
    color: #666;
    font-weight: 600;
}

.metric-value {
    font-size: 2em;
    font-weight: bold;
    color: #333;
}

.metric-percent {
    font-size: 0.9em;
    color: #666;
    margin-top: 5px;
}

.metric-help {
    font-size: 0.8em;
    color: #999;
    margin-top: 5px;
}

.filters {
    padding: 20px 30px;
    background: #f8f9fa;
    border-bottom: 1px solid #eee;
}

.filter-controls {
    display: flex;
    gap: 20px;
    align-items: center;
    flex-wrap: wrap;
}

.filter-controls label {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 0.9em;
}

button {
    background: #007bff;
    color: white;
    border: none;
    padding: 8px 16px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.9em;
}

button:hover {
    background: #0056b3;
}

.results {
    padding: 30px;
}

.category {
    margin-bottom: 40px;
}

.category-title {
    color: #495057;
    border-bottom: 2px solid #007bff;
    padding-bottom: 10px;
    margin-bottom: 20px;
}

.results-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(400px, 1fr));
    gap: 20px;
}

.result-card {
    background: white;
    border: 1px solid #dee2e6;
    border-radius: 6px;
    padding: 20px;
    transition: all 0.2s ease;
}

.result-card:hover {
    box-shadow: 0 4px 12px rgba(0,0,0,0.1);
    transform: translateY(-2px);
}

.result-card.passed {
    border-left: 4px solid #28a745;
}

.result-card.failed {
    border-left: 4px solid #dc3545;
}

.test-name {
    margin: 0 0 10px 0;
    font-size: 1.1em;
    color: #333;
}

.test-status {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 15px;
}

.status-badge {
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 0.8em;
    font-weight: bold;
}

.passed .status-badge {
    background: #d4edda;
    color: #155724;
}

.failed .status-badge {
    background: #f8d7da;
    color: #721c24;
}

.diff-value {
    font-size: 0.9em;
    color: #666;
}

.metrics {
    margin-bottom: 15px;
}

.metric {
    display: flex;
    justify-content: space-between;
    margin-bottom: 5px;
    font-size: 0.9em;
}

.metric .label {
    color: #666;
}

.metric .value {
    font-weight: 500;
}

.thumbnails {
    margin-top: 15px;
    padding-top: 15px;
    border-top: 1px solid #eee;
}

.thumbnail-row {
    display: flex;
    gap: 10px;
    justify-content: space-around;
}

.thumbnail {
    text-align: center;
    flex: 1;
}

.thumbnail img {
    max-width: 100%;
    max-height: 120px;
    border: 1px solid #ddd;
    border-radius: 4px;
    cursor: pointer;
}

.thumbnail img:hover {
    border-color: #007bff;
}

.thumbnail label {
    display: block;
    margin-top: 5px;
    font-size: 0.8em;
    color: #666;
}

.diff-regions {
    margin-top: 15px;
    padding-top: 15px;
    border-top: 1px solid #eee;
}

.diff-regions h5 {
    margin: 0 0 10px 0;
    font-size: 0.9em;
    color: #666;
    text-transform: uppercase;
}

.regions-list {
    max-height: 150px;
    overflow-y: auto;
}

.region-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 5px 0;
    font-size: 0.8em;
    border-bottom: 1px solid #f1f1f1;
}

.region-item.even {
    background: #f8f9fa;
}

.region-bounds {
    color: #333;
    font-family: monospace;
}

.region-type {
    padding: 2px 6px;
    border-radius: 3px;
    background: #e9ecef;
    color: #495057;
}

.region-type.type-pixel { background: #fff3cd; color: #856404; }
.region-type.type-position { background: #d4edda; color: #155724; }
.region-type.type-timing { background: #cce7ff; color: #004085; }
.region-type.type-font { background: #f8d7da; color: #721c24; }
.region-type.type-effects { background: #e2e3e5; color: #383d41; }
.region-type.type-transform { background: #ffeeba; color: #533f03; }

.region-diff {
    color: #666;
    font-family: monospace;
}

.hidden {
    display: none !important;
}

/* Responsive design */
@media (max-width: 768px) {
    .container {
        margin: 10px;
        border-radius: 0;
    }
    
    .results-grid {
        grid-template-columns: 1fr;
    }
    
    .filter-controls {
        flex-direction: column;
        align-items: flex-start;
    }
    
    .summary-grid {
        grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    }
}"#
        .to_string()
    }

    /// Generate JavaScript for interactive features
    fn generate_javascript() -> String {
        r#"// JavaScript for interactive compatibility report
document.addEventListener('DOMContentLoaded', function() {
    // Filter controls
    const showPassed = document.getElementById('show-passed');
    const showFailed = document.getElementById('show-failed');
    const statusFilter = document.getElementById('status-filter');
    const diffThreshold = document.getElementById('diff-threshold');
    const diffValue = document.getElementById('diff-value');
    const resetFilters = document.getElementById('reset-filters');

    // Update diff threshold display
    if (diffThreshold && diffValue) {
        diffThreshold.addEventListener('input', function() {
            diffValue.textContent = this.value + '%';
            applyFilters();
        });
    }

    // Apply filters
    function applyFilters() {
        const cards = document.querySelectorAll('.result-card');
        const showPassedChecked = showPassed ? showPassed.checked : true;
        const showFailedChecked = showFailed ? showFailed.checked : true;
        const statusFilterValue = statusFilter ? statusFilter.value : 'all';
        const maxDiff = diffThreshold ? parseFloat(diffThreshold.value) : 100;

        cards.forEach(card => {
            const status = card.dataset.status;
            const diff = parseFloat(card.dataset.diff) || 0;
            
            let visible = true;
            
            // Status filter
            if (statusFilterValue === 'passed' && status !== 'passed') {
                visible = false;
            } else if (statusFilterValue === 'failed' && status !== 'failed') {
                visible = false;
            }
            
            // Checkbox filters
            if (status === 'passed' && !showPassedChecked) {
                visible = false;
            } else if (status === 'failed' && !showFailedChecked) {
                visible = false;
            }
            
            // Difference threshold
            if (diff > maxDiff) {
                visible = false;
            }
            
            card.classList.toggle('hidden', !visible);
        });

        updateResultsCount();
    }

    // Update visible results count
    function updateResultsCount() {
        const visibleCards = document.querySelectorAll('.result-card:not(.hidden)');
        const totalCards = document.querySelectorAll('.result-card');
        
        // Update summary if exists
        const summaryElement = document.querySelector('.results h2');
        if (summaryElement) {
            summaryElement.textContent = `Test Results (${visibleCards.length} of ${totalCards.length} shown)`;
        }
    }

    // Event listeners
    if (showPassed) {
        showPassed.addEventListener('change', applyFilters);
    }
    if (showFailed) {
        showFailed.addEventListener('change', applyFilters);
    }
    if (statusFilter) {
        statusFilter.addEventListener('change', applyFilters);
    }
    if (resetFilters) {
        resetFilters.addEventListener('click', function() {
            if (showPassed) showPassed.checked = true;
            if (showFailed) showFailed.checked = true;
            if (statusFilter) statusFilter.value = 'all';
            if (diffThreshold) {
                diffThreshold.value = '10';
                diffValue.textContent = '10.0%';
            }
            applyFilters();
        });
    }

    // Image modal functionality
    const images = document.querySelectorAll('.thumbnail img');
    images.forEach(img => {
        img.addEventListener('click', function() {
            showImageModal(this.src, this.alt);
        });
    });

    function showImageModal(src, alt) {
        // Create modal
        const modal = document.createElement('div');
        modal.className = 'image-modal';
        modal.innerHTML = `
            <div class="modal-backdrop">
                <div class="modal-content">
                    <img src="${src}" alt="${alt}">
                    <div class="modal-caption">${alt}</div>
                    <button class="modal-close">&times;</button>
                </div>
            </div>
        `;

        // Add modal styles
        modal.style.cssText = `
            position: fixed;
            top: 0;
            left: 0;
            width: 100%;
            height: 100%;
            background: rgba(0,0,0,0.8);
            display: flex;
            justify-content: center;
            align-items: center;
            z-index: 1000;
        `;

        const content = modal.querySelector('.modal-content');
        content.style.cssText = `
            position: relative;
            max-width: 90%;
            max-height: 90%;
            background: white;
            border-radius: 8px;
            padding: 20px;
            text-align: center;
        `;

        const image = modal.querySelector('img');
        image.style.cssText = `
            max-width: 100%;
            max-height: 70vh;
            border-radius: 4px;
        `;

        const closeBtn = modal.querySelector('.modal-close');
        closeBtn.style.cssText = `
            position: absolute;
            top: 10px;
            right: 15px;
            background: none;
            border: none;
            font-size: 24px;
            cursor: pointer;
            color: #666;
        `;

        // Close modal handlers
        closeBtn.addEventListener('click', () => modal.remove());
        modal.addEventListener('click', (e) => {
            if (e.target === modal || e.target.className === 'modal-backdrop') {
                modal.remove();
            }
        });

        document.addEventListener('keydown', function escHandler(e) {
            if (e.key === 'Escape') {
                modal.remove();
                document.removeEventListener('keydown', escHandler);
            }
        });

        document.body.appendChild(modal);
    }

    // Initialize filters
    applyFilters();

    // Sort functionality
    addSortControls();

    function addSortControls() {
        const resultsSection = document.querySelector('.results');
        if (!resultsSection) return;

        const sortControls = document.createElement('div');
        sortControls.className = 'sort-controls';
        sortControls.innerHTML = `
            <label>
                Sort by:
                <select id="sort-by">
                    <option value="name">Test Name</option>
                    <option value="status">Status</option>
                    <option value="diff">Difference</option>
                    <option value="performance">Performance</option>
                </select>
            </label>
            <label>
                <input type="checkbox" id="sort-reverse"> Reverse Order
            </label>
        `;

        sortControls.style.cssText = `
            margin-bottom: 20px;
            padding: 15px;
            background: #f8f9fa;
            border-radius: 6px;
            display: flex;
            gap: 20px;
            align-items: center;
        `;

        const h2 = resultsSection.querySelector('h2');
        if (h2) {
            h2.insertAdjacentElement('afterend', sortControls);
        }

        const sortBy = document.getElementById('sort-by');
        const sortReverse = document.getElementById('sort-reverse');

        function applySorting() {
            const grids = document.querySelectorAll('.results-grid');
            grids.forEach(grid => {
                const cards = Array.from(grid.querySelectorAll('.result-card'));
                
                cards.sort((a, b) => {
                    let aValue, bValue;
                    
                    switch (sortBy.value) {
                        case 'name':
                            aValue = a.querySelector('.test-name').textContent;
                            bValue = b.querySelector('.test-name').textContent;
                            break;
                        case 'status':
                            aValue = a.dataset.status;
                            bValue = b.dataset.status;
                            break;
                        case 'diff':
                            aValue = parseFloat(a.dataset.diff) || 0;
                            bValue = parseFloat(b.dataset.diff) || 0;
                            break;
                        case 'performance':
                            aValue = parseFloat(a.querySelector('.metric .value')?.textContent) || 0;
                            bValue = parseFloat(b.querySelector('.metric .value')?.textContent) || 0;
                            break;
                        default:
                            return 0;
                    }
                    
                    let result;
                    if (typeof aValue === 'string') {
                        result = aValue.localeCompare(bValue);
                    } else {
                        result = aValue - bValue;
                    }
                    
                    return sortReverse.checked ? -result : result;
                });
                
                // Re-append sorted cards
                cards.forEach(card => grid.appendChild(card));
            });
        }

        sortBy.addEventListener('change', applySorting);
        sortReverse.addEventListener('change', applySorting);
    }
});
"#.to_string()
    }

    /// Generate HTML footer
    fn generate_html_footer() -> String {
        {
            #[cfg(not(feature = "nostd"))]
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| format!("{} seconds since epoch", d.as_secs()))
                .unwrap_or_else(|_| "unknown time".to_string());
            #[cfg(feature = "nostd")]
            let timestamp = "unknown time".to_string();

            format!(
                r#"
        <footer style="padding: 20px 30px; background: #f8f9fa; border-top: 1px solid #eee; text-align: center; color: #666;">
            <p>Report generated on {timestamp} by ASS Renderer Compatibility Testing Framework</p>
            <p>Compare with libass reference implementation for pixel-perfect subtitle rendering</p>
        </footer>
    </div>
</body>
</html>"#
            )
        }
    }

    /// Generate summary JSON for programmatic access
    #[cfg(feature = "serde")]
    fn generate_summary_json(&self) -> Result<String, RenderError> {
        let total_tests = self.results.len();
        let passed_tests = self.results.iter().filter(|r| r.passed).count();

        let summary = serde_json::json!({
            "summary": {
                "total_tests": total_tests,
                "passed_tests": passed_tests,
                "failed_tests": total_tests - passed_tests,
                "pass_rate": if total_tests > 0 { passed_tests as f64 / total_tests as f64 } else { 0.0 },
                "average_difference": if total_tests > 0 {
                    self.results.iter().map(|r| r.pixel_diff_percentage).sum::<f64>() / total_tests as f64
                } else { 0.0 },
                "maximum_difference": self.results.iter().map(|r| r.pixel_diff_percentage).fold(0.0, f64::max)
            },
            "results": self.results.iter().map(|r| {
                serde_json::json!({
                    "test_name": r.test_name,
                    "passed": r.passed,
                    "pixel_diff_percentage": r.pixel_diff_percentage,
                    "max_pixel_diff": r.max_pixel_diff,
                    "performance_ratio": r.performance_ratio,
                    "diff_regions_count": r.diff_regions.len()
                })
            }).collect::<Vec<_>>()
        });

        serde_json::to_string_pretty(&summary)
            .map_err(|e| RenderError::BackendError(format!("Failed to serialize summary: {e}")))
    }

    /// Categorize test by name pattern
    fn categorize_test(test_name: &str) -> String {
        if test_name.contains("basic") || test_name.contains("text") {
            "Basic Text Rendering".to_string()
        } else if test_name.contains("animation")
            || test_name.contains("move")
            || test_name.contains("fade")
        {
            "Animations".to_string()
        } else if test_name.contains("transform")
            || test_name.contains("rotation")
            || test_name.contains("scale")
        {
            "Transformations".to_string()
        } else if test_name.contains("karaoke") {
            "Karaoke Effects".to_string()
        } else if test_name.contains("drawing") || test_name.contains("clip") {
            "Drawing Commands".to_string()
        } else if test_name.contains("effect")
            || test_name.contains("blur")
            || test_name.contains("shadow")
        {
            "Visual Effects".to_string()
        } else if test_name.contains("edge") || test_name.contains("error") {
            "Edge Cases".to_string()
        } else {
            "Other".to_string()
        }
    }
}

/// Quick function to generate a basic report from results
#[cfg(not(feature = "nostd"))]
pub fn generate_compatibility_report(
    results: Vec<CompatibilityResult>,
    output_dir: &str,
) -> Result<String, RenderError> {
    let config = ReportConfig::default();
    let mut generator = VisualReportGenerator::new(output_dir.to_string(), config);
    generator.add_results(results);
    generator.generate_report()
}
