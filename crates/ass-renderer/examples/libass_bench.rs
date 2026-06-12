//! Performance comparison: our software backend vs libass (dev-only; needs the
//! `libass-compare` feature and a native libass).
//!
//! Renders the same `.ass` over a sweep of frame times with each engine and
//! reports per-frame timing. Our `render_frame` does the full pipeline including
//! the final RGBA composite; libass's `ass_render_frame` produces the bitmap list
//! (the player composites), so ours is timed doing slightly more work.
//!
//! Usage:
//! ```text
//! cargo run --features full,libass-compare --example libass_bench -- \
//!     --ass FILE [--size 1280x720] [--frames 300] [--duration 1000] [--family Arial]
//! ```
//! `--duration` is the time span in centiseconds the frame sweep covers.

use ass_core::parser::Script;
use ass_renderer::backends::BackendType;
use ass_renderer::debug::libass::Libass;
use ass_renderer::renderer::{RenderContext, Renderer};
use std::path::PathBuf;
use std::time::Instant;

struct Config {
    ass: PathBuf,
    width: u32,
    height: u32,
    frames: u32,
    duration_cs: u32,
    family: String,
}

fn next_val(argv: &[String], i: &mut usize) -> Result<String, String> {
    *i += 1;
    argv.get(*i)
        .cloned()
        .ok_or_else(|| format!("missing value for {}", argv[*i - 1]))
}

fn parse_config() -> Result<Config, String> {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut ass: Option<PathBuf> = None;
    let (mut width, mut height, mut frames, mut duration_cs) = (1280u32, 720u32, 300u32, 1000u32);
    let mut family = String::from("Arial");
    let mut i = 0;
    while i < argv.len() {
        match argv[i].as_str() {
            "--ass" => ass = Some(PathBuf::from(next_val(&argv, &mut i)?)),
            "--size" => {
                let v = next_val(&argv, &mut i)?;
                let (w, h) = v.split_once('x').ok_or_else(|| format!("bad --size {v}"))?;
                width = w.parse().map_err(|_| format!("bad width {w}"))?;
                height = h.parse().map_err(|_| format!("bad height {h}"))?;
            }
            "--frames" => {
                frames = next_val(&argv, &mut i)?
                    .parse()
                    .map_err(|_| "bad --frames")?
            }
            "--duration" => {
                duration_cs = next_val(&argv, &mut i)?
                    .parse()
                    .map_err(|_| "bad --duration")?;
            }
            "--family" => family = next_val(&argv, &mut i)?,
            other => return Err(format!("unknown arg {other}")),
        }
        i += 1;
    }
    let ass = ass.ok_or_else(|| "--ass is required".to_string())?;
    Ok(Config {
        ass,
        width,
        height,
        frames: frames.max(1),
        duration_cs,
        family,
    })
}

fn report(label: &str, total_ms: f64, frames: u32) {
    let per = total_ms / f64::from(frames);
    println!(
        "{label:8} {total_ms:8.1} ms total   {per:7.4} ms/frame   {:6.0} fps",
        if per > 0.0 { 1000.0 / per } else { 0.0 }
    );
}

fn run() -> Result<(), String> {
    let cfg = parse_config()?;
    let text = std::fs::read_to_string(&cfg.ass).map_err(|e| format!("read ass: {e}"))?;
    let warmup = 5u32;

    // Frame times (centiseconds) swept evenly across the duration.
    let times: Vec<u32> = (0..cfg.frames)
        .map(|i| i * cfg.duration_cs / cfg.frames)
        .collect();

    // --- our software backend (full pipeline incl. final composite) ---
    let script = Script::parse(&text).map_err(|e| format!("parse ass: {e:?}"))?;
    let ctx = RenderContext::new(cfg.width, cfg.height);
    let mut renderer =
        Renderer::new(BackendType::Software, ctx).map_err(|e| format!("renderer: {e}"))?;
    for _ in 0..warmup {
        renderer
            .render_frame(&script, 0)
            .map_err(|e| format!("warmup render: {e}"))?;
    }
    let mut sink = 0u64;
    let t = Instant::now();
    for &time_cs in &times {
        let frame = renderer
            .render_frame(&script, time_cs)
            .map_err(|e| format!("render: {e}"))?;
        sink = sink.wrapping_add(frame.data().len() as u64);
    }
    let ours_ms = t.elapsed().as_secs_f64() * 1000.0;

    // --- our bitmap-list output (libass-style: emit positioned bitmaps, no
    // full-frame clear/composite/copy — the apples-to-apples shape of libass) ---
    for _ in 0..warmup {
        renderer
            .render_frame_bitmaps(&script, 0)
            .map_err(|e| format!("warmup bitmaps: {e}"))?;
    }
    let mut sink_b = 0u64;
    let tb = Instant::now();
    for &time_cs in &times {
        let bitmaps = renderer
            .render_frame_bitmaps(&script, time_cs)
            .map_err(|e| format!("render bitmaps: {e}"))?;
        sink_b = sink_b.wrapping_add(bitmaps.len() as u64);
    }
    let ours_bitmaps_ms = tb.elapsed().as_secs_f64() * 1000.0;

    // --- libass (ass_render_frame only; player would composite) ---
    let lib = Libass::new(cfg.width, cfg.height).map_err(|e| format!("libass init: {e}"))?;
    lib.set_fonts(None, &cfg.family, true)
        .map_err(|e| format!("libass fonts: {e}"))?;
    let track = lib
        .read_track(&text)
        .map_err(|e| format!("libass track: {e}"))?;
    for _ in 0..warmup {
        lib.render_count(&track, 0);
    }
    let mut sink2 = 0usize;
    let t2 = Instant::now();
    for &time_cs in &times {
        sink2 = sink2.wrapping_add(lib.render_count(&track, i64::from(time_cs) * 10));
    }
    let libass_ms = t2.elapsed().as_secs_f64() * 1000.0;

    println!(
        "ass: {}  {}x{}  frames={}  duration={}cs",
        cfg.ass.display(),
        cfg.width,
        cfg.height,
        cfg.frames,
        cfg.duration_cs
    );
    report("ours", ours_ms, cfg.frames);
    report("ours-bmp", ours_bitmaps_ms, cfg.frames);
    report("libass", libass_ms, cfg.frames);
    println!(
        "ratio ours/libass: {:.2}x   ours-bmp/libass: {:.2}x   (checksums {sink}/{sink_b}/{sink2})",
        if libass_ms > 0.0 {
            ours_ms / libass_ms
        } else {
            0.0
        },
        if libass_ms > 0.0 {
            ours_bitmaps_ms / libass_ms
        } else {
            0.0
        }
    );
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("libass_bench error: {e}");
        std::process::exit(1);
    }
}
