//! Text, dialogue, and linting analysis benchmark functions for
//! `parser_benchmarks`.

use ass_core::{
    analysis::{
        events::{dialogue_info::DialogueInfo, text_analysis::TextAnalysis},
        linting::rules::{invalid_tag::InvalidTagRule, performance::PerformanceRule},
        linting::LintRule,
        ScriptAnalysis,
    },
    parser::Script,
    utils::{create_test_event, generate_script_with_issues},
};
use criterion::{black_box, BenchmarkId, Criterion};
use std::hint::black_box as std_black_box;

/// Benchmark text analysis performance
pub fn bench_text_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_analysis");

    // Generate different text complexities
    let simple_text = "Simple dialogue text";
    let moderate_text = r"Text with {\b1}bold{\b0} and {\i1}italic{\i0} formatting";
    let complex_text = r"{\pos(100,200)\fad(500,500)\b1\i1\c&H00FF00&}Complex text{\b0\i0\c&HFFFFFF&} with {\t(0,1000,\frz360)}animation{\t(1000,2000,\frz0)}";
    let extreme_text = r"{\pos(100,200)\move(100,200,500,400)\fad(300,300)\t(0,500,\fscx120\fscy120)\t(500,1000,\fscx100\fscy100)\b1\i1\u1\s1\bord2\shad2\c&H00FF00&\3c&H0000FF&\4c&H000000&\alpha&H00\3a&H80}Extreme complexity{\b0\i0\u0\s0\r} {\k50}with {\k30}karaoke {\k40}timing";

    let texts = [
        ("simple", simple_text),
        ("moderate", moderate_text),
        ("complex", complex_text),
        ("extreme", extreme_text),
    ];

    for (name, text) in &texts {
        group.bench_with_input(BenchmarkId::new("analyze", name), text, |b, text| {
            b.iter(|| {
                let result = TextAnalysis::analyze(black_box(text));
                std_black_box(result)
            });
        });
    }

    group.finish();
}

/// Benchmark dialogue info analysis
pub fn bench_dialogue_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("dialogue_analysis");

    // Create synthetic events for testing
    let simple_event = create_test_event("0:00:00.00", "0:00:05.00", "Simple text");
    let moderate_event = create_test_event(
        "0:00:05.00",
        "0:00:10.00",
        r"Text with {\b1}formatting{\b0}",
    );
    let complex_event = create_test_event(
        "0:00:10.00",
        "0:00:15.00",
        r"{\pos(100,200)\t(0,1000,\frz360)}Complex animation{\r}",
    );

    let events = [
        ("simple", &simple_event),
        ("moderate", &moderate_event),
        ("complex", &complex_event),
    ];

    for (name, event) in &events {
        group.bench_with_input(BenchmarkId::new("analyze", name), event, |b, event| {
            b.iter(|| {
                let result = DialogueInfo::analyze(black_box(event));
                std_black_box(result)
            });
        });
    }

    group.finish();
}

/// Benchmark linting rules performance
pub fn bench_linting(c: &mut Criterion) {
    let mut group = c.benchmark_group("linting");

    let sizes = [100, 1000, 5000];

    for &size in &sizes {
        // Generate script with some intentional issues
        let script_text = generate_script_with_issues(size);
        let script = Script::parse(&script_text).unwrap();

        // Test InvalidTagRule
        let invalid_tag_rule = InvalidTagRule;
        let analysis = ScriptAnalysis::analyze(&script).unwrap();
        group.bench_with_input(
            BenchmarkId::new("invalid_tag", size),
            &analysis,
            |b, analysis| {
                b.iter(|| {
                    let result = invalid_tag_rule.check_script(black_box(analysis));
                    std_black_box(result)
                });
            },
        );

        // Test PerformanceRule
        let performance_rule = PerformanceRule;
        let analysis = ScriptAnalysis::analyze(&script).unwrap();
        group.bench_with_input(
            BenchmarkId::new("performance", size),
            &analysis,
            |b, analysis| {
                b.iter(|| {
                    let result = performance_rule.check_script(black_box(analysis));
                    std_black_box(result)
                });
            },
        );
    }

    group.finish();
}
