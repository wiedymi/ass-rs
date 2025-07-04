use std::env;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use ass_core::Script;
use ass_render::SoftwareRenderer;
use rayon::prelude::*;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 5 {
        eprintln!("Usage: {} <subs.ass> <font1.ttf[,font2.ttf,...]> <out_dir> <width>x<height> [fps] [duration_sec]", args[0]);
        std::process::exit(1);
    }
    let subs_path = &args[1];
    let font_arg = &args[2];
    let out_dir = &args[3];
    let res = &args[4];
    let fps: f32 = if args.len() > 5 {
        args[5].parse().unwrap_or(30.0)
    } else {
        30.0
    };
    let duration: f32 = if args.len() > 6 {
        args[6].parse().unwrap_or(10.0)
    } else {
        10.0
    };

    let (width, height) = parse_resolution(res)?;

    fs::create_dir_all(out_dir)?;

    let ass_bytes = fs::read(subs_path)?;

    // Allow comma-separated list of fonts or a path to a directory containing ttf/otf files.
    let font_datas: Vec<Vec<u8>> = if font_arg.contains(',') {
        font_arg
            .split(',')
            .map(|p| fs::read(p.trim()))
            .collect::<Result<_, _>>()?
    } else {
        let path = Path::new(font_arg);
        if path.is_dir() {
            fs::read_dir(path)?
                .filter_map(|e| e.ok())
                .filter(|e| {
                    if let Some(ext) = e.path().extension() {
                        ext == "ttf" || ext == "otf"
                    } else {
                        false
                    }
                })
                .map(|e| fs::read(e.path()))
                .collect::<Result<_, _>>()?
        } else {
            vec![fs::read(path)?]
        }
    };

    // Convert to 'static lifetime by leaking memory (reasonable for short-lived CLI process)
    let font_slices: Vec<&'static [u8]> = font_datas
        .into_iter()
        .map(|v| Box::leak(v.into_boxed_slice()) as &'static [u8])
        .collect();

    let script = Script::parse(&ass_bytes);
    let renderer = SoftwareRenderer::new_multi(&script, font_slices);

    let renderer = Arc::new(renderer);

    let total_frames = (duration * fps) as usize;
    let dt = 1.0 / fps;

    println!("Rendering {total_frames} frames at {fps} fps...");

    (0..total_frames)
        .into_par_iter()
        .try_for_each(|frame_idx| -> anyhow::Result<()> {
            let t = frame_idx as f32 * dt;
            let buffer = renderer.render_bitmap(t as f64, width, height, 42.0);

            let img = image::RgbaImage::from_raw(width, height, buffer).expect("buffer size");
            let file_name = format!("{frame_idx:06}.png");
            let path = Path::new(out_dir).join(file_name);
            img.save(path)?;
            Ok(())
        })?;

    println!("Done.");
    Ok(())
}

fn parse_resolution(res: &str) -> anyhow::Result<(u32, u32)> {
    let mut parts = res.split('x');
    let w = parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("bad res"))?
        .parse()?;
    let h = parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("bad res"))?
        .parse()?;
    Ok((w, h))
}
