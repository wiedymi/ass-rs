//! Fixed software pipeline implementation with proper style resolution

#[cfg(feature = "nostd")]
use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};
#[cfg(not(feature = "nostd"))]
use std::{
    string::{String, ToString},
    vec::Vec,
};

use crate::collision::PositionedEvent;
use crate::pipeline::{
    shaping::GlyphRenderer, text_segmenter::segment_text_with_tags, IntermediateLayer, Pipeline,
};
use crate::renderer::RenderContext;
use crate::utils::{DirtyRegion, RenderError};
use ahash::AHashMap;
#[cfg(feature = "analysis-integration")]
use ass_core::analysis::ScriptAnalysis;
use ass_core::parser::{Event, Script};
use fontdb::Database as FontDatabase;

mod animation;
mod drawing;
mod position;
mod style;
mod text;
mod wrap;
use style::OwnedStyle;

/// Software rendering pipeline with proper style inheritance
pub struct SoftwarePipeline {
    /// Font database for text rendering
    font_database: FontDatabase,
    /// Glyph renderer
    #[allow(dead_code)] // Glyph rendering component - used in future rendering features
    glyph_renderer: GlyphRenderer,
    /// Collision resolver for subtitle positioning
    collision_resolver: crate::collision::CollisionResolver,
    /// Render cache for performance
    cache: crate::cache::RenderCache,
    /// Current script styles map for quick lookup
    styles_map: AHashMap<String, OwnedStyle>,
    /// Default style for fallback
    default_style: Option<OwnedStyle>,
    /// Script playback resolution from PlayResX/PlayResY
    play_res_x: f32,
    play_res_y: f32,
    /// Script layout resolution from LayoutResX/LayoutResY (if present)
    layout_res_x: Option<f32>,
    layout_res_y: Option<f32>,
    /// Whether to scale border and shadow with video resolution
    scaled_border_and_shadow: bool,
    /// DPI scale factor for font rendering (default: 0.9)
    /// libass uses 72 DPI, some systems use 96 DPI
    /// Empirically adjusted to 0.9 for better libass visual match
    dpi_scale: f32,
    /// Script `WrapStyle` header (0 smart, 1 greedy, 2 none, 3 smart); the `\q`
    /// override takes precedence per event. Defaults to 0.
    wrap_style: u8,
}

impl Default for SoftwarePipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl SoftwarePipeline {
    /// Create a new fixed software pipeline
    pub fn new() -> Self {
        let mut font_database = FontDatabase::new();
        font_database.load_system_fonts();

        Self {
            font_database,
            glyph_renderer: GlyphRenderer::new(),
            collision_resolver: crate::collision::CollisionResolver::new(1920.0, 1080.0),
            cache: crate::cache::RenderCache::with_limits(5000, 2000),
            styles_map: AHashMap::new(),
            default_style: None,
            play_res_x: 1920.0, // Default resolution
            play_res_y: 1080.0, // Default resolution
            layout_res_x: None,
            layout_res_y: None,
            scaled_border_and_shadow: true, // Default to true per ASS spec
            dpi_scale: 0.9,                 // Adjusted for better libass compatibility (was 0.75)
            wrap_style: 0,
        }
    }

    /// Create with specific dimensions
    pub fn with_dimensions(width: f32, height: f32) -> Self {
        let mut font_database = FontDatabase::new();
        font_database.load_system_fonts();

        Self {
            font_database,
            glyph_renderer: GlyphRenderer::new(),
            collision_resolver: crate::collision::CollisionResolver::new(width, height),
            cache: crate::cache::RenderCache::with_limits(5000, 2000),
            styles_map: AHashMap::new(),
            default_style: None,
            play_res_x: width,  // Use provided dimensions as default
            play_res_y: height, // Use provided dimensions as default
            layout_res_x: None,
            layout_res_y: None,
            scaled_border_and_shadow: true, // Default to true per ASS spec
            dpi_scale: 0.9,                 // Adjusted for better libass compatibility (was 0.75)
            wrap_style: 0,
        }
    }

    /// Set DPI scale factor (default is 0.9 for libass compatibility)
    /// Use 1.0 for 96 DPI, 0.9 for empirically matched libass rendering
    pub fn set_dpi_scale(&mut self, scale: f32) {
        self.dpi_scale = scale;
    }

    /// Get current DPI scale factor
    pub fn dpi_scale(&self) -> f32 {
        self.dpi_scale
    }

    fn process_event(
        &mut self,
        event: &Event,
        time_cs: u32,
        context: &RenderContext,
    ) -> Result<Vec<IntermediateLayer>, RenderError> {
        // Get text segments with their individual tags
        let segments = segment_text_with_tags(event.text, None)?;

        if segments.is_empty() {
            return Ok(Vec::new());
        }

        // Check if this is a drawing command
        if let Some(draw_level) = segments[0].tags.drawing_mode {
            if draw_level > 0 {
                // Clone the style to avoid borrow issues
                let style_cloned = self
                    .styles_map
                    .get(event.style)
                    .or(self.default_style.as_ref())
                    .cloned();

                return self.process_drawing_command(
                    &segments[0],
                    event,
                    style_cloned.as_ref(),
                    time_cs,
                    context,
                );
            }
        }

        // Clone the style to avoid borrow issues
        let style_cloned = self
            .styles_map
            .get(event.style)
            .or(self.default_style.as_ref())
            .cloned();

        // Process text segments with proper style inheritance
        self.process_text_segments(segments, event, style_cloned.as_ref(), time_cs, context)
    }
}

#[allow(dead_code)] // Utility for karaoke effects - used in future features
fn calculate_karaoke_progress(time_cs: u32, start_time_cs: u32, duration_cs: u32) -> f32 {
    if time_cs < start_time_cs {
        return 0.0;
    }
    let elapsed = time_cs - start_time_cs;
    if elapsed >= duration_cs {
        return 1.0;
    }
    elapsed as f32 / duration_cs as f32
}

impl Pipeline for SoftwarePipeline {
    fn prepare_script(
        &mut self,
        script: &Script,
        #[cfg(feature = "analysis-integration")] analysis: Option<&ScriptAnalysis>,
        #[cfg(not(feature = "analysis-integration"))] _analysis: Option<()>,
    ) -> Result<(), RenderError> {
        // Load embedded and referenced fonts from the script
        super::font_loader::load_script_fonts(script, &mut self.font_database);

        // Clear and rebuild styles map
        self.styles_map.clear();
        self.default_style = None;

        // If we have analysis with resolved styles (which handle LayoutRes->PlayRes scaling),
        // we should use those instead of raw styles
        #[cfg(feature = "analysis-integration")]
        let _use_resolved_styles = analysis.is_some();
        #[cfg(not(feature = "analysis-integration"))]
        let _use_resolved_styles = false;

        // Extract script info and styles from the script
        for section in script.sections() {
            match section {
                ass_core::parser::Section::ScriptInfo(info) => {
                    // Extract PlayResX and PlayResY from script info
                    if let Some((res_x, res_y)) = info.play_resolution() {
                        self.play_res_x = res_x as f32;
                        self.play_res_y = res_y as f32;
                    }

                    // Extract WrapStyle (0 smart / 1 greedy / 2 none / 3 smart)
                    self.wrap_style = info.wrap_style();

                    // Extract LayoutResX and LayoutResY if present
                    if let Some((layout_x, layout_y)) = info.layout_resolution() {
                        self.layout_res_x = Some(layout_x as f32);
                        self.layout_res_y = Some(layout_y as f32);

                        // If LayoutRes differs from PlayRes, we need to scale styles
                        // This is done later when processing styles
                    }

                    // Extract ScaledBorderAndShadow setting
                    // Default is "yes" per ASS spec, but can be "no" to disable scaling
                    if let Some(scaled_value) = info.get_field("ScaledBorderAndShadow") {
                        self.scaled_border_and_shadow = scaled_value.to_lowercase() != "no";
                    }
                }
                ass_core::parser::Section::Styles(styles) => {
                    // Calculate LayoutRes->PlayRes scaling factors if LayoutRes is present
                    let layout_to_play_scale_x = if let Some(layout_x) = self.layout_res_x {
                        if layout_x != self.play_res_x {
                            self.play_res_x / layout_x
                        } else {
                            1.0
                        }
                    } else {
                        1.0
                    };

                    let layout_to_play_scale_y = if let Some(layout_y) = self.layout_res_y {
                        if layout_y != self.play_res_y {
                            self.play_res_y / layout_y
                        } else {
                            1.0
                        }
                    } else {
                        1.0
                    };

                    let needs_layout_scaling =
                        layout_to_play_scale_x != 1.0 || layout_to_play_scale_y != 1.0;

                    for style in styles {
                        let style_name = style.name.to_string();
                        let mut owned_style = OwnedStyle::from_style(style);

                        // Apply LayoutRes->PlayRes scaling if needed
                        if needs_layout_scaling {
                            // Scale font size (using Y scale as per libass)
                            if let Ok(font_size) = owned_style.fontsize.parse::<f32>() {
                                owned_style.fontsize =
                                    (font_size * layout_to_play_scale_y).to_string();
                            }

                            // Scale margins
                            if let Ok(margin_l) = owned_style.margin_l.parse::<f32>() {
                                owned_style.margin_l =
                                    (margin_l * layout_to_play_scale_x).to_string();
                            }
                            if let Ok(margin_r) = owned_style.margin_r.parse::<f32>() {
                                owned_style.margin_r =
                                    (margin_r * layout_to_play_scale_x).to_string();
                            }
                            if let Ok(margin_v) = owned_style.margin_v.parse::<f32>() {
                                owned_style.margin_v =
                                    (margin_v * layout_to_play_scale_y).to_string();
                            }

                            // Scale outline and shadow if ScaledBorderAndShadow is enabled
                            if self.scaled_border_and_shadow {
                                if let Ok(outline) = owned_style.outline.parse::<f32>() {
                                    owned_style.outline =
                                        (outline * layout_to_play_scale_y).to_string();
                                }
                                if let Ok(shadow) = owned_style.shadow.parse::<f32>() {
                                    owned_style.shadow =
                                        (shadow * layout_to_play_scale_y).to_string();
                                }
                            }

                            // Scale spacing
                            if let Ok(spacing) = owned_style.spacing.parse::<f32>() {
                                owned_style.spacing =
                                    (spacing * layout_to_play_scale_x).to_string();
                            }
                        }

                        if style_name == "Default" || style_name == "*Default" {
                            self.default_style = Some(owned_style.clone());
                        }

                        self.styles_map.insert(style_name, owned_style);
                    }
                }
                _ => {}
            }
        }

        // If no default style found, use the first one
        if self.default_style.is_none() && !self.styles_map.is_empty() {
            self.default_style = self.styles_map.values().next().cloned();
        }

        Ok(())
    }

    fn script(&self) -> Option<&Script<'_>> {
        None // We don't store the script reference directly
    }

    fn process_events(
        &mut self,
        events: &[&Event],
        time_cs: u32,
        context: &RenderContext,
    ) -> Result<Vec<IntermediateLayer>, RenderError> {
        // Clear collision resolver for this frame (but keep dimensions)
        self.collision_resolver.clear();

        // Pre-allocate with estimated capacity to reduce allocations
        let mut all_layers = Vec::with_capacity(events.len() * 3);

        // Sort events first by layer, then by start time to ensure proper ordering
        let mut sorted_events = events.to_vec();
        sorted_events.sort_by(|a, b| {
            let layer_a = a.layer.parse::<i32>().unwrap_or(0);
            let layer_b = b.layer.parse::<i32>().unwrap_or(0);
            let start_a = a.start_time_cs().unwrap_or(0);
            let start_b = b.start_time_cs().unwrap_or(0);

            // Sort by layer first, then by start time
            layer_a.cmp(&layer_b).then(start_a.cmp(&start_b))
        });

        let scale_y = context.height() as f32 / self.play_res_y;

        // Process each event, applying collision resolution so simultaneous
        // non-positioned events stack instead of overlapping (libass "Normal"
        // collisions). Positioned events (\pos/\move) are exempt and do not
        // participate in stacking.
        for event in sorted_events {
            let mut event_layers = self.process_event(event, time_cs, context)?;

            if !Self::event_is_positioned(event) {
                if let Some(bbox) = self.event_bounding_box(&event_layers) {
                    let positioned = PositionedEvent {
                        bbox,
                        layer: event.layer.parse::<i32>().unwrap_or(0),
                        margin_v: self.event_margin_v(event, scale_y) as i32,
                        margin_l: 0,
                        margin_r: 0,
                        alignment: self.effective_alignment(event),
                        priority: 0,
                    };
                    let resolved = self.collision_resolver.find_position(positioned);
                    let dy = resolved.y - bbox.y;
                    if dy.abs() > 0.5 {
                        Self::offset_layers_y(&mut event_layers, dy);
                    }
                }
            }

            all_layers.extend(event_layers);
        }

        Ok(all_layers)
    }

    fn compute_dirty_regions(
        &self,
        events: &[&Event],
        time_cs: u32,
        prev_time_cs: u32,
    ) -> Result<Vec<DirtyRegion>, RenderError> {
        let mut regions = Vec::new();

        for event in events {
            let was_active = event.start_time_cs().unwrap_or(0) <= prev_time_cs
                && event.end_time_cs().unwrap_or(u32::MAX) > prev_time_cs;
            let is_active = event.start_time_cs().unwrap_or(0) <= time_cs
                && event.end_time_cs().unwrap_or(u32::MAX) > time_cs;

            if was_active != is_active {
                // Event visibility changed, mark entire screen as dirty for now
                regions.push(DirtyRegion::full_screen());
                break;
            }
        }

        Ok(regions)
    }
}
