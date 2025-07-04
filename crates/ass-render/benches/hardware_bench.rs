use ass_core::Script;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

#[cfg(feature = "hardware")]
use ass_render::HardwareRenderer;

#[cfg(feature = "software")]
use ass_render::SoftwareRenderer;

use std::time::Duration;
use tokio::runtime::Builder;

fn create_test_script(num_lines: usize) -> Script {
    let mut events = String::new();
    events.push_str(
        "Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n",
    );

    for i in 0..num_lines {
        let start_time = i as f64 * 0.1; // Stagger dialogue lines
        let end_time = start_time + 5.0;
        events.push_str(&format!(
            "Dialogue: 0,{}:{}:{:02}.{:02},{}:{}:{:02}.{:02},Default,,0,0,0,,Test line {}\n",
            (start_time as u32) / 3600,
            ((start_time as u32) % 3600) / 60,
            (start_time as u32) % 60,
            ((start_time * 100.0) as u32) % 100,
            (end_time as u32) / 3600,
            ((end_time as u32) % 3600) / 60,
            (end_time as u32) % 60,
            ((end_time * 100.0) as u32) % 100,
            i + 1
        ));
    }

    let ass_content = format!(
        r#"[Script Info]
Title: Benchmark Script

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,32,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
{}
"#,
        events
    );

    Script::parse(ass_content.as_bytes())
}

fn create_mock_font_data() -> Vec<u8> {
    // Generate a reasonable amount of mock font data for benchmarking
    vec![0u8; 65536] // 64KB of mock font data
}

#[cfg(feature = "software")]
fn bench_software_rendering(c: &mut Criterion) {
    let font_data = create_mock_font_data();
    let font_static: &'static [u8] = Box::leak(font_data.into_boxed_slice());

    let mut group = c.benchmark_group("software_rendering");

    for num_lines in [1, 10, 100, 500].iter() {
        let script = create_test_script(*num_lines);
        let renderer = SoftwareRenderer::new(&script, font_static);

        group.throughput(Throughput::Elements(*num_lines as u64));
        group.bench_with_input(
            BenchmarkId::new("render_frame", num_lines),
            num_lines,
            |b, _| {
                b.iter(|| {
                    let frame = black_box(renderer.render(1.0));
                    frame
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("render_bitmap", num_lines),
            num_lines,
            |b, _| {
                b.iter(|| {
                    let bitmap = black_box(renderer.render_bitmap(1.0, 1920, 1080, 32.0));
                    bitmap
                });
            },
        );
    }
    group.finish();
}

#[cfg(feature = "hardware")]
fn bench_hardware_rendering(c: &mut Criterion) {
    let rt = Builder::new_current_thread().enable_all().build().unwrap();
    let font_data = create_mock_font_data();

    let mut group = c.benchmark_group("hardware_rendering");
    group.measurement_time(Duration::from_secs(10)); // Longer measurement for GPU operations

    for num_lines in [1, 10, 100, 500].iter() {
        let script = create_test_script(*num_lines);

        // Try to create hardware renderer
        let renderer_result =
            rt.block_on(async { HardwareRenderer::new(&script, &font_data).await });

        match renderer_result {
            Ok(mut renderer) => {
                group.throughput(Throughput::Elements(*num_lines as u64));
                group.bench_with_input(
                    BenchmarkId::new("render_frame", num_lines),
                    num_lines,
                    |b, _| {
                        b.iter(|| {
                            let frame = black_box(renderer.render(1.0));
                            frame
                        });
                    },
                );

                group.bench_with_input(
                    BenchmarkId::new("render_to_texture", num_lines),
                    num_lines,
                    |b, _| {
                        b.iter(|| {
                            let result = rt.block_on(async {
                                renderer.render_to_texture(1.0, 1920, 1080, 32.0).await
                            });
                            black_box(result)
                        });
                    },
                );
            }
            Err(e) => {
                println!("Skipping hardware rendering benchmarks: {}", e);
                return;
            }
        }
    }
    group.finish();
}

fn bench_glyph_caching(c: &mut Criterion) {
    let font_data = create_mock_font_data();
    let rt = Builder::new_current_thread().enable_all().build().unwrap();

    let mut group = c.benchmark_group("glyph_caching");

    // Test various text complexities
    let test_texts = vec![
        ("ascii", "Hello World! This is a test."),
        ("unicode", "Hello 世界! Café naïve résumé"),
        (
            "complex",
            "The quick brown fox jumps over the lazy dog. 0123456789!@#$%^&*()",
        ),
        (
            "repeated",
            "aaaaaaaaaa bbbbbbbbbb cccccccccc dddddddddd eeeeeeeeee",
        ),
    ];

    for (test_name, text) in test_texts {
        let script_content = format!(
            r#"[Script Info]
Title: Glyph Test

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,32,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,{}
"#,
            text
        );

        let script = Script::parse(script_content.as_bytes());

        #[cfg(feature = "hardware")]
        {
            let renderer_result =
                rt.block_on(async { HardwareRenderer::new(&script, &font_data).await });

            if let Ok(mut renderer) = renderer_result {
                group.bench_function(&format!("hardware_glyph_{}", test_name), |b| {
                    b.iter(|| {
                        let result = rt.block_on(async {
                            renderer.render_to_texture(1.0, 800, 600, 24.0).await
                        });
                        black_box(result)
                    });
                });
            }
        }

        #[cfg(feature = "software")]
        {
            let font_static: &'static [u8] = Box::leak(font_data.clone().into_boxed_slice());
            let renderer = SoftwareRenderer::new(&script, font_static);

            group.bench_function(&format!("software_glyph_{}", test_name), |b| {
                b.iter(|| {
                    let bitmap = black_box(renderer.render_bitmap(1.0, 800, 600, 24.0));
                    bitmap
                });
            });
        }
    }
    group.finish();
}

fn bench_resolution_scaling(c: &mut Criterion) {
    let font_data = create_mock_font_data();
    let rt = Builder::new_current_thread().enable_all().build().unwrap();
    let script = create_test_script(50); // Fixed number of lines

    let mut group = c.benchmark_group("resolution_scaling");

    let resolutions = vec![
        ("720p", 1280, 720),
        ("1080p", 1920, 1080),
        ("1440p", 2560, 1440),
        ("4K", 3840, 2160),
    ];

    for (res_name, width, height) in resolutions {
        #[cfg(feature = "hardware")]
        {
            let renderer_result =
                rt.block_on(async { HardwareRenderer::new(&script, &font_data).await });

            if let Ok(mut renderer) = renderer_result {
                group.throughput(Throughput::Elements((width * height) as u64));
                group.bench_function(&format!("hardware_{}", res_name), |b| {
                    b.iter(|| {
                        let result = rt.block_on(async {
                            renderer.render_to_texture(1.0, width, height, 32.0).await
                        });
                        black_box(result)
                    });
                });
            }
        }

        #[cfg(feature = "software")]
        {
            let font_static: &'static [u8] = Box::leak(font_data.clone().into_boxed_slice());
            let renderer = SoftwareRenderer::new(&script, font_static);

            group.throughput(Throughput::Elements((width * height) as u64));
            group.bench_function(&format!("software_{}", res_name), |b| {
                b.iter(|| {
                    let bitmap = black_box(renderer.render_bitmap(1.0, width, height, 32.0));
                    bitmap
                });
            });
        }
    }
    group.finish();
}

fn bench_font_size_scaling(c: &mut Criterion) {
    let font_data = create_mock_font_data();
    let rt = Builder::new_current_thread().enable_all().build().unwrap();
    let script = create_test_script(20);

    let mut group = c.benchmark_group("font_size_scaling");

    let font_sizes = vec![12.0, 16.0, 24.0, 32.0, 48.0, 64.0, 96.0];

    for font_size in font_sizes {
        #[cfg(feature = "hardware")]
        {
            let renderer_result =
                rt.block_on(async { HardwareRenderer::new(&script, &font_data).await });

            if let Ok(mut renderer) = renderer_result {
                group.bench_function(&format!("hardware_size_{}", font_size as u32), |b| {
                    b.iter(|| {
                        let result = rt.block_on(async {
                            renderer.render_to_texture(1.0, 1920, 1080, font_size).await
                        });
                        black_box(result)
                    });
                });
            }
        }

        #[cfg(feature = "software")]
        {
            let font_static: &'static [u8] = Box::leak(font_data.clone().into_boxed_slice());
            let renderer = SoftwareRenderer::new(&script, font_static);

            group.bench_function(&format!("software_size_{}", font_size as u32), |b| {
                b.iter(|| {
                    let bitmap = black_box(renderer.render_bitmap(1.0, 1920, 1080, font_size));
                    bitmap
                });
            });
        }
    }
    group.finish();
}

// Define benchmark groups
#[cfg(feature = "software")]
criterion_group!(
    software_benches,
    bench_software_rendering,
    bench_glyph_caching,
    bench_resolution_scaling,
    bench_font_size_scaling
);

#[cfg(feature = "hardware")]
criterion_group!(
    hardware_benches,
    bench_hardware_rendering,
    bench_glyph_caching,
    bench_resolution_scaling,
    bench_font_size_scaling
);

// Main benchmark runner
#[cfg(all(feature = "software", feature = "hardware"))]
criterion_main!(software_benches, hardware_benches);

#[cfg(all(feature = "software", not(feature = "hardware")))]
criterion_main!(software_benches);

#[cfg(all(feature = "hardware", not(feature = "software")))]
criterion_main!(hardware_benches);

#[cfg(not(any(feature = "software", feature = "hardware")))]
fn main() {
    println!("No rendering features enabled. Enable 'software' or 'hardware' features to run benchmarks.");
}
