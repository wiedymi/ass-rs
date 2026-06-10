//! In-process libass A/B comparison (dev-only; needs the `libass-compare`
//! feature and a native libass via vcpkg/pkg-config).
//!
//! Renders an `.ass` with the software backend and with libass directly,
//! composites both over black, and prints a concise pixel-diff report plus
//! libass's per-line bitmap geometry — so spacing/placement gaps are read off
//! libass's own output (the `ASS_Image` rectangles) instead of guessed.
//!
//! Usage:
//! ```text
//! cargo run --features full,libass-compare --example libass_ffi_compare -- \
//!     --ass FILE --size 1280x720 --time 200 [--family Arial] [--out DIR] [--tol 16]
//! ```
//! `--time` is in centiseconds. Uses system fonts on both sides by default so
//! results line up with the ffmpeg harness; pass `--fonts-dir` for a pinned set.

use ass_core::parser::Script;
use ass_renderer::backends::BackendType;
use ass_renderer::debug::libass::{Libass, LibassRect};
use ass_renderer::renderer::{RenderContext, Renderer};
use image::RgbImage;
use std::path::PathBuf;

struct Config {
    ass: PathBuf,
    width: u32,
    height: u32,
    time_cs: u32,
    family: String,
    fonts_dir: Option<String>,
    out: PathBuf,
    tol: u8,
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
    let (mut width, mut height, mut time_cs, mut tol) = (1280u32, 720u32, 0u32, 16u8);
    let mut family = String::from("Arial");
    let mut fonts_dir = None;
    let mut out = PathBuf::from("target/libass-ffi");
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
            "--time" => time_cs = next_val(&argv, &mut i)?.parse().map_err(|_| "bad --time")?,
            "--family" => family = next_val(&argv, &mut i)?,
            "--fonts-dir" => fonts_dir = Some(next_val(&argv, &mut i)?),
            "--out" => out = PathBuf::from(next_val(&argv, &mut i)?),
            "--tol" => tol = next_val(&argv, &mut i)?.parse().map_err(|_| "bad --tol")?,
            other => return Err(format!("unknown arg {other}")),
        }
        i += 1;
    }
    let ass = ass.ok_or_else(|| "--ass is required".to_string())?;
    Ok(Config {
        ass,
        width,
        height,
        time_cs,
        family,
        fonts_dir,
        out,
        tol,
    })
}

fn composite_over_black(rgba: &[u8], pixels: usize) -> Vec<u8> {
    let mut out = vec![0u8; pixels * 3];
    for i in 0..pixels {
        let a = u32::from(rgba[i * 4 + 3]);
        for c in 0..3 {
            out[i * 3 + c] = ((u32::from(rgba[i * 4 + c]) * a) / 255) as u8;
        }
    }
    out
}

fn render_ours(cfg: &Config, script: &Script) -> Result<Vec<u8>, String> {
    let ctx = RenderContext::new(cfg.width, cfg.height);
    let mut renderer =
        Renderer::new(BackendType::Software, ctx).map_err(|e| format!("renderer: {e}"))?;
    let frame = renderer
        .render_frame(script, cfg.time_cs)
        .map_err(|e| format!("render: {e}"))?;
    Ok(composite_over_black(
        frame.data(),
        (cfg.width * cfg.height) as usize,
    ))
}

fn render_libass(cfg: &Config, ass_text: &str) -> Result<(Vec<u8>, Vec<LibassRect>), String> {
    let lib = Libass::new(cfg.width, cfg.height).map_err(|e| format!("libass init: {e}"))?;
    let use_system = cfg.fonts_dir.is_none();
    lib.set_fonts(cfg.fonts_dir.as_deref(), &cfg.family, use_system)
        .map_err(|e| format!("libass fonts: {e}"))?;
    let frame = lib
        .render(ass_text, i64::from(cfg.time_cs) * 10)
        .map_err(|e| format!("libass render: {e}"))?;
    let rgb = composite_over_black(&frame.rgba, (cfg.width * cfg.height) as usize);
    Ok((rgb, frame.rects))
}

struct PixelDiff {
    pct: f64,
    mae: f64,
    maxe: u8,
    bbox: Option<(u32, u32, u32, u32)>,
}

/// Pixel diff of two packed-RGB buffers.
fn pixel_diff(a: &[u8], b: &[u8], w: u32, h: u32, tol: u8) -> PixelDiff {
    let total = (w * h) as usize;
    let (mut over, mut sum, mut maxe) = (0usize, 0u64, 0u8);
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
    for idx in 0..total {
        let mut worst = 0u8;
        for c in 0..3 {
            let d = a[idx * 3 + c].abs_diff(b[idx * 3 + c]);
            sum += u64::from(d);
            worst = worst.max(d);
        }
        maxe = maxe.max(worst);
        if worst > tol {
            over += 1;
            let (x, y) = (idx as u32 % w, idx as u32 / w);
            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x);
            y1 = y1.max(y);
        }
    }
    PixelDiff {
        pct: over as f64 * 100.0 / total as f64,
        mae: sum as f64 / (total * 3) as f64,
        maxe,
        bbox: (over > 0).then_some((x0, y0, x1, y1)),
    }
}

/// Print line bands with each band's height, inter-line spacing, and (when a
/// reference is given) the center delta vs that reference (+ = lower).
fn print_bands(label: &str, bands: &[(u32, u32)], reference: Option<&[(u32, u32)]>) {
    for (n, &(top, bottom)) in bands.iter().enumerate() {
        let center = (top + bottom) / 2;
        let mut extra = String::new();
        if n > 0 {
            let prev = (bands[n - 1].0 + bands[n - 1].1) / 2;
            extra.push_str(&format!("  spacing={}", center - prev));
        }
        if let Some(&(rt, rb)) = reference.and_then(|r| r.get(n)) {
            extra.push_str(&format!(
                "  vs_libass={}",
                i64::from(center) - i64::from((rt + rb) / 2)
            ));
        }
        println!(
            "  {label} line {n}: y[{top}..{bottom}] h={} center={center}{extra}",
            bottom - top
        );
    }
}

/// Detect vertical coverage bands (text lines) in a packed-RGB frame by finding
/// runs of rows that contain non-black pixels. Returns (top, bottom) per band.
fn frame_line_bands(rgb: &[u8], w: u32, h: u32) -> Vec<(u32, u32)> {
    let mut bands: Vec<(u32, u32)> = Vec::new();
    let mut run_start: Option<u32> = None;
    for y in 0..h {
        let mut lit = 0u32;
        for x in 0..w {
            let idx = ((y * w + x) * 3) as usize;
            if rgb[idx] > 16 || rgb[idx + 1] > 16 || rgb[idx + 2] > 16 {
                lit += 1;
            }
        }
        let on = lit >= 3;
        match (on, run_start) {
            (true, None) => run_start = Some(y),
            (false, Some(start)) => {
                bands.push((start, y - 1));
                run_start = None;
            }
            _ => {}
        }
    }
    if let Some(start) = run_start {
        bands.push((start, h - 1));
    }
    bands
}

fn run() -> Result<(), String> {
    let cfg = parse_config()?;
    std::fs::create_dir_all(&cfg.out).map_err(|e| format!("create out dir: {e}"))?;
    let text = std::fs::read_to_string(&cfg.ass).map_err(|e| format!("read ass: {e}"))?;
    let script = Script::parse(&text).map_err(|e| format!("parse ass: {e:?}"))?;

    let ours = render_ours(&cfg, &script)?;
    let (libass, rects) = render_libass(&cfg, &text)?;
    let diff = pixel_diff(&ours, &libass, cfg.width, cfg.height, cfg.tol);
    // Measure both sides with the SAME method (thresholded ink on the composited
    // frame) so the geometry comparison is apples-to-apples. The raw ASS_Image
    // rects below include faint AA fringe and are reported only as context.
    let ours_bands = frame_line_bands(&ours, cfg.width, cfg.height);
    let libass_bands = frame_line_bands(&libass, cfg.width, cfg.height);

    RgbImage::from_raw(cfg.width, cfg.height, ours)
        .ok_or_else(|| "build ours image".to_string())?
        .save(cfg.out.join("ours.png"))
        .map_err(|e| format!("save ours: {e}"))?;
    RgbImage::from_raw(cfg.width, cfg.height, libass)
        .ok_or_else(|| "build libass image".to_string())?
        .save(cfg.out.join("libass.png"))
        .map_err(|e| format!("save libass: {e}"))?;

    println!(
        "ass: {}  t={}cs  {}x{}  tol={}",
        cfg.ass.display(),
        cfg.time_cs,
        cfg.width,
        cfg.height,
        cfg.tol
    );
    println!(
        "diff: {:.3}% px>tol  MAE={:.2}  MAXE={}",
        diff.pct, diff.mae, diff.maxe
    );
    match diff.bbox {
        Some((x0, y0, x1, y1)) => println!("region: bbox x[{x0}..{x1}] y[{y0}..{y1}]"),
        None => println!("region: (identical within tol)"),
    }
    print_bands("libass", &libass_bands, None);
    print_bands("ours  ", &ours_bands, Some(&libass_bands));
    println!("libass raw bitmaps: {}", rects.len());
    println!("wrote: {}/{{ours,libass}}.png", cfg.out.display());
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("libass_ffi_compare error: {e}");
        std::process::exit(1);
    }
}
