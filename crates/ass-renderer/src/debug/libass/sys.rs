//! Raw libass C bindings — the minimal subset needed to render an `.ass` to the
//! `ASS_Image` list for A/B comparison. Linked via `build.rs` when the
//! `libass-compare` feature is on. All items are `unsafe` to call.

#![allow(unsafe_code, non_camel_case_types)]

use core::ffi::{c_char, c_int, c_void};

/// Mirror of libass `ASS_Image` (ass.h). One single-colour coverage bitmap.
#[repr(C)]
pub struct ASS_Image {
    pub w: c_int,
    pub h: c_int,
    pub stride: c_int,
    pub bitmap: *mut u8,
    /// 0xRRGGBBAA, where the low byte is transparency (0 = opaque).
    pub color: u32,
    pub dst_x: c_int,
    pub dst_y: c_int,
    pub next: *mut ASS_Image,
    pub image_type: c_int,
}

extern "C" {
    pub fn ass_library_init() -> *mut c_void;
    pub fn ass_library_done(library: *mut c_void);
    pub fn ass_set_fonts_dir(library: *mut c_void, fonts_dir: *const c_char);
    pub fn ass_renderer_init(library: *mut c_void) -> *mut c_void;
    pub fn ass_renderer_done(renderer: *mut c_void);
    pub fn ass_set_frame_size(renderer: *mut c_void, w: c_int, h: c_int);
    pub fn ass_set_storage_size(renderer: *mut c_void, w: c_int, h: c_int);
    pub fn ass_set_fonts(
        renderer: *mut c_void,
        default_font: *const c_char,
        default_family: *const c_char,
        default_font_provider: c_int,
        config: *const c_char,
        update: c_int,
    );
    pub fn ass_read_memory(
        library: *mut c_void,
        data: *mut c_char,
        size: usize,
        codepage: *mut c_char,
    ) -> *mut c_void;
    pub fn ass_free_track(track: *mut c_void);
    pub fn ass_render_frame(
        renderer: *mut c_void,
        track: *mut c_void,
        now: i64,
        detect_change: *mut c_int,
    ) -> *mut ASS_Image;
}
