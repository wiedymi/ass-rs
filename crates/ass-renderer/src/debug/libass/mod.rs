//! Safe wrapper over libass for A/B comparison against our software backend.
//!
//! Renders an `.ass` document through libass and returns a straight-alpha RGBA
//! frame (over transparent) plus the raw `ASS_Image` rectangles libass emitted,
//! in draw order. The rectangles expose libass's exact per-bitmap placement so a
//! geometric discrepancy (line spacing, glyph advance, outline extent) can be
//! measured directly rather than inferred from pixels.

#![allow(unsafe_code)]

mod sys;

use crate::utils::RenderError;
use core::ffi::{c_int, c_void};
use std::ffi::CString;

/// libass default font provider ids (`ASS_DefaultFontProvider`).
const FONTPROVIDER_NONE: c_int = 0;
const FONTPROVIDER_AUTODETECT: c_int = 1;

/// Placement and colour of one libass output bitmap (`ASS_Image`).
#[derive(Debug, Clone, Copy)]
pub struct LibassRect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    /// libass `color`: 0xRRGGBBAA, low byte is transparency (0 = opaque).
    pub color: u32,
}

/// A libass-rendered frame: straight-alpha RGBA over transparent plus the raw
/// bitmap rectangles (draw order).
pub struct LibassFrame {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub rects: Vec<LibassRect>,
}

/// Owns a libass library + renderer handle pair.
pub struct Libass {
    library: *mut c_void,
    renderer: *mut c_void,
    width: u32,
    height: u32,
}

impl Libass {
    /// Initialize libass for a `width`x`height` output frame.
    pub fn new(width: u32, height: u32) -> Result<Self, RenderError> {
        // SAFETY: init calls return owned handles or null, which we check.
        unsafe {
            let library = sys::ass_library_init();
            if library.is_null() {
                return Err(RenderError::BackendError("ass_library_init failed".into()));
            }
            let renderer = sys::ass_renderer_init(library);
            if renderer.is_null() {
                sys::ass_library_done(library);
                return Err(RenderError::BackendError("ass_renderer_init failed".into()));
            }
            sys::ass_set_frame_size(renderer, width as c_int, height as c_int);
            sys::ass_set_storage_size(renderer, width as c_int, height as c_int);
            Ok(Self {
                library,
                renderer,
                width,
                height,
            })
        }
    }

    /// Configure font lookup. Pass `fonts_dir` with a single bundled font and
    /// `use_system = false` for deterministic glyphs; `use_system = true` lets
    /// libass autodetect system fonts (matches our backend's default loader).
    pub fn set_fonts(
        &self,
        fonts_dir: Option<&str>,
        default_family: &str,
        use_system: bool,
    ) -> Result<(), RenderError> {
        let family = CString::new(default_family)
            .map_err(|_| RenderError::BackendError("invalid font family".into()))?;
        let dir = fonts_dir
            .map(CString::new)
            .transpose()
            .map_err(|_| RenderError::BackendError("invalid fonts dir".into()))?;
        let provider = if use_system {
            FONTPROVIDER_AUTODETECT
        } else {
            FONTPROVIDER_NONE
        };
        // SAFETY: all pointers are valid for the duration of the calls; libass
        // copies what it retains.
        unsafe {
            if let Some(dir) = &dir {
                sys::ass_set_fonts_dir(self.library, dir.as_ptr());
            }
            sys::ass_set_fonts(
                self.renderer,
                core::ptr::null(),
                family.as_ptr(),
                provider,
                core::ptr::null(),
                1,
            );
        }
        Ok(())
    }

    /// Render `ass_text` at `time_ms` and composite the result.
    pub fn render(&self, ass_text: &str, time_ms: i64) -> Result<LibassFrame, RenderError> {
        let mut buf = ass_text.as_bytes().to_vec();
        // SAFETY: `buf` is a valid mutable region of `len` bytes; ass_read_memory
        // parses it in place and does not retain the pointer afterwards.
        let track = unsafe {
            sys::ass_read_memory(
                self.library,
                buf.as_mut_ptr().cast(),
                buf.len(),
                core::ptr::null_mut(),
            )
        };
        if track.is_null() {
            return Err(RenderError::BackendError("ass_read_memory failed".into()));
        }
        let mut frame = LibassFrame {
            width: self.width,
            height: self.height,
            rgba: vec![0u8; (self.width * self.height * 4) as usize],
            rects: Vec::new(),
        };
        // SAFETY: render_frame returns a list owned by the renderer, valid until
        // the next render/teardown; we only read it before freeing the track.
        unsafe {
            let mut detect: c_int = 0;
            let mut img = sys::ass_render_frame(self.renderer, track, time_ms, &mut detect);
            while !img.is_null() {
                let image = &*img;
                composite_image(&mut frame, image);
                frame.rects.push(LibassRect {
                    x: image.dst_x,
                    y: image.dst_y,
                    w: image.w,
                    h: image.h,
                    color: image.color,
                });
                img = image.next;
            }
            sys::ass_free_track(track);
        }
        Ok(frame)
    }

    /// Parse an `.ass` document into a reusable track (for benchmarking, so the
    /// parse cost is not paid per frame).
    pub fn read_track(&self, ass_text: &str) -> Result<LibassTrack, RenderError> {
        let mut buf = ass_text.as_bytes().to_vec();
        // SAFETY: `buf` is a valid mutable region; ass_read_memory parses in place.
        let track = unsafe {
            sys::ass_read_memory(
                self.library,
                buf.as_mut_ptr().cast(),
                buf.len(),
                core::ptr::null_mut(),
            )
        };
        if track.is_null() {
            return Err(RenderError::BackendError("ass_read_memory failed".into()));
        }
        Ok(LibassTrack { track })
    }

    /// Render a pre-parsed track at `time_ms` and return the number of output
    /// bitmaps. Times only libass's frame rendering (`ass_render_frame`); the
    /// returned count keeps the call from being optimized away in benchmarks.
    pub fn render_count(&self, track: &LibassTrack, time_ms: i64) -> usize {
        // SAFETY: `track` is a valid track owned by `self.library`; the returned
        // list is owned by the renderer and only walked here.
        unsafe {
            let mut detect: c_int = 0;
            let mut img = sys::ass_render_frame(self.renderer, track.track, time_ms, &mut detect);
            let mut count = 0;
            while !img.is_null() {
                count += 1;
                img = (*img).next;
            }
            count
        }
    }
}

/// A parsed libass track, freed on drop.
pub struct LibassTrack {
    track: *mut c_void,
}

impl Drop for LibassTrack {
    fn drop(&mut self) {
        // SAFETY: `track` came from ass_read_memory and is freed only here.
        unsafe {
            sys::ass_free_track(self.track);
        }
    }
}

impl Drop for Libass {
    fn drop(&mut self) {
        // SAFETY: both handles were created in `new` and freed only here.
        unsafe {
            sys::ass_renderer_done(self.renderer);
            sys::ass_library_done(self.library);
        }
    }
}

/// Source-over composite one libass coverage bitmap into the RGBA accumulator.
fn composite_image(frame: &mut LibassFrame, image: &sys::ASS_Image) {
    if image.bitmap.is_null() || image.w <= 0 || image.h <= 0 {
        return;
    }
    let r = ((image.color >> 24) & 0xFF) as u8;
    let g = ((image.color >> 16) & 0xFF) as u8;
    let b = ((image.color >> 8) & 0xFF) as u8;
    let opacity = 255 - (image.color & 0xFF); // ASS-style inverted alpha
    let (fw, fh) = (frame.width as i32, frame.height as i32);
    for row in 0..image.h {
        let py = image.dst_y + row;
        if py < 0 || py >= fh {
            continue;
        }
        for col in 0..image.w {
            let px = image.dst_x + col;
            if px < 0 || px >= fw {
                continue;
            }
            // SAFETY: row < h and col < w, and stride is the row length in bytes.
            let cov =
                u32::from(unsafe { *image.bitmap.offset((row * image.stride + col) as isize) });
            let src_a = cov * opacity / 255;
            if src_a == 0 {
                continue;
            }
            let idx = ((py * fw + px) * 4) as usize;
            over(&mut frame.rgba[idx..idx + 4], r, g, b, src_a as u8);
        }
    }
}

/// Straight-alpha source-over of (r, g, b, a) onto an RGBA destination pixel.
fn over(dst: &mut [u8], r: u8, g: u8, b: u8, a: u8) {
    let sa = u32::from(a);
    let da = u32::from(dst[3]);
    let inv = 255 - sa;
    let out_a = sa + da * inv / 255;
    if out_a == 0 {
        return;
    }
    for (i, &sc) in [r, g, b].iter().enumerate() {
        let dc = u32::from(dst[i]);
        dst[i] = ((u32::from(sc) * sa + dc * da * inv / 255) / out_a) as u8;
    }
    dst[3] = out_a as u8;
}
