//! Software (CPU) rendering backend using tiny-skia

#[cfg(feature = "nostd")]
use alloc::{boxed::Box, format, vec, vec::Vec};
#[cfg(not(feature = "nostd"))]
use std::{boxed::Box, vec::Vec};

use crate::backends::{BackendFeature, BackendType, RenderBackend};
use crate::cache::{RenderCache, TextCacheKey};
use crate::pipeline::{IntermediateLayer, Pipeline, SoftwarePipeline};
use crate::renderer::RenderContext;
use crate::utils::{DirtyRegion, RenderError};
use tiny_skia::{Pixmap, Transform};

/// Software rendering backend using tiny-skia
pub struct SoftwareBackend {
    pixmap: Pixmap,
    font_database: fontdb::Database,
    glyph_renderer: crate::pipeline::shaping::GlyphRenderer,
    cache: RenderCache,
    #[cfg(feature = "backend-metrics")]
    metrics: super::BackendMetrics,
}

impl SoftwareBackend {
    /// Create a new software backend
    pub fn new(context: &RenderContext) -> Result<Self, RenderError> {
        let pixmap =
            Pixmap::new(context.width(), context.height()).ok_or(RenderError::InvalidDimensions)?;

        // Initialize font database once and load system fonts
        let mut font_database = fontdb::Database::new();
        font_database.load_system_fonts();

        // Also load common fallback fonts and ensure Japanese font support
        #[cfg(not(feature = "nostd"))]
        {
            // Try to load additional Unicode fonts for better coverage
            if let Ok(home) = std::env::var("HOME") {
                let font_dirs = [
                    format!("{}/.fonts", home),
                    "/usr/share/fonts".to_string(),
                    "/usr/local/share/fonts".to_string(),
                    "/System/Library/Fonts".to_string(), // macOS
                    "/System/Library/Fonts/Supplemental".to_string(), // macOS CJK fonts
                    "/Library/Fonts".to_string(),        // macOS user fonts
                    "C:\\Windows\\Fonts".to_string(),    // Windows
                ];

                for dir in &font_dirs {
                    if std::path::Path::new(dir).exists() {
                        font_database.load_fonts_dir(dir);
                    }
                }
            }
        }

        // Check for common Japanese fonts (silently)
        #[cfg(debug_assertions)]
        {
            let japanese_fonts = [
                "Noto Sans CJK JP",
                "Hiragino Sans",
                "Hiragino Kaku Gothic Pro",
                "Yu Gothic",
                "MS Gothic",
                "Meiryo",
                "Noto Sans JP",
            ];
            let mut found_japanese = false;
            for font_name in &japanese_fonts {
                let query = fontdb::Query {
                    families: &[fontdb::Family::Name(font_name)],
                    weight: fontdb::Weight::NORMAL,
                    stretch: fontdb::Stretch::Normal,
                    style: fontdb::Style::Normal,
                };
                if font_database.query(&query).is_some() {
                    found_japanese = true;
                    break;
                }
            }

            if !found_japanese {
                #[cfg(not(feature = "nostd"))]
                eprintln!("Warning: No Japanese fonts found. Japanese text may not render correctly.");
            }
        }

        Ok(Self {
            pixmap,
            font_database,
            glyph_renderer: crate::pipeline::shaping::GlyphRenderer::new(),
            cache: RenderCache::with_limits(2000, 1000), // Increased cache limits for better performance
            #[cfg(feature = "backend-metrics")]
            metrics: super::BackendMetrics::new(),
        })
    }

    /// Resize the backend pixmap
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), RenderError> {
        self.pixmap = Pixmap::new(width, height).ok_or(RenderError::InvalidDimensions)?;
        Ok(())
    }

    fn composite_layer(
        &mut self,
        layer: &IntermediateLayer,
        _context: &RenderContext,
    ) -> Result<(), RenderError> {
        match layer {
            IntermediateLayer::Raster(raster_data) => {
                self.draw_raster_layer(raster_data)?;
            }
            IntermediateLayer::Vector(path_data) => {
                self.draw_vector_layer(path_data)?;
            }
            IntermediateLayer::Text(text_data) => {
                self.draw_text_layer(text_data)?;
            }
        }
        Ok(())
    }

    fn draw_raster_layer(&mut self, data: &crate::pipeline::RasterData) -> Result<(), RenderError> {
        if data.pixels.len() != (data.width * data.height * 4) as usize {
            return Err(RenderError::InvalidBufferSize {
                expected: (data.width * data.height * 4) as usize,
                actual: data.pixels.len(),
            });
        }

        let src_pixmap = Pixmap::from_vec(
            data.pixels.clone(),
            tiny_skia::IntSize::from_wh(data.width, data.height)
                .ok_or(RenderError::InvalidDimensions)?,
        )
        .ok_or(RenderError::InvalidPixmap)?;

        let transform = Transform::from_translate(data.x as f32, data.y as f32);

        // Use SourceOver blend mode for proper alpha compositing
        let mut paint = tiny_skia::PixmapPaint::default();
        paint.blend_mode = tiny_skia::BlendMode::SourceOver;
        
        self.pixmap.draw_pixmap(
            0,
            0,
            src_pixmap.as_ref(),
            &paint,
            transform,
            None,
        );

        Ok(())
    }

    fn draw_vector_layer(&mut self, data: &crate::pipeline::VectorData) -> Result<(), RenderError> {
        eprintln!("DRAWING RENDER: draw_vector_layer called with color {:?}", data.color);
        
        let mut paint = tiny_skia::Paint::default();
        // Ensure we're setting color with proper alpha handling
        // tiny-skia expects premultiplied alpha internally
        paint.set_color_rgba8(data.color[0], data.color[1], data.color[2], data.color[3]);
        paint.anti_alias = true;
        paint.blend_mode = tiny_skia::BlendMode::SourceOver;

        if let Some(path) = &data.path {
            eprintln!("DRAWING RENDER: Filling path with color RGBA({}, {}, {}, {})", 
                data.color[0], data.color[1], data.color[2], data.color[3]);
            
            eprintln!("DRAWING RENDER: Path bounds: {:?}", path.bounds());
            
            self.pixmap.fill_path(
                path,
                &paint,
                tiny_skia::FillRule::Winding,
                Transform::identity(),
                None,
            );
        }

        if let Some(stroke) = &data.stroke {
            paint.set_color_rgba8(
                stroke.color[0],
                stroke.color[1],
                stroke.color[2],
                stroke.color[3],
            );

            let mut sk_stroke = tiny_skia::Stroke::default();
            sk_stroke.width = stroke.width;

            if let Some(path) = &data.path {
                self.pixmap
                    .stroke_path(path, &paint, &sk_stroke, Transform::identity(), None);
            }
        }

        Ok(())
    }

    fn draw_text_layer(&mut self, data: &crate::pipeline::TextData) -> Result<(), RenderError> {
        use crate::pipeline::shaping::shape_text_with_style;

        // Extract bold/italic from effects
        let bold = data
            .effects
            .iter()
            .any(|e| matches!(e, crate::pipeline::TextEffect::Bold));
        let italic = data
            .effects
            .iter()
            .any(|e| matches!(e, crate::pipeline::TextEffect::Italic));
        let underline = data
            .effects
            .iter()
            .any(|e| matches!(e, crate::pipeline::TextEffect::Underline));
        let strikethrough = data
            .effects
            .iter()
            .any(|e| matches!(e, crate::pipeline::TextEffect::Strikethrough));

        // Create cache key
        let cache_key = TextCacheKey {
            text: data.text.clone(),
            font_family: data.font_family.clone(),
            font_size: data.font_size as u32,
            bold,
            italic,
        };

        // Try to get from cache or shape the text
        let shaped = if let Some(cached) = self.cache.get_shaped_text(&cache_key) {
            cached
        } else {
            // Shape the text
            let shaped_text = shape_text_with_style(
                &data.text,
                &data.font_family,
                data.font_size,
                bold,
                italic,
                &self.font_database,
            )?;
            self.cache.store_shaped_text(cache_key, shaped_text)
        };

        // Find font for rendering
        let query = fontdb::Query {
            families: &[
                fontdb::Family::Name(&data.font_family),
                fontdb::Family::SansSerif,
            ],
            weight: if bold {
                fontdb::Weight::BOLD
            } else {
                fontdb::Weight::NORMAL
            },
            stretch: fontdb::Stretch::Normal,
            style: if italic {
                fontdb::Style::Italic
            } else {
                fontdb::Style::Normal
            },
        };

        let font_id = self.font_database.query(&query).ok_or_else(|| {
            RenderError::FontError(format!("Font '{}' not found", data.font_family))
        })?;

        // Render glyphs to paths using cached renderer with spacing
        let paths =
            self.glyph_renderer
                .render_shaped_text(&*shaped, font_id, &self.font_database, data.spacing)?;
        
        #[cfg(all(debug_assertions, not(feature = "nostd")))]
        if data.text.contains("Чысценькая") {
            eprintln!("  Rendered {} glyph paths", paths.len());
        }

        // Build base transform with rotation and scaling
        // The data.x and data.y are the top-left corner of the text box
        // But glyphs are positioned from their baseline, so we need to adjust y by adding the baseline offset
        let baseline_y = data.y + (*shaped).baseline;
        
        #[cfg(all(debug_assertions, not(feature = "nostd")))]
        {
            // Safely truncate text for debug output, respecting UTF-8 boundaries
            let debug_text = if data.text.len() > 20 {
                let mut end = 20;
                while !data.text.is_char_boundary(end) && end > 0 {
                    end -= 1;
                }
                &data.text[..end]
            } else {
                &data.text
            };
            eprintln!("Drawing text: '{}' at ({}, {}), baseline={}, baseline_y={}", 
                debug_text, data.x, data.y, (*shaped).baseline, baseline_y);
            if baseline_y > 1080.0 || baseline_y < -100.0 {
                eprintln!("WARNING: Baseline off-screen! data.y={}, baseline={}, baseline_y={}", 
                    data.y, (*shaped).baseline, baseline_y);
            }
        }
        
        let mut base_transform = Transform::from_translate(data.x, baseline_y);

        // Check for rotation, scaling, and shear effects
        #[cfg(all(debug_assertions, not(feature = "nostd")))]
        if data.text.contains("Чысценькая") {
            eprintln!("  Applying {} effects to text", data.effects.len());
            for effect in &data.effects {
                eprintln!("    Effect: {:?}", effect);
            }
        }
        
        for effect in &data.effects {
            match effect {
                crate::pipeline::TextEffect::Rotation { x, y, z } => {
                    // Apply Z rotation (2D rotation) - convert degrees to radians
                    if *z != 0.0 {
                        let angle_rad = z * core::f32::consts::PI / 180.0;
                        // To rotate around text center, we need to:
                        // 1. Translate back to origin
                        // 2. Rotate
                        // 3. Translate back to position
                        // Get the center of the text for rotation
                        // The text is positioned at its baseline, so we need to calculate center
                        // relative to the text's actual bounding box
                        let text_center_x = (*shaped).width / 2.0;
                        let text_center_y = (*shaped).height / 2.0;
                        
                        // Move center to origin, rotate, then move back
                        base_transform = base_transform
                            .pre_translate(text_center_x, text_center_y)
                            .pre_rotate(angle_rad)
                            .pre_translate(-text_center_x, -text_center_y);
                        
                        #[cfg(all(debug_assertions, not(feature = "nostd")))]
                        eprintln!("Applied rotation: {} degrees ({} radians) around center ({}, {})", 
                            z, angle_rad, text_center_x, text_center_y);
                    }

                    // Approximate 3D rotations with skew transformations
                    // X rotation (rotation around horizontal axis) - affects vertical skew
                    if *x != 0.0 {
                        let angle_rad = x * core::f32::consts::PI / 180.0;
                        // Use perspective approximation: tan(angle) for small angles
                        let skew_y = angle_rad.sin() * 0.5; // Scale down for visual effect
                        base_transform =
                            Transform::from_skew(0.0, skew_y).pre_concat(base_transform);
                    }

                    // Y rotation (rotation around vertical axis) - affects horizontal skew
                    if *y != 0.0 {
                        let angle_rad = y * core::f32::consts::PI / 180.0;
                        // Use perspective approximation
                        let skew_x = angle_rad.sin() * 0.5; // Scale down for visual effect
                        base_transform =
                            Transform::from_skew(skew_x, 0.0).pre_concat(base_transform);
                    }
                }
                crate::pipeline::TextEffect::Scale { x, y } => {
                    // Font Y-scale is already applied to the font size during shaping
                    // But X-scale needs to be applied as a transform since fonts don't support
                    // asymmetric scaling through size alone
                    // Apply X-scale transform if it's different from Y-scale
                    let x_scale = *x / 100.0;
                    let y_scale = *y / 100.0;
                    
                    #[cfg(all(debug_assertions, not(feature = "nostd")))]
                    eprintln!("SCALE: Applying scale transform - x={:.2}, y={:.2} for text '{}'", 
                        x_scale, y_scale, data.text);
                    
                    // Apply scale transform
                    // Note: Y-scale is already partially applied to font size during shaping,
                    // but we still need to apply the transform for proper scaling
                    if (x_scale - 1.0).abs() > 0.01 || (y_scale - 1.0).abs() > 0.01 {
                        // Get the center of the text for scaling
                        let text_center_x = (*shaped).width / 2.0;
                        let text_center_y = (*shaped).height / 2.0 - (*shaped).baseline;
                        
                        base_transform = base_transform
                            .pre_translate(text_center_x, text_center_y)
                            .pre_scale(x_scale, 1.0)  // X-scale, Y is in font size already
                            .pre_translate(-text_center_x, -text_center_y);
                    }
                }
                crate::pipeline::TextEffect::Shear { x, y } => {
                    // Apply shear transformation (for \fax and \fay tags)
                    base_transform = Transform::from_skew(*x, *y).pre_concat(base_transform);
                }
                _ => {}
            }
        }

        // Create clip mask if needed
        let clip_mask = data.effects.iter().find_map(|e| {
            if let crate::pipeline::TextEffect::Clip {
                x1,
                y1,
                x2,
                y2,
                inverse,
            } = e
            {
                // Create a mask for clipping
                let width = self.pixmap.width();
                let height = self.pixmap.height();

                if let Some(mut mask) = tiny_skia::Mask::new(width, height) {
                    // Fill the mask with the clipping region
                    let mut builder = tiny_skia::PathBuilder::new();
                    builder.move_to(*x1, *y1);
                    builder.line_to(*x2, *y1);
                    builder.line_to(*x2, *y2);
                    builder.line_to(*x1, *y2);
                    builder.close();

                    if let Some(clip_path) = builder.finish() {
                        // Fill the mask with the appropriate pattern
                        mask.fill_path(
                            &clip_path,
                            tiny_skia::FillRule::Winding,
                            !*inverse, // For normal clip, fill inside; for inverse, fill outside
                            Transform::identity(),
                        );

                        return Some(mask);
                    }
                }
                None
            } else {
                None
            }
        });

        // Apply effects in order: shadow, outline, then main text
        for effect in &data.effects {
            match effect {
                crate::pipeline::TextEffect::Shadow {
                    color,
                    x_offset,
                    y_offset,
                } => {
                    // Draw shadow first
                    let mut shadow_paint = tiny_skia::Paint::default();
                    shadow_paint.set_color_rgba8(color[0], color[1], color[2], color[3]);
                    shadow_paint.anti_alias = true;
                    shadow_paint.blend_mode = tiny_skia::BlendMode::SourceOver;

                    let shadow_transform = base_transform.pre_translate(*x_offset, *y_offset);

                    for path in &paths {
                        if let Some(transformed) = path.clone().transform(shadow_transform) {
                            self.pixmap.fill_path(
                                &transformed,
                                &shadow_paint,
                                tiny_skia::FillRule::Winding,
                                Transform::identity(),
                                clip_mask.as_ref(),
                            );
                        }
                    }
                }
                _ => {}
            }
        }

        // Check for edge blur effect (applies to outline only)
        let edge_blur_radius = data.effects.iter().find_map(|e| {
            if let crate::pipeline::TextEffect::EdgeBlur { radius } = e {
                Some(*radius)
            } else {
                None
            }
        });

        // Draw outline if present
        for effect in &data.effects {
            match effect {
                crate::pipeline::TextEffect::Outline { color, width } => {
                    let mut outline_paint = tiny_skia::Paint::default();
                    outline_paint.set_color_rgba8(color[0], color[1], color[2], color[3]);
                    outline_paint.anti_alias = true;
                    outline_paint.blend_mode = tiny_skia::BlendMode::SourceOver;

                    // Create stroke configuration for path expansion
                    let mut stroke = tiny_skia::Stroke::default();
                    stroke.width = *width * 0.6; // Further reduce width to match libass
                    stroke.line_cap = tiny_skia::LineCap::Square;
                    stroke.line_join = tiny_skia::LineJoin::Miter;

                    // If edge blur is needed, render outline to temporary pixmap first
                    if let Some(blur_radius) = edge_blur_radius {
                        if blur_radius > 0.0 {
                            let blur_size = (blur_radius * 2.0).ceil() as u32;
                            let outline_width =
                                ((*shaped).width + blur_size as f32 * 2.0 + *width * 2.0).ceil()
                                    as u32;
                            let outline_height =
                                ((*shaped).height + blur_size as f32 * 2.0 + *width * 2.0).ceil()
                                    as u32;

                            if let Some(mut temp_pixmap) =
                                Pixmap::new(outline_width, outline_height)
                            {
                                temp_pixmap.fill(tiny_skia::Color::TRANSPARENT);

                                // Draw outline to temporary pixmap
                                let temp_transform = Transform::from_translate(
                                    blur_size as f32 + *width,
                                    blur_size as f32 + *width,
                                );

                                let mut stroker = tiny_skia::PathStroker::new();
                                for path in &paths {
                                    if let Some(transformed) =
                                        path.clone().transform(temp_transform)
                                    {
                                        // Expand the path to create an outline shape
                                        if let Some(outlined_path) = stroker.stroke(&transformed, &stroke, 1.0) {
                                            // Fill the expanded outline path
                                            temp_pixmap.fill_path(
                                                &outlined_path,
                                                &outline_paint,
                                                tiny_skia::FillRule::Winding,
                                                Transform::identity(),
                                                None,
                                            );
                                        }
                                    }
                                }

                                // Apply blur to the outline
                                apply_box_blur(&mut temp_pixmap, blur_radius);

                                // Draw blurred outline to main pixmap
                                let blend_transform = base_transform.pre_translate(
                                    -(blur_size as f32) - *width,
                                    -(blur_size as f32) - *width,
                                );

                                let mut paint = tiny_skia::PixmapPaint::default();
                                paint.blend_mode = tiny_skia::BlendMode::SourceOver;
                                
                                self.pixmap.draw_pixmap(
                                    0,
                                    0,
                                    temp_pixmap.as_ref(),
                                    &paint,
                                    blend_transform,
                                    clip_mask.as_ref(),
                                );
                            }
                        }
                    } else {
                        // Draw outline using path expansion (like libass)
                        // This creates a filled outline rather than a stroked one
                        let mut stroker = tiny_skia::PathStroker::new();
                        
                        for path in &paths {
                            if let Some(transformed) = path.clone().transform(base_transform) {
                                // Expand the path to create an outline shape
                                if let Some(outlined_path) = stroker.stroke(&transformed, &stroke, 1.0) {
                                    // Fill the expanded outline path
                                    self.pixmap.fill_path(
                                        &outlined_path,
                                        &outline_paint,
                                        tiny_skia::FillRule::Winding,
                                        Transform::identity(),
                                        clip_mask.as_ref(),
                                    );
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // Draw main text
        let mut text_paint = tiny_skia::Paint::default();
        text_paint.set_color_rgba8(data.color[0], data.color[1], data.color[2], data.color[3]);
        text_paint.anti_alias = true;
        text_paint.blend_mode = tiny_skia::BlendMode::SourceOver;
        
        #[cfg(all(debug_assertions, not(feature = "nostd")))]
        eprintln!("Drawing main text with color: R={}, G={}, B={}, A={}", 
            data.color[0], data.color[1], data.color[2], data.color[3]);

        // Check for blur effect
        let blur_radius = data.effects.iter().find_map(|e| {
            if let crate::pipeline::TextEffect::Blur { radius } = e {
                Some(*radius)
            } else {
                None
            }
        });

        // Check for karaoke effect
        let karaoke_info = data.effects.iter().find_map(|e| {
            if let crate::pipeline::TextEffect::Karaoke { progress, style } = e {
                #[cfg(all(debug_assertions, not(feature = "nostd")))]
                eprintln!("KARAOKE DETECTED: progress={}, style={}", progress, style);
                Some((*progress, *style))
            } else {
                None
            }
        });

        // Use base_transform built above with rotation/scaling
        let text_transform = base_transform;

        // Apply blur if needed
        if let Some(radius) = blur_radius {
            // Create a temporary pixmap for blurred text
            let blur_size = (radius * 2.0).ceil() as u32;
            let text_width = ((*shaped).width + blur_size as f32 * 2.0).ceil() as u32;
            let text_height = ((*shaped).height + blur_size as f32 * 2.0).ceil() as u32;

            if let Some(mut temp_pixmap) = Pixmap::new(text_width, text_height) {
                temp_pixmap.fill(tiny_skia::Color::TRANSPARENT);

                // Draw text to temporary pixmap
                let temp_transform = Transform::from_translate(blur_size as f32, blur_size as f32);
                for path in &paths {
                    if let Some(transformed) = path.clone().transform(temp_transform) {
                        temp_pixmap.fill_path(
                            &transformed,
                            &text_paint,
                            tiny_skia::FillRule::Winding,
                            Transform::identity(),
                            clip_mask.as_ref(),
                        );
                    }
                }

                // Apply simple box blur
                apply_box_blur(&mut temp_pixmap, radius);

                // Draw blurred result to main pixmap
                let blend_transform =
                    Transform::from_translate(data.x - blur_size as f32, data.y - blur_size as f32);
                let mut paint = tiny_skia::PixmapPaint::default();
                paint.blend_mode = tiny_skia::BlendMode::SourceOver;
                
                self.pixmap.draw_pixmap(
                    0,
                    0,
                    temp_pixmap.as_ref(),
                    &paint,
                    blend_transform,
                    None,
                );
            }
        } else if let Some((progress, karaoke_style)) = karaoke_info {
            // Draw with karaoke effect based on style
            
            #[cfg(all(debug_assertions, not(feature = "nostd")))]
            eprintln!("KARAOKE RENDERING: progress={}, style={}, original color=({},{},{},{})", 
                progress, karaoke_style, data.color[0], data.color[1], data.color[2], data.color[3]);
            
            let mut karaoke_paint = tiny_skia::Paint::default();
            
            // For basic karaoke (\k), it's binary: either sung or not sung
            // Progress > 0 means the syllable timing has started
            // For \kf and \K, we'd need sweep effect (not fully implemented yet)
            
            if karaoke_style == 0 {  // Basic karaoke (\k)
                // Binary switching based on progress
                // During the syllable duration (0 < progress < 1), show as sung
                // After (progress >= 1), keep as sung
                if progress > 0.0 {
                    // Sung - use highlight color (typically secondary color, we'll use yellow)
                    karaoke_paint.set_color_rgba8(255, 255, 0, data.color[3]); // Yellow
                    #[cfg(all(debug_assertions, not(feature = "nostd")))]
                    eprintln!("KARAOKE COLOR: Sung - Yellow (255,255,0,{})", data.color[3]);
                } else {
                    // Not yet sung - use original color  
                    karaoke_paint.set_color_rgba8(data.color[0], data.color[1], data.color[2], data.color[3]);
                    #[cfg(all(debug_assertions, not(feature = "nostd")))]
                    eprintln!("KARAOKE COLOR: Not sung - Original ({},{},{},{})", 
                        data.color[0], data.color[1], data.color[2], data.color[3]);
                }
            } else {
                // For other styles (fill, outline, sweep), use simple approach for now
                // TODO: Implement proper sweep effect
                if progress >= 1.0 {
                    karaoke_paint.set_color_rgba8(255, 255, 0, data.color[3]);
                } else if progress <= 0.0 {
                    karaoke_paint.set_color_rgba8(data.color[0], data.color[1], data.color[2], data.color[3]);
                } else {
                    // Simple interpolation as placeholder
                    let r = (data.color[0] as f32 * (1.0 - progress) + 255.0 * progress) as u8;
                    let g = (data.color[1] as f32 * (1.0 - progress) + 255.0 * progress) as u8;
                    let b = (data.color[2] as f32 * (1.0 - progress) + 0.0 * progress) as u8;
                    karaoke_paint.set_color_rgba8(r, g, b, data.color[3]);
                }
            }
            karaoke_paint.anti_alias = true;
            karaoke_paint.blend_mode = tiny_skia::BlendMode::SourceOver;
            
            // Draw text with karaoke color
            for path in &paths {
                if let Some(transformed) = path.clone().transform(text_transform) {
                    self.pixmap.fill_path(
                        &transformed,
                        &karaoke_paint,
                        tiny_skia::FillRule::Winding,
                        Transform::identity(),
                        clip_mask.as_ref(),
                    );
                }
            }
            
        } else {
            // Draw without blur or karaoke
            #[cfg(all(debug_assertions, not(feature = "nostd")))]
            eprintln!("Drawing {} paths for main text at transform ({}, {})", 
                paths.len(), text_transform.tx, text_transform.ty);
            
            for (i, path) in paths.iter().enumerate() {
                if let Some(transformed) = path.clone().transform(text_transform) {
                    #[cfg(all(debug_assertions, not(feature = "nostd")))]
                    {
                        if data.text.contains("Чысценькая") {
                            let bounds = transformed.bounds();
                            eprintln!("  Glyph {}: bounds = ({:.1}, {:.1}, {:.1}, {:.1})", 
                                i, bounds.left(), bounds.top(), bounds.right(), bounds.bottom());
                        }
                        if i == 0 {
                            eprintln!("Drawing path 0 for main text, bounds: {:?}", transformed.bounds());
                        }
                    }
                    
                    self.pixmap.fill_path(
                        &transformed,
                        &text_paint,
                        tiny_skia::FillRule::Winding,
                        Transform::identity(),
                        clip_mask.as_ref(),
                    );
                }
            }
        }

        // Draw underline if present
        if underline {
            // Position underline according to libass formula: baseline + descent/2
            // baseline_y is already calculated as data.y + (*shaped).baseline
            // descent is negative, so we need to subtract half of its absolute value
            let underline_y = baseline_y - (*shaped).descent / 2.0;
            let mut builder = tiny_skia::PathBuilder::new();
            builder.move_to(data.x, underline_y);
            builder.line_to(data.x + (*shaped).width, underline_y);

            if let Some(underline_path) = builder.finish() {
                let mut stroke = tiny_skia::Stroke::default();
                stroke.width = data.font_size * 0.08;
                stroke.line_cap = tiny_skia::LineCap::Round;
                self.pixmap.stroke_path(
                    &underline_path,
                    &text_paint,
                    &stroke,
                    Transform::identity(),
                    clip_mask.as_ref(),
                );
            }
        }

        // Draw strikethrough if present
        if strikethrough {
            // Position strikethrough according to libass formula: baseline - ascent/3
            // baseline_y is already calculated as data.y + (*shaped).baseline
            let strike_y = baseline_y - (*shaped).ascent / 3.0;
            let mut builder = tiny_skia::PathBuilder::new();
            builder.move_to(data.x, strike_y);
            builder.line_to(data.x + (*shaped).width, strike_y);

            if let Some(strike_path) = builder.finish() {
                let mut stroke = tiny_skia::Stroke::default();
                stroke.width = data.font_size * 0.06;
                stroke.line_cap = tiny_skia::LineCap::Round;
                self.pixmap.stroke_path(
                    &strike_path,
                    &text_paint,
                    &stroke,
                    Transform::identity(),
                    clip_mask.as_ref(),
                );
            }
        }

        Ok(())
    }
}

/// Apply a simple box blur to a pixmap
fn apply_box_blur(pixmap: &mut Pixmap, radius: f32) {
    if radius <= 0.0 {
        return;
    }

    let radius = radius.round() as i32;
    let width = pixmap.width() as i32;
    let height = pixmap.height() as i32;
    let data = pixmap.data_mut();

    // Create temporary buffer for horizontal pass
    let mut temp = vec![0u8; data.len()];

    // Horizontal blur pass
    for y in 0..height {
        for x in 0..width {
            let mut r = 0u32;
            let mut g = 0u32;
            let mut b = 0u32;
            let mut a = 0u32;
            let mut count = 0u32;

            for dx in -radius..=radius {
                let sx = (x + dx).clamp(0, width - 1);
                let idx = ((y * width + sx) * 4) as usize;
                r += data[idx] as u32;
                g += data[idx + 1] as u32;
                b += data[idx + 2] as u32;
                a += data[idx + 3] as u32;
                count += 1;
            }

            let out_idx = ((y * width + x) * 4) as usize;
            temp[out_idx] = (r / count) as u8;
            temp[out_idx + 1] = (g / count) as u8;
            temp[out_idx + 2] = (b / count) as u8;
            temp[out_idx + 3] = (a / count) as u8;
        }
    }

    // Vertical blur pass
    for y in 0..height {
        for x in 0..width {
            let mut r = 0u32;
            let mut g = 0u32;
            let mut b = 0u32;
            let mut a = 0u32;
            let mut count = 0u32;

            for dy in -radius..=radius {
                let sy = (y + dy).clamp(0, height - 1);
                let idx = ((sy * width + x) * 4) as usize;
                r += temp[idx] as u32;
                g += temp[idx + 1] as u32;
                b += temp[idx + 2] as u32;
                a += temp[idx + 3] as u32;
                count += 1;
            }

            let out_idx = ((y * width + x) * 4) as usize;
            data[out_idx] = (r / count) as u8;
            data[out_idx + 1] = (g / count) as u8;
            data[out_idx + 2] = (b / count) as u8;
            data[out_idx + 3] = (a / count) as u8;
        }
    }
}

/// Apply optimized box blur using SIMD when available
#[cfg(feature = "simd")]
fn apply_box_blur_simd(pixmap: &mut Pixmap, radius: f32) {
    if radius <= 0.0 {
        return;
    }

    // Use SIMD instructions for faster blur
    // This is a placeholder - real SIMD implementation would use intrinsics
    apply_box_blur(pixmap, radius);
}

impl RenderBackend for SoftwareBackend {
    fn backend_type(&self) -> BackendType {
        BackendType::Software
    }

    fn create_pipeline(&self) -> Result<Box<dyn Pipeline>, RenderError> {
        Ok(Box::new(SoftwarePipeline::new()))
    }

    fn composite_layers(
        &self,
        layers: &[IntermediateLayer],
        context: &RenderContext,
    ) -> Result<Vec<u8>, RenderError> {
        // Create a new backend instance for this frame
        // This is intentional - the backend must be stateless due to Arc<dyn RenderBackend>
        let mut backend = Self::new(context)?;

        // Clear pixmap
        backend.pixmap.fill(tiny_skia::Color::TRANSPARENT);

        // Composite each layer
        for layer in layers {
            backend.composite_layer(layer, context)?;
        }

        // Return RGBA data
        let data = backend.pixmap.data().to_vec();
        
        #[cfg(all(debug_assertions, not(feature = "nostd")))]
        {
            // Sample some pixels to check alpha
            if data.len() >= 4 {
                let sample1 = &data[0..4];
                let sample2 = if data.len() >= 4000 { &data[4000..4004] } else { &data[0..4] };
                eprintln!("PIXMAP SAMPLES: [0]={:?}, [1000]={:?}", sample1, sample2);
                
                // Count non-opaque pixels
                let non_opaque = data.chunks_exact(4).filter(|p| p[3] != 255).count();
                let transparent = data.chunks_exact(4).filter(|p| p[3] == 0).count();
                eprintln!("PIXMAP ALPHA STATS: {} non-opaque pixels, {} transparent pixels out of {}", 
                    non_opaque, transparent, data.len() / 4);
            }
        }
        
        Ok(data)
    }

    fn composite_layers_incremental(
        &self,
        layers: &[IntermediateLayer],
        dirty_regions: &[DirtyRegion],
        previous_frame: &[u8],
        context: &RenderContext,
    ) -> Result<Vec<u8>, RenderError> {
        // Create a new backend instance for this frame
        let mut backend = Self::new(context)?;

        // Copy previous frame
        if previous_frame.len() == backend.pixmap.data().len() {
            backend.pixmap.data_mut().copy_from_slice(previous_frame);
        } else {
            backend.pixmap.fill(tiny_skia::Color::TRANSPARENT);
        }

        // Only redraw dirty regions
        for region in dirty_regions {
            // TODO: Create clip mask for dirty region
            // tiny_skia doesn't expose ClipMask publicly
            let _ = region; // TODO: Apply clipping

            // Composite layers within this region
            for layer in layers {
                if layer.intersects_region(region) {
                    backend.composite_layer(layer, context)?;
                }
            }
        }

        Ok(backend.pixmap.data().to_vec())
    }

    fn supports_feature(&self, feature: BackendFeature) -> bool {
        match feature {
            BackendFeature::IncrementalRendering => true,
            BackendFeature::HardwareAcceleration => false,
            BackendFeature::ComputeShaders => false,
            BackendFeature::AsyncRendering => false,
        }
    }

    #[cfg(feature = "backend-metrics")]
    fn metrics(&self) -> Option<super::BackendMetrics> {
        Some(self.metrics.clone())
    }
}
