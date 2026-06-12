//! Render an `.ass` file to a raw RGB24 frame stream on stdout, for piping into
//! ffmpeg to produce a preview video. Subtitles are composited over black.
//!
//! Usage:
//! ```text
//! render_video --ass FILE --size 1280x720 --fps 24 --start 0 --end 143410 \
//!     | ffmpeg -f rawvideo -pixel_format rgb24 -video_size 1280x720 \
//!         -framerate 24 -i - -c:v libx264 -crf 20 -pix_fmt yuv420p out.mp4
//! ```
//! `--start`/`--end` are in centiseconds. Progress is logged to stderr.

use ass_core::parser::Script;
use ass_renderer::backends::BackendType;
use ass_renderer::renderer::{RenderContext, Renderer};
use std::io::Write;

fn main() {
    if let Err(e) = run() {
        eprintln!("render_video error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut ass: Option<String> = None;
    let (mut width, mut height) = (1280u32, 720u32);
    let mut fps = 24.0_f64;
    let mut start_cs = 0u32;
    let mut end_cs = 0u32;
    let mut i = 0;
    while i < argv.len() {
        let next = |i: &mut usize| -> Result<String, String> {
            *i += 1;
            argv.get(*i)
                .cloned()
                .ok_or_else(|| format!("missing value for {}", argv[*i - 1]))
        };
        match argv[i].as_str() {
            "--ass" => ass = Some(next(&mut i)?),
            "--size" => {
                let v = next(&mut i)?;
                let (w, h) = v.split_once('x').ok_or_else(|| format!("bad --size {v}"))?;
                width = w.parse().map_err(|_| "bad width")?;
                height = h.parse().map_err(|_| "bad height")?;
            }
            "--fps" => fps = next(&mut i)?.parse().map_err(|_| "bad --fps")?,
            "--start" => start_cs = next(&mut i)?.parse().map_err(|_| "bad --start")?,
            "--end" => end_cs = next(&mut i)?.parse().map_err(|_| "bad --end")?,
            other => return Err(format!("unknown arg {other}")),
        }
        i += 1;
    }
    let ass = ass.ok_or_else(|| "--ass is required".to_string())?;
    if end_cs <= start_cs {
        return Err("--end must be greater than --start".into());
    }

    let text = std::fs::read_to_string(&ass).map_err(|e| format!("read ass: {e}"))?;
    let script = Script::parse(&text).map_err(|e| format!("parse ass: {e:?}"))?;
    let ctx = RenderContext::new(width, height);
    let mut renderer =
        Renderer::new(BackendType::Software, ctx).map_err(|e| format!("renderer: {e}"))?;

    let span_cs = f64::from(end_cs - start_cs);
    let n_frames = (span_cs / 100.0 * fps).ceil() as u64;
    let pixels = (width * height) as usize;
    let mut rgb = vec![0u8; pixels * 3];

    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::with_capacity(pixels * 3, stdout.lock());

    eprintln!("rendering {n_frames} frames at {width}x{height} {fps} fps over black");
    for f in 0..n_frames {
        let time_cs = start_cs + (f as f64 * 100.0 / fps).round() as u32;
        let frame = renderer
            .render_frame(&script, time_cs)
            .map_err(|e| format!("render frame {f}: {e}"))?;
        let data = frame.data();
        // Composite straight-alpha RGBA over black -> RGB24.
        for p in 0..pixels {
            let a = u32::from(data[p * 4 + 3]);
            rgb[p * 3] = (u32::from(data[p * 4]) * a / 255) as u8;
            rgb[p * 3 + 1] = (u32::from(data[p * 4 + 1]) * a / 255) as u8;
            rgb[p * 3 + 2] = (u32::from(data[p * 4 + 2]) * a / 255) as u8;
        }
        out.write_all(&rgb)
            .map_err(|e| format!("write frame {f}: {e}"))?;
        if f.is_multiple_of(500) {
            eprintln!("  frame {f}/{n_frames} (t={time_cs}cs)");
        }
    }
    out.flush().map_err(|e| format!("flush: {e}"))?;
    eprintln!("done: {n_frames} frames");
    Ok(())
}
