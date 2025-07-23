//! Comprehensive benchmarks for ASS parsing and analysis
//!
//! Tests parsing performance against project targets:
//! - <5ms for typical 1KB scripts
//! - <10MB peak memory usage
//! - <1.1x input memory ratio
//!
//! Generates synthetic ASS data programmatically to test various
//! complexity scenarios without external file dependencies.

use ass_core::{
    analysis::{
        events::{dialogue_info::DialogueInfo, text_analysis::TextAnalysis},
        linting::rules::{invalid_tag::InvalidTagRule, performance::PerformanceRule},
        linting::LintRule,
        ScriptAnalysis,
    },
    parser::{ast::EventType, streaming::StreamingParser, Event, Script, Section},
};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::fmt::Write;
use std::hint::black_box as std_black_box;

/// Synthetic ASS script generator for benchmarking
struct ScriptGenerator {
    /// Script title for metadata
    title: String,
    /// Number of styles to generate
    styles_count: usize,
    /// Number of events to generate
    events_count: usize,
    /// Complexity level for generated content
    complexity_level: ComplexityLevel,
}

/// Script complexity levels for testing
#[derive(Debug, Clone, Copy)]
enum ComplexityLevel {
    /// Simple text with minimal formatting
    Simple,
    /// Moderate formatting and some animations
    Moderate,
    /// Heavy animations, complex styling, karaoke
    Complex,
    /// Extreme complexity to stress-test parser
    Extreme,
}

impl ScriptGenerator {
    /// Create generator for simple scripts
    fn simple(events_count: usize) -> Self {
        Self {
            title: "Simple Benchmark Script".to_string(),
            styles_count: 1,
            events_count,
            complexity_level: ComplexityLevel::Simple,
        }
    }

    /// Create generator for moderate complexity scripts
    fn moderate(events_count: usize) -> Self {
        Self {
            title: "Moderate Benchmark Script".to_string(),
            styles_count: 5,
            events_count,
            complexity_level: ComplexityLevel::Moderate,
        }
    }

    /// Create generator for complex scripts
    fn complex(events_count: usize) -> Self {
        Self {
            title: "Complex Benchmark Script".to_string(),
            styles_count: 10,
            events_count,
            complexity_level: ComplexityLevel::Complex,
        }
    }

    /// Create generator for extreme complexity scripts
    fn extreme(events_count: usize) -> Self {
        Self {
            title: "Extreme Benchmark Script".to_string(),
            styles_count: 20,
            events_count,
            complexity_level: ComplexityLevel::Extreme,
        }
    }

    /// Generate complete ASS script as string
    fn generate(&self) -> String {
        let mut script =
            String::with_capacity(1000 + (self.styles_count * 200) + (self.events_count * 150));

        // Script Info section
        script.push_str(&self.generate_script_info());
        script.push('\n');

        // V4+ Styles section
        script.push_str(&self.generate_styles());
        script.push('\n');

        // Events section
        script.push_str(&self.generate_events());

        script
    }

    /// Generate Script Info section
    fn generate_script_info(&self) -> String {
        format!(
            r"[Script Info]
Title: {}
ScriptType: v4.00+
WrapStyle: 0
ScaledBorderAndShadow: yes
PlayResX: 1920
PlayResY: 1080",
            self.title
        )
    }

    /// Generate V4+ Styles section
    fn generate_styles(&self) -> String {
        let mut styles = String::from(
            "[V4+ Styles]\n\
            Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n"
        );

        for i in 0..self.styles_count {
            let style_name = if i == 0 {
                "Default"
            } else {
                &format!("Style{i}")
            };
            let fontsize = 20 + (i * 2);
            let color = format!("&H00{:06X}&", i * 0x0011_1111);

            writeln!(
                styles,
                "Style: {style_name},Arial,{fontsize},{color},{color},{color},&H00000000&,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1"
            ).unwrap();
        }

        styles
    }

    /// Generate Events section
    fn generate_events(&self) -> String {
        let mut events = String::from(
            "[Events]\n\
            Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n",
        );

        for i in 0..self.events_count {
            let start_cs = u32::try_from(i * 3000).unwrap_or(u32::MAX);
            let end_cs = u32::try_from(i * 3000 + 2500).unwrap_or(u32::MAX);
            let start_time = Self::format_time(start_cs); // 3 seconds apart
            let end_time = Self::format_time(end_cs); // 2.5 second duration
            let style = if self.styles_count > 1 {
                format!("Style{}", i % self.styles_count)
            } else {
                "Default".to_string()
            };
            let text = self.generate_dialogue_text(i);

            writeln!(
                events,
                "Dialogue: 0,{start_time},{end_time},{style},Speaker,0,0,0,,{text}"
            )
            .unwrap();
        }

        events
    }

    /// Format time in ASS format (H:MM:SS.cc)
    fn format_time(centiseconds: u32) -> String {
        let hours = centiseconds / 360_000;
        let minutes = (centiseconds % 360_000) / 6_000;
        let seconds = (centiseconds % 6000) / 100;
        let cs = centiseconds % 100;
        format!("{hours}:{minutes:02}:{seconds:02}.{cs:02}")
    }

    /// Generate dialogue text based on complexity level
    fn generate_dialogue_text(&self, event_index: usize) -> String {
        let base_text = format!("This is dialogue line number {}", event_index + 1);

        match self.complexity_level {
            ComplexityLevel::Simple => base_text,
            ComplexityLevel::Moderate => {
                format!(r"{{\b1}}{base_text}{{\b0}} with {{\i1}}some{{\i0}} formatting")
            }
            ComplexityLevel::Complex => {
                format!(
                    r"{{\pos(100,200)\fad(500,500)\b1\i1\c&H00FF00&}}{base_text}{{\b0\i0\c&HFFFFFF&}} with {{\t(0,1000,\frz360)}}animation{{\t(1000,2000,\frz0)}}"
                )
            }
            ComplexityLevel::Extreme => {
                format!(
                    r"{{\pos(100,200)\move(100,200,500,400)\fad(300,300)\t(0,500,\fscx120\fscy120)\t(500,1000,\fscx100\fscy100)\b1\i1\u1\s1\bord2\shad2\c&H00FF00&\3c&H0000FF&\4c&H000000&\alpha&H00\3a&H80}}{base_text}{{\b0\i0\u0\s0\r}} {{\k50}}with {{\k30}}karaoke {{\k40}}timing {{\k60}}and {{\k45}}complex {{\k35}}animations"
                )
            }
        }
    }
}

/// Benchmark basic parsing performance
fn bench_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing");

    // Test different script sizes
    let sizes = [10, 100, 1000, 5000];

    for &size in &sizes {
        let simple_script = ScriptGenerator::simple(size).generate();
        let moderate_script = ScriptGenerator::moderate(size).generate();
        let complex_script = ScriptGenerator::complex(size).generate();
        let extreme_script = ScriptGenerator::extreme(size).generate();

        group.throughput(Throughput::Bytes(simple_script.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("simple", size),
            &simple_script,
            |b, script| {
                b.iter(|| {
                    let result = Script::parse(black_box(script));
                    std_black_box(result)
                });
            },
        );

        group.throughput(Throughput::Bytes(moderate_script.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("moderate", size),
            &moderate_script,
            |b, script| {
                b.iter(|| {
                    let result = Script::parse(black_box(script));
                    std_black_box(result)
                });
            },
        );

        group.throughput(Throughput::Bytes(complex_script.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("complex", size),
            &complex_script,
            |b, script| {
                b.iter(|| {
                    let parsed = Script::parse(black_box(script));
                    black_box(parsed)
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("extreme", size),
            &extreme_script,
            |b, script| {
                b.iter(|| {
                    let parsed = Script::parse(black_box(script));
                    black_box(parsed)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark streaming parser performance
fn bench_streaming(c: &mut Criterion) {
    let mut group = c.benchmark_group("streaming");

    let sizes = [100, 1000, 5000];
    let chunk_sizes = [1024, 4096, 16384];

    for &size in &sizes {
        let script = ScriptGenerator::moderate(size).generate();

        for &chunk_size in &chunk_sizes {
            group.throughput(Throughput::Bytes(script.len() as u64));
            group.bench_with_input(
                BenchmarkId::new(format!("size_{size}_chunk_{chunk_size}"), ""),
                &(script.as_str(), chunk_size),
                |b, (script, chunk_size)| {
                    b.iter(|| {
                        let mut parser = StreamingParser::new();
                        let chunks = script.as_bytes().chunks(*chunk_size);

                        for chunk in chunks {
                            let result = parser.feed_chunk(black_box(chunk));
                            let _ = std_black_box(result);
                        }
                    });
                },
            );
        }
    }

    group.finish();
}

/// Benchmark text analysis performance
fn bench_text_analysis(c: &mut Criterion) {
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
fn bench_dialogue_analysis(c: &mut Criterion) {
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
fn bench_linting(c: &mut Criterion) {
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

/// Benchmark memory usage patterns
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");

    let sizes = [100, 1000, 5000, 10000];

    for &size in &sizes {
        let complex_script = ScriptGenerator::complex(size).generate();
        let extreme_script = ScriptGenerator::extreme(size).generate();

        group.bench_with_input(
            BenchmarkId::new("parse_and_analyze_complex", size),
            &complex_script,
            |b, script| {
                b.iter(|| {
                    // Parse script
                    let parsed = Script::parse(black_box(script)).unwrap();

                    // Analyze all events
                    if let Some(Section::Events(events)) = parsed
                        .sections()
                        .iter()
                        .find(|s| matches!(s, Section::Events(_)))
                    {
                        for event in events {
                            let dialogue_info = DialogueInfo::analyze(event);
                            let _ = std_black_box(dialogue_info);
                        }
                    }

                    std_black_box(parsed)
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("parse_and_analyze_extreme", size),
            &extreme_script,
            |b, script| {
                b.iter(|| {
                    // Parse script
                    let parsed = Script::parse(black_box(script)).unwrap();

                    // Analyze all events
                    if let Some(Section::Events(events)) = parsed
                        .sections()
                        .iter()
                        .find(|s| matches!(s, Section::Events(_)))
                    {
                        for event in events {
                            let dialogue_info = DialogueInfo::analyze(event);
                            let _ = std_black_box(dialogue_info);
                        }
                    }

                    std_black_box(parsed)
                });
            },
        );
    }

    group.finish();
}

/// Helper function to create test events
const fn create_test_event<'a>(start: &'a str, end: &'a str, text: &'a str) -> Event<'a> {
    Event {
        event_type: EventType::Dialogue,
        layer: "0",
        start,
        end,
        style: "Default",
        name: "",
        margin_l: "0",
        margin_r: "0",
        margin_v: "0",
        effect: "",
        text,
    }
}

/// Generate script with intentional issues for linting benchmarks
fn generate_script_with_issues(event_count: usize) -> String {
    let mut script = String::from(
        "[Script Info]\n\
        Title: Test Script\n\n\
        [V4+ Styles]\n\
        Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n\
        Style: Default,Arial,20,&H00FFFFFF&,&H000000FF&,&H00000000&,&H00000000&,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1\n\n\
        [Events]\n\
        Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n"
    );

    for i in 0..event_count {
        let start_time = format!("0:{:02}:{:02}.00", i / 60, i % 60);
        let end_time = format!("0:{:02}:{:02}.50", i / 60, i % 60);

        // Add some problematic content every 10th event
        let text = if i % 10 == 0 {
            r"Text with {\} empty tag and {\invalidtag} unknown tag"
        } else if i % 7 == 0 {
            // Very complex animation that might cause performance issues
            r"{\pos(100,200)\move(100,200,500,400,0,5000)\t(0,1000,\frz360)\t(1000,2000,\fscx200\fscy200)\t(2000,3000,\alpha&HFF&)\t(3000,4000,\alpha&H00&)\t(4000,5000,\c&HFF0000&)}Performance heavy animation"
        } else {
            let line_num = i + 1;
            &format!("Normal dialogue line {line_num}")
        };

        writeln!(
            script,
            "Dialogue: 0,{start_time},{end_time},Default,Speaker,0,0,0,,{text}"
        )
        .unwrap();
    }

    script
}

/// Benchmark UU-decoding performance for embedded media
fn bench_uu_decoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("uu_decoding");
    group.sample_size(1000);

    // Generate test UU-encoded data of various sizes
    let small_data = [
        "#0V%T", // "Cat" - 3 bytes
        "`",
    ];

    let medium_data = [
        "M5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O", // ~50 bytes
        "M5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O",
        "M5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O",
        "`",
    ];

    let large_data: Vec<&str> = (0..100)
        .map(|_| "M9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F")
        .chain(std::iter::once("`"))
        .collect();

    group.bench_function("small_uu_decode", |b| {
        b.iter(|| black_box(ass_core::utils::decode_uu_data(small_data.iter().copied())).unwrap());
    });

    group.bench_function("medium_uu_decode", |b| {
        b.iter(|| black_box(ass_core::utils::decode_uu_data(medium_data.iter().copied())).unwrap());
    });

    group.bench_function("large_uu_decode", |b| {
        b.iter(|| black_box(ass_core::utils::decode_uu_data(large_data.iter().copied())).unwrap());
    });

    group.finish();
}

/// Benchmark style resolution and analysis performance
fn bench_style_resolution(c: &mut Criterion) {
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

[Events]
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

/// Generate scripts with different numbers of potentially overlapping events
fn generate_overlapping_script(event_count: usize) -> String {
    use std::fmt::Write;
    let mut script = String::from(
        r"[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,2,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
",
    );

    for i in 0..event_count {
        // Create overlapping events: each event overlaps with several others
        let start_time = i * 2; // 2 second intervals
        let end_time = start_time + 5; // 5 second duration (overlaps next 2-3 events)
        writeln!(
            &mut script,
            "Dialogue: 0,0:{:02}:{:02}.00,0:{:02}:{:02}.00,Default,,0,0,0,,Event {} text",
            start_time / 60,
            start_time % 60,
            end_time / 60,
            end_time % 60,
            i
        )
        .unwrap();
    }

    script
}

/// Benchmark overlap detection performance with many events
fn bench_overlap_detection(c: &mut Criterion) {
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

criterion_group!(
    benches,
    bench_parsing,
    bench_streaming,
    bench_text_analysis,
    bench_dialogue_analysis,
    bench_linting,
    bench_memory_usage,
    bench_uu_decoding,
    bench_style_resolution,
    bench_overlap_detection
);
criterion_main!(benches);
