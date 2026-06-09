//! Shared helper functions for the incremental parsing benchmarks.
//!
//! Provides change-construction utilities and the apply/parse primitives used
//! by the individual benchmark groups.

#[cfg(not(feature = "std"))]
use alloc::{format, string::ToString};
use ass_core::parser::{incremental::TextChange, Script};

/// Check if running in quick mode (for CI or quick tests)
pub fn is_quick_bench() -> bool {
    std::env::var("QUICK_BENCH").is_ok()
}

/// Create a change within a single section
pub fn create_section_change(script_text: &str, size: usize) -> TextChange {
    // Find Events section and create a change within it
    script_text.find("[Events]").map_or_else(
        || {
            // Fallback to middle of script
            let mid = script_text.len() / 2;
            TextChange {
                range: mid..mid + 10,
                new_text: "x".repeat(size),
                line_range: 5..6,
            }
        },
        |events_start| {
            let change_start = events_start + 50; // Skip header
            TextChange {
                range: change_start..change_start + 20,
                new_text: "x".repeat(size),
                line_range: 10..11,
            }
        },
    )
}

/// Create a change that spans multiple sections
pub fn create_cross_section_change(script_text: &str, size: usize) -> TextChange {
    // Find boundary between sections
    script_text.find("[Events]").map_or_else(
        || {
            let mid = script_text.len() / 2;
            TextChange {
                range: mid..mid + 20,
                new_text: "x".repeat(size),
                line_range: 5..7,
            }
        },
        |styles_end| {
            let change_start = styles_end.saturating_sub(20);
            TextChange {
                range: change_start..styles_end + 20,
                new_text: format!("\n{}\n[Events]\n", "x".repeat(size)),
                line_range: 8..12,
            }
        },
    )
}

/// Create a change at section boundary
pub fn create_section_boundary_change(script_text: &str) -> TextChange {
    script_text.find("[Events]").map_or_else(
        || TextChange {
            range: 0..0,
            new_text: "[New Section]\n".to_string(),
            line_range: 1..2,
        },
        |events_start| TextChange {
            range: events_start..events_start,
            new_text: "\n[Custom Section]\nTest: Value\n\n".to_string(),
            line_range: 10..14,
        },
    )
}

/// Create malformed change for error recovery testing
pub fn create_malformed_change(script_text: &str) -> TextChange {
    let mid = script_text.len() / 2;
    TextChange {
        range: mid..mid + 10,
        new_text: "{`[Events]` malformed {\\tag} content \\}".to_string(),
        line_range: 5..6,
    }
}

/// Create very large change for stress testing
pub fn create_large_change(script_text: &str, size: usize) -> TextChange {
    let mid = script_text.len() / 2;
    TextChange {
        range: mid..mid + 100,
        new_text: "x".repeat(size),
        line_range: 10..15,
    }
}

/// Apply a text change to source text (placeholder implementation)
pub fn apply_text_change(text: &str, change: &TextChange) -> String {
    let mut result = String::with_capacity(text.len() + change.new_text.len());
    result.push_str(&text[..change.range.start]);
    result.push_str(&change.new_text);
    result.push_str(&text[change.range.end..]);
    result
}

/// Apply incremental change (placeholder for actual incremental parser)
pub fn apply_incremental_change(text: &str, change: &TextChange) -> Result<String, String> {
    // For now, simulate incremental parsing by applying the change and re-parsing
    // In the real implementation, this would use the actual incremental parser
    let modified_text = apply_text_change(text, change);

    // Simulate incremental parsing overhead (minimal compared to full parse)
    match Script::parse(&modified_text) {
        Ok(_) => Ok(modified_text),
        Err(e) => Err(format!("Parse error: {e:?}")),
    }
}
