use ass_core::{plugin::AnimationState, Script};
use ass_render::{SoftwareRenderer, TextDirection, TextShaper};
use wasm_bindgen::prelude::*;
use web_sys::{console, CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

// Set up panic hook for better debugging
#[cfg(feature = "console_error_panic_hook")]
pub use console_error_panic_hook::set_once as set_panic_hook;

// Performance optimizations: pre-allocate common buffer sizes
static mut BUFFER_POOL: Option<Vec<Vec<u8>>> = None;
static mut POOL_INITIALIZED: bool = false;

const COMMON_RESOLUTIONS: &[(u32, u32)] = &[
    (640, 360),   // 360p
    (854, 480),   // 480p
    (1280, 720),  // 720p
    (1920, 1080), // 1080p
    (3840, 2160), // 4K
];

#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    set_panic_hook();

    // Initialize buffer pool for common resolutions
    unsafe {
        if !POOL_INITIALIZED {
            let mut pool = Vec::new();
            for &(w, h) in COMMON_RESOLUTIONS {
                let size = (w * h * 4) as usize;
                pool.push(vec![0u8; size]);
            }
            BUFFER_POOL = Some(pool);
            POOL_INITIALIZED = true;
        }
    }
}

/// Get a pre-allocated buffer or create a new one
fn get_buffer(size: usize) -> Vec<u8> {
    unsafe {
        if let Some(ref mut pool) = BUFFER_POOL {
            // Try to find a suitable buffer in the pool
            for buffer in pool.iter_mut() {
                if buffer.len() >= size {
                    let mut result = Vec::with_capacity(size);
                    result.resize(size, 0);
                    return result;
                }
            }
        }
    }
    // Fallback: allocate new buffer
    vec![0u8; size]
}

/// Main WASM API entry point for ASS subtitle processing
#[wasm_bindgen]
pub struct AssProcessor {
    script: Script,
    text_shaper: TextShaper,
    animation_state: AnimationState,
}

#[wasm_bindgen]
impl AssProcessor {
    /// Create a new ASS processor
    #[wasm_bindgen(constructor)]
    pub fn new(ass_bytes: &[u8]) -> Result<AssProcessor, JsValue> {
        let script = Script::parse(ass_bytes);
        let text_shaper = TextShaper::new();
        let animation_state = AnimationState::new();

        Ok(AssProcessor {
            script,
            text_shaper,
            animation_state,
        })
    }

    /// Get script metadata as JSON
    #[wasm_bindgen]
    pub fn get_metadata(&self) -> String {
        serde_json::json!({
            "sections": self.script.sections().len(),
            "serialized_size": self.script.serialize().len(),
        })
        .to_string()
    }

    /// Parse and validate the script
    #[wasm_bindgen]
    pub fn validate(&self) -> bool {
        // Basic validation - could be more sophisticated
        !self.script.serialize().is_empty()
    }

    /// Get all registered tag names
    #[wasm_bindgen]
    pub fn get_available_tags(&self) -> Vec<String> {
        ass_core::plugin::get_all_tag_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }
}

/// Enhanced renderer with performance optimizations and comprehensive API
#[wasm_bindgen]
pub struct RendererHandle {
    inner: SoftwareRenderer,
    processor: AssProcessor,
    // Performance optimization fields
    last_time: f64,
    cached_frame: Option<Vec<u8>>,
    cached_params: Option<(u32, u32, f32)>, // width, height, font_size
}

#[wasm_bindgen]
impl RendererHandle {
    /// Create a new renderer with font data
    #[wasm_bindgen(constructor)]
    pub fn new(ass_bytes: &[u8], font_bytes: &[u8]) -> Result<RendererHandle, JsValue> {
        let processor = AssProcessor::new(ass_bytes)?;
        let renderer = SoftwareRenderer::new(
            &processor.script,
            Box::leak(font_bytes.to_vec().into_boxed_slice()),
        );

        Ok(RendererHandle {
            inner: renderer,
            processor,
            last_time: -1.0,
            cached_frame: None,
            cached_params: None,
        })
    }

    /// Create renderer with multiple fonts for better glyph coverage
    #[wasm_bindgen]
    pub fn new_multi_font(ass_bytes: &[u8], fonts: &JsValue) -> Result<RendererHandle, JsValue> {
        let processor = AssProcessor::new(ass_bytes)?;

        // Convert JS array of font bytes to Vec<&'static [u8]>
        let fonts_array: js_sys::Array = fonts.clone().into();
        let mut font_data_vec = Vec::new();

        for i in 0..fonts_array.length() {
            let font_js = fonts_array.get(i);
            let font_bytes: js_sys::Uint8Array = font_js.into();
            let font_vec = font_bytes.to_vec();
            let font_static: &'static [u8] = Box::leak(font_vec.into_boxed_slice());
            font_data_vec.push(font_static);
        }

        if font_data_vec.is_empty() {
            return Err(JsValue::from_str("No fonts provided"));
        }

        let renderer = SoftwareRenderer::new_multi(&processor.script, font_data_vec);

        Ok(RendererHandle {
            inner: renderer,
            processor,
            last_time: -1.0,
            cached_frame: None,
            cached_params: None,
        })
    }

    /// Render a frame to RGBA bytes (width*height*4) with caching optimization
    #[wasm_bindgen]
    pub fn render_rgba(
        &mut self,
        time_sec: f64,
        width: u32,
        height: u32,
        font_size: f32,
    ) -> Vec<u8> {
        let current_params = (width, height, font_size);

        // Check if we can reuse cached frame (same time and params)
        if let (Some(ref cached), Some(cached_params)) = (&self.cached_frame, self.cached_params) {
            if (self.last_time - time_sec).abs() < 0.001 && cached_params == current_params {
                return cached.clone();
            }
        }

        let result = self.inner.render_bitmap(time_sec, width, height, font_size);

        // Update cache
        self.last_time = time_sec;
        self.cached_frame = Some(result.clone());
        self.cached_params = Some(current_params);

        result
    }

    /// Legacy method for backward compatibility
    #[wasm_bindgen]
    pub fn render(&mut self, time_sec: f64, width: u32, height: u32, font_size: f32) -> Vec<u8> {
        self.render_rgba(time_sec, width, height, font_size)
    }

    /// High-performance streaming render for video playback
    #[wasm_bindgen]
    pub fn render_stream(
        &mut self,
        time_sec: f64,
        width: u32,
        height: u32,
        font_size: f32,
        _fps: f32,
    ) -> js_sys::Uint8Array {
        let buffer_size = (width * height * 4) as usize;
        let mut buffer = get_buffer(buffer_size);

        // Render directly into pre-allocated buffer
        let frame_data = self.inner.render_bitmap(time_sec, width, height, font_size);

        if frame_data.len() <= buffer.len() {
            buffer[..frame_data.len()].copy_from_slice(&frame_data);
            buffer.truncate(frame_data.len());
        } else {
            buffer = frame_data;
        }

        // Return as Uint8Array for zero-copy transfer to JS
        js_sys::Uint8Array::from(&buffer[..])
    }

    /// Get frame timing information for optimization
    #[wasm_bindgen]
    pub fn get_timing_info(&self) -> JsValue {
        JsValue::from_serde(&serde_json::json!({
            "last_render_time": self.last_time,
            "has_cached_frame": self.cached_frame.is_some(),
            "cached_params": self.cached_params
        }))
        .unwrap_or(JsValue::NULL)
    }

    /// Clear cache to free memory
    #[wasm_bindgen]
    pub fn clear_cache(&mut self) {
        self.cached_frame = None;
        self.cached_params = None;
        self.last_time = -1.0;
    }

    /// Render directly to HTML5 Canvas with advanced options
    #[wasm_bindgen]
    pub fn render_to_canvas(
        &mut self,
        canvas: &HtmlCanvasElement,
        time_sec: f64,
        font_size: f32,
    ) -> Result<(), JsValue> {
        self.render_to_canvas_with_options(canvas, time_sec, font_size, &JsValue::NULL)
    }

    /// Render to canvas with advanced rendering options
    #[wasm_bindgen]
    pub fn render_to_canvas_with_options(
        &mut self,
        canvas: &HtmlCanvasElement,
        time_sec: f64,
        font_size: f32,
        options: &JsValue,
    ) -> Result<(), JsValue> {
        let context = canvas
            .get_context("2d")?
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()?;

        let width = canvas.width();
        let height = canvas.height();

        // Parse rendering options
        let blend_mode = if !options.is_null() && !options.is_undefined() {
            let opts = js_sys::Object::from(options.clone());
            js_sys::Reflect::get(&opts, &JsValue::from_str("blendMode"))
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_else(|| "source-over".to_string())
        } else {
            "source-over".to_string()
        };

        let alpha = if !options.is_null() && !options.is_undefined() {
            let opts = js_sys::Object::from(options.clone());
            js_sys::Reflect::get(&opts, &JsValue::from_str("alpha"))
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or(1.0)
        } else {
            1.0
        };

        // Set advanced canvas properties
        context.set_global_composite_operation(&blend_mode)?;
        context.set_global_alpha(alpha);

        let rgba_data = self.render_rgba(time_sec, width, height, font_size);

        // Convert RGBA to ImageData
        let image_data = ImageData::new_with_u8_clamped_array_and_sh(
            wasm_bindgen::Clamped(&rgba_data),
            width,
            height,
        )?;

        context.put_image_data(&image_data, 0.0, 0.0)?;

        Ok(())
    }

    /// Render to canvas with real-time video overlay
    #[wasm_bindgen]
    pub fn render_video_overlay(
        &mut self,
        canvas: &HtmlCanvasElement,
        video_element: &web_sys::HtmlVideoElement,
        time_sec: f64,
        font_size: f32,
    ) -> Result<(), JsValue> {
        let context = canvas
            .get_context("2d")?
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()?;

        let width = canvas.width();
        let height = canvas.height();

        // Clear canvas
        context.clear_rect(0.0, 0.0, width as f64, height as f64);

        // Draw video frame first
        context.draw_image_with_html_video_element_and_dw_and_dh(
            video_element,
            0.0,
            0.0,
            width as f64,
            height as f64,
        )?;

        // Render subtitles on top with proper blending
        context.set_global_composite_operation("source-over")?;
        
        let subtitle_data = self.render_rgba(time_sec, width, height, font_size);
        
        // Only render pixels with non-zero alpha to avoid overwriting video
        let image_data = ImageData::new_with_u8_clamped_array_and_sh(
            wasm_bindgen::Clamped(&subtitle_data),
            width,
            height,
        )?;

        context.put_image_data(&image_data, 0.0, 0.0)?;

        Ok(())
    }

    /// Render with off-screen canvas for performance
    #[wasm_bindgen]
    pub fn render_offscreen(
        &mut self,
        time_sec: f64,
        width: u32,
        height: u32,
        font_size: f32,
    ) -> Result<web_sys::OffscreenCanvas, JsValue> {
        let offscreen_canvas = web_sys::OffscreenCanvas::new(width, height)?;
        
        let context = offscreen_canvas
            .get_context("2d")?
            .unwrap()
            .dyn_into::<web_sys::OffscreenCanvasRenderingContext2d>()?;

        let rgba_data = self.render_rgba(time_sec, width, height, font_size);

        let image_data = ImageData::new_with_u8_clamped_array_and_sh(
            wasm_bindgen::Clamped(&rgba_data),
            width,
            height,
        )?;

        context.put_image_data(&image_data, 0.0, 0.0)?;

        Ok(offscreen_canvas)
    }

    /// Get timing information for all dialogues
    #[wasm_bindgen]
    pub fn get_dialogue_timings(&self) -> String {
        // Extract timing information from the script
        let serialized = self.processor.script.serialize();
        let mut timings = Vec::new();

        for line in serialized.lines() {
            if let Some(rest) = line.strip_prefix("Dialogue:") {
                if let Some(dialogue) = parse_dialogue_timing(rest) {
                    timings.push(dialogue);
                }
            }
        }

        serde_json::to_string(&timings).unwrap_or_else(|_| "[]".to_string())
    }

    /// Check if there are any subtitles at the given time
    #[wasm_bindgen]
    pub fn has_subtitles_at_time(&self, time_sec: f64) -> bool {
        let serialized = self.processor.script.serialize();

        for line in serialized.lines() {
            if let Some(rest) = line.strip_prefix("Dialogue:") {
                if let Some(dialogue) = parse_dialogue_timing(rest) {
                    if time_sec >= dialogue.start && time_sec <= dialogue.end {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Get metadata about the renderer state
    #[wasm_bindgen]
    pub fn get_renderer_info(&self) -> String {
        serde_json::json!({
            "last_render_time": self.last_time,
            "has_cached_frame": self.cached_frame.is_some(),
            "cached_params": self.cached_params,
            "sections_count": self.processor.script.sections().len()
        })
        .to_string()
    }
}

/// Text analysis and shaping utilities
#[wasm_bindgen]
pub struct TextAnalyzer {
    shaper: TextShaper,
}

#[wasm_bindgen]
impl TextAnalyzer {
    /// Create a new text analyzer
    #[wasm_bindgen(constructor)]
    pub fn new() -> TextAnalyzer {
        TextAnalyzer {
            shaper: TextShaper::new(),
        }
    }

    /// Analyze text properties (direction, script, etc.)
    #[wasm_bindgen]
    pub fn analyze_text(&self, text: &str) -> String {
        // Simple text analysis without rustybuzz
        let has_rtl = text.chars().any(|c| {
            matches!(c, '\u{0590}'..='\u{05FF}' | '\u{0600}'..='\u{06FF}' | '\u{0700}'..='\u{074F}')
        });

        let has_arabic = text.chars().any(|c| matches!(c, '\u{0600}'..='\u{06FF}'));
        let has_hebrew = text.chars().any(|c| matches!(c, '\u{0590}'..='\u{05FF}'));

        let script_name = if has_arabic {
            "Arabic"
        } else if has_hebrew {
            "Hebrew"
        } else {
            "Latin"
        };

        let direction_name = if has_rtl {
            "RightToLeft"
        } else {
            "LeftToRight"
        };

        serde_json::json!({
            "script": script_name,
            "direction": direction_name,
            "length": text.len(),
            "char_count": text.chars().count(),
        })
        .to_string()
    }

    /// Shape text and return glyph information
    #[wasm_bindgen]
    pub fn shape_text(&mut self, text: &str, font_size: f32) -> String {
        self.shaper.set_font_size(font_size);

        let direction = if text.chars().any(|c| matches!(c, '\u{0590}'..='\u{06FF}')) {
            TextDirection::RightToLeft
        } else {
            TextDirection::LeftToRight
        };

        match self.shaper.shape_text(text, "default", direction) {
            Ok(shaped) => serde_json::json!({
                "glyphs": shaped.glyphs.len(),
                "total_advance": shaped.total_advance,
                "line_height": shaped.line_height,
            })
            .to_string(),
            Err(_) => "{}".to_string(),
        }
    }
}

/// Utility functions for ASS processing
#[wasm_bindgen]
pub fn normalize_ass(src: &str) -> String {
    // Simple normalization - could be more sophisticated
    src.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Parse timing from dialogue line
#[wasm_bindgen]
pub fn parse_dialogue_timing_js(dialogue_line: &str) -> String {
    if let Some(dialogue) = parse_dialogue_timing(dialogue_line) {
        serde_json::to_string(&dialogue).unwrap_or_else(|_| "{}".to_string())
    } else {
        "{}".to_string()
    }
}

/// Convert time format (H:MM:SS.CC to seconds)
#[wasm_bindgen]
pub fn time_to_seconds(time_str: &str) -> f64 {
    parse_time_to_seconds(time_str).unwrap_or(0.0)
}

/// Convert seconds to time format (H:MM:SS.CC)
#[wasm_bindgen]
pub fn seconds_to_time(seconds: f64) -> String {
    format_seconds_to_time(seconds)
}

// Helper structures and functions

#[derive(serde::Serialize)]
struct DialogueTiming {
    start: f64,
    end: f64,
    text: String,
}

fn parse_dialogue_timing(rest: &str) -> Option<DialogueTiming> {
    let parts: Vec<&str> = rest.split(',').collect();
    if parts.len() < 10 {
        return None;
    }

    let start = parse_time_to_seconds(parts[1].trim()).ok()?;
    let end = parse_time_to_seconds(parts[2].trim()).ok()?;
    let text = parts[9..].join(",").trim().to_string();

    Some(DialogueTiming { start, end, text })
}

fn parse_time_to_seconds(time_str: &str) -> Result<f64, ()> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() != 3 {
        return Err(());
    }

    let hours: f64 = parts[0].parse().map_err(|_| ())?;
    let minutes: f64 = parts[1].parse().map_err(|_| ())?;

    let sec_parts: Vec<&str> = parts[2].split('.').collect();
    if sec_parts.len() != 2 {
        return Err(());
    }

    let seconds: f64 = sec_parts[0].parse().map_err(|_| ())?;
    let centiseconds: f64 = sec_parts[1].parse().map_err(|_| ())?;

    Ok(hours * 3600.0 + minutes * 60.0 + seconds + centiseconds / 100.0)
}

fn format_seconds_to_time(seconds: f64) -> String {
    let total_seconds = seconds as u64;
    let centiseconds = ((seconds - total_seconds as f64) * 100.0) as u64;

    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let secs = total_seconds % 60;

    format!("{}:{:02}:{:02}.{:02}", hours, minutes, secs, centiseconds)
}

/// Utility function to parse ASS without creating a renderer
#[wasm_bindgen]
pub fn parse_ass_script(ass_bytes: &[u8]) -> Result<JsValue, JsError> {
    let script = Script::parse(ass_bytes);
    let serialized = script.serialize();

    Ok(JsValue::from_str(&serialized))
}

/// Real-time subtitle processor for live streaming
#[wasm_bindgen]
pub struct RealTimeProcessor {
    renderer: RendererHandle,
    performance_monitor: PerformanceMonitor,
    target_fps: f64,
    frame_interval: f64,
    last_frame_time: f64,
    render_ahead_buffer: Vec<(f64, Vec<u8>)>, // (time, frame_data)
    buffer_size: usize,
}

#[wasm_bindgen]
impl RealTimeProcessor {
    /// Create a new real-time processor
    #[wasm_bindgen(constructor)]
    pub fn new(
        ass_bytes: &[u8],
        font_bytes: &[u8],
        target_fps: f64,
        buffer_size: Option<usize>,
    ) -> Result<RealTimeProcessor, JsValue> {
        let renderer = RendererHandle::new(ass_bytes, font_bytes)?;
        let performance_monitor = PerformanceMonitor::new(100);
        let buffer_size = buffer_size.unwrap_or(30); // Default 30 frame buffer

        Ok(RealTimeProcessor {
            renderer,
            performance_monitor,
            target_fps,
            frame_interval: 1.0 / target_fps,
            last_frame_time: 0.0,
            render_ahead_buffer: Vec::with_capacity(buffer_size),
            buffer_size,
        })
    }

    /// Process and render a frame with timing optimization
    #[wasm_bindgen]
    pub fn process_frame(
        &mut self,
        time_sec: f64,
        width: u32,
        height: u32,
        font_size: f32,
    ) -> js_sys::Uint8Array {
        self.performance_monitor.start_frame();

        // Check if we have a pre-rendered frame in the buffer
        if let Some(index) = self.render_ahead_buffer
            .iter()
            .position(|(t, _)| (t - time_sec).abs() < self.frame_interval / 2.0)
        {
            let (_, frame_data) = self.render_ahead_buffer.remove(index);
            self.performance_monitor.end_frame();
            return js_sys::Uint8Array::from(&frame_data[..]);
        }

        // Render frame on demand
        let frame_data = self.renderer.render_rgba(time_sec, width, height, font_size);
        
        // Render ahead for smooth playback
        self.render_ahead_frames(time_sec, width, height, font_size);

        self.performance_monitor.end_frame();
        js_sys::Uint8Array::from(&frame_data[..])
    }

    /// Render frames ahead of time for smooth playback
    fn render_ahead_frames(&mut self, current_time: f64, width: u32, height: u32, font_size: f32) {
        // Clean old frames from buffer
        self.render_ahead_buffer.retain(|(t, _)| *t > current_time - self.frame_interval);

        // Render ahead only if buffer has space and performance allows
        if self.render_ahead_buffer.len() < self.buffer_size 
            && self.performance_monitor.get_average_frame_time() < self.frame_interval * 0.5 {
            
            let ahead_time = current_time + (self.render_ahead_buffer.len() + 1) as f64 * self.frame_interval;
            
            // Only render if there are subtitles at this time
            if self.renderer.has_subtitles_at_time(ahead_time) {
                let frame_data = self.renderer.render_rgba(ahead_time, width, height, font_size);
                self.render_ahead_buffer.push((ahead_time, frame_data));
            }
        }
    }

    /// Set the target FPS for processing
    #[wasm_bindgen]
    pub fn set_target_fps(&mut self, fps: f64) {
        self.target_fps = fps;
        self.frame_interval = 1.0 / fps;
    }

    /// Get real-time performance statistics
    #[wasm_bindgen]
    pub fn get_performance_stats(&self) -> String {
        serde_json::json!({
            "current_fps": self.performance_monitor.get_fps(),
            "average_frame_time": self.performance_monitor.get_average_frame_time(),
            "target_fps": self.target_fps,
            "buffer_size": self.render_ahead_buffer.len(),
            "max_buffer_size": self.buffer_size,
        }).to_string()
    }

    /// Process multiple frames in batch for better performance
    #[wasm_bindgen]
    pub fn process_batch(
        &mut self,
        start_time: f64,
        frame_count: u32,
        width: u32,
        height: u32,
        font_size: f32,
    ) -> js_sys::Array {
        let result = js_sys::Array::new();
        
        for i in 0..frame_count {
            let time = start_time + i as f64 * self.frame_interval;
            let frame = self.process_frame(time, width, height, font_size);
            result.push(&frame);
        }
        
        result
    }

    /// Clear render-ahead buffer
    #[wasm_bindgen]
    pub fn clear_buffer(&mut self) {
        self.render_ahead_buffer.clear();
    }
}

/// WebGL-based renderer for hardware acceleration
#[wasm_bindgen]
pub struct WebGLRenderer {
    canvas: HtmlCanvasElement,
    gl_context: web_sys::WebGlRenderingContext,
    program: web_sys::WebGlProgram,
    texture: web_sys::WebGlTexture,
    renderer: RendererHandle,
}

#[wasm_bindgen]
impl WebGLRenderer {
    /// Create a new WebGL renderer
    #[wasm_bindgen(constructor)]
    pub fn new(
        canvas: HtmlCanvasElement,
        ass_bytes: &[u8],
        font_bytes: &[u8],
    ) -> Result<WebGLRenderer, JsValue> {
        let gl_context = canvas
            .get_context("webgl")?
            .unwrap()
            .dyn_into::<web_sys::WebGlRenderingContext>()?;

        let renderer = RendererHandle::new(ass_bytes, font_bytes)?;

        // Create shader program
        let program = create_shader_program(&gl_context)?;
        
        // Create texture
        let texture = gl_context.create_texture().unwrap();

        Ok(WebGLRenderer {
            canvas,
            gl_context,
            program,
            texture,
            renderer,
        })
    }

    /// Render using WebGL for better performance
    #[wasm_bindgen]
    pub fn render_webgl(
        &mut self,
        time_sec: f64,
        font_size: f32,
    ) -> Result<(), JsValue> {
        let width = self.canvas.width();
        let height = self.canvas.height();

        // Get subtitle data
        let subtitle_data = self.renderer.render_rgba(time_sec, width, height, font_size);

        // Upload to GPU texture
        self.gl_context.bind_texture(web_sys::WebGlRenderingContext::TEXTURE_2D, Some(&self.texture));
        
        self.gl_context.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            web_sys::WebGlRenderingContext::TEXTURE_2D,
            0,
            web_sys::WebGlRenderingContext::RGBA as i32,
            width as i32,
            height as i32,
            0,
            web_sys::WebGlRenderingContext::RGBA,
            web_sys::WebGlRenderingContext::UNSIGNED_BYTE,
            Some(&subtitle_data),
        )?;

        // Render the texture
        self.gl_context.use_program(Some(&self.program));
        self.gl_context.viewport(0, 0, width as i32, height as i32);
        self.gl_context.draw_arrays(web_sys::WebGlRenderingContext::TRIANGLE_STRIP, 0, 4);

        Ok(())
    }
}

/// Create a simple shader program for texture rendering
fn create_shader_program(gl: &web_sys::WebGlRenderingContext) -> Result<web_sys::WebGlProgram, JsValue> {
    let vertex_shader_source = r#"
        attribute vec2 a_position;
        attribute vec2 a_texCoord;
        varying vec2 v_texCoord;
        
        void main() {
            gl_Position = vec4(a_position, 0.0, 1.0);
            v_texCoord = a_texCoord;
        }
    "#;

    let fragment_shader_source = r#"
        precision mediump float;
        varying vec2 v_texCoord;
        uniform sampler2D u_texture;
        
        void main() {
            gl_FragColor = texture2D(u_texture, v_texCoord);
        }
    "#;

    let vertex_shader = compile_shader(gl, web_sys::WebGlRenderingContext::VERTEX_SHADER, vertex_shader_source)?;
    let fragment_shader = compile_shader(gl, web_sys::WebGlRenderingContext::FRAGMENT_SHADER, fragment_shader_source)?;

    let program = gl.create_program().unwrap();
    gl.attach_shader(&program, &vertex_shader);
    gl.attach_shader(&program, &fragment_shader);
    gl.link_program(&program);

    if !gl.get_program_parameter(&program, web_sys::WebGlRenderingContext::LINK_STATUS).as_bool().unwrap() {
        return Err(JsValue::from_str("Failed to link shader program"));
    }

    Ok(program)
}

/// Compile a shader
fn compile_shader(
    gl: &web_sys::WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<web_sys::WebGlShader, JsValue> {
    let shader = gl.create_shader(shader_type).unwrap();
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if !gl.get_shader_parameter(&shader, web_sys::WebGlRenderingContext::COMPILE_STATUS).as_bool().unwrap() {
        let error = gl.get_shader_info_log(&shader).unwrap_or_else(|| "Unknown error".to_string());
        return Err(JsValue::from_str(&format!("Shader compilation error: {}", error)));
    }

    Ok(shader)
}

/// Performance monitoring utilities
#[wasm_bindgen]
pub struct PerformanceMonitor {
    start_time: f64,
    frame_times: Vec<f64>,
    max_samples: usize,
}

#[wasm_bindgen]
impl PerformanceMonitor {
    #[wasm_bindgen(constructor)]
    pub fn new(max_samples: usize) -> PerformanceMonitor {
        PerformanceMonitor {
            start_time: web_sys::window().unwrap().performance().unwrap().now(),
            frame_times: Vec::with_capacity(max_samples),
            max_samples,
        }
    }

    #[wasm_bindgen]
    pub fn start_frame(&mut self) {
        self.start_time = web_sys::window().unwrap().performance().unwrap().now();
    }

    #[wasm_bindgen]
    pub fn end_frame(&mut self) {
        let end_time = web_sys::window().unwrap().performance().unwrap().now();

        let frame_time = end_time - self.start_time;

        if self.frame_times.len() >= self.max_samples {
            self.frame_times.remove(0);
        }
        self.frame_times.push(frame_time);
    }

    #[wasm_bindgen]
    pub fn get_average_frame_time(&self) -> f64 {
        if self.frame_times.is_empty() {
            0.0
        } else {
            self.frame_times.iter().sum::<f64>() / self.frame_times.len() as f64
        }
    }

    #[wasm_bindgen]
    pub fn get_fps(&self) -> f64 {
        let avg_time = self.get_average_frame_time();
        if avg_time > 0.0 {
            1000.0 / avg_time
        } else {
            0.0
        }
    }
}

/// WASM-specific error handling
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

/// Log performance information to console
#[wasm_bindgen]
pub fn log_performance(message: &str) {
    console::log_1(&format!("ASS-RS Performance: {}", message).into());
}
