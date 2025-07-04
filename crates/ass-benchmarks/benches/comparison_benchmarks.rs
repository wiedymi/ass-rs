use ass_core::Script;
use ass_render::SoftwareRenderer;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::time::Duration;

// Optional libass comparison - only compile if libass is available
#[cfg(feature = "libass-comparison")]
use libass::{Library, Renderer, Track};

/// Comprehensive benchmark suite comparing ass-rs with native libass
/// Run with: cargo bench --features libass-comparison

fn bench_parsing_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing_comparison");
    group.measurement_time(Duration::from_secs(10));

    let ass_data = include_bytes!("../assets/all_cases.ass");
    let ass_str = std::str::from_utf8(ass_data).unwrap();

    // Benchmark ass-rs parsing
    group.bench_function("ass_rs_parse", |b| {
        b.iter(|| {
            let _script = Script::parse(black_box(ass_data));
        });
    });

    // Benchmark native libass parsing (if available)
    #[cfg(feature = "libass-comparison")]
    {
        group.bench_function("libass_parse", |b| {
            b.iter(|| {
                let lib = Library::new().unwrap();
                let mut track = Track::new(&lib);
                track.set_data(black_box(ass_str)).unwrap();
            });
        });
    }

    group.finish();
}

fn bench_rendering_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("rendering_comparison");
    group.measurement_time(Duration::from_secs(15));

    let ass_data = include_bytes!("../assets/all_cases.ass");
    let ass_str = std::str::from_utf8(ass_data).unwrap();

    // Setup ass-rs
    let script = Script::parse(ass_data);
    let mock_font = create_mock_font();
    let ass_rs_renderer = SoftwareRenderer::new_multi(&script, vec![mock_font]);

    if let Ok(renderer) = ass_rs_renderer {
        group.bench_function("ass_rs_render_640x360", |b| {
            b.iter(|| {
                let _result = renderer.render_bitmap(
                    black_box(10.0),
                    black_box(640),
                    black_box(360),
                    black_box(24.0),
                );
            });
        });

        group.bench_function("ass_rs_render_1920x1080", |b| {
            b.iter(|| {
                let _result = renderer.render_bitmap(
                    black_box(10.0),
                    black_box(1920),
                    black_box(1080),
                    black_box(24.0),
                );
            });
        });
    }

    // Setup and benchmark native libass (if available)
    #[cfg(feature = "libass-comparison")]
    {
        let lib = Library::new().unwrap();
        let mut renderer = Renderer::new(&lib).unwrap();
        let mut track = Track::new(&lib);
        track.set_data(ass_str).unwrap();

        renderer.set_frame_size(640, 360);
        renderer.set_fonts(None, None, None, None, true).unwrap();

        group.bench_function("libass_render_640x360", |b| {
            b.iter(|| {
                let _images = renderer.render_frame(&track, black_box(10000)); // 10 seconds in ms
            });
        });

        renderer.set_frame_size(1920, 1080);
        group.bench_function("libass_render_1920x1080", |b| {
            b.iter(|| {
                let _images = renderer.render_frame(&track, black_box(10000));
            });
        });
    }

    group.finish();
}

fn bench_memory_usage_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage_comparison");
    group.measurement_time(Duration::from_secs(10));

    let ass_data = include_bytes!("../assets/all_cases.ass");
    let ass_str = std::str::from_utf8(ass_data).unwrap();

    // Test memory usage patterns for multiple script instances
    group.bench_function("ass_rs_multiple_scripts", |b| {
        b.iter(|| {
            let scripts: Vec<_> = (0..100)
                .map(|_| Script::parse(black_box(ass_data)))
                .collect();
            black_box(scripts);
        });
    });

    #[cfg(feature = "libass-comparison")]
    {
        group.bench_function("libass_multiple_tracks", |b| {
            b.iter(|| {
                let lib = Library::new().unwrap();
                let tracks: Vec<_> = (0..100)
                    .map(|_| {
                        let mut track = Track::new(&lib);
                        track.set_data(black_box(ass_str)).unwrap();
                        track
                    })
                    .collect();
                black_box(tracks);
            });
        });
    }

    group.finish();
}

fn bench_serialization_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization_comparison");
    group.measurement_time(Duration::from_secs(10));

    let ass_data = include_bytes!("../assets/all_cases.ass");
    let script = Script::parse(ass_data);

    // Benchmark ass-rs serialization
    group.bench_function("ass_rs_serialize", |b| {
        b.iter(|| {
            let _serialized = black_box(&script).serialize();
        });
    });

    // Note: libass doesn't have direct serialization, so we skip that comparison

    group.finish();
}

fn bench_complex_scripts_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("complex_scripts_comparison");
    group.measurement_time(Duration::from_secs(15));

    // Generate complex scripts with various features
    let simple_script = create_test_script(100, ScriptComplexity::Simple);
    let complex_script = create_test_script(100, ScriptComplexity::Complex);
    let unicode_script = create_unicode_test_script();

    // Test ass-rs with different script complexities
    for (name, script_data) in [
        ("simple", simple_script.as_bytes()),
        ("complex", complex_script.as_bytes()),
        ("unicode", unicode_script.as_bytes()),
    ]
    .iter()
    {
        group.bench_with_input(
            BenchmarkId::new("ass_rs_parse", name),
            script_data,
            |b, data| {
                b.iter(|| Script::parse(black_box(data)));
            },
        );

        #[cfg(feature = "libass-comparison")]
        {
            let script_str = std::str::from_utf8(script_data).unwrap();
            group.bench_with_input(
                BenchmarkId::new("libass_parse", name),
                &script_str,
                |b, data| {
                    b.iter(|| {
                        let lib = Library::new().unwrap();
                        let mut track = Track::new(&lib);
                        track.set_data(black_box(data)).unwrap();
                    });
                },
            );
        }
    }

    group.finish();
}

fn bench_timeline_processing_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("timeline_processing_comparison");
    group.measurement_time(Duration::from_secs(15));

    let ass_data = include_bytes!("../assets/all_cases.ass");
    let ass_str = std::str::from_utf8(ass_data).unwrap();

    // Setup
    let script = Script::parse(ass_data);
    let mock_font = create_mock_font();

    if let Ok(renderer) = SoftwareRenderer::new_multi(&script, vec![mock_font]) {
        // Test rendering at multiple timestamps (simulating video playback)
        group.bench_function("ass_rs_timeline_30fps", |b| {
            b.iter(|| {
                // Simulate 1 second of 30fps rendering
                for frame in 0..30 {
                    let timestamp = frame as f64 / 30.0;
                    let _result = renderer.render_bitmap(
                        black_box(timestamp),
                        black_box(640),
                        black_box(360),
                        black_box(24.0),
                    );
                }
            });
        });

        group.bench_function("ass_rs_timeline_60fps", |b| {
            b.iter(|| {
                // Simulate 1 second of 60fps rendering
                for frame in 0..60 {
                    let timestamp = frame as f64 / 60.0;
                    let _result = renderer.render_bitmap(
                        black_box(timestamp),
                        black_box(640),
                        black_box(360),
                        black_box(24.0),
                    );
                }
            });
        });
    }

    #[cfg(feature = "libass-comparison")]
    {
        let lib = Library::new().unwrap();
        let mut renderer = Renderer::new(&lib).unwrap();
        let mut track = Track::new(&lib);
        track.set_data(ass_str).unwrap();
        renderer.set_frame_size(640, 360);
        renderer.set_fonts(None, None, None, None, true).unwrap();

        group.bench_function("libass_timeline_30fps", |b| {
            b.iter(|| {
                for frame in 0..30 {
                    let timestamp_ms = (frame * 1000) / 30; // Convert to milliseconds
                    let _images = renderer.render_frame(&track, black_box(timestamp_ms));
                }
            });
        });

        group.bench_function("libass_timeline_60fps", |b| {
            b.iter(|| {
                for frame in 0..60 {
                    let timestamp_ms = (frame * 1000) / 60;
                    let _images = renderer.render_frame(&track, black_box(timestamp_ms));
                }
            });
        });
    }

    group.finish();
}

fn bench_large_scripts_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_scripts_comparison");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10); // Fewer samples for large benchmarks

    // Generate scripts of different sizes
    let sizes = [100, 500, 1000, 5000];

    for size in sizes.iter() {
        let large_script = create_test_script(*size, ScriptComplexity::Medium);
        let script_bytes = large_script.as_bytes();

        group.bench_with_input(
            BenchmarkId::new("ass_rs_parse_large", size),
            &script_bytes,
            |b, data| {
                b.iter(|| Script::parse(black_box(data)));
            },
        );

        #[cfg(feature = "libass-comparison")]
        {
            group.bench_with_input(
                BenchmarkId::new("libass_parse_large", size),
                &large_script,
                |b, data| {
                    b.iter(|| {
                        let lib = Library::new().unwrap();
                        let mut track = Track::new(&lib);
                        track.set_data(black_box(data)).unwrap();
                    });
                },
            );
        }
    }

    group.finish();
}

fn bench_feature_coverage_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("feature_coverage_comparison");
    group.measurement_time(Duration::from_secs(10));

    // Test specific ASS features
    let feature_scripts = [
        ("basic_formatting", create_basic_formatting_script()),
        ("advanced_tags", create_advanced_tags_script()),
        ("animations", create_animation_script()),
        ("positioning", create_positioning_script()),
    ];

    for (feature_name, script_content) in feature_scripts.iter() {
        let script_bytes = script_content.as_bytes();

        group.bench_with_input(
            BenchmarkId::new("ass_rs", feature_name),
            &script_bytes,
            |b, data| {
                b.iter(|| {
                    let script = Script::parse(black_box(data));
                    // Also test serialization to ensure full processing
                    let _serialized = script.serialize();
                });
            },
        );

        #[cfg(feature = "libass-comparison")]
        {
            group.bench_with_input(
                BenchmarkId::new("libass", feature_name),
                script_content,
                |b, data| {
                    b.iter(|| {
                        let lib = Library::new().unwrap();
                        let mut track = Track::new(&lib);
                        track.set_data(black_box(data)).unwrap();
                    });
                },
            );
        }
    }

    group.finish();
}

// Helper functions and types

#[derive(Clone, Copy)]
enum ScriptComplexity {
    Simple,
    Medium,
    Complex,
}

fn create_test_script(line_count: usize, complexity: ScriptComplexity) -> String {
    let mut script = String::from("[Script Info]\nTitle: Benchmark Test\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");

    for i in 0..line_count {
        let start_time = format!("0:00:{:02}.{:02}", (i * 2) / 100, (i * 2) % 100);
        let end_time = format!("0:00:{:02}.{:02}", ((i + 2) * 2) / 100, ((i + 2) * 2) % 100);

        let text = match complexity {
            ScriptComplexity::Simple => format!("Simple text line {}", i),
            ScriptComplexity::Medium => format!(
                "{{\\b1}}Bold text {}{{\\b0}} {{\\i1}}italic{}{{\\i0}}",
                i, i
            ),
            ScriptComplexity::Complex => format!(
                "{{\\pos({},{})}}{{\\frz{}}}{{\\c&H{:06X}&}}{{\\fscx{}}}{{\\fscy{}}}Complex text {}{{\\r}}",
                (i % 640), (i % 360), (i % 360), (i * 0x111) & 0xFFFFFF,
                100 + (i % 50), 100 + (i % 50), i
            ),
        };

        script.push_str(&format!(
            "Dialogue: 0,{},{},Default,,0,0,0,,{}\n",
            start_time, end_time, text
        ));
    }

    script
}

fn create_unicode_test_script() -> String {
    let mut script = String::from("[Script Info]\nTitle: Unicode Test\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");

    let unicode_texts = [
        "こんにちは世界！ Hello World!",
        "Привет мир! 🌍🚀✨",
        "مرحبا بالعالم العربي",
        "你好世界 Chinese text",
        "🎭🎪🎨🎯🎲 Emoji test",
        "Ελληνικά Greek text",
        "עברית Hebrew text",
        "ไทย Thai text",
    ];

    for (i, text) in unicode_texts.iter().enumerate() {
        let start_time = format!("0:00:{:02}.00", i * 3);
        let end_time = format!("0:00:{:02}.00", (i + 1) * 3);
        script.push_str(&format!(
            "Dialogue: 0,{},{},Default,,0,0,0,,{}\n",
            start_time, end_time, text
        ));
    }

    script
}

fn create_basic_formatting_script() -> String {
    r#"[Script Info]
Title: Basic Formatting Test

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,{\b1}Bold text{\b0}
Dialogue: 0,0:00:06.00,0:00:10.00,Default,,0,0,0,,{\i1}Italic text{\i0}
Dialogue: 0,0:00:11.00,0:00:15.00,Default,,0,0,0,,{\u1}Underlined text{\u0}
Dialogue: 0,0:00:16.00,0:00:20.00,Default,,0,0,0,,{\c&HFF0000&}Red text{\r}
"#
    .to_string()
}

fn create_advanced_tags_script() -> String {
    r#"[Script Info]
Title: Advanced Tags Test

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,{\pos(320,180)}Positioned text
Dialogue: 0,0:00:06.00,0:00:10.00,Default,,0,0,0,,{\frz45}Rotated text
Dialogue: 0,0:00:11.00,0:00:15.00,Default,,0,0,0,,{\fscx150\fscy150}Scaled text
Dialogue: 0,0:00:16.00,0:00:20.00,Default,,0,0,0,,{\alpha&H80&}Semi-transparent
"#
    .to_string()
}

fn create_animation_script() -> String {
    r#"[Script Info]
Title: Animation Test

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,{\move(100,100,500,100)}Moving text
Dialogue: 0,0:00:06.00,0:00:10.00,Default,,0,0,0,,{\t(\frz360)}Spinning text
Dialogue: 0,0:00:11.00,0:00:15.00,Default,,0,0,0,,{\t(\fscx200\fscy200)}Growing text
Dialogue: 0,0:00:16.00,0:00:20.00,Default,,0,0,0,,{\fade(255,0,255,0,1000,4000)}Fading text
"#
    .to_string()
}

fn create_positioning_script() -> String {
    r#"[Script Info]
Title: Positioning Test

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,{\an1}Bottom left
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,{\an2}Bottom center
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,{\an3}Bottom right
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,{\an4}Middle left
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,{\an5}Middle center
"#
    .to_string()
}

fn create_mock_font() -> &'static [u8] {
    static MOCK_FONT: &[u8] = &[
        0x00, 0x01, 0x00, 0x00, // sfnt version
        0x00, 0x04, // number of tables
        0x00, 0x80, // search range
        0x00, 0x02, // entry selector
        0x00, 0x00, // range shift
    ];
    MOCK_FONT
}

criterion_group!(
    comparison_benches,
    bench_parsing_comparison,
    bench_rendering_comparison,
    bench_memory_usage_comparison,
    bench_serialization_comparison,
    bench_complex_scripts_comparison,
    bench_timeline_processing_comparison,
    bench_large_scripts_comparison,
    bench_feature_coverage_comparison
);

criterion_main!(comparison_benches);
