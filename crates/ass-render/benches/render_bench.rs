use ass_core::Script;
use ass_render::SoftwareRenderer;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::sync::Arc;
use std::time::Duration;

// Create a valid mock font or return None if we can't
fn create_mock_renderer(script: &Script) -> Option<SoftwareRenderer> {
    // Try to create a more valid TTF mock
    let mut font_data = vec![0u8; 2048];

    // Basic TTF/OTF header structure
    font_data[0] = 0x00; // sfnt version
    font_data[1] = 0x01;
    font_data[2] = 0x00;
    font_data[3] = 0x00;

    // Number of tables (minimal)
    font_data[4] = 0x00;
    font_data[5] = 0x04; // 4 tables

    let font_slice: &'static [u8] = Box::leak(font_data.into_boxed_slice());

    // Try to create renderer, return None if it fails
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        SoftwareRenderer::new_multi(script, vec![font_slice])
    }))
    .ok()
}

fn benchmark_render_basic(c: &mut Criterion) {
    let ass_data = include_bytes!("../../../assets/all_cases.ass");
    let script = Script::parse(ass_data);

    match create_mock_renderer(&script) {
        Some(renderer) => {
            let mut group = c.benchmark_group("render_basic");
            group.measurement_time(Duration::from_secs(10));

            group.bench_function("640x360", |b| {
                b.iter(|| {
                    let _result = renderer.render_bitmap(
                        black_box(10.0),
                        black_box(640),
                        black_box(360),
                        black_box(24.0),
                    );
                });
            });

            group.bench_function("1280x720", |b| {
                b.iter(|| {
                    let _result = renderer.render_bitmap(
                        black_box(10.0),
                        black_box(1280),
                        black_box(720),
                        black_box(24.0),
                    );
                });
            });

            group.finish();
        }
        None => {
            // Skip benchmark if we can't create a renderer
            eprintln!("Skipping render_basic benchmark: no valid font available");
        }
    }
}

fn benchmark_render_different_sizes(c: &mut Criterion) {
    let ass_data = include_bytes!("../../../assets/all_cases.ass");
    let script = Script::parse(ass_data);

    match create_mock_renderer(&script) {
        Some(renderer) => {
            let mut group = c.benchmark_group("render_sizes");
            group.measurement_time(Duration::from_secs(15));

            let sizes = [
                (320, 240),   // QVGA
                (640, 360),   // nHD
                (640, 480),   // VGA
                (1280, 720),  // HD
                (1920, 1080), // Full HD
                (2560, 1440), // QHD
                (3840, 2160), // 4K UHD
            ];

            for (width, height) in sizes.iter() {
                group.bench_with_input(
                    BenchmarkId::new("size", format!("{}x{}", width, height)),
                    &(*width, *height),
                    |b, &(w, h)| {
                        b.iter(|| {
                            let _result = renderer.render_bitmap(
                                black_box(10.0),
                                black_box(w),
                                black_box(h),
                                black_box(24.0),
                            );
                        });
                    },
                );
            }
            group.finish();
        }
        None => {
            eprintln!("Skipping render_sizes benchmark: no valid font available");
        }
    }
}

fn benchmark_render_font_sizes(c: &mut Criterion) {
    let ass_data = include_bytes!("../../../assets/all_cases.ass");
    let script = Script::parse(ass_data);

    match create_mock_renderer(&script) {
        Some(renderer) => {
            let mut group = c.benchmark_group("render_font_sizes");
            group.measurement_time(Duration::from_secs(10));

            let font_sizes = [8.0, 12.0, 16.0, 20.0, 24.0, 32.0, 48.0, 64.0, 96.0];
            for font_size in font_sizes.iter() {
                group.bench_with_input(
                    BenchmarkId::new("font_size", format!("{}", font_size)),
                    font_size,
                    |b, &size| {
                        b.iter(|| {
                            let _result = renderer.render_bitmap(
                                black_box(10.0),
                                black_box(640),
                                black_box(360),
                                black_box(size),
                            );
                        });
                    },
                );
            }
            group.finish();
        }
        None => {
            eprintln!("Skipping render_font_sizes benchmark: no valid font available");
        }
    }
}

fn benchmark_render_timeline(c: &mut Criterion) {
    let ass_data = include_bytes!("../../../assets/all_cases.ass");
    let script = Script::parse(ass_data);

    match create_mock_renderer(&script) {
        Some(renderer) => {
            let mut group = c.benchmark_group("render_timeline");
            group.measurement_time(Duration::from_secs(15));

            let timestamps = [0.0, 5.0, 10.0, 15.0, 20.0, 30.0, 60.0, 120.0];
            for timestamp in timestamps.iter() {
                group.bench_with_input(
                    BenchmarkId::new("time", format!("{}", timestamp)),
                    timestamp,
                    |b, &time| {
                        b.iter(|| {
                            let _result = renderer.render_bitmap(
                                black_box(time),
                                black_box(640),
                                black_box(360),
                                black_box(24.0),
                            );
                        });
                    },
                );
            }

            // Test rapid timestamp changes (simulating seeking)
            group.bench_function("rapid_seeking", |b| {
                b.iter(|| {
                    let timestamps = [1.0, 30.0, 5.0, 45.0, 10.0, 60.0, 15.0];
                    for &time in timestamps.iter() {
                        let _result = renderer.render_bitmap(
                            black_box(time),
                            black_box(640),
                            black_box(360),
                            black_box(24.0),
                        );
                    }
                });
            });

            group.finish();
        }
        None => {
            eprintln!("Skipping render_timeline benchmark: no valid font available");
        }
    }
}

fn benchmark_render_script_complexity(c: &mut Criterion) {
    // Simple script
    let simple_script = b"[Events]\nDialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,Simple text";
    let simple = Script::parse(simple_script);

    // Complex script with multiple features
    let complex_script = include_bytes!("../../../assets/all_cases.ass");
    let complex = Script::parse(complex_script);

    // Very complex script with animations and effects
    let extreme_script = create_extreme_complexity_script();
    let extreme = Script::parse(extreme_script.as_bytes());

    let simple_renderer = create_mock_renderer(&simple);
    let complex_renderer = create_mock_renderer(&complex);
    let extreme_renderer = create_mock_renderer(&extreme);

    if let (Some(simple_renderer), Some(complex_renderer), Some(extreme_renderer)) =
        (simple_renderer, complex_renderer, extreme_renderer)
    {
        let mut group = c.benchmark_group("render_complexity");
        group.measurement_time(Duration::from_secs(15));

        group.bench_function("simple_script", |b| {
            b.iter(|| {
                let _result = simple_renderer.render_bitmap(
                    black_box(2.0),
                    black_box(640),
                    black_box(360),
                    black_box(24.0),
                );
            });
        });

        group.bench_function("complex_script", |b| {
            b.iter(|| {
                let _result = complex_renderer.render_bitmap(
                    black_box(15.0),
                    black_box(640),
                    black_box(360),
                    black_box(24.0),
                );
            });
        });

        group.bench_function("extreme_script", |b| {
            b.iter(|| {
                let _result = extreme_renderer.render_bitmap(
                    black_box(2.0),
                    black_box(640),
                    black_box(360),
                    black_box(24.0),
                );
            });
        });

        group.finish();
    } else {
        eprintln!("Skipping render_complexity benchmark: no valid font available");
    }
}

fn benchmark_render_memory_patterns(c: &mut Criterion) {
    let ass_data = include_bytes!("../../../assets/all_cases.ass");
    let script = Script::parse(ass_data);

    match create_mock_renderer(&script) {
        Some(renderer) => {
            let mut group = c.benchmark_group("render_memory");
            group.measurement_time(Duration::from_secs(15));

            // Test repeated rendering (cache effects)
            group.bench_function("repeated_render", |b| {
                b.iter(|| {
                    for _ in 0..10 {
                        let _result = renderer.render_bitmap(
                            black_box(10.0),
                            black_box(640),
                            black_box(360),
                            black_box(24.0),
                        );
                    }
                });
            });

            // Test rendering with different timestamps
            group.bench_function("varied_timestamps", |b| {
                b.iter(|| {
                    for i in 0..10 {
                        let _result = renderer.render_bitmap(
                            black_box(i as f64),
                            black_box(640),
                            black_box(360),
                            black_box(24.0),
                        );
                    }
                });
            });

            // Test memory usage with different output sizes
            group.bench_function("varied_output_sizes", |b| {
                let sizes = [(320, 240), (640, 360), (1280, 720), (1920, 1080)];
                b.iter(|| {
                    for (w, h) in sizes.iter() {
                        let _result = renderer.render_bitmap(
                            black_box(10.0),
                            black_box(*w),
                            black_box(*h),
                            black_box(24.0),
                        );
                    }
                });
            });

            group.finish();
        }
        None => {
            eprintln!("Skipping render_memory benchmark: no valid font available");
        }
    }
}

fn benchmark_render_edge_cases(c: &mut Criterion) {
    // Test with minimal and edge case content
    let empty_script = b"[Events]\n";
    let empty = Script::parse(empty_script);

    let unicode_script = b"[Events]\nDialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,Unicode: \xe3\x81\x93\xe3\x82\x93\xe3\x81\xab\xe3\x81\xa1\xe3\x81\xaf \xf0\x9f\x8c\x8d";
    let unicode = Script::parse(unicode_script);

    let malformed_script =
        b"[Events]\nDialogue: 0,invalid,0:00:05.00,Default,,0,0,0,,{\\incomplete tag";
    let malformed = Script::parse(malformed_script);

    let long_text_script = create_long_text_script();
    let long_text = Script::parse(long_text_script.as_bytes());

    let empty_renderer = create_mock_renderer(&empty);
    let unicode_renderer = create_mock_renderer(&unicode);
    let malformed_renderer = create_mock_renderer(&malformed);
    let long_text_renderer = create_mock_renderer(&long_text);

    if let (
        Some(empty_renderer),
        Some(unicode_renderer),
        Some(malformed_renderer),
        Some(long_text_renderer),
    ) = (
        empty_renderer,
        unicode_renderer,
        malformed_renderer,
        long_text_renderer,
    ) {
        let mut group = c.benchmark_group("render_edge_cases");
        group.measurement_time(Duration::from_secs(10));

        group.bench_function("empty_script", |b| {
            b.iter(|| {
                let _result = empty_renderer.render_bitmap(
                    black_box(2.0),
                    black_box(640),
                    black_box(360),
                    black_box(24.0),
                );
            });
        });

        group.bench_function("unicode_content", |b| {
            b.iter(|| {
                let _result = unicode_renderer.render_bitmap(
                    black_box(2.0),
                    black_box(640),
                    black_box(360),
                    black_box(24.0),
                );
            });
        });

        group.bench_function("malformed_content", |b| {
            b.iter(|| {
                let _result = malformed_renderer.render_bitmap(
                    black_box(2.0),
                    black_box(640),
                    black_box(360),
                    black_box(24.0),
                );
            });
        });

        group.bench_function("long_text_content", |b| {
            b.iter(|| {
                let _result = long_text_renderer.render_bitmap(
                    black_box(2.0),
                    black_box(640),
                    black_box(360),
                    black_box(24.0),
                );
            });
        });

        group.finish();
    } else {
        eprintln!("Skipping render_edge_cases benchmark: no valid font available");
    }
}

fn benchmark_render_performance_scenarios(c: &mut Criterion) {
    let ass_data = include_bytes!("../../../assets/all_cases.ass");
    let script = Script::parse(ass_data);

    match create_mock_renderer(&script) {
        Some(renderer) => {
            let mut group = c.benchmark_group("render_performance_scenarios");
            group.measurement_time(Duration::from_secs(20));

            // Simulate 30fps video playback for 1 second
            group.bench_function("video_30fps_1sec", |b| {
                b.iter(|| {
                    for frame in 0..30 {
                        let timestamp = frame as f64 / 30.0;
                        let _result = renderer.render_bitmap(
                            black_box(timestamp),
                            black_box(1920),
                            black_box(1080),
                            black_box(24.0),
                        );
                    }
                });
            });

            // Simulate 60fps video playback for 1 second
            group.bench_function("video_60fps_1sec", |b| {
                b.iter(|| {
                    for frame in 0..60 {
                        let timestamp = frame as f64 / 60.0;
                        let _result = renderer.render_bitmap(
                            black_box(timestamp),
                            black_box(1920),
                            black_box(1080),
                            black_box(24.0),
                        );
                    }
                });
            });

            // Simulate live streaming scenario (lower resolution, real-time)
            group.bench_function("live_stream_720p", |b| {
                b.iter(|| {
                    for frame in 0..30 {
                        let timestamp = frame as f64 / 30.0;
                        let _result = renderer.render_bitmap(
                            black_box(timestamp),
                            black_box(1280),
                            black_box(720),
                            black_box(20.0),
                        );
                    }
                });
            });

            // Simulate batch processing scenario
            group.bench_function("batch_processing", |b| {
                b.iter(|| {
                    let timestamps = [1.0, 5.0, 10.0, 15.0, 20.0, 25.0, 30.0];
                    for &timestamp in timestamps.iter() {
                        let _result = renderer.render_bitmap(
                            black_box(timestamp),
                            black_box(1920),
                            black_box(1080),
                            black_box(24.0),
                        );
                    }
                });
            });

            group.finish();
        }
        None => {
            eprintln!("Skipping render_performance_scenarios benchmark: no valid font available");
        }
    }
}

fn benchmark_render_concurrent_simulation(c: &mut Criterion) {
    let ass_data = include_bytes!("../../../assets/all_cases.ass");
    let script = Script::parse(ass_data);

    match create_mock_renderer(&script) {
        Some(renderer) => {
            let mut group = c.benchmark_group("render_concurrent");
            group.measurement_time(Duration::from_secs(20));

            // Create Arc outside the closure
            let renderer_arc = Arc::new(renderer);

            // Simulate multiple concurrent rendering requests
            group.bench_function("concurrent_simulation", |b| {
                use std::sync::Arc;
                use std::thread;

                b.iter(|| {
                    let handles: Vec<_> = (0..4)
                        .map(|i| {
                            let renderer = Arc::clone(&renderer_arc);
                            thread::spawn(move || {
                                for frame in 0..5 {
                                    let timestamp = (i * 5 + frame) as f64 / 30.0;
                                    let _result = renderer.render_bitmap(
                                        black_box(timestamp),
                                        black_box(640),
                                        black_box(360),
                                        black_box(24.0),
                                    );
                                }
                            })
                        })
                        .collect();

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            });

            group.finish();
        }
        None => {
            eprintln!("Skipping render_concurrent benchmark: no valid font available");
        }
    }
}

// Helper functions for creating test scripts

fn create_extreme_complexity_script() -> String {
    r#"[Script Info]
Title: Extreme Complexity Test

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,{\move(0,0,640,360,0,4000)\t(0,1000,\frz360\fscx200\fscy200)\fade(255,0,255,0,500,3500)\c&HFF00FF&\3c&H00FFFF&\be1\blur3}Extreme animation test
Dialogue: 0,0:00:02.00,0:00:06.00,Default,,0,0,0,,{\pos(320,180)\t(0,2000,\frx360\fry360)\c&HFF0000&\t(1000,3000,\c&H00FF00&)\t(2000,4000,\c&H0000FF&)}Multi-axis rotation with color transitions
Dialogue: 0,0:00:03.00,0:00:07.00,Default,,0,0,0,,{\move(100,100,540,260,0,3000)\t(0,1500,\fscx300\fscy50)\t(1500,3000,\fscx50\fscy300)\shad5\bord3}Complex scaling animation
Dialogue: 0,0:00:04.00,0:00:08.00,Default,,0,0,0,,{\pos(320,180)\t(0,4000,\frz720)\fade(0,255,0,255,0,1000,3000,4000)\alpha&H80&}Double rotation with complex fade
"#.to_string()
}

fn create_long_text_script() -> String {
    let long_text = "This is an extremely long subtitle line that contains a lot of text and should test the performance of the renderer when dealing with lengthy content. ".repeat(10);

    format!(
        r#"[Script Info]
Title: Long Text Test

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:10.00,Default,,0,0,0,,{}
"#,
        long_text
    )
}

criterion_group!(
    benches,
    benchmark_render_basic,
    benchmark_render_different_sizes,
    benchmark_render_font_sizes,
    benchmark_render_timeline,
    benchmark_render_script_complexity,
    benchmark_render_memory_patterns,
    benchmark_render_edge_cases,
    benchmark_render_performance_scenarios,
    benchmark_render_concurrent_simulation
);
criterion_main!(benches);
