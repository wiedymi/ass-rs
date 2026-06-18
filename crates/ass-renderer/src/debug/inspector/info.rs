#[cfg(not(feature = "nostd"))]
use std::collections::HashSet;
#[cfg(not(feature = "nostd"))]
use std::fmt;

#[cfg(feature = "nostd")]
use alloc::collections::BTreeSet as HashSet;
#[cfg(feature = "nostd")]
use alloc::vec::Vec;
#[cfg(feature = "nostd")]
use core::fmt;

#[derive(Debug, Clone, Copy)]
pub struct PixelInfo {
    pub x: u32,
    pub y: u32,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl PixelInfo {
    pub fn to_hex(&self) -> String {
        format!(
            "#{r:02X}{g:02X}{b:02X}{a:02X}",
            r = self.r,
            g = self.g,
            b = self.b,
            a = self.a
        )
    }

    pub fn to_css(&self) -> String {
        if self.a == 255 {
            format!("rgb({r}, {g}, {b})", r = self.r, g = self.g, b = self.b)
        } else {
            format!(
                "rgba({r}, {g}, {b}, {a:.2})",
                r = self.r,
                g = self.g,
                b = self.b,
                a = self.a as f32 / 255.0
            )
        }
    }

    pub fn luminance(&self) -> f32 {
        // Relative luminance formula
        0.2126 * (self.r as f32) + 0.7152 * (self.g as f32) + 0.0722 * (self.b as f32)
    }

    pub fn is_grayscale(&self) -> bool {
        self.r == self.g && self.g == self.b
    }

    pub fn color_distance(&self, other: &PixelInfo) -> f32 {
        let dr = (self.r as f32 - other.r as f32).powi(2);
        let dg = (self.g as f32 - other.g as f32).powi(2);
        let db = (self.b as f32 - other.b as f32).powi(2);
        let da = (self.a as f32 - other.a as f32).powi(2);

        (dr + dg + db + da).sqrt()
    }

    pub fn is_similar(&self, other: &PixelInfo, threshold: u8) -> bool {
        let diff = |a: u8, b: u8| -> u8 { a.abs_diff(b) };

        diff(self.r, other.r) <= threshold
            && diff(self.g, other.g) <= threshold
            && diff(self.b, other.b) <= threshold
            && diff(self.a, other.a) <= threshold
    }

    pub fn to_ascii(&self) -> char {
        if self.a == 0 {
            ' '
        } else if self.a > 200 {
            '█'
        } else if self.a > 150 {
            '▓'
        } else if self.a > 100 {
            '▒'
        } else if self.a > 50 {
            '░'
        } else {
            '·'
        }
    }

    pub fn print_detailed(&self) {
        println!("╔════════════════════════════════════╗");
        println!("║         Pixel Information          ║");
        println!("╚════════════════════════════════════╝");
        println!("Position: ({x}, {y})", x = self.x, y = self.y);
        println!(
            "RGBA: ({r}, {g}, {b}, {a})",
            r = self.r,
            g = self.g,
            b = self.b,
            a = self.a
        );
        println!("Hex: {hex}", hex = self.to_hex());
        println!("CSS: {css}", css = self.to_css());
        println!("Luminance: {luminance:.2}", luminance = self.luminance());
        println!(
            "Grayscale: {}",
            if self.is_grayscale() { "Yes" } else { "No" }
        );
        println!(
            "Opacity: {opacity:.1}%",
            opacity = (self.a as f32 / 255.0) * 100.0
        );

        // Color classification
        let color_name = if self.a == 0 {
            "Transparent"
        } else if self.is_grayscale() {
            if self.r > 200 {
                "White"
            } else if self.r > 150 {
                "Light Gray"
            } else if self.r > 100 {
                "Gray"
            } else if self.r > 50 {
                "Dark Gray"
            } else {
                "Black"
            }
        } else {
            // Simple color classification
            if self.r > self.g && self.r > self.b {
                "Reddish"
            } else if self.g > self.r && self.g > self.b {
                "Greenish"
            } else if self.b > self.r && self.b > self.g {
                "Bluish"
            } else {
                "Mixed"
            }
        };

        println!("Color: {color_name}");
        println!("════════════════════════════════════");
    }
}

#[derive(Debug)]
pub struct RegionInfo {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<PixelInfo>,
    pub histogram: ColorHistogram,
}

impl RegionInfo {
    pub(super) fn empty() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            pixels: Vec::new(),
            histogram: ColorHistogram::new(),
        }
    }

    pub fn print_summary(&self) {
        println!(
            "Region: {}x{} at ({}, {})",
            self.width, self.height, self.x, self.y
        );
        let pixel_count = self.pixels.len();
        println!("Total pixels: {pixel_count}");

        if !self.pixels.is_empty() {
            let non_transparent = self.pixels.iter().filter(|p| p.a > 0).count();
            println!(
                "Non-transparent: {} ({:.1}%)",
                non_transparent,
                (non_transparent as f32 / self.pixels.len() as f32) * 100.0
            );

            self.histogram.print_summary();
        }
    }
}

#[derive(Debug)]
pub struct ColorHistogram {
    pub red_sum: u64,
    pub green_sum: u64,
    pub blue_sum: u64,
    pub alpha_sum: u64,
    pub count: usize,
    pub unique_colors: HashSet<(u8, u8, u8, u8)>,
}

impl ColorHistogram {
    pub(super) fn new() -> Self {
        Self {
            red_sum: 0,
            green_sum: 0,
            blue_sum: 0,
            alpha_sum: 0,
            count: 0,
            unique_colors: HashSet::new(),
        }
    }

    pub(super) fn add_pixel(&mut self, pixel: &PixelInfo) {
        self.red_sum += pixel.r as u64;
        self.green_sum += pixel.g as u64;
        self.blue_sum += pixel.b as u64;
        self.alpha_sum += pixel.a as u64;
        self.count += 1;
        self.unique_colors
            .insert((pixel.r, pixel.g, pixel.b, pixel.a));
    }

    pub fn average_color(&self) -> Option<[u8; 4]> {
        if self.count == 0 {
            return None;
        }

        Some([
            (self.red_sum / self.count as u64) as u8,
            (self.green_sum / self.count as u64) as u8,
            (self.blue_sum / self.count as u64) as u8,
            (self.alpha_sum / self.count as u64) as u8,
        ])
    }

    fn print_summary(&self) {
        if let Some(avg) = self.average_color() {
            println!(
                "Average color: RGBA({}, {}, {}, {})",
                avg[0], avg[1], avg[2], avg[3]
            );
        }
        let color_count = self.unique_colors.len();
        println!("Unique colors: {color_count}");
    }
}

#[derive(Debug)]
pub struct BoundsInfo {
    pub min_x: u32,
    pub min_y: u32,
    pub max_x: u32,
    pub max_y: u32,
    pub width: u32,
    pub height: u32,
}

impl fmt::Display for BoundsInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Bounds: ({}, {}) to ({}, {}) [{}x{}]",
            self.min_x, self.min_y, self.max_x, self.max_y, self.width, self.height
        )
    }
}

#[derive(Debug)]
pub struct LineScanSegment {
    pub y: u32,
    pub start_x: u32,
    pub end_x: u32,
    pub pixels: Vec<PixelInfo>,
    pub avg_color: [u8; 4],
}

impl LineScanSegment {
    pub(super) fn is_similar(&self, pixel: &PixelInfo) -> bool {
        if self.pixels.is_empty() {
            return false;
        }

        // Check if pixel is similar to average
        let threshold = 30;
        let diff = |a: u8, b: u8| -> u8 { a.abs_diff(b) };

        diff(self.avg_color[0], pixel.r) <= threshold
            && diff(self.avg_color[1], pixel.g) <= threshold
            && diff(self.avg_color[2], pixel.b) <= threshold
    }

    pub(super) fn calculate_average(&mut self) {
        if self.pixels.is_empty() {
            return;
        }

        let sum: (u64, u64, u64, u64) = self.pixels.iter().fold((0, 0, 0, 0), |acc, p| {
            (
                acc.0 + p.r as u64,
                acc.1 + p.g as u64,
                acc.2 + p.b as u64,
                acc.3 + p.a as u64,
            )
        });

        let count = self.pixels.len() as u64;
        self.avg_color = [
            (sum.0 / count) as u8,
            (sum.1 / count) as u8,
            (sum.2 / count) as u8,
            (sum.3 / count) as u8,
        ];
    }

    pub fn width(&self) -> u32 {
        self.end_x - self.start_x + 1
    }
}

#[derive(Debug)]
pub struct PixelComparison {
    pub pixel1: PixelInfo,
    pub pixel2: PixelInfo,
    pub color_distance: f32,
    pub is_similar: bool,
}

impl fmt::Display for PixelComparison {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "Pixel 1 @ ({}, {}): {}",
            self.pixel1.x,
            self.pixel1.y,
            self.pixel1.to_hex()
        )?;
        writeln!(
            f,
            "Pixel 2 @ ({}, {}): {}",
            self.pixel2.x,
            self.pixel2.y,
            self.pixel2.to_hex()
        )?;
        writeln!(f, "Distance: {:.2}", self.color_distance)?;
        writeln!(f, "Similar: {}", if self.is_similar { "Yes" } else { "No" })?;
        Ok(())
    }
}
