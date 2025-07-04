//! Simple software renderer that interprets basic bold and italic tags.

use crate::model::{Frame, Pos, RenderedLine, Segment, StyleState};
use ass_core::override_parser;
use ass_core::Script;
use fontdue::Font;
use std::f32::consts::PI;

// Optimized glyph cache with reduced mutex contention
use std::cell::RefCell;
use std::collections::HashMap;

// Use thread-local cache for better performance
thread_local! {
    static THREAD_GLYPH_CACHE: RefCell<HashMap<(char, u32, u8), (fontdue::Metrics, Vec<u8>)>> =
        RefCell::new(HashMap::with_capacity(1024));
}

// Global cache as fallback
type GlobalGlyphCache =
    std::sync::RwLock<std::collections::HashMap<(char, u32, u8), (fontdue::Metrics, Vec<u8>)>>;

// Pre-allocated glyph slots for common characters
const COMMON_CHARS: &[char] = &[
    ' ', 'a', 'e', 'i', 'o', 'u', 't', 'n', 's', 'h', 'r', 'd', 'l', 'c', 'u', 'm', 'w', 'f', 'g',
    'y', 'p', 'b', 'v', 'k', 'j', 'x', 'q', 'z', 'A', 'E', 'I', 'O', 'U', 'T', 'N', 'S', 'H', 'R',
    'D', 'L', 'C', 'M', 'W', 'F', 'G', 'Y', 'P', 'B', 'V', 'K', 'J', 'X', 'Q', 'Z', '0', '1', '2',
    '3', '4', '5', '6', '7', '8', '9', '.', ',', '!', '?', ':', ';', '-', '\'', '"', '(', ')',
];

pub struct SoftwareRenderer {
    dialogues: Vec<Dialogue>,
    /// One or more fonts to search for glyphs, in priority order.
    fonts: Vec<Font>,
    /// Global glyph cache with font index: key is (char, font_size, font_index)
    glyph_cache: GlobalGlyphCache,
}

#[derive(Debug)]
struct Dialogue {
    start: f64, // seconds
    end: f64,   // seconds
    text: String,
    pos: Option<Pos>,
}

// Parameters for blitting rotated text
struct BlitParams {
    dst_x: f32,
    dst_y: f32,
    angle_deg: f32,
    rgb: (u8, u8, u8),
    global_alpha: u8,
}

impl SoftwareRenderer {
    /// Create a renderer with a single font (kept for backward-compatibility).
    pub fn new(script: &Script, font_data: &'static [u8]) -> Self {
        Self::new_multi(script, vec![font_data])
    }

    /// Create a renderer with multiple fonts. The first font that contains a glyph will be used.
    pub fn new_multi(script: &Script, fonts_data: Vec<&'static [u8]>) -> Self {
        let mut dialogues = Vec::new();
        // naive approach: serialize script and parse Dialogue lines
        let serialized = script.serialize();
        for line in serialized.lines() {
            if let Some(rest) = line.strip_prefix("Dialogue:") {
                if let Some(mut d) = parse_dialogue_line(rest.trim()) {
                    // extract pos override early (for simplicity)
                    if let Some(p) = extract_pos(&d.text) {
                        d.pos = Some(p);
                    }
                    dialogues.push(d);
                }
            }
        }
        let mut fonts = Vec::with_capacity(fonts_data.len());
        for data in fonts_data {
            if let Ok(f) = Font::from_bytes(data, fontdue::FontSettings::default()) {
                fonts.push(f);
            }
        }

        // If no valid fonts provided, create a minimal fallback font for testing
        if fonts.is_empty() {
            // For testing purposes, create a minimal pseudo-font that can at least provide metrics
            eprintln!("Warning: No valid fonts supplied to SoftwareRenderer. Creating minimal fallback for testing.");
            // We can't create a valid font from nothing, but we can handle this case gracefully
            // Rather than panic, we'll create an empty font vector and handle it in rendering
        }

        let mut renderer = Self {
            dialogues,
            fonts,
            glyph_cache: std::sync::RwLock::new(std::collections::HashMap::with_capacity(2048)),
        };

        // Pre-cache common characters for better performance (only if we have fonts)
        if !renderer.fonts.is_empty() {
            renderer.pre_cache_common_glyphs();
        }

        renderer
    }

    /// Pre-cache common characters for better performance
    fn pre_cache_common_glyphs(&mut self) {
        const COMMON_SIZES: &[u32] = &[12, 16, 18, 20, 24, 28, 32, 36, 48];

        for &ch in COMMON_CHARS {
            for &size in COMMON_SIZES {
                for (font_idx, font) in self.fonts.iter().enumerate() {
                    if font.lookup_glyph_index(ch) != 0 {
                        let (metrics, bitmap) = font.rasterize(ch, size as f32);
                        let key = (ch, size, font_idx as u8);

                        if let Ok(mut cache) = self.glyph_cache.write() {
                            cache.insert(key, (metrics, bitmap));
                        }
                        break; // Found in this font, no need to check others
                    }
                }
            }
        }
    }

    /// Optimized glyph lookup with thread-local caching
    fn get_glyph(&self, ch: char, size: f32) -> (fontdue::Metrics, Vec<u8>) {
        // Safety check: return default metrics if no fonts available
        if self.fonts.is_empty() {
            return (
                fontdue::Metrics {
                    xmin: 0,
                    ymin: 0,
                    width: 8,
                    height: 12,
                    advance_width: 8.0,
                    advance_height: 12.0,
                    bounds: fontdue::OutlineBounds {
                        xmin: 0.0,
                        ymin: 0.0,
                        width: 8.0,
                        height: 12.0,
                    },
                },
                vec![255; 8 * 12], // Simple filled rectangle
            );
        }

        let size_key = size.round() as u32;

        // Try thread-local cache first (fastest)
        let thread_result = THREAD_GLYPH_CACHE.with(|cache| {
            let cache_ref = cache.borrow();
            for (font_idx, _font) in self.fonts.iter().enumerate() {
                let key = (ch, size_key, font_idx as u8);
                if let Some((metrics, bitmap)) = cache_ref.get(&key) {
                    return Some((*metrics, bitmap.clone()));
                }
            }
            None
        });

        if let Some(result) = thread_result {
            return result;
        }

        // Try global cache (medium speed)
        if let Ok(cache) = self.glyph_cache.read() {
            for (font_idx, _font) in self.fonts.iter().enumerate() {
                let key = (ch, size_key, font_idx as u8);
                if let Some((metrics, bitmap)) = cache.get(&key) {
                    // Cache in thread-local for next time
                    THREAD_GLYPH_CACHE.with(|thread_cache| {
                        let mut cache_ref = thread_cache.borrow_mut();
                        cache_ref.insert(key, (*metrics, bitmap.clone()));
                    });
                    return (*metrics, bitmap.clone());
                }
            }
        }

        // Compute and cache (slowest path)
        for (font_idx, font) in self.fonts.iter().enumerate() {
            if font.lookup_glyph_index(ch) != 0 {
                let (metrics, bitmap) = font.rasterize(ch, size);
                let key = (ch, size_key, font_idx as u8);

                // Cache in both global and thread-local
                if let Ok(mut cache) = self.glyph_cache.write() {
                    cache.insert(key, (metrics.clone(), bitmap.clone()));
                }

                THREAD_GLYPH_CACHE.with(|thread_cache| {
                    let mut cache_ref = thread_cache.borrow_mut();
                    cache_ref.insert(key, (metrics, bitmap.clone()));
                });

                return (metrics, bitmap);
            }
        }

        // Fallback to first font (safe since we checked fonts.is_empty() above)
        self.fonts[0].rasterize(ch, size)
    }

    /// Render a frame corresponding to time (in seconds).
    pub fn render(&self, time: f64) -> Frame {
        let mut lines_out = Vec::new();
        for dlg in &self.dialogues {
            if time < dlg.start || time > dlg.end {
                continue;
            }
            let mut line = render_dialogue(&dlg.text);
            // Apply fade alpha
            if let Some((f_in, f_out)) = line.fade {
                let rel_start = time - dlg.start; // seconds
                let rel_end = dlg.end - time; // seconds remaining
                let a_in = f_in as f64 / 1000.0;
                let a_out = f_out as f64 / 1000.0;
                let alpha_factor = if rel_start < a_in {
                    (rel_start / a_in) as f32
                } else if rel_end < a_out {
                    (rel_end / a_out) as f32
                } else {
                    1.0
                };
                line.alpha = alpha_factor.clamp(0.0, 1.0);
            }
            lines_out.push(line);
        }
        Frame { lines: lines_out }
    }

    pub fn render_bitmap(&self, time: f64, width: u32, height: u32, font_size: f32) -> Vec<u8> {
        let mut buffer = vec![0u8; (width * height * 4) as usize];
        let frame = self.render(time);
        let mut cursor_y = 0f32;
        for line in frame.lines {
            // Compute line width with scaling
            let mut line_width = 0f32;
            for seg in &line.segments {
                let effective_size = font_size * (seg.style.font_size / 32.0);
                let scale_x = seg.style.font_scale_x / 100.0;
                for ch in seg.text.chars() {
                    // Skip rendering if no fonts available
                    if self.fonts.is_empty() {
                        line_width += 8.0 * scale_x; // Use default width
                        continue;
                    }

                    let (metrics, _bitmap) = self.get_glyph(ch, effective_size);
                    line_width += metrics.advance_width * scale_x;
                }
            }

            // horizontal anchor
            let mut cursor_x = match line.align % 3 {
                1 => 0.0,                               // left
                2 => (width as f32 - line_width) / 2.0, // center
                0 => width as f32 - line_width,         // right (align 3,6,9)
                _ => 0.0,
            };

            // vertical anchor baseline
            // basic line height estimate
            let line_height = font_size + 4.0;
            if cursor_y == 0.0 {
                cursor_y = match (line.align - 1) / 3 {
                    0 => height as f32 - line_height,         // bottom
                    1 => (height as f32 - line_height) / 2.0, // middle
                    2 => 0.0,                                 // top
                    _ => 0.0,
                };
            }

            for seg in line.segments {
                let line_alpha = line.alpha;
                let seg_alpha = seg.style.alpha;
                let combined_alpha = (line_alpha * seg_alpha * 255.0) as u8;

                let effective_size = font_size * (seg.style.font_size / 32.0);
                let scale_x = seg.style.font_scale_x / 100.0;
                let scale_y = seg.style.font_scale_y / 100.0;

                for ch in seg.text.chars() {
                    // Skip rendering if no fonts available
                    if self.fonts.is_empty() {
                        cursor_x += 8.0 * scale_x; // Use default width and skip
                        continue;
                    }

                    let (metrics, bitmap) = self.get_glyph(ch, effective_size);
                    let color = seg.style.color;
                    let rgb = (
                        ((color >> 16) & 0xFF) as u8,
                        ((color >> 8) & 0xFF) as u8,
                        (color & 0xFF) as u8,
                    );

                    let has_rotation = line.rot_z.abs() > 0.01
                        || line.rot_x.abs() > 0.01
                        || line.rot_y.abs() > 0.01;

                    if !has_rotation {
                        // Simple blit without rotation
                        for y in 0..metrics.height {
                            for x in 0..metrics.width {
                                let alpha = bitmap[y * metrics.width + x];
                                if alpha == 0 {
                                    continue;
                                }
                                let scaled_x = (x as f32 * scale_x) as i32;
                                let scaled_y = (y as f32 * scale_y) as i32;
                                let px = cursor_x as i32 + metrics.xmin + scaled_x;
                                let py = cursor_y as i32 + metrics.ymin + scaled_y;
                                if px < 0 || py < 0 {
                                    continue;
                                }
                                let (px, py) = (px as u32, py as u32);
                                if px >= width || py >= height {
                                    continue;
                                }
                                let idx = ((py * width + px) * 4) as usize;
                                let final_alpha = alpha.saturating_mul(combined_alpha) / 255;
                                buffer[idx] = rgb.0;
                                buffer[idx + 1] = rgb.1;
                                buffer[idx + 2] = rgb.2;
                                buffer[idx + 3] = final_alpha;
                            }
                        }
                    } else {
                        blit_rot(
                            &mut buffer,
                            width,
                            height,
                            &bitmap,
                            metrics.width,
                            metrics.height,
                            &BlitParams {
                                dst_x: cursor_x + metrics.xmin as f32,
                                dst_y: cursor_y + metrics.ymin as f32,
                                angle_deg: line.rot_z,
                                rgb,
                                global_alpha: combined_alpha,
                            },
                        );
                    }
                    cursor_x += metrics.advance_width * scale_x;
                }
            }
            cursor_y += line_height;
            if cursor_y >= height as f32 {
                break;
            }
        }
        buffer
    }
}

fn parse_time(s: &str) -> Option<f64> {
    // format h:mm:ss.cs (cs=centiseconds 0..99)
    let mut parts = s.split(':');
    let h = parts.next()?.parse::<u32>().ok()?;
    let m = parts.next()?.parse::<u32>().ok()?;
    let sec_cs = parts.next()?; // contains seconds and centiseconds "ss.cs"
    let mut sec_parts = sec_cs.split('.');
    let sec = sec_parts.next()?.parse::<u32>().ok()?;
    let cs = sec_parts.next()?.parse::<u32>().ok()?; // centiseconds
    let total_seconds = h as f64 * 3600.0 + m as f64 * 60.0 + sec as f64 + cs as f64 / 100.0;
    Some(total_seconds)
}

fn parse_dialogue_line(rest: &str) -> Option<Dialogue> {
    // Fields separated by commas, but text may contain commas. So split first 9 commas.
    let mut parts = rest.splitn(10, ',');
    let _layer = parts.next()?;
    let start = parse_time(parts.next()?.trim())?;
    let end = parse_time(parts.next()?.trim())?;
    // skip Style, Name, MarginL, MarginR, MarginV, Effect
    for _ in 0..6 {
        parts.next()?;
    }
    let text = parts.next()?.to_string();
    Some(Dialogue {
        start,
        end,
        text,
        pos: None,
    })
}

fn render_dialogue(text: &str) -> RenderedLine {
    // We'll iterate through characters, keep style state.
    let mut segments: Vec<Segment> = Vec::new();
    let mut state = StyleState::default();
    let mut pos = 0;
    let bytes = text.as_bytes();
    while pos < bytes.len() {
        if bytes[pos] == b'{' {
            // find closing '}'
            if let Some(close) = text[pos + 1..].find('}') {
                let start_span = pos + 1;
                let end_span = pos + 1 + close;
                let span = ass_core::Span {
                    start: start_span,
                    end: end_span,
                };
                let tag_instances = override_parser::parse_override_block(bytes, span);
                for inst in tag_instances {
                    if let Some(tag) = inst.plugin {
                        match tag.name() {
                            "b" => {
                                let arg_slice = &bytes[inst.args.start..inst.args.end];
                                state.bold = arg_slice.first().map(|b| *b != b'0').unwrap_or(true);
                            }
                            "i" => {
                                let arg_slice = &bytes[inst.args.start..inst.args.end];
                                state.italic =
                                    arg_slice.first().map(|b| *b != b'0').unwrap_or(true);
                            }
                            "u" => {
                                let arg_slice = &bytes[inst.args.start..inst.args.end];
                                state.underline =
                                    arg_slice.first().map(|b| *b != b'0').unwrap_or(true);
                            }
                            "s" => {
                                let arg_slice = &bytes[inst.args.start..inst.args.end];
                                state.strikethrough =
                                    arg_slice.first().map(|b| *b != b'0').unwrap_or(true);
                            }
                            "fs" => {
                                let s = std::str::from_utf8(&bytes[inst.args.start..inst.args.end])
                                    .unwrap_or("")
                                    .trim();
                                if let Ok(size) = s.parse::<f32>() {
                                    if size > 0.0 && size <= 200.0 {
                                        state.font_size = size;
                                    }
                                }
                            }
                            "fscx" => {
                                let s = std::str::from_utf8(&bytes[inst.args.start..inst.args.end])
                                    .unwrap_or("")
                                    .trim();
                                if let Ok(scale) = s.parse::<f32>() {
                                    state.font_scale_x = scale.clamp(1.0, 1000.0);
                                }
                            }
                            "fscy" => {
                                let s = std::str::from_utf8(&bytes[inst.args.start..inst.args.end])
                                    .unwrap_or("")
                                    .trim();
                                if let Ok(scale) = s.parse::<f32>() {
                                    state.font_scale_y = scale.clamp(1.0, 1000.0);
                                }
                            }
                            "bord" => {
                                let s = std::str::from_utf8(&bytes[inst.args.start..inst.args.end])
                                    .unwrap_or("")
                                    .trim();
                                if let Ok(border) = s.parse::<f32>() {
                                    state.border_size = border.clamp(0.0, 10.0);
                                }
                            }
                            "shad" => {
                                let s = std::str::from_utf8(&bytes[inst.args.start..inst.args.end])
                                    .unwrap_or("")
                                    .trim();
                                if let Ok(shadow) = s.parse::<f32>() {
                                    state.shadow_depth = shadow.clamp(0.0, 10.0);
                                }
                            }
                            "be" => {
                                let s = std::str::from_utf8(&bytes[inst.args.start..inst.args.end])
                                    .unwrap_or("")
                                    .trim();
                                if let Ok(blur) = s.parse::<f32>() {
                                    state.blur_edges = blur.clamp(0.0, 10.0);
                                }
                            }
                            "alpha" => {
                                let s = std::str::from_utf8(&bytes[inst.args.start..inst.args.end])
                                    .unwrap_or("")
                                    .trim();
                                // ASS alpha is &Hxx& format where xx is hex alpha (00 = opaque, FF = transparent)
                                if s.starts_with("&H") && s.ends_with('&') {
                                    let hex = &s[2..s.len() - 1];
                                    if let Ok(alpha_val) = u8::from_str_radix(hex, 16) {
                                        state.alpha = 1.0 - (alpha_val as f32 / 255.0);
                                    }
                                }
                            }
                            "an" => {
                                let s = std::str::from_utf8(&bytes[inst.args.start..inst.args.end])
                                    .unwrap_or("")
                                    .trim();
                                if let Ok(n) = s.parse::<u8>() {
                                    if (1..=9).contains(&n) {
                                        state.align = n;
                                    }
                                }
                            }
                            "frz" => {
                                let s = std::str::from_utf8(&bytes[inst.args.start..inst.args.end])
                                    .unwrap_or("")
                                    .trim();
                                if let Ok(v) = s.parse::<f32>() {
                                    state.rot_z = v;
                                }
                            }
                            "frx" => {
                                let s = std::str::from_utf8(&bytes[inst.args.start..inst.args.end])
                                    .unwrap_or("")
                                    .trim();
                                if let Ok(v) = s.parse::<f32>() {
                                    state.rot_x = v;
                                }
                            }
                            "fry" => {
                                let s = std::str::from_utf8(&bytes[inst.args.start..inst.args.end])
                                    .unwrap_or("")
                                    .trim();
                                if let Ok(v) = s.parse::<f32>() {
                                    state.rot_y = v;
                                }
                            }
                            "fad" => {
                                let s = std::str::from_utf8(&bytes[inst.args.start..inst.args.end])
                                    .unwrap_or("");
                                let mut nums =
                                    s.split(',').filter_map(|p| p.trim().parse::<u16>().ok());
                                if let (Some(a), Some(b)) = (nums.next(), nums.next()) {
                                    state.fade = Some((a, b));
                                }
                            }
                            "c" => {
                                // expect &HBBGGRR& pattern
                                let slice = &bytes[inst.args.start..inst.args.end];
                                // find first digit/letter
                                let s = std::str::from_utf8(slice).unwrap_or("");
                                // remove leading & and H and trailing &
                                if let Some(start_hex) = s.find('H') {
                                    if let Some(end_amp) = s[start_hex + 1..].find('&') {
                                        let hex = &s[start_hex + 1..start_hex + 1 + end_amp];
                                        if let Ok(val) = u32::from_str_radix(hex, 16) {
                                            // ASS order is BBGGRR
                                            let b = (val >> 16) & 0xFF;
                                            let g = (val >> 8) & 0xFF;
                                            let r = val & 0xFF;
                                            state.color = (r << 16) | (g << 8) | b;
                                        }
                                    }
                                }
                            }
                            "r" => {
                                // Reset to default style
                                state = StyleState::default();
                            }
                            "pos" => { /* handled separately */ }
                            "move" => { /* handled separately */ }
                            "clip" => { /* TODO: implement clipping */ }
                            _ => {}
                        }
                    }
                }
                pos = end_span + 1; // skip closing
                continue;
            }
        }
        // normal character until next '{'
        let next_special = text[pos..].find('{').unwrap_or(text.len() - pos);
        let segment_text = &text[pos..pos + next_special];

        // Convert to static string by leaking (for simplicity in this demo)
        let static_text = Box::leak(segment_text.to_owned().into_boxed_str());

        segments.push(Segment {
            text: static_text,
            style: state,
        });
        pos += next_special;
    }

    RenderedLine {
        segments,
        alpha: state.alpha,
        rot_x: state.rot_x,
        rot_y: state.rot_y,
        rot_z: state.rot_z,
        fade: state.fade,
        align: state.align,
        pos: None,
        movement: None,
    }
}

fn extract_pos(text: &str) -> Option<Pos> {
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'{' {
            if let Some(close) = text[i + 1..].find('}') {
                let block = &text[i + 1..i + 1 + close];
                if let Some(tag_start) = block.find("\\pos(") {
                    let rest = &block[tag_start + 5..];
                    if let Some(paren_end) = rest.find(')') {
                        let coords = &rest[..paren_end];
                        let mut parts = coords.split(',');
                        let x = parts.next()?.trim().parse::<f32>().ok()?;
                        let y = parts.next()?.trim().parse::<f32>().ok()?;
                        return Some(Pos { x, y });
                    }
                }
                i += close + 2;
                continue;
            }
        }
        i += 1;
    }
    None
}

// -------------- rotation helper --------------

fn blit_rot(
    buffer: &mut [u8],
    buf_w: u32,
    buf_h: u32,
    glyph: &[u8],
    gw: usize,
    gh: usize,
    params: &BlitParams,
) {
    if params.angle_deg.abs() < 0.01 {
        return;
    }
    let angle = params.angle_deg * PI / 180.0;
    let (s, c) = angle.sin_cos();
    let cx = gw as f32 / 2.0;
    let cy = gh as f32 / 2.0;
    for gy in 0..gh {
        for gx in 0..gw {
            let a = glyph[gy * gw + gx];
            if a == 0 {
                continue;
            }
            let ox = gx as f32 - cx;
            let oy = gy as f32 - cy;
            let rx = ox * c - oy * s + cx;
            let ry = ox * s + oy * c + cy;
            let px = (params.dst_x + rx).round() as i32;
            let py = (params.dst_y + ry).round() as i32;
            if px < 0 || py < 0 {
                continue;
            }
            let (px, py) = (px as u32, py as u32);
            if px >= buf_w || py >= buf_h {
                continue;
            }
            let idx = ((py * buf_w + px) * 4) as usize;
            let final_a = a.saturating_mul(params.global_alpha) / 255;
            buffer[idx] = params.rgb.0;
            buffer[idx + 1] = params.rgb.1;
            buffer[idx + 2] = params.rgb.2;
            buffer[idx + 3] = final_a;
        }
    }
}
