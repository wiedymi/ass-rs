//! Measure the GPU resident-layer steady-state collapse for a video overlay.
//!
//! The hybrid GPU backend caches the composited subtitle in a resident texture
//! ([`GpuBackend::render_subtitle_layer`]) and, while that subtitle is
//! unchanged, presents it with a single textured-quad blend
//! ([`GpuBackend::present_frame`]) — no CPU re-rasterize, no tile upload, no
//! readback. This example quantifies that collapse two ways:
//!
//! * **Steady state** — build the layer once for a fixed timestamp, then time
//!   `--present-iters` back-to-back `present_frame` calls: the headline
//!   "one quad blend of a cached texture" per-frame cost. Because the present
//!   pass always draws the cached layer as one full-screen quad, this cost is
//!   independent of how many tiles built the layer.
//! * **Realistic 60 fps playback** — step a time span at the requested frame
//!   rate and gate each frame exactly as [`Renderer`] does: select the active
//!   events, derive the static-frame cache key (animated `\t`/`\move`/`\fad`/
//!   `\k` events are never cacheable), rebuild the layer only when the key
//!   changes, and present every frame. Reports the cache-hit rate and the
//!   average cost on hit vs miss frames.
//!
//! Usage:
//! ```text
//! cargo run -p ass-renderer --no-default-features --features full,gpu \
//!     --example gpu_playback -- --ass FILE --size 1280x720 \
//!     --span 2700,12000 --static-time 12500 --present-iters 2000
//! ```
//! All times are centiseconds.

use std::time::Instant;

use ass_core::parser::{Event, Script};
use ass_renderer::backends::gpu::GpuBackend;
use ass_renderer::backends::BackendType;
use ass_renderer::renderer::{EventSelector, RenderContext, Renderer};

/// CPU composite baseline (ms/frame) from the prior `gpu_compare --bench` run.
const CPU_COMPOSITE_MS: f64 = 0.46;
/// GPU composite-with-readback baseline (ms/frame) from the same prior run.
const GPU_READBACK_MS: f64 = 1.49;

/// Parsed command-line configuration.
struct Cfg {
    ass: String,
    width: u32,
    height: u32,
    span: (u32, u32),
    static_time: u32,
    present_iters: u32,
    fps: u32,
}

fn parse_cfg() -> Result<Cfg, String> {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut cfg = Cfg {
        ass: concat!(env!("CARGO_MANIFEST_DIR"), "/benches/benchmark.ass").to_string(),
        width: 1280,
        height: 720,
        span: (2700, 12000),
        static_time: 12500,
        present_iters: 2000,
        fps: 60,
    };
    let mut i = 0;
    let val = |argv: &[String], i: &mut usize| -> Result<String, String> {
        *i += 1;
        argv.get(*i)
            .cloned()
            .ok_or_else(|| format!("missing value for {}", argv[*i - 1]))
    };
    while i < argv.len() {
        match argv[i].as_str() {
            "--ass" => cfg.ass = val(&argv, &mut i)?,
            "--size" => {
                let v = val(&argv, &mut i)?;
                let (w, h) = v.split_once('x').ok_or("bad --size")?;
                cfg.width = w.parse().map_err(|_| "bad width")?;
                cfg.height = h.parse().map_err(|_| "bad height")?;
            }
            "--span" => {
                let v = val(&argv, &mut i)?;
                let (s, e) = v.split_once(',').ok_or("bad --span")?;
                cfg.span = (
                    s.trim().parse().map_err(|_| "bad span start")?,
                    e.trim().parse().map_err(|_| "bad span end")?,
                );
            }
            "--static-time" => {
                cfg.static_time = val(&argv, &mut i)?.parse().map_err(|_| "bad --static-time")?;
            }
            "--present-iters" => {
                cfg.present_iters =
                    val(&argv, &mut i)?.parse().map_err(|_| "bad --present-iters")?;
            }
            "--fps" => cfg.fps = val(&argv, &mut i)?.parse().map_err(|_| "bad --fps")?,
            other => return Err(format!("unknown arg {other}")),
        }
        i += 1;
    }
    if cfg.fps == 0 {
        return Err("--fps must be > 0".into());
    }
    if cfg.span.1 <= cfg.span.0 {
        return Err("--span end must exceed start".into());
    }
    Ok(cfg)
}

/// Mirror of `Renderer::event_is_animated`: text whose output changes between
/// frames (so it must never be served from the static layer cache).
fn event_is_animated(text: &str) -> bool {
    text.contains("\\t")
        || text.contains("\\move")
        || text.contains("\\fad")
        || text.contains("\\k")
        || text.contains("\\K")
}

/// The static-frame cache key the renderer uses: `None` when any active event is
/// animated (never cacheable, so every frame re-renders), else the active
/// events' text spans as `(ptr, len)` pairs — identical spans mean identical
/// output, hence a present-only cache hit.
fn cache_key(events: &[&Event]) -> Option<Vec<(usize, usize)>> {
    if events.iter().any(|e| event_is_animated(e.text)) {
        return None;
    }
    Some(
        events
            .iter()
            .map(|e| (e.text.as_ptr() as usize, e.text.len()))
            .collect(),
    )
}

/// Build the subtitle layer once at `cfg.static_time`, then time `present_iters`
/// back-to-back present passes. Returns the steady-state ms/frame.
fn steady_state(
    cfg: &Cfg,
    software: &mut Renderer,
    gpu: &mut GpuBackend,
    script: &Script,
) -> Result<f64, String> {
    let (w, h) = (cfg.width, cfg.height);
    let tiles = software
        .render_frame_bitmaps(script, cfg.static_time)
        .map_err(|e| format!("tiles @ {}: {e}", cfg.static_time))?;
    gpu.render_subtitle_layer(&tiles, w, h)
        .map_err(|e| format!("render layer: {e}"))?;

    for _ in 0..16 {
        gpu.present_frame(w, h)
            .map_err(|e| format!("present warmup: {e}"))?;
    }
    let start = Instant::now();
    for _ in 0..cfg.present_iters {
        gpu.present_frame(w, h)
            .map_err(|e| format!("present: {e}"))?;
    }
    let ms = start.elapsed().as_secs_f64() * 1000.0 / f64::from(cfg.present_iters.max(1));
    println!(
        "steady-state present-only: layer cached from {} tiles @ t={}cs, \
         {} present iters -> {ms:.4} ms/frame",
        tiles.len(),
        cfg.static_time,
        cfg.present_iters
    );
    Ok(ms)
}

/// Step `cfg.span` at `cfg.fps`, gating each frame with the renderer's static
/// cache key: rebuild the layer only when the key changes, present every frame.
fn playback(
    cfg: &Cfg,
    software: &mut Renderer,
    gpu: &mut GpuBackend,
    script: &Script,
) -> Result<(), String> {
    let (w, h) = (cfg.width, cfg.height);
    let (start_cs, end_cs) = cfg.span;
    let frames = (end_cs - start_cs) * cfg.fps / 100 + 1;

    let mut gate = EventSelector::new();
    let mut last_key: Option<Vec<(usize, usize)>> = None;
    let (mut hits, mut misses) = (0u64, 0u64);
    let (mut hit_ms, mut miss_ms) = (0.0_f64, 0.0_f64);

    for k in 0..frames {
        let t = start_cs + k * 100 / cfg.fps;
        let frame_start = Instant::now();
        let active = gate
            .select_active(script, t)
            .map_err(|e| format!("select @ {t}: {e}"))?;
        let key = cache_key(&active.events);
        let hit = key.is_some() && last_key == key;
        if !hit {
            let tiles = software
                .render_frame_bitmaps(script, t)
                .map_err(|e| format!("tiles @ {t}: {e}"))?;
            gpu.render_subtitle_layer(&tiles, w, h)
                .map_err(|e| format!("layer @ {t}: {e}"))?;
        }
        last_key = key;
        gpu.present_frame(w, h)
            .map_err(|e| format!("present @ {t}: {e}"))?;
        let dt = frame_start.elapsed().as_secs_f64() * 1000.0;
        if hit {
            hits += 1;
            hit_ms += dt;
        } else {
            misses += 1;
            miss_ms += dt;
        }
    }

    let total = (hits + misses).max(1);
    let hit_rate = hits as f64 * 100.0 / total as f64;
    let avg_ms = (hit_ms + miss_ms) / total as f64;
    let avg_hit = if hits > 0 { hit_ms / hits as f64 } else { 0.0 };
    let avg_miss = if misses > 0 { miss_ms / misses as f64 } else { 0.0 };
    println!(
        "realistic playback @ {} fps over [{start_cs}..{end_cs}]cs: {total} frames, \
         {misses} re-render(s), hit-rate={hit_rate:.1}%",
        cfg.fps
    );
    println!(
        "  avg {avg_ms:.4} ms/frame  |  hit (present-only) {avg_hit:.4} ms  |  \
         miss (rebuild+present) {avg_miss:.4} ms"
    );
    Ok(())
}

fn run() -> Result<(), String> {
    let cfg = parse_cfg()?;
    let text = std::fs::read_to_string(&cfg.ass).map_err(|e| format!("read ass: {e}"))?;
    let script = Script::parse(&text).map_err(|e| format!("parse ass: {e:?}"))?;

    let mut software = Renderer::new(
        BackendType::Software,
        RenderContext::new(cfg.width, cfg.height),
    )
    .map_err(|e| format!("software renderer: {e}"))?;
    let mut gpu =
        GpuBackend::new(cfg.width, cfg.height).map_err(|e| format!("gpu backend: {e}"))?;

    println!(
        "GPU resident-layer playback on {} at {}x{}",
        cfg.ass, cfg.width, cfg.height
    );
    let present_ms = steady_state(&cfg, &mut software, &mut gpu, &script)?;
    playback(&cfg, &mut software, &mut gpu, &script)?;

    println!("baselines: CPU composite {CPU_COMPOSITE_MS:.2} ms/frame, GPU full+readback {GPU_READBACK_MS:.2} ms/frame");
    let best_baseline = CPU_COMPOSITE_MS.min(GPU_READBACK_MS);
    if present_ms < best_baseline {
        println!(
            "steady-state present-only ({present_ms:.4} ms) collapses below both baselines: \
             one quad blend of a cached texture, no re-rasterize/upload/readback."
        );
    } else {
        println!(
            "steady-state present-only ({present_ms:.4} ms) did NOT beat both baselines; \
             note device.poll(Wait) per frame adds fixed overhead a real swapchain present amortizes."
        );
    }
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("gpu_playback error: {e}");
        std::process::exit(1);
    }
}
