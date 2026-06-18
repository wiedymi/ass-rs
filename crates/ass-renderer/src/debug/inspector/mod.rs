use crate::Frame;

#[cfg(feature = "nostd")]
use alloc::vec::Vec;

mod info;

pub use info::{
    BoundsInfo, ColorHistogram, LineScanSegment, PixelComparison, PixelInfo, RegionInfo,
};

/// Frame inspector for detailed pixel-level debugging
pub struct FrameInspector {
    frame: Option<Frame>,
    cursor_x: u32,
    cursor_y: u32,
}

impl Default for FrameInspector {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameInspector {
    pub fn new() -> Self {
        Self {
            frame: None,
            cursor_x: 0,
            cursor_y: 0,
        }
    }

    pub fn load_frame(&mut self, frame: Frame) {
        self.cursor_x = frame.width() / 2;
        self.cursor_y = frame.height() / 2;
        self.frame = Some(frame);
    }

    pub fn move_cursor(&mut self, dx: i32, dy: i32) {
        if let Some(ref frame) = self.frame {
            let new_x = (self.cursor_x as i32 + dx)
                .max(0)
                .min(frame.width() as i32 - 1) as u32;
            let new_y = (self.cursor_y as i32 + dy)
                .max(0)
                .min(frame.height() as i32 - 1) as u32;
            self.cursor_x = new_x;
            self.cursor_y = new_y;
        }
    }

    pub fn set_cursor(&mut self, x: u32, y: u32) {
        if let Some(ref frame) = self.frame {
            self.cursor_x = x.min(frame.width() - 1);
            self.cursor_y = y.min(frame.height() - 1);
        }
    }

    pub fn get_pixel_at_cursor(&self) -> Option<PixelInfo> {
        self.get_pixel_at(self.cursor_x, self.cursor_y)
    }

    pub fn get_pixel_at(&self, x: u32, y: u32) -> Option<PixelInfo> {
        let frame = self.frame.as_ref()?;

        if x >= frame.width() || y >= frame.height() {
            return None;
        }

        let idx = ((y * frame.width() + x) * 4) as usize;
        let pixels = frame.pixels();

        if idx + 3 >= pixels.len() {
            return None;
        }

        Some(PixelInfo {
            x,
            y,
            r: pixels[idx],
            g: pixels[idx + 1],
            b: pixels[idx + 2],
            a: pixels[idx + 3],
        })
    }

    pub fn get_region(&self, x: u32, y: u32, width: u32, height: u32) -> RegionInfo {
        let frame = match self.frame.as_ref() {
            Some(f) => f,
            None => return RegionInfo::empty(),
        };

        let mut pixels = Vec::new();
        let mut histogram = ColorHistogram::new();

        let x_end = (x + width).min(frame.width());
        let y_end = (y + height).min(frame.height());

        for py in y..y_end {
            for px in x..x_end {
                if let Some(pixel) = self.get_pixel_at(px, py) {
                    histogram.add_pixel(&pixel);
                    pixels.push(pixel);
                }
            }
        }

        RegionInfo {
            x,
            y,
            width: x_end - x,
            height: y_end - y,
            pixels,
            histogram,
        }
    }

    pub fn find_non_transparent_bounds(&self) -> Option<BoundsInfo> {
        let frame = self.frame.as_ref()?;
        let pixels = frame.pixels();

        let mut min_x = frame.width();
        let mut min_y = frame.height();
        let mut max_x = 0u32;
        let mut max_y = 0u32;
        let mut found = false;

        for y in 0..frame.height() {
            for x in 0..frame.width() {
                let idx = ((y * frame.width() + x) * 4) as usize;
                if idx + 3 < pixels.len() && pixels[idx + 3] > 0 {
                    found = true;
                    min_x = min_x.min(x);
                    min_y = min_y.min(y);
                    max_x = max_x.max(x);
                    max_y = max_y.max(y);
                }
            }
        }

        if !found {
            return None;
        }

        Some(BoundsInfo {
            min_x,
            min_y,
            max_x,
            max_y,
            width: max_x - min_x + 1,
            height: max_y - min_y + 1,
        })
    }

    pub fn scan_line(&self, y: u32) -> Vec<LineScanSegment> {
        let frame = match self.frame.as_ref() {
            Some(f) => f,
            None => return Vec::new(),
        };

        if y >= frame.height() {
            return Vec::new();
        }

        let mut segments = Vec::new();
        let mut current_segment: Option<LineScanSegment> = None;

        for x in 0..frame.width() {
            if let Some(pixel) = self.get_pixel_at(x, y) {
                if pixel.a > 0 {
                    // Non-transparent pixel
                    match current_segment.as_mut() {
                        Some(seg) if seg.is_similar(&pixel) => {
                            seg.end_x = x;
                            seg.pixels.push(pixel);
                        }
                        _ => {
                            if let Some(seg) = current_segment.take() {
                                segments.push(seg);
                            }
                            current_segment = Some(LineScanSegment {
                                y,
                                start_x: x,
                                end_x: x,
                                pixels: vec![pixel],
                                avg_color: [pixel.r, pixel.g, pixel.b, pixel.a],
                            });
                        }
                    }
                } else if let Some(seg) = current_segment.take() {
                    segments.push(seg);
                }
            }
        }

        if let Some(mut seg) = current_segment {
            seg.calculate_average();
            segments.push(seg);
        }

        segments
    }

    pub fn compare_pixels(&self, x1: u32, y1: u32, x2: u32, y2: u32) -> Option<PixelComparison> {
        let pixel1 = self.get_pixel_at(x1, y1)?;
        let pixel2 = self.get_pixel_at(x2, y2)?;

        Some(PixelComparison {
            pixel1,
            pixel2,
            color_distance: pixel1.color_distance(&pixel2),
            is_similar: pixel1.is_similar(&pixel2, 30),
        })
    }

    pub fn print_cursor_info(&self) {
        if let Some(pixel) = self.get_pixel_at_cursor() {
            pixel.print_detailed();
        } else {
            println!("No frame loaded or cursor out of bounds");
        }
    }

    pub fn print_ascii_preview(&self, width: u32, height: u32) {
        let frame = match self.frame.as_ref() {
            Some(f) => f,
            None => {
                println!("No frame loaded");
                return;
            }
        };

        let scale_x = frame.width() as f32 / width as f32;
        let scale_y = frame.height() as f32 / height as f32;

        println!(
            "ASCII Preview ({}x{} -> {}x{}):",
            frame.width(),
            frame.height(),
            width,
            height
        );
        let border = "─".repeat(width as usize);
        println!("┌{border}┐");

        for y in 0..height {
            print!("│");
            for x in 0..width {
                let sample_x = (x as f32 * scale_x) as u32;
                let sample_y = (y as f32 * scale_y) as u32;

                if let Some(pixel) = self.get_pixel_at(sample_x, sample_y) {
                    print!("{}", pixel.to_ascii());
                } else {
                    print!(" ");
                }
            }
            println!("│");
        }

        let border = "─".repeat(width as usize);
        println!("└{border}┘");
        println!("Legend: █=opaque ▓=semi ░=faint ·=trace  =transparent");
    }
}
