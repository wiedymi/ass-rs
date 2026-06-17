//! Render an `.ass` file to a raw RGB24 frame stream on stdout using libass (via
//! the in-process FFI oracle), for piping into ffmpeg to produce a reference
//! video. The libass counterpart of `render_video`, so the two can be encoded
//! with identical settings and compared side by side. Subtitles are composited
//! over the same background.
//!
//! Usage:
//! ```text
//! render_video_libass --ass FILE --size 1280x720 --fps 24 --start 0 --end 143410 \
//!     | ffmpeg -f rawvideo -pixel_format rgb24 -video_size 1280x720 \
//!         -framerate 24 -i - -c:v libx264 -crf 20 -pix_fmt yuv420p out.mp4
//! ```
//! `--start`/`--end` are in centiseconds. libass receives `time_cs * 10` as its
//! millisecond clock, matching the FFI oracle (`libass_ffi_compare`). Progress is
//! logged to stderr.

use ass_renderer::debug::libass::Libass;
use std::io::Write;

/// Build a `width*height*3` RGB background buffer. Modes: `black`, `white`,
/// `gray`, `checker` (a two-tone grid, the canonical subtitle-preview backdrop),
/// or a solid `#RRGGBB` colour. Mirrors `render_video`'s background so the two
/// streams share an identical backdrop.
fn build_background(mode: &str, width: u32, height: u32) -> Result<Vec<u8>, String> {
    let pixels = (width * height) as usize;
    let mut bg = vec![0u8; pixels * 3];
    let mut fill = |c: [u8; 3]| {
        for px in bg.chunks_exact_mut(3) {
            px.copy_from_slice(&c);
        }
    };
    match mode {
        "black" => {}
        "white" => fill([255, 255, 255]),
        "gray" => fill([96, 96, 96]),
        "checker" => {
            let sq = 24u32;
            for y in 0..height {
                for x in 0..width {
                    let c = if (x / sq + y / sq).is_multiple_of(2) {
                        56
                    } else {
                        96
                    };
                    let i = (y * width + x) as usize * 3;
                    bg[i] = c;
                    bg[i + 1] = c;
                    bg[i + 2] = c;
                }
            }
        }
        hex if hex.starts_with('#') && hex.len() == 7 => {
            let p =
                |a, b| u8::from_str_radix(&hex[a..b], 16).map_err(|_| format!("bad --bg {hex}"));
            fill([p(1, 3)?, p(3, 5)?, p(5, 7)?]);
        }
        other => {
            return Err(format!(
                "unknown --bg '{other}' (black|white|gray|checker|#RRGGBB)"
            ))
        }
    }
    Ok(bg)
}

fn main() {
    if let Err(e) = run() {
        eprintln!("render_video_libass error: {e}");
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
    let mut bg_mode = String::from("checker");
    let mut family = String::from("Arial");
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
            "--bg" => bg_mode = next(&mut i)?,
            "--family" => family = next(&mut i)?,
            other => return Err(format!("unknown arg {other}")),
        }
        i += 1;
    }
    let ass = ass.ok_or_else(|| "--ass is required".to_string())?;
    if end_cs <= start_cs {
        return Err("--end must be greater than --start".into());
    }

    let text = std::fs::read_to_string(&ass).map_err(|e| format!("read ass: {e}"))?;

    // Match the FFI oracle's libass config: frame == storage size, system fonts,
    // and the default family applied when a style's font is unavailable.
    let lib = Libass::new(width, height).map_err(|e| format!("libass init: {e}"))?;
    lib.set_fonts(None, &family, true)
        .map_err(|e| format!("libass set_fonts: {e}"))?;
    let track = lib
        .read_track(&text)
        .map_err(|e| format!("libass read_track: {e}"))?;

    let span_cs = f64::from(end_cs - start_cs);
    let n_frames = (span_cs / 100.0 * fps).ceil() as u64;
    let pixels = (width * height) as usize;
    let mut rgb = vec![0u8; pixels * 3];
    let bg = build_background(&bg_mode, width, height)?;

    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::with_capacity(pixels * 3, stdout.lock());

    eprintln!("rendering {n_frames} frames at {width}x{height} {fps} fps on '{bg_mode}' (libass)");
    for f in 0..n_frames {
        let time_cs = start_cs + (f as f64 * 100.0 / fps).round() as u32;
        // The oracle feeds libass `time_cs * 10` as its ms clock (the ASS cs->ms
        // quirk); keep the same mapping so frames line up with our renderer.
        let frame = lib.render_track(&track, i64::from(time_cs) * 10);
        let data = &frame.rgba;
        // Composite straight-alpha RGBA over the background -> RGB24.
        for p in 0..pixels {
            let a = u32::from(data[p * 4 + 3]);
            let ia = 255 - a;
            for c in 0..3 {
                rgb[p * 3 + c] =
                    ((u32::from(data[p * 4 + c]) * a + u32::from(bg[p * 3 + c]) * ia) / 255) as u8;
            }
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
