//! Direct FFI bindings to libass for debugging and comparison
//!
//! This module provides direct FFI bindings to libass library.
//! It requires libass to be installed on the system:
//! - macOS: brew install libass
//! - Ubuntu/Debian: apt-get install libass-dev
//! - Fedora: dnf install libass-devel

#![allow(unsafe_code)] // Required for FFI

use std::ffi::CString;
use std::os::raw::{c_char, c_double, c_int, c_long};
use std::ptr;

#[repr(C)]
pub struct ASS_Library {
    _private: [u8; 0],
}

#[repr(C)]
pub struct ASS_Renderer {
    _private: [u8; 0],
}

#[repr(C)]
pub struct ASS_Track {
    _private: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct ASS_Image {
    pub w: c_int,
    pub h: c_int,
    pub stride: c_int,
    pub bitmap: *mut u8,
    pub color: u32,
    pub dst_x: c_int,
    pub dst_y: c_int,
    pub next: *mut ASS_Image,
    pub type_: c_int,
}

#[link(name = "ass")]
extern "C" {
    // Library functions
    pub fn ass_library_init() -> *mut ASS_Library;
    pub fn ass_library_done(library: *mut ASS_Library);
    pub fn ass_library_version() -> c_int;

    // Renderer functions
    pub fn ass_renderer_init(library: *mut ASS_Library) -> *mut ASS_Renderer;
    pub fn ass_renderer_done(renderer: *mut ASS_Renderer);
    pub fn ass_set_frame_size(renderer: *mut ASS_Renderer, w: c_int, h: c_int);
    pub fn ass_set_storage_size(renderer: *mut ASS_Renderer, w: c_int, h: c_int);
    pub fn ass_set_margins(renderer: *mut ASS_Renderer, t: c_int, b: c_int, l: c_int, r: c_int);
    pub fn ass_set_use_margins(renderer: *mut ASS_Renderer, use_: c_int);
    pub fn ass_set_pixel_aspect(renderer: *mut ASS_Renderer, par: c_double);
    pub fn ass_set_font_scale(renderer: *mut ASS_Renderer, font_scale: c_double);
    pub fn ass_set_hinting(renderer: *mut ASS_Renderer, ht: c_int);
    pub fn ass_set_line_spacing(renderer: *mut ASS_Renderer, line_spacing: c_double);
    pub fn ass_set_line_position(renderer: *mut ASS_Renderer, line_position: c_double);

    // Font configuration
    pub fn ass_set_fonts(
        renderer: *mut ASS_Renderer,
        default_font: *const c_char,
        default_family: *const c_char,
        dfp: c_int,
        config: *const c_char,
        update: c_int,
    );

    pub fn ass_fonts_update(renderer: *mut ASS_Renderer) -> c_int;
    pub fn ass_set_fonts_dir(library: *mut ASS_Library, fonts_dir: *const c_char);

    // Track functions
    pub fn ass_new_track(library: *mut ASS_Library) -> *mut ASS_Track;
    pub fn ass_free_track(track: *mut ASS_Track);
    pub fn ass_process_data(track: *mut ASS_Track, data: *const c_char, size: c_int);
    pub fn ass_process_chunk(
        track: *mut ASS_Track,
        data: *const c_char,
        size: c_int,
        timecode: c_long,
        duration: c_long,
    );
    pub fn ass_read_file(
        library: *mut ASS_Library,
        fname: *const c_char,
        codepage: *const c_char,
    ) -> *mut ASS_Track;

    // Rendering
    pub fn ass_render_frame(
        renderer: *mut ASS_Renderer,
        track: *mut ASS_Track,
        now: c_long,
        detect_change: *mut c_int,
    ) -> *mut ASS_Image;
}

/// Safe wrapper for libass library
pub struct LibassLibrary {
    ptr: *mut ASS_Library,
}

impl LibassLibrary {
    pub fn new() -> Option<Self> {
        unsafe {
            let ptr = ass_library_init();
            if ptr.is_null() {
                None
            } else {
                Some(Self { ptr })
            }
        }
    }

    pub fn version() -> i32 {
        unsafe { ass_library_version() as i32 }
    }

    pub fn as_ptr(&self) -> *mut ASS_Library {
        self.ptr
    }

    pub fn set_fonts_dir(&mut self, path: &str) {
        let c_path = CString::new(path).unwrap();
        unsafe {
            ass_set_fonts_dir(self.ptr, c_path.as_ptr());
        }
    }
}

impl Drop for LibassLibrary {
    fn drop(&mut self) {
        unsafe {
            if !self.ptr.is_null() {
                ass_library_done(self.ptr);
            }
        }
    }
}

/// Safe wrapper for libass renderer
pub struct LibassRenderer {
    ptr: *mut ASS_Renderer,
    _library: *mut ASS_Library, // Keep reference to prevent early drop
}

impl LibassRenderer {
    pub fn new(library: &LibassLibrary) -> Option<Self> {
        unsafe {
            let ptr = ass_renderer_init(library.as_ptr());
            if ptr.is_null() {
                None
            } else {
                Some(Self {
                    ptr,
                    _library: library.as_ptr(),
                })
            }
        }
    }

    pub fn set_frame_size(&mut self, width: i32, height: i32) {
        unsafe {
            ass_set_frame_size(self.ptr, width, height);
        }
    }

    pub fn set_storage_size(&mut self, width: i32, height: i32) {
        unsafe {
            ass_set_storage_size(self.ptr, width, height);
        }
    }

    pub fn set_margins(&mut self, top: i32, bottom: i32, left: i32, right: i32) {
        unsafe {
            ass_set_margins(self.ptr, top, bottom, left, right);
        }
    }

    pub fn set_use_margins(&mut self, use_margins: bool) {
        unsafe {
            ass_set_use_margins(self.ptr, if use_margins { 1 } else { 0 });
        }
    }

    pub fn set_pixel_aspect(&mut self, aspect: f64) {
        unsafe {
            ass_set_pixel_aspect(self.ptr, aspect);
        }
    }

    pub fn set_font_scale(&mut self, scale: f64) {
        unsafe {
            ass_set_font_scale(self.ptr, scale);
        }
    }

    pub fn set_fonts(&mut self, default_font: Option<&str>, default_family: Option<&str>) {
        unsafe {
            let font = default_font
                .and_then(|s| CString::new(s).ok())
                .map(|s| s.as_ptr())
                .unwrap_or(ptr::null());

            let default_family_cstr = CString::new("sans-serif").unwrap();
            let family = default_family
                .and_then(|s| CString::new(s).ok())
                .map(|s| s.as_ptr())
                .unwrap_or_else(|| default_family_cstr.as_ptr());

            ass_set_fonts(self.ptr, font, family, 1, ptr::null(), 1);
        }
    }

    pub fn render_frame(
        &mut self,
        track: &mut LibassTrack,
        time_ms: i64,
    ) -> Option<Vec<ASS_Image>> {
        unsafe {
            let mut change = 0;
            let image_ptr = ass_render_frame(self.ptr, track.as_ptr(), time_ms, &mut change);

            if image_ptr.is_null() {
                return None;
            }

            let mut images = Vec::new();
            let mut current = image_ptr;

            while !current.is_null() {
                images.push((*current).clone());
                current = (*current).next;
            }

            Some(images)
        }
    }

    pub fn as_ptr(&self) -> *mut ASS_Renderer {
        self.ptr
    }
}

impl Drop for LibassRenderer {
    fn drop(&mut self) {
        unsafe {
            if !self.ptr.is_null() {
                ass_renderer_done(self.ptr);
            }
        }
    }
}

/// Safe wrapper for libass track
pub struct LibassTrack {
    ptr: *mut ASS_Track,
}

impl LibassTrack {
    pub fn new(library: &LibassLibrary) -> Option<Self> {
        unsafe {
            let ptr = ass_new_track(library.as_ptr());
            if ptr.is_null() {
                None
            } else {
                Some(Self { ptr })
            }
        }
    }

    pub fn process_data(&mut self, data: &[u8]) {
        unsafe {
            ass_process_data(
                self.ptr,
                data.as_ptr() as *const c_char,
                data.len() as c_int,
            );
        }
    }

    pub fn process_chunk(&mut self, data: &str, timecode: i64, duration: i64) {
        let c_data = CString::new(data).unwrap();
        unsafe {
            ass_process_chunk(
                self.ptr,
                c_data.as_ptr(),
                data.len() as c_int,
                timecode,
                duration,
            );
        }
    }

    pub fn as_ptr(&mut self) -> *mut ASS_Track {
        self.ptr
    }
}

impl Drop for LibassTrack {
    fn drop(&mut self) {
        unsafe {
            if !self.ptr.is_null() {
                ass_free_track(self.ptr);
            }
        }
    }
}
