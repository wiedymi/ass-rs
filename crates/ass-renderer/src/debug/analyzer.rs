use crate::Frame;

#[cfg(feature = "nostd")]
use alloc::vec::Vec;

/// Frame analyzer for detailed text-based debugging
pub struct FrameAnalyzer {
    enable_pixel_histogram: bool,
    enable_region_analysis: bool,
    enable_text_detection: bool,
}

impl FrameAnalyzer {
    pub fn new() -> Self {
        Self {
            enable_pixel_histogram: true,
            enable_region_analysis: true,
            enable_text_detection: true,
        }
    }

    pub fn analyze(&self, frame: &Frame) -> AnalysisReport {
        let mut report = AnalysisReport::new(frame.width(), frame.height());

        if self.enable_pixel_histogram {
            report.pixel_histogram = self.calculate_pixel_histogram(frame);
        }

        if self.enable_region_analysis {
            report.regions = self.detect_regions(frame);
        }

        if self.enable_text_detection {
            report.text_areas = self.detect_text_areas(frame);
        }

        report.calculate_statistics(frame);
        report
    }

    fn calculate_pixel_histogram(&self, frame: &Frame) -> PixelHistogram {
        let pixels = frame.pixels();
        let mut histogram = PixelHistogram::default();

        for chunk in pixels.chunks(4) {
            if chunk.len() == 4 {
                let r = chunk[0];
                let g = chunk[1];
                let b = chunk[2];
                let a = chunk[3];

                histogram.red[r as usize] += 1;
                histogram.green[g as usize] += 1;
                histogram.blue[b as usize] += 1;
                histogram.alpha[a as usize] += 1;

                if a > 0 {
                    histogram.non_transparent_count += 1;

                    // Classify pixel
                    if r == g && g == b {
                        if r > 200 {
                            histogram.white_pixels += 1;
                        } else if r < 50 {
                            histogram.black_pixels += 1;
                        } else {
                            histogram.gray_pixels += 1;
                        }
                    } else {
                        histogram.colored_pixels += 1;
                    }
                }
            }
        }

        histogram
    }

    fn detect_regions(&self, frame: &Frame) -> Vec<Region> {
        let mut regions = Vec::new();
        let pixels = frame.pixels();
        let width = frame.width() as usize;
        let height = frame.height() as usize;

        // Simple region detection using connected components
        let mut visited = vec![false; width * height];

        for y in 0..height {
            for x in 0..width {
                let idx = y * width + x;
                let pixel_idx = idx * 4;

                if !visited[idx] && pixel_idx + 3 < pixels.len() && pixels[pixel_idx + 3] > 0 {
                    // Found a non-transparent, unvisited pixel
                    let region = self.flood_fill(frame, &mut visited, x, y);
                    if region.pixel_count > 10 {
                        // Filter out tiny regions
                        regions.push(region);
                    }
                }
            }
        }

        regions
    }

    fn flood_fill(
        &self,
        frame: &Frame,
        visited: &mut [bool],
        start_x: usize,
        start_y: usize,
    ) -> Region {
        let pixels = frame.pixels();
        let width = frame.width() as usize;
        let height = frame.height() as usize;

        let mut region = Region {
            min_x: start_x as u32,
            min_y: start_y as u32,
            max_x: start_x as u32,
            max_y: start_y as u32,
            pixel_count: 0,
            avg_color: [0, 0, 0, 0],
        };

        let mut stack = vec![(start_x, start_y)];
        let mut color_sum = [0u64; 4];

        while let Some((x, y)) = stack.pop() {
            if x >= width || y >= height {
                continue;
            }

            let idx = y * width + x;
            if visited[idx] {
                continue;
            }

            let pixel_idx = idx * 4;
            if pixel_idx + 3 >= pixels.len() || pixels[pixel_idx + 3] == 0 {
                continue;
            }

            visited[idx] = true;
            region.pixel_count += 1;

            // Update bounds
            region.min_x = region.min_x.min(x as u32);
            region.min_y = region.min_y.min(y as u32);
            region.max_x = region.max_x.max(x as u32);
            region.max_y = region.max_y.max(y as u32);

            // Accumulate color
            for i in 0..4 {
                color_sum[i] += pixels[pixel_idx + i] as u64;
            }

            // Add neighbors
            if x > 0 {
                stack.push((x - 1, y));
            }
            if x + 1 < width {
                stack.push((x + 1, y));
            }
            if y > 0 {
                stack.push((x, y - 1));
            }
            if y + 1 < height {
                stack.push((x, y + 1));
            }
        }

        // Calculate average color
        if region.pixel_count > 0 {
            for i in 0..4 {
                region.avg_color[i] = (color_sum[i] / region.pixel_count as u64) as u8;
            }
        }

        region
    }

    fn detect_text_areas(&self, frame: &Frame) -> Vec<TextArea> {
        let regions = self.detect_regions(frame);
        let mut text_areas = Vec::new();

        for region in regions {
            // Heuristics to identify text regions:
            // - Aspect ratio typical of text lines
            // - High contrast colors (usually white/yellow text)
            // - Reasonable size

            let width = region.max_x - region.min_x + 1;
            let height = region.max_y - region.min_y + 1;
            let aspect_ratio = width as f32 / height as f32;

            // Text typically has aspect ratio > 2 for horizontal text
            // and reasonable height (20-200 pixels for typical subtitles)
            if aspect_ratio > 1.5 && height > 15 && height < 200 {
                // Check if it's high contrast (likely text)
                let is_bright = region.avg_color[0] > 200
                    || region.avg_color[1] > 200
                    || region.avg_color[2] > 200;

                if is_bright {
                    text_areas.push(TextArea {
                        x: region.min_x,
                        y: region.min_y,
                        width,
                        height,
                        confidence: calculate_text_confidence(&region),
                        estimated_font_size: estimate_font_size(height),
                    });
                }
            }
        }

        text_areas
    }
}

#[derive(Debug, Clone)]
pub struct AnalysisReport {
    pub width: u32,
    pub height: u32,
    pub total_pixels: usize,
    pub non_transparent_pixels: usize,
    pub transparency_percentage: f32,
    pub pixel_histogram: PixelHistogram,
    pub regions: Vec<Region>,
    pub text_areas: Vec<TextArea>,
    pub dominant_color: [u8; 4],
    pub contrast_ratio: f32,
}

impl AnalysisReport {
    fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            total_pixels: (width * height) as usize,
            non_transparent_pixels: 0,
            transparency_percentage: 0.0,
            pixel_histogram: PixelHistogram::default(),
            regions: Vec::new(),
            text_areas: Vec::new(),
            dominant_color: [0, 0, 0, 0],
            contrast_ratio: 0.0,
        }
    }

    fn calculate_statistics(&mut self, frame: &Frame) {
        self.non_transparent_pixels = self.pixel_histogram.non_transparent_count;
        self.transparency_percentage = ((self.total_pixels - self.non_transparent_pixels) as f32
            / self.total_pixels as f32)
            * 100.0;

        // Calculate dominant color
        self.dominant_color = self.calculate_dominant_color();

        // Calculate contrast ratio (simplified)
        self.contrast_ratio = self.calculate_contrast_ratio(frame);
    }

    fn calculate_dominant_color(&self) -> [u8; 4] {
        let hist = &self.pixel_histogram;

        // Find the peak in each channel
        let mut dominant = [0u8; 4];

        for (i, channel) in [&hist.red, &hist.green, &hist.blue, &hist.alpha]
            .iter()
            .enumerate()
        {
            let mut max_count = 0;
            let mut max_value = 0;

            for (value, &count) in channel.iter().enumerate() {
                if count > max_count {
                    max_count = count;
                    max_value = value;
                }
            }

            dominant[i] = max_value as u8;
        }

        dominant
    }

    fn calculate_contrast_ratio(&self, frame: &Frame) -> f32 {
        // Simplified contrast calculation
        // In a real implementation, this would calculate Weber contrast or Michelson contrast
        if self.pixel_histogram.white_pixels > 0 && self.pixel_histogram.black_pixels > 0 {
            let white_ratio =
                self.pixel_histogram.white_pixels as f32 / self.non_transparent_pixels as f32;
            let black_ratio =
                self.pixel_histogram.black_pixels as f32 / self.non_transparent_pixels as f32;

            // High contrast if we have both bright and dark pixels
            (white_ratio * black_ratio * 100.0).min(1.0)
        } else {
            0.0
        }
    }

    pub fn print_summary(&self) {
        println!("\n╔════════════════════════════════════════╗");
        println!("║         Frame Analysis Report          ║");
        println!("╚════════════════════════════════════════╝");

        println!("\n📊 Basic Statistics:");
        println!("  • Resolution: {}x{}", self.width, self.height);
        println!("  • Total Pixels: {}", self.total_pixels);
        println!(
            "  • Non-transparent: {} ({:.1}%)",
            self.non_transparent_pixels,
            (self.non_transparent_pixels as f32 / self.total_pixels as f32) * 100.0
        );
        println!("  • Transparency: {:.1}%", self.transparency_percentage);

        println!("\n🎨 Color Distribution:");
        println!("  • White pixels: {}", self.pixel_histogram.white_pixels);
        println!("  • Black pixels: {}", self.pixel_histogram.black_pixels);
        println!("  • Gray pixels: {}", self.pixel_histogram.gray_pixels);
        println!(
            "  • Colored pixels: {}",
            self.pixel_histogram.colored_pixels
        );
        println!(
            "  • Dominant color: RGBA({}, {}, {}, {})",
            self.dominant_color[0],
            self.dominant_color[1],
            self.dominant_color[2],
            self.dominant_color[3]
        );

        println!("\n🔍 Region Detection:");
        println!("  • Detected regions: {}", self.regions.len());
        for (i, region) in self.regions.iter().enumerate() {
            println!(
                "    {}. Box: ({}, {}) to ({}, {}) | Pixels: {} | Color: RGBA({}, {}, {}, {})",
                i + 1,
                region.min_x,
                region.min_y,
                region.max_x,
                region.max_y,
                region.pixel_count,
                region.avg_color[0],
                region.avg_color[1],
                region.avg_color[2],
                region.avg_color[3]
            );
        }

        println!("\n📝 Text Detection:");
        if self.text_areas.is_empty() {
            println!("  • No text areas detected");
        } else {
            println!("  • Detected text areas: {}", self.text_areas.len());
            for (i, area) in self.text_areas.iter().enumerate() {
                println!(
                    "    {}. Position: ({}, {}) | Size: {}x{} | Font: ~{}px | Confidence: {:.1}%",
                    i + 1,
                    area.x,
                    area.y,
                    area.width,
                    area.height,
                    area.estimated_font_size,
                    area.confidence * 100.0
                );
            }
        }

        println!("\n⚡ Performance Indicators:");
        println!("  • Contrast ratio: {:.2}", self.contrast_ratio);
        println!("  • Memory usage: {} KB", (self.total_pixels * 4) / 1024);

        println!("\n═══════════════════════════════════════");
    }

    pub fn to_json(&self) -> String {
        format!(
            r#"{{
    "resolution": {{ "width": {}, "height": {} }},
    "pixels": {{
        "total": {},
        "non_transparent": {},
        "transparency_percentage": {:.2}
    }},
    "color_distribution": {{
        "white": {},
        "black": {},
        "gray": {},
        "colored": {}
    }},
    "regions": {},
    "text_areas": {},
    "dominant_color": [{}, {}, {}, {}],
    "contrast_ratio": {:.2}
}}"#,
            self.width,
            self.height,
            self.total_pixels,
            self.non_transparent_pixels,
            self.transparency_percentage,
            self.pixel_histogram.white_pixels,
            self.pixel_histogram.black_pixels,
            self.pixel_histogram.gray_pixels,
            self.pixel_histogram.colored_pixels,
            self.regions.len(),
            self.text_areas.len(),
            self.dominant_color[0],
            self.dominant_color[1],
            self.dominant_color[2],
            self.dominant_color[3],
            self.contrast_ratio
        )
    }
}

#[derive(Debug, Clone)]
pub struct PixelHistogram {
    pub red: [usize; 256],
    pub green: [usize; 256],
    pub blue: [usize; 256],
    pub alpha: [usize; 256],
    pub non_transparent_count: usize,
    pub white_pixels: usize,
    pub black_pixels: usize,
    pub gray_pixels: usize,
    pub colored_pixels: usize,
}

impl Default for PixelHistogram {
    fn default() -> Self {
        Self {
            red: [0; 256],
            green: [0; 256],
            blue: [0; 256],
            alpha: [0; 256],
            non_transparent_count: 0,
            white_pixels: 0,
            black_pixels: 0,
            gray_pixels: 0,
            colored_pixels: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Region {
    pub min_x: u32,
    pub min_y: u32,
    pub max_x: u32,
    pub max_y: u32,
    pub pixel_count: usize,
    pub avg_color: [u8; 4],
}

#[derive(Debug, Clone)]
pub struct TextArea {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub confidence: f32,
    pub estimated_font_size: u32,
}

fn calculate_text_confidence(region: &Region) -> f32 {
    // Simple heuristic for text confidence
    let width = region.max_x - region.min_x + 1;
    let height = region.max_y - region.min_y + 1;
    let aspect = width as f32 / height as f32;

    let mut confidence: f32 = 0.0;

    // Good aspect ratio for text
    if aspect > 2.0 && aspect < 20.0 {
        confidence += 0.3;
    }

    // High brightness (typical for subtitles)
    let brightness =
        (region.avg_color[0] as f32 + region.avg_color[1] as f32 + region.avg_color[2] as f32)
            / 3.0;
    if brightness > 200.0 {
        confidence += 0.4;
    }

    // Reasonable size
    if height > 20 && height < 150 {
        confidence += 0.3;
    }

    confidence.min(1.0)
}

fn estimate_font_size(height: u32) -> u32 {
    // Rough estimation: font size is about 75% of bounding box height
    (height as f32 * 0.75) as u32
}
