//! Sweep an `.ass` over a time range and rank frames by how far our software
//! render diverges from libass (dev-only; needs `libass-compare`). Parses the
//! script and initializes libass once, then for each sampled time composites both
//! over black and reports a pixel diff plus a per-side text-line-band count — so
//! "we draw extra/overlapping lines" (band-count mismatch) is separated from
//! "we draw the same lines slightly off" (high diff, equal bands).
//!
//! Usage:
//! ```text
//! cargo run --features full,libass-compare --example parity_sweep -- \
//!     --ass FILE --size 1280x720 --start 0 --end 143410 --step 200 \
//!     [--family Arial] [--worst 25] [--dump 8] [--out DIR]
//! ```
//! Times are centiseconds.

use ass_core::parser::Script;
use ass_renderer::backends::BackendType;
use ass_renderer::debug::libass::Libass;
use ass_renderer::renderer::{RenderContext, Renderer};
use image::RgbImage;
use std::path::PathBuf;

struct Cfg {
    ass: PathBuf,
    width: u32,
    height: u32,
    start: u32,
    end: u32,
    step: u32,
    family: String,
    worst: usize,
    dump: usize,
    out: PathBuf,
}

fn parse_cfg() -> Result<Cfg, String> {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut c = Cfg {
        ass: PathBuf::new(),
        width: 1280,
        height: 720,
        start: 0,
        end: 0,
        step: 200,
        family: "Arial".into(),
        worst: 25,
        dump: 8,
        out: PathBuf::from("target/parity-sweep"),
    };
    let mut have_ass = false;
    let mut i = 0;
    let val = |argv: &[String], i: &mut usize| -> Result<String, String> {
        *i += 1;
        argv.get(*i)
            .cloned()
            .ok_or_else(|| format!("missing value for {}", argv[*i - 1]))
    };
    while i < argv.len() {
        match argv[i].as_str() {
            "--ass" => {
                c.ass = PathBuf::from(val(&argv, &mut i)?);
                have_ass = true;
            }
            "--size" => {
                let v = val(&argv, &mut i)?;
                let (w, h) = v.split_once('x').ok_or("bad --size")?;
                c.width = w.parse().map_err(|_| "bad width")?;
                c.height = h.parse().map_err(|_| "bad height")?;
            }
            "--start" => c.start = val(&argv, &mut i)?.parse().map_err(|_| "bad --start")?,
            "--end" => c.end = val(&argv, &mut i)?.parse().map_err(|_| "bad --end")?,
            "--step" => c.step = val(&argv, &mut i)?.parse().map_err(|_| "bad --step")?,
            "--family" => c.family = val(&argv, &mut i)?,
            "--worst" => c.worst = val(&argv, &mut i)?.parse().map_err(|_| "bad --worst")?,
            "--dump" => c.dump = val(&argv, &mut i)?.parse().map_err(|_| "bad --dump")?,
            "--out" => c.out = PathBuf::from(val(&argv, &mut i)?),
            other => return Err(format!("unknown arg {other}")),
        }
        i += 1;
    }
    if !have_ass {
        return Err("--ass is required".into());
    }
    if c.end <= c.start {
        return Err("--end must be > --start".into());
    }
    Ok(c)
}

fn over_black(rgba: &[u8], px: usize) -> Vec<u8> {
    let mut out = vec![0u8; px * 3];
    for i in 0..px {
        let a = u32::from(rgba[i * 4 + 3]);
        for c in 0..3 {
            out[i * 3 + c] = (u32::from(rgba[i * 4 + c]) * a / 255) as u8;
        }
    }
    out
}

/// Fraction of pixels (%) whose worst channel differs by more than `tol`, and the
/// total summed ink on each side (to flag over/under-draw).
fn diff_stats(a: &[u8], b: &[u8], tol: u8) -> (f64, u64, u64) {
    let total = a.len() / 3;
    let mut over = 0usize;
    let (mut ia, mut ib) = (0u64, 0u64);
    for i in 0..total {
        let mut worst = 0u8;
        for c in 0..3 {
            let (x, y) = (a[i * 3 + c], b[i * 3 + c]);
            worst = worst.max(x.abs_diff(y));
            ia += u64::from(x);
            ib += u64::from(y);
        }
        if worst > tol {
            over += 1;
        }
    }
    (over as f64 * 100.0 / total as f64, ia, ib)
}

/// Count vertical text bands (runs of rows with >=3 lit pixels) — a proxy for the
/// number of on-screen text lines, so a mismatch flags extra/overlapping lines.
fn band_count(rgb: &[u8], w: u32, h: u32) -> u32 {
    let mut bands = 0u32;
    let mut in_run = false;
    for y in 0..h {
        let mut lit = 0u32;
        for x in 0..w {
            let i = ((y * w + x) * 3) as usize;
            if rgb[i] > 16 || rgb[i + 1] > 16 || rgb[i + 2] > 16 {
                lit += 1;
            }
        }
        let on = lit >= 3;
        if on && !in_run {
            bands += 1;
        }
        in_run = on;
    }
    bands
}

struct FrameStat {
    t: u32,
    diff: f64,
    ink_ratio: f64,
    ours_bands: u32,
    libass_bands: u32,
}

fn run() -> Result<(), String> {
    let cfg = parse_cfg()?;
    std::fs::create_dir_all(&cfg.out).map_err(|e| format!("mkdir: {e}"))?;
    let text = std::fs::read_to_string(&cfg.ass).map_err(|e| format!("read: {e}"))?;
    let script = Script::parse(&text).map_err(|e| format!("parse: {e:?}"))?;

    let ctx = RenderContext::new(cfg.width, cfg.height);
    let mut renderer =
        Renderer::new(BackendType::Software, ctx).map_err(|e| format!("renderer: {e}"))?;

    let lib = Libass::new(cfg.width, cfg.height).map_err(|e| format!("libass: {e}"))?;
    lib.set_fonts(None, &cfg.family, true)
        .map_err(|e| format!("fonts: {e}"))?;
    let track = lib.read_track(&text).map_err(|e| format!("track: {e}"))?;

    let px = (cfg.width * cfg.height) as usize;
    let mut stats: Vec<FrameStat> = Vec::new();

    let mut t = cfg.start;
    let mut n = 0u32;
    while t <= cfg.end {
        let ours_frame = renderer
            .render_frame(&script, t)
            .map_err(|e| format!("ours @ {t}: {e}"))?;
        let ours = over_black(ours_frame.data(), px);
        let lib_frame = lib.render_track(&track, i64::from(t) * 10);
        let libass = over_black(&lib_frame.rgba, px);

        let (diff, ia, ib) = diff_stats(&ours, &libass, 16);
        let ink_ratio = if ib > 0 {
            ia as f64 / ib as f64
        } else {
            f64::from(ia > 0)
        };
        stats.push(FrameStat {
            t,
            diff,
            ink_ratio,
            ours_bands: band_count(&ours, cfg.width, cfg.height),
            libass_bands: band_count(&libass, cfg.width, cfg.height),
        });

        n += 1;
        if n.is_multiple_of(200) {
            eprintln!("  swept {n} frames (t={t}cs)");
        }
        t += cfg.step;
    }

    // Aggregate.
    let count = stats.len();
    let mean_diff = stats.iter().map(|s| s.diff).sum::<f64>() / count as f64;
    let band_mismatch: Vec<&FrameStat> = stats
        .iter()
        .filter(|s| s.ours_bands != s.libass_bands)
        .collect();
    let extra_lines = band_mismatch
        .iter()
        .filter(|s| s.ours_bands > s.libass_bands)
        .count();
    let missing_lines = band_mismatch
        .iter()
        .filter(|s| s.ours_bands < s.libass_bands)
        .count();

    println!(
        "swept {count} frames [{}..{}] step {}cs",
        cfg.start, cfg.end, cfg.step
    );
    println!("mean diff: {mean_diff:.3}%   band-count mismatches: {} ({extra_lines} extra-line, {missing_lines} missing-line)", band_mismatch.len());

    println!("\n== band-count mismatch frames (extra/overlapping or missing lines) ==");
    let mut bm = band_mismatch.clone();
    bm.sort_by(|a, b| {
        (b.ours_bands as i64 - b.libass_bands as i64)
            .abs()
            .cmp(&(a.ours_bands as i64 - a.libass_bands as i64).abs())
    });
    for s in bm.iter().take(cfg.worst) {
        println!(
            "  t={:>7}cs  ours_bands={} libass_bands={}  diff={:.2}%  ink={:.2}x",
            s.t, s.ours_bands, s.libass_bands, s.diff, s.ink_ratio
        );
    }

    println!("\n== worst {} frames by pixel diff ==", cfg.worst);
    let mut by_diff: Vec<&FrameStat> = stats.iter().collect();
    by_diff.sort_by(|a, b| b.diff.partial_cmp(&a.diff).unwrap());
    for s in by_diff.iter().take(cfg.worst) {
        println!(
            "  t={:>7}cs  diff={:.2}%  ink={:.2}x  bands {}/{}",
            s.t, s.diff, s.ink_ratio, s.ours_bands, s.libass_bands
        );
    }

    // Dump PNGs for the worst-by-diff frames for visual inspection.
    println!(
        "\n== dumping {} worst frames to {} ==",
        cfg.dump,
        cfg.out.display()
    );
    for s in by_diff.iter().take(cfg.dump) {
        let of = renderer
            .render_frame(&script, s.t)
            .map_err(|e| e.to_string())?;
        let ours = over_black(of.data(), px);
        let lf = lib.render_track(&track, i64::from(s.t) * 10);
        let libass = over_black(&lf.rgba, px);
        let save = |name: &str, buf: Vec<u8>| -> Result<(), String> {
            RgbImage::from_raw(cfg.width, cfg.height, buf)
                .ok_or("img")?
                .save(cfg.out.join(name))
                .map_err(|e| e.to_string())
        };
        save(&format!("t{}_ours.png", s.t), ours)?;
        save(&format!("t{}_libass.png", s.t), libass)?;
        println!(
            "  t={}cs (diff {:.2}%, bands {}/{})",
            s.t, s.diff, s.ours_bands, s.libass_bands
        );
    }
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("parity_sweep error: {e}");
        std::process::exit(1);
    }
}
