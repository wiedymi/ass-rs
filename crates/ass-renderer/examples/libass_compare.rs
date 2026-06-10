//! libass comparison harness.
//!
//! Renders an `.ass` file with the software backend and, separately, with
//! ffmpeg's bundled libass, composites both over opaque black, and prints a
//! concise diff report aimed at locating rendering discrepancies. Also writes
//! `ours.png`, `libass.png` and `diff.png` (a red heat map of over-tolerance
//! pixels) for visual root-causing.
//!
//! The report is intentionally terse: a percentage of pixels exceeding a
//! per-channel tolerance, mean/max absolute error, and the bounding box and
//! centroid of the differing region so a discrepancy can be traced to a spot.
//!
//! Usage:
//! ```text
//! cargo run --example libass_compare -- \
//!     --ass FILE --size 1280x720 --time 200 [--ffmpeg ffmpeg] [--out DIR] [--tol 16]
//! ```
//! `--time` is in centiseconds and must fall inside a dialogue's Start..End.
//! `--size` should equal the script's `PlayResX`/`PlayResY` for a 1:1 compare.
//! Both sides resolve fonts from the system set, so pin the same font on both.

use ass_core::parser::Script;
use ass_renderer::backends::BackendType;
use ass_renderer::renderer::{RenderContext, Renderer};
use image::{Rgb, RgbImage};
use std::path::{Path, PathBuf};
use std::process::Command;

struct Config {
    ass: PathBuf,
    width: u32,
    height: u32,
    time_cs: u32,
    ffmpeg: String,
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
    let mut ffmpeg = String::from("ffmpeg");
    let mut out = PathBuf::from("target/libass-compare");
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
            "--ffmpeg" => ffmpeg = next_val(&argv, &mut i)?,
            "--out" => out = PathBuf::from(next_val(&argv, &mut i)?),
            "--tol" => tol = next_val(&argv, &mut i)?.parse().map_err(|_| "bad --tol")?,
            other => return Err(format!("unknown arg {other}")),
        }
        i += 1;
    }
    let ass = ass.ok_or_else(|| "--ass is required".to_string())?;
    Ok(Config { ass, width, height, time_cs, ffmpeg, out, tol })
}

/// Alpha-composite an RGBA buffer over opaque black, yielding packed RGB.
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
    Ok(composite_over_black(frame.data(), (cfg.width * cfg.height) as usize))
}

/// Invoke ffmpeg's libass `ass` filter to rasterize the script over black at the
/// requested time, then load the resulting PNG as packed RGB.
fn render_libass(cfg: &Config, ref_png: &Path) -> Result<Vec<u8>, String> {
    let abs = cfg.ass.canonicalize().map_err(|e| format!("ass path: {e}"))?;
    let mut path = abs.to_string_lossy().replace('\\', "/");
    if let Some(stripped) = path.strip_prefix("//?/") {
        path = stripped.to_string();
    }
    let filter = format!("ass='{}'", path.replace(':', "\\:"));
    let dur = cfg.time_cs / 100 + 2;
    let seek = format!(
        "{:02}:{:02}:{:02}.{:02}",
        cfg.time_cs / 360_000,
        (cfg.time_cs / 6_000) % 60,
        (cfg.time_cs / 100) % 60,
        cfg.time_cs % 100,
    );
    let source = format!("color=c=black:s={}x{}:r=25:d={dur}", cfg.width, cfg.height);
    let status = Command::new(&cfg.ffmpeg)
        .args(["-hide_banner", "-loglevel", "error", "-f", "lavfi", "-i"])
        .arg(source)
        .args(["-vf"])
        .arg(filter)
        .args(["-ss"])
        .arg(seek)
        .args(["-frames:v", "1", "-y"])
        .arg(ref_png)
        .status()
        .map_err(|e| format!("spawn ffmpeg ({}): {e}", cfg.ffmpeg))?;
    if !status.success() {
        return Err(format!("ffmpeg exited with {status}"));
    }
    let img = image::open(ref_png)
        .map_err(|e| format!("open libass png: {e}"))?
        .to_rgb8();
    if img.width() != cfg.width || img.height() != cfg.height {
        return Err(format!(
            "libass size {}x{} != requested {}x{}",
            img.width(),
            img.height(),
            cfg.width,
            cfg.height
        ));
    }
    Ok(img.into_raw())
}

struct Report {
    total: usize,
    diff_px: usize,
    mae: f64,
    maxe: u8,
    bbox: Option<(u32, u32, u32, u32)>,
    centroid: Option<(u32, u32)>,
    ours_cov: usize,
    libass_cov: usize,
}

/// Compare two packed-RGB buffers, returning a concise report and a red heat map
/// marking pixels whose worst channel exceeds `tol`.
fn diff(ours: &[u8], libass: &[u8], width: u32, height: u32, tol: u8) -> (Report, RgbImage) {
    let mut heat = RgbImage::new(width, height);
    let total = (width * height) as usize;
    let (mut diff_px, mut sum_abs, mut maxe) = (0usize, 0u64, 0u8);
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (u32::MAX, u32::MAX, 0u32, 0u32);
    let (mut sum_x, mut sum_y) = (0u64, 0u64);
    let (mut ours_cov, mut libass_cov) = (0usize, 0usize);
    for idx in 0..total {
        let (mut worst, mut o_on, mut l_on) = (0u8, false, false);
        for c in 0..3 {
            let o = ours[idx * 3 + c];
            let l = libass[idx * 3 + c];
            let ad = o.abs_diff(l);
            sum_abs += u64::from(ad);
            worst = worst.max(ad);
            o_on |= o > tol;
            l_on |= l > tol;
        }
        ours_cov += usize::from(o_on);
        libass_cov += usize::from(l_on);
        maxe = maxe.max(worst);
        if worst > tol {
            let (x, y) = (idx as u32 % width, idx as u32 / width);
            diff_px += 1;
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);
            sum_x += u64::from(x);
            sum_y += u64::from(y);
            heat.put_pixel(x, y, Rgb([(u32::from(worst) * 4).min(255) as u8, 0, 0]));
        }
    }
    let mae = sum_abs as f64 / (total * 3) as f64;
    let (bbox, centroid) = if diff_px > 0 {
        let n = diff_px as u64;
        (
            Some((min_x, min_y, max_x, max_y)),
            Some(((sum_x / n) as u32, (sum_y / n) as u32)),
        )
    } else {
        (None, None)
    };
    let report = Report { total, diff_px, mae, maxe, bbox, centroid, ours_cov, libass_cov };
    (report, heat)
}

fn print_report(cfg: &Config, r: &Report) {
    let pct = r.diff_px as f64 * 100.0 / r.total as f64;
    println!(
        "ass: {}  t={}cs  {}x{}  tol={}",
        cfg.ass.display(),
        cfg.time_cs,
        cfg.width,
        cfg.height,
        cfg.tol
    );
    println!(
        "diff: {pct:.3}% px>tol ({}/{})  MAE={:.2}  MAXE={}",
        r.diff_px, r.total, r.mae, r.maxe
    );
    match (r.bbox, r.centroid) {
        (Some((x0, y0, x1, y1)), Some((cx, cy))) => {
            println!("region: bbox x[{x0}..{x1}] y[{y0}..{y1}]  centroid=({cx},{cy})");
        }
        _ => println!("region: (no pixels over tol)"),
    }
    let ratio = if r.libass_cov > 0 {
        r.ours_cov as f64 / r.libass_cov as f64
    } else {
        0.0
    };
    println!(
        "coverage: ours={} libass={} ratio={ratio:.2}",
        r.ours_cov, r.libass_cov
    );
    println!("wrote: {}/{{ours,libass,diff}}.png", cfg.out.display());
    let verdict = if pct < 1.0 {
        "PASS (<1% over tol)"
    } else if pct < 5.0 {
        "WARN (1-5% over tol)"
    } else {
        "FAIL (>5% over tol)"
    };
    println!("verdict: {verdict}");
}

fn run() -> Result<(), String> {
    let cfg = parse_config()?;
    std::fs::create_dir_all(&cfg.out).map_err(|e| format!("create out dir: {e}"))?;
    let text = std::fs::read_to_string(&cfg.ass).map_err(|e| format!("read ass: {e}"))?;
    let script = Script::parse(&text).map_err(|e| format!("parse ass: {e:?}"))?;
    let ours = render_ours(&cfg, &script)?;
    let ref_png = cfg.out.join("libass.png");
    let libass = render_libass(&cfg, &ref_png)?;
    let (report, heat) = diff(&ours, &libass, cfg.width, cfg.height, cfg.tol);
    RgbImage::from_raw(cfg.width, cfg.height, ours)
        .ok_or_else(|| "build ours image".to_string())?
        .save(cfg.out.join("ours.png"))
        .map_err(|e| format!("save ours: {e}"))?;
    heat.save(cfg.out.join("diff.png"))
        .map_err(|e| format!("save diff: {e}"))?;
    print_report(&cfg, &report);
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("libass_compare error: {e}");
        std::process::exit(1);
    }
}
