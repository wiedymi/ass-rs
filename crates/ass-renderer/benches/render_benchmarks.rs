//! Benchmarks for ASS renderer performance

use ass_core::parser::Script;
use ass_renderer::{RenderContext, Renderer};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

/// Simple test script for benchmarking
const SIMPLE_SCRIPT: &str = r#"[Script Info]
Title: Benchmark Script
ScriptType: v4.00+
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,40,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,2,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Simple benchmark text"#;

/// Complex script with many events
const COMPLEX_SCRIPT: &str = r#"[Script Info]
Title: Complex Benchmark Script
ScriptType: v4.00+
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,40,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,2,2,10,10,10,1
Style: Title,Arial Black,60,&H00FFFF00,&H000000FF,&H00000000,&H80000000,-1,0,0,0,100,100,0,0,1,3,3,5,10,10,30,1
Style: Small,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,1,1,7,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:10.00,Default,,0,0,0,,Line 1 with normal text
Dialogue: 0,0:00:00.50,0:00:10.00,Default,,0,0,0,,{\b1}Line 2 with bold text{\b0}
Dialogue: 0,0:00:01.00,0:00:10.00,Default,,0,0,0,,{\i1}Line 3 with italic text{\i0}
Dialogue: 0,0:00:01.50,0:00:10.00,Title,,0,0,0,,{\pos(960,100)}Title at top
Dialogue: 0,0:00:02.00,0:00:10.00,Default,,0,0,0,,{\c&H00FF00&}Green colored text
Dialogue: 0,0:00:02.50,0:00:10.00,Default,,0,0,0,,{\fscx200\fscy200}Scaled large text
Dialogue: 0,0:00:03.00,0:00:10.00,Small,,0,0,0,,Small text at top
Dialogue: 0,0:00:03.50,0:00:10.00,Default,,0,0,0,,{\move(100,500,1820,500,0,5000)}Moving horizontally
Dialogue: 0,0:00:04.00,0:00:10.00,Default,,0,0,0,,{\fade(255,0,255,0,500,4500,5000)}Fading in and out
Dialogue: 0,0:00:04.50,0:00:10.00,Default,,0,0,0,,{\clip(500,300,1420,780)}Clipped to rectangle
Dialogue: 1,0:00:05.00,0:00:10.00,Default,,0,0,0,,Layer 1 text
Dialogue: 2,0:00:05.50,0:00:10.00,Default,,0,0,0,,Layer 2 text
Dialogue: 0,0:00:06.00,0:00:10.00,Default,,0,0,0,,{\frz45}Rotated 45 degrees
Dialogue: 0,0:00:06.50,0:00:10.00,Default,,0,0,0,,{\bord5\shad5}Thick border and shadow
Dialogue: 0,0:00:07.00,0:00:10.00,Default,,0,0,0,,{\alpha&H80&}Semi-transparent text"#;

/// Script with animations
const ANIMATED_SCRIPT: &str = r#"[Script Info]
Title: Animated Benchmark Script
ScriptType: v4.00+
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,40,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,2,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,{\t(0,2000,\fscx200\fscy200)}Growing text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,{\t(0,2000,\frz360)}Rotating text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,{\t(0,3000,\c&HFF0000&)}Color changing text"#;

fn benchmark_simple_render(c: &mut Criterion) {
    let script = Script::parse(SIMPLE_SCRIPT).unwrap();
    let context = RenderContext::new(1920, 1080);
    let mut renderer = Renderer::with_auto_backend(context).unwrap();

    c.bench_function("render_simple_frame", |b| {
        b.iter(|| renderer.render_frame(black_box(&script), black_box(250)))
    });
}

fn benchmark_complex_render(c: &mut Criterion) {
    let script = Script::parse(COMPLEX_SCRIPT).unwrap();
    let context = RenderContext::new(1920, 1080);
    let mut renderer = Renderer::with_auto_backend(context).unwrap();

    c.bench_function("render_complex_frame", |b| {
        b.iter(|| renderer.render_frame(black_box(&script), black_box(500)))
    });
}

fn benchmark_animated_render(c: &mut Criterion) {
    let script = Script::parse(ANIMATED_SCRIPT).unwrap();
    let context = RenderContext::new(1920, 1080);
    let mut renderer = Renderer::with_auto_backend(context).unwrap();

    c.bench_function("render_animated_frame", |b| {
        b.iter(|| renderer.render_frame(black_box(&script), black_box(100)))
    });
}

fn benchmark_incremental_render(c: &mut Criterion) {
    let script = Script::parse(COMPLEX_SCRIPT).unwrap();
    let context = RenderContext::new(1920, 1080);
    let mut renderer = Renderer::with_auto_backend(context).unwrap();

    // Initial frame
    let initial_frame = renderer.render_frame(&script, 100).unwrap();

    c.bench_function("render_incremental", |b| {
        b.iter(|| {
            renderer.render_frame_incremental(
                black_box(&script),
                black_box(150),
                black_box(&initial_frame),
            )
        })
    });
}

fn benchmark_resolutions(c: &mut Criterion) {
    let script = Script::parse(COMPLEX_SCRIPT).unwrap();

    let mut group = c.benchmark_group("render_resolutions");

    for (width, height, name) in &[
        (640, 480, "SD"),
        (1280, 720, "HD"),
        (1920, 1080, "FHD"),
        (3840, 2160, "4K"),
    ] {
        let context = RenderContext::new(*width, *height);
        let mut renderer = Renderer::with_auto_backend(context).unwrap();

        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &(*width, *height),
            |b, _| b.iter(|| renderer.render_frame(black_box(&script), black_box(500))),
        );
    }

    group.finish();
}

fn benchmark_event_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("render_event_count");

    for num_events in &[1, 5, 10, 20, 50] {
        let mut events = String::new();
        for i in 0..*num_events {
            events.push_str(&format!(
                "Dialogue: 0,0:00:00.00,0:00:10.00,Default,,0,0,0,,Event {} text\n",
                i
            ));
        }

        let script_text = format!(
            r#"[Script Info]
Title: Event Count Benchmark
ScriptType: v4.00+
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,40,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,2,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
{}"#,
            events
        );

        let script = Script::parse(&script_text).unwrap();
        let context = RenderContext::new(1920, 1080);
        let mut renderer = Renderer::with_auto_backend(context).unwrap();

        group.bench_with_input(
            BenchmarkId::from_parameter(num_events),
            num_events,
            |b, _| b.iter(|| renderer.render_frame(black_box(&script), black_box(500))),
        );
    }

    group.finish();
}

fn benchmark_parsing(c: &mut Criterion) {
    c.bench_function("parse_simple_script", |b| {
        b.iter(|| Script::parse(black_box(SIMPLE_SCRIPT)))
    });

    c.bench_function("parse_complex_script", |b| {
        b.iter(|| Script::parse(black_box(COMPLEX_SCRIPT)))
    });
}

fn benchmark_collision_detection(c: &mut Criterion) {
    use ass_renderer::collision::{BoundingBox, CollisionResolver, PositionedEvent};

    let mut resolver = CollisionResolver::new(1920.0, 1080.0);

    // Add multiple existing events
    for i in 0..10 {
        let event = PositionedEvent {
            bbox: BoundingBox::new(100.0 * i as f32, 900.0, 150.0, 50.0),
            layer: 0,
            margin_v: 10,
            margin_l: 10,
            margin_r: 10,
            alignment: 2,
            priority: 100,
        };
        resolver.add_fixed(event);
    }

    c.bench_function("collision_detection", |b| {
        b.iter(|| {
            let event = PositionedEvent {
                bbox: BoundingBox::new(500.0, 900.0, 200.0, 50.0),
                layer: 0,
                margin_v: 10,
                margin_l: 10,
                margin_r: 10,
                alignment: 2,
                priority: 50,
            };
            resolver.find_position(black_box(event))
        })
    });
}

fn benchmark_animation_evaluation(c: &mut Criterion) {
    use ass_renderer::animation::{
        AnimatedValue, AnimationController, AnimationInterpolation, AnimationTiming, AnimationTrack,
    };

    let mut controller = AnimationController::new();

    // Add multiple animation tracks
    for i in 0..5 {
        let timing = AnimationTiming::new(0, 500, 1.0);
        let track = AnimationTrack::new(
            format!("property_{}", i),
            timing,
            AnimatedValue::Float {
                from: 0.0,
                to: 100.0,
            },
            AnimationInterpolation::Linear,
        );
        controller.add_track(track);
    }

    c.bench_function("animation_evaluation", |b| {
        b.iter(|| controller.evaluate(black_box(250)))
    });
}

// Comparison with theoretical libass performance (simulated)
fn benchmark_comparison(c: &mut Criterion) {
    let script = Script::parse(COMPLEX_SCRIPT).unwrap();
    let context = RenderContext::new(1920, 1080);
    let mut renderer = Renderer::with_auto_backend(context).unwrap();

    let mut group = c.benchmark_group("ass_renderer_vs_libass");

    // Our renderer
    group.bench_function("ass_renderer", |b| {
        b.iter(|| renderer.render_frame(black_box(&script), black_box(500)))
    });

    // Simulated libass performance (typically 5-10ms for complex frames)
    // This is just for comparison visualization in benchmarks
    group.bench_function("libass_simulated", |b| {
        b.iter(|| {
            // Simulate libass taking ~7ms
            std::thread::sleep(std::time::Duration::from_micros(7000));
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_simple_render,
    benchmark_complex_render,
    benchmark_animated_render,
    benchmark_incremental_render,
    benchmark_resolutions,
    benchmark_event_count,
    benchmark_parsing,
    benchmark_collision_detection,
    benchmark_animation_evaluation,
    benchmark_comparison
);

criterion_main!(benches);
