//! Correctness gate for the hybrid GPU compositor: render the same frame with the
//! CPU [`SoftwareBackend`] and the [`GpuBackend`], composite each over black and
//! report how far they diverge (MAE, max per-channel diff, fraction of pixels over
//! a tolerance). The GPU path reuses the software backend's cached tiles and only
//! moves compositing to wgpu, so the two outputs must agree to within a few
//! least-significant bits of blend/format rounding.
//!
//! Usage:
//! ```text
//! cargo run -p ass-renderer --no-default-features --features full,gpu \
//!     --example gpu_compare -- --ass FILE --size 1280x720 --times 100,12500,90000
//! ```
//! Times are centiseconds. Defaults target `benches/benchmark.ass` at 1280x720.
//! Passing `--bench N` additionally times the GPU and software compositors over
//! `N` iterations per timestamp and prints milliseconds per frame for each.

use std::time::Instant;

use ass_core::parser::Script;
use ass_renderer::backends::coverage::{composite_bitmap, RenderBitmap};
use ass_renderer::backends::gpu::GpuBackend;
use ass_renderer::backends::BackendType;
use ass_renderer::renderer::{RenderContext, Renderer};

/// Parsed command-line configuration.
struct Cfg {
    ass: String,
    width: u32,
    height: u32,
    times: Vec<u32>,
    tol: u8,
    bench: u32,
}

fn parse_cfg() -> Result<Cfg, String> {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut cfg = Cfg {
        ass: concat!(env!("CARGO_MANIFEST_DIR"), "/benches/benchmark.ass").to_string(),
        width: 1280,
        height: 720,
        times: vec![100, 12500, 90000, 133090],
        tol: 4,
        bench: 0,
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
            "--times" => {
                cfg.times = val(&argv, &mut i)?
                    .split(',')
                    .map(|s| s.trim().parse().map_err(|_| format!("bad time {s}")))
                    .collect::<Result<Vec<u32>, String>>()?;
            }
            "--tol" => cfg.tol = val(&argv, &mut i)?.parse().map_err(|_| "bad --tol")?,
            "--bench" => cfg.bench = val(&argv, &mut i)?.parse().map_err(|_| "bad --bench")?,
            other => return Err(format!("unknown arg {other}")),
        }
        i += 1;
    }
    Ok(cfg)
}

/// Composite a straight-alpha RGBA frame over an opaque black background, yielding
/// a `px * 3` RGB buffer (the same convention the other parity tools use).
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

/// Per-channel agreement between two equal-length byte buffers: mean-absolute
/// error, maximum absolute difference, and the fraction of pixels (%) whose worst
/// channel differs by more than `tol`.
fn diff_stats(a: &[u8], b: &[u8], tol: u8) -> (f64, u8, f64) {
    let mut sum = 0u64;
    let mut max = 0u8;
    let mut over = 0usize;
    for (ca, cb) in a.chunks_exact(3).zip(b.chunks_exact(3)) {
        let mut worst = 0u8;
        for c in 0..3 {
            let d = ca[c].abs_diff(cb[c]);
            sum += u64::from(d);
            worst = worst.max(d);
        }
        max = max.max(worst);
        if worst > tol {
            over += 1;
        }
    }
    let count = (a.len() / 3).max(1);
    (sum as f64 / a.len() as f64, max, over as f64 * 100.0 / count as f64)
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
    let mut gpu = Renderer::new(BackendType::Gpu, RenderContext::new(cfg.width, cfg.height))
        .map_err(|e| format!("gpu renderer: {e}"))?;

    let px = (cfg.width * cfg.height) as usize;
    println!(
        "comparing GPU vs software on {} at {}x{} (tol={})",
        cfg.ass, cfg.width, cfg.height, cfg.tol
    );

    let mut worst_mae = 0.0_f64;
    let mut worst_max = 0u8;
    for &t in &cfg.times {
        let sw_frame = software
            .render_frame(&script, t)
            .map_err(|e| format!("software @ {t}: {e}"))?;
        let gpu_frame = gpu
            .render_frame(&script, t)
            .map_err(|e| format!("gpu @ {t}: {e}"))?;

        // Raw straight-RGBA agreement across all four channels.
        let (raw_mae, raw_max) = {
            let (sw, gp) = (sw_frame.data(), gpu_frame.data());
            let mut sum = 0u64;
            let mut max = 0u8;
            for (a, b) in sw.iter().zip(gp.iter()) {
                let d = a.abs_diff(*b);
                sum += u64::from(d);
                max = max.max(d);
            }
            (sum as f64 / sw.len() as f64, max)
        };

        let sw_rgb = over_black(sw_frame.data(), px);
        let gpu_rgb = over_black(gpu_frame.data(), px);
        let (mae, max, over) = diff_stats(&sw_rgb, &gpu_rgb, cfg.tol);
        worst_mae = worst_mae.max(mae);
        worst_max = worst_max.max(max);

        println!(
            "  t={t:>7}cs  over-black: MAE={mae:.4} max={max} pix>tol={over:.4}%   \
             raw-RGBA: MAE={raw_mae:.4} max={raw_max}"
        );
    }

    println!("worst over-black MAE={worst_mae:.4}  worst max-diff={worst_max}");
    if worst_mae < 2.0 {
        println!("PASS: GPU compositor matches the software compositor (MAE < 2.0)");
    } else {
        println!("FAIL: GPU output diverges from software (MAE >= 2.0)");
    }

    if cfg.bench > 0 {
        bench(&cfg, &script, &mut software)?;
    }
    Ok(())
}

/// Time the GPU and software compositors on identical tiles. Each timestamp's
/// tile list is produced once via the (shared) software pipeline, then composited
/// `cfg.bench` times per backend. Tile production is excluded so the numbers
/// isolate the compositing step the backends own. The GPU figure includes its
/// per-frame upload and the blocking readback, so on this light workload it is
/// expected to trail the CPU — the point is honest measurement, not a speedup.
fn bench(cfg: &Cfg, script: &Script, tiles_src: &mut Renderer) -> Result<(), String> {
    let frames: Vec<Vec<RenderBitmap>> = cfg
        .times
        .iter()
        .map(|&t| {
            tiles_src
                .render_frame_bitmaps(script, t)
                .map_err(|e| format!("tiles @ {t}: {e}"))
        })
        .collect::<Result<_, _>>()?;
    let total = u64::from(cfg.bench) * frames.len() as u64;
    if total == 0 {
        return Err("nothing to bench (no timestamps)".into());
    }
    let px = (cfg.width * cfg.height) as usize;

    let mut buf = vec![0u8; px * 4];
    let composite_sw = |buf: &mut [u8], frame: &[RenderBitmap]| {
        buf.fill(0);
        for bmp in frame {
            composite_bitmap(buf, cfg.width, cfg.height, bmp);
        }
    };
    for frame in &frames {
        composite_sw(&mut buf, frame);
    }
    let sw_start = Instant::now();
    for _ in 0..cfg.bench {
        for frame in &frames {
            composite_sw(&mut buf, frame);
        }
    }
    let sw_ms = sw_start.elapsed().as_secs_f64() * 1000.0 / total as f64;

    let mut gpu = GpuBackend::new(cfg.width, cfg.height).map_err(|e| format!("gpu backend: {e}"))?;
    for frame in &frames {
        gpu.composite_bitmaps(frame, cfg.width, cfg.height)
            .map_err(|e| format!("gpu warmup: {e}"))?;
    }
    let gpu_start = Instant::now();
    for _ in 0..cfg.bench {
        for frame in &frames {
            gpu.composite_bitmaps(frame, cfg.width, cfg.height)
                .map_err(|e| format!("gpu composite: {e}"))?;
        }
    }
    let gpu_ms = gpu_start.elapsed().as_secs_f64() * 1000.0 / total as f64;

    let avg_tiles = frames.iter().map(Vec::len).sum::<usize>() as f64 / frames.len() as f64;
    println!(
        "bench composite-only ({} iters x {} frames, avg {avg_tiles:.0} tiles/frame): \
         software={sw_ms:.3} ms/frame  gpu={gpu_ms:.3} ms/frame",
        cfg.bench,
        frames.len()
    );
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("gpu_compare error: {e}");
        std::process::exit(1);
    }
}
