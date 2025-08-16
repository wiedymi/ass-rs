//! New process_event implementation with proper segment handling

use crate::pipeline::{IntermediateLayer, TextData, TextEffect, VectorData};
use crate::pipeline::text_segmenter::segment_text_with_tags;
use crate::pipeline::shaping::shape_text_with_style;
use crate::pipeline::drawing::process_drawing_commands;
use crate::pipeline::animation::{calculate_move_progress, calculate_fade_progress};
use crate::renderer::RenderContext;
use crate::utils::RenderError;
use ass_core::parser::Event;
use smallvec::SmallVec;

impl super::SoftwarePipeline {
    /// Process a single event into layers with proper text segmentation
    pub fn process_event_segmented(
        &mut self,
        event: &Event,
        time_cs: u32,
        context: &RenderContext,
    ) -> Result<Vec<IntermediateLayer>, RenderError> {
        // Get resolved style from analysis if available
        let resolved_style = self.analysis.as_ref().and_then(|analysis| {
            analysis.resolve_style(&event.style)
        });
        
        // Get text segments with their individual tags
        let segments = segment_text_with_tags(&event.text, None)?;
        
        if segments.is_empty() {
            return Ok(Vec::new());
        }
        
        // Check if this is a drawing command (should be in first segment)
        if let Some(draw_level) = segments[0].tags.drawing_mode {
            if draw_level > 0 {
                // Process drawing command
                let plain_text = segments[0].text.clone();
                let tags = &segments[0].tags;
                
                let path_opt = process_drawing_commands(&plain_text)?;
                
                if let Some(path) = path_opt {
                    let color = tags.colors.primary.unwrap_or_else(|| {
                        resolved_style
                            .map(|s| s.primary_color())
                            .unwrap_or([255, 255, 255, 255])
                    });
                    
                    let (x, y) = if let Some((px, py)) = tags.position {
                        (px, py)
                    } else {
                        self.calculate_position(event, context, tags)
                    };
                    
                    // Apply transform to path if needed
                    let transformed_path = if let Some((x1, y1, x2, y2, t1, t2)) = tags.movement {
                        let progress = calculate_move_progress(time_cs, t1, t2);
                        let curr_x = x1 + (x2 - x1) * progress;
                        let curr_y = y1 + (y2 - y1) * progress;
                        path.transform(tiny_skia::Transform::from_translate(curr_x, curr_y))
                    } else {
                        path.transform(tiny_skia::Transform::from_translate(x, y))
                    };
                    
                    return Ok(vec![IntermediateLayer::Vector(VectorData {
                        path: transformed_path,
                        color,
                        stroke: None,
                        bounds: None,
                    })]);
                }
                return Ok(Vec::new());
            }
        }
        
        // Process text segments
        let mut all_layers = Vec::new();
        let mut current_x = 0.0;
        let mut current_y = 0.0;
        let mut line_height = 0.0;
        
        // Get base position from first segment (pos/move tags apply to whole event)
        let base_tags = &segments[0].tags;
        let (base_x, base_y) = if let Some((px, py)) = base_tags.position {
            (px, py)
        } else if let Some((x1, y1, x2, y2, t1, t2)) = base_tags.movement {
            let progress = calculate_move_progress(time_cs, t1, t2);
            let x = x1 + (x2 - x1) * progress;
            let y = y1 + (y2 - y1) * progress;
            (x, y)
        } else {
            self.calculate_position(event, context, base_tags)
        };
        
        current_x = base_x;
        current_y = base_y;
        
        for segment in segments {
            if segment.text.is_empty() {
                continue;
            }
            
            let tags = &segment.tags;
            
            // Handle line breaks in segment text
            let lines: Vec<&str> = segment.text.split('\n').collect();
            
            for (line_idx, line_text) in lines.iter().enumerate() {
                if line_idx > 0 {
                    // Move to next line
                    current_y += line_height * 1.2;
                    current_x = base_x;
                }
                
                if line_text.is_empty() {
                    continue;
                }
                
                // Get font information for this segment
                let font_family = tags.font.name.as_deref().unwrap_or_else(|| {
                    resolved_style
                        .map(|s| s.font_name())
                        .unwrap_or("Arial")
                });
                
                let base_font_size = tags.font.size.unwrap_or_else(|| {
                    resolved_style
                        .map(|s| s.font_size())
                        .unwrap_or(40.0)
                });
                
                let font_scale_x = tags.font.scale_x.unwrap_or(100.0) / 100.0;
                let font_scale_y = tags.font.scale_y.unwrap_or(100.0) / 100.0;
                let font_size = base_font_size * font_scale_y;
                line_height = font_size;
                
                // Get formatting
                let bold = tags.formatting.bold.unwrap_or_else(|| {
                    resolved_style.map(|s| s.is_bold()).unwrap_or(false)
                });
                let italic = tags.formatting.italic.unwrap_or_else(|| {
                    resolved_style.map(|s| s.is_italic()).unwrap_or(false)
                });
                
                // Shape text
                let shaped = shape_text_with_style(
                    line_text,
                    font_family,
                    font_size,
                    bold,
                    italic,
                    &self.font_database,
                )?;
                
                // Get colors for this segment - THIS IS THE KEY FIX
                let primary_color = tags.colors.primary.unwrap_or_else(|| {
                    resolved_style
                        .map(|s| s.primary_color())
                        .unwrap_or([255, 255, 255, 255])
                });
                
                // Apply alpha override if present
                let mut color = primary_color;
                if let Some(alpha) = tags.colors.alpha {
                    color[3] = alpha;
                }
                
                // Apply fade if present
                if let Some(fade) = &tags.fade {
                    // For \fad(t1,t2), t1 is fade-in duration, t2 is fade-out duration
                    // Calculate actual fade times relative to event
                    let event_start = event.start_time_cs().unwrap_or(0);
                    let event_end = event.end_time_cs().unwrap_or(u32::MAX);
                    
                    let fade_alpha = if fade.alpha_middle.is_some() {
                        // Complex fade with 7 parameters - times are absolute
                        let fade_progress = calculate_fade_progress(time_cs, fade.time_start, fade.time_end);
                        fade.alpha_start as f32 + (fade.alpha_end as f32 - fade.alpha_start as f32) * fade_progress
                    } else {
                        // Simple fade - times are durations
                        let fade_in_end = event_start + fade.time_start;
                        let fade_out_start = event_end.saturating_sub(fade.time_end);
                        
                        if time_cs < fade_in_end {
                            // During fade in
                            let progress = (time_cs.saturating_sub(event_start)) as f32 / fade.time_start.max(1) as f32;
                            255.0 * progress.min(1.0)
                        } else if time_cs >= fade_out_start && fade_out_start < event_end {
                            // During fade out
                            let progress = (event_end.saturating_sub(time_cs)) as f32 / fade.time_end.max(1) as f32;
                            255.0 * progress.min(1.0)
                        } else {
                            // Fully visible
                            255.0
                        }
                    };
                    
                    color[3] = ((color[3] as f32 * (fade_alpha / 255.0)) as u8).min(255);
                }
                
                // Create text layer for this segment
                let mut layer = TextData {
                    text: line_text.to_string(),
                    font_family: font_family.to_string(),
                    font_size,
                    color,
                    x: current_x,
                    y: current_y,
                    effects: SmallVec::new(),
                };
                
                // Apply effects
                if bold {
                    layer.effects.push(TextEffect::Bold);
                }
                
                if italic {
                    layer.effects.push(TextEffect::Italic);
                }
                
                if tags.formatting.underline.unwrap_or(false) {
                    layer.effects.push(TextEffect::Underline);
                }
                
                if tags.formatting.strikeout.unwrap_or(false) {
                    layer.effects.push(TextEffect::Strikethrough);
                }
                
                // Add outline
                let outline_width = tags.formatting.border.unwrap_or_else(|| {
                    resolved_style.map(|s| s.outline()).unwrap_or(0.0)
                });
                if outline_width > 0.0 {
                    let outline_color = tags.colors.outline.unwrap_or_else(|| {
                        resolved_style
                            .map(|s| s.outline_color())
                            .unwrap_or([0, 0, 0, 255])
                    });
                    layer.effects.push(TextEffect::Outline {
                        color: outline_color,
                        width: outline_width,
                    });
                }
                
                // Add shadow
                let shadow_depth = tags.formatting.shadow.unwrap_or_else(|| {
                    resolved_style.map(|s| s.shadow()).unwrap_or(0.0)
                });
                if shadow_depth > 0.0 {
                    let shadow_color = tags.colors.shadow.unwrap_or_else(|| {
                        resolved_style
                            .map(|s| s.secondary_color())
                            .unwrap_or([64, 64, 64, 128])
                    });
                    layer.effects.push(TextEffect::Shadow {
                        color: shadow_color,
                        x_offset: shadow_depth,
                        y_offset: shadow_depth,
                    });
                }
                
                // Update x position for next segment on same line
                if let Some(advance) = shaped.total_advance() {
                    current_x += advance * font_scale_x;
                }
                
                all_layers.push(IntermediateLayer::Text(layer));
            }
        }
        
        Ok(all_layers)
    }
}