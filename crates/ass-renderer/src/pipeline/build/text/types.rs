//! Shared layout types for the software pipeline's text builder: style-derived
//! defaults, per-call and per-line context, pen state, and resolved effect
//! colours. Consumed by the descendant `defaults`/`position`/`effects` modules
//! and the orchestrator in the parent module.

use ass_core::parser::Event;

use crate::renderer::RenderContext;

/// Style-derived defaults (font, size, colours, formatting) resolved once per
/// event. Borrows the run's resolved style for its zero-copy font name.
pub(super) struct TextDefaults<'a> {
    pub(super) font_name: &'a str,
    pub(super) font_size_base: f32,
    pub(super) bold: bool,
    pub(super) italic: bool,
    pub(super) underline: bool,
    pub(super) strikeout: bool,
    pub(super) primary_color: [u8; 4],
    pub(super) secondary_color: [u8; 4],
    pub(super) outline_color: [u8; 4],
    pub(super) back_color: [u8; 4],
    pub(super) outline: f32,
    pub(super) shadow: f32,
    pub(super) scale_x: f32,
    pub(super) scale_y: f32,
    pub(super) spacing: f32,
    pub(super) alignment: u8,
}

/// Per-call rendering context shared by the per-segment helpers (constant
/// across every line and segment of the event).
pub(super) struct RunCtx<'a, 'b> {
    pub(super) event: &'a Event<'b>,
    pub(super) context: &'a RenderContext,
    pub(super) time_cs: u32,
    pub(super) scale_x: f32,
    pub(super) scale_y: f32,
}

/// Per-line layout constants consumed when positioning each segment.
pub(super) struct LineLayout {
    pub(super) is_multi_segment: bool,
    pub(super) line_total_width: f32,
    pub(super) num_lines: usize,
    pub(super) line_index: usize,
    pub(super) estimated_line_height: f32,
    pub(super) line_spacing_multiplier: f32,
    pub(super) line_y_offset: f32,
}

/// Pen state carried across the segments of a single line.
pub(super) struct Pen {
    pub(super) current_x: f32,
    pub(super) needs_initial_position: bool,
}

/// Resolved outline and shadow colours handed to the effects builder.
pub(super) struct EffectColors {
    pub(super) outline_color: [u8; 4],
    pub(super) shadow_color: [u8; 4],
}
