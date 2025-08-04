//! Simple memory profiling tool for ASS parser
//!
//! Run with: cargo run --bin memory-profile --features=benches

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{format, string::ToString};
#[allow(clippy::missing_docs_in_private_items, clippy::cast_precision_loss)]
use ass_core::{parser::Script, utils::ScriptGenerator};
use std::process::Command;

/// Get current process memory usage (RSS) in bytes
#[cfg(target_os = "macos")]
fn get_memory_usage() -> Option<usize> {
    let output = Command::new("ps")
        .args(["-o", "rss=", "-p", &std::process::id().to_string()])
        .output()
        .ok()?;

    let rss_kb = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<usize>()
        .ok()?;

    Some(rss_kb * 1024) // Convert KB to bytes
}

#[cfg(not(target_os = "macos"))]
fn get_memory_usage() -> Option<usize> {
    // Linux version would read from /proc/self/status
    None
}

fn format_bytes(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1_048_576 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.2} MB", bytes as f64 / 1_048_576.0)
    }
}

fn profile_memory(name: &str, script_text: &str) {
    println!("\n=== Memory Profile: {name} ===");
    println!("Input size: {}", format_bytes(script_text.len()));

    // Measure memory before parsing
    let mem_before = get_memory_usage();

    // Parse the script
    let start = std::time::Instant::now();
    let script = Script::parse(script_text).expect("Failed to parse");
    let parse_time = start.elapsed();

    // Measure memory after parsing
    let mem_after = get_memory_usage();

    // Calculate statistics
    let sections = script.sections().len();

    println!("Parse time: {:.2}ms", parse_time.as_secs_f64() * 1000.0);
    println!("Sections: {sections}");

    if let (Some(before), Some(after)) = (mem_before, mem_after) {
        let used = after.saturating_sub(before);
        let ratio = used as f64 / script_text.len() as f64;

        println!("Memory before: {}", format_bytes(before));
        println!("Memory after: {}", format_bytes(after));
        println!("Memory used: {}", format_bytes(used));
        println!("Memory ratio: {ratio:.2}x input size");

        if ratio <= 1.1 {
            println!("✅ PASS: Memory usage within target (<1.1x)");
        } else {
            println!("❌ FAIL: Memory usage exceeds target (>1.1x)");
        }
    } else {
        println!("⚠️  Memory measurement not available on this platform");
    }
}

fn main() {
    println!("ASS Parser Memory Profiler");
    println!("=========================");

    // Test various sizes
    let test_cases = [
        (
            "Small (100 events)",
            ScriptGenerator::simple(100).generate(),
        ),
        (
            "Medium (1000 events)",
            ScriptGenerator::moderate(1000).generate(),
        ),
        (
            "Large (10000 events)",
            ScriptGenerator::complex(10000).generate(),
        ),
        (
            "Anime Episode",
            ScriptGenerator::anime_realistic(10000).generate(),
        ),
        ("Movie", ScriptGenerator::movie_realistic(30000).generate()),
    ];

    for (name, script) in &test_cases {
        profile_memory(name, script);
    }

    // Summary
    println!("\n=== Summary ===");
    println!("Target: <1.1x input memory ratio");
    println!("This is a simple profiler. For detailed analysis, use:");
    println!("  - macOS: Instruments or leaks");
    println!("  - Linux: valgrind --tool=massif");
    println!("  - Windows: Visual Studio Diagnostic Tools");
}
