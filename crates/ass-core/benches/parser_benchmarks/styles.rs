//! Style resolution and overlap detection benchmark functions for
//! `parser_benchmarks`.

use ass_core::utils::generate_overlapping_script;
use criterion::{black_box, Criterion};

/// Benchmark style resolution and analysis performance
pub fn bench_style_resolution(c: &mut Criterion) {
    let mut group = c.benchmark_group("style_resolution");
    group.sample_size(500);

    let script_with_many_styles = r"[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,2,2,10,10,10,1
Style: Title,Impact,72,&H00FFD700,&H000000FF,&H00000000,&H80000000,1,0,0,0,120,120,2,0,1,3,3,2,0,0,0,1
Style: Subtitle,Calibri,45,&H00E6E6FA,&H000000FF,&H00404040,&H80000000,0,1,0,0,95,95,1,0,1,1,1,8,20,20,20,1
Style: Caption,Verdana,30,&H00FFFF00,&H000000FF,&H00800080,&H80000000,0,0,1,0,105,105,0.5,0,1,1.5,1.5,1,15,15,15,1
Style: Heading,Times,60,&H0000FFFF,&H000000FF,&H00FF0000,&H80000000,1,1,0,0,110,110,1.5,0,1,2.5,2.5,5,5,5,5,1
Style: Quote,Georgia,35,&H00C0C0C0,&H000000FF,&H00606060,&H80000000,0,1,0,0,98,98,0.8,0,1,1.2,1.2,7,25,25,25,1
Style: Code,Courier,28,&H0000FF00,&H000000FF,&H00008000,&H80000000,0,0,0,0,90,90,0,0,1,1,1,3,30,30,30,1
Style: Warning,Arial,40,&H000080FF,&H000000FF,&H00000080,&H80000000,1,0,0,0,115,115,1,0,1,2,2,2,10,10,10,1

[Events\]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Default style text
Dialogue: 0,0:00:05.00,0:00:10.00,Title,,0,0,0,,Title style text
Dialogue: 0,0:00:10.00,0:00:15.00,Subtitle,,0,0,0,,Subtitle style text
Dialogue: 0,0:00:15.00,0:00:20.00,Caption,,0,0,0,,Caption style text
";

    let script = ass_core::Script::parse(script_with_many_styles).unwrap();

    group.bench_function("style_validation", |b| {
        b.iter(|| {
            let analysis = black_box(ass_core::analysis::ScriptAnalysis::analyze(&script)).unwrap();
            let styles = analysis.resolved_styles();
            black_box(styles.len())
        });
    });

    group.bench_function("style_conflict_detection", |b| {
        let analysis = ass_core::analysis::ScriptAnalysis::analyze(&script).unwrap();
        b.iter(|| {
            let resolved_styles = analysis.resolved_styles();
            // Access style properties to trigger any conflict detection
            for style in resolved_styles {
                black_box(style.name);
            }
        });
    });

    group.finish();
}

/// Benchmark overlap detection performance with many events
pub fn bench_overlap_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("overlap_detection");
    group.sample_size(100);

    let small_script = generate_overlapping_script(50);
    let medium_script = generate_overlapping_script(200);
    let large_script = generate_overlapping_script(1000);

    group.bench_function("overlap_50_events", |b| {
        let script = ass_core::Script::parse(&small_script).unwrap();
        let analysis = ass_core::analysis::ScriptAnalysis::analyze(&script).unwrap();

        b.iter(|| {
            black_box(ass_core::analysis::find_overlapping_dialogue_events(
                analysis.dialogue_info(),
            ))
        });
    });

    group.bench_function("overlap_200_events", |b| {
        let script = ass_core::Script::parse(&medium_script).unwrap();
        let analysis = ass_core::analysis::ScriptAnalysis::analyze(&script).unwrap();

        b.iter(|| {
            black_box(ass_core::analysis::find_overlapping_dialogue_events(
                analysis.dialogue_info(),
            ))
        });
    });

    group.bench_function("overlap_1000_events", |b| {
        let script = ass_core::Script::parse(&large_script).unwrap();
        let analysis = ass_core::analysis::ScriptAnalysis::analyze(&script).unwrap();

        b.iter(|| {
            black_box(ass_core::analysis::find_overlapping_dialogue_events(
                analysis.dialogue_info(),
            ))
        });
    });

    group.finish();
}
