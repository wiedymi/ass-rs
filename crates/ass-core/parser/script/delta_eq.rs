//! Span-insensitive equality checks for delta computation.
//!
//! Provides comparison helpers used by [`calculate_delta`](super::calculate_delta)
//! to determine whether sections and their contained items differ in content
//! rather than merely in position (which would change spans).

use crate::parser::ast::{Event, Font, Graphic, Section, Style};

/// Compare two sections for equality while ignoring span differences
///
/// This is used by delta calculation to determine if sections have actually
/// changed in content, not just in position (which would change spans).
pub(super) fn sections_equal_ignoring_spans(old: &Section<'_>, new: &Section<'_>) -> bool {
    use Section::{Events, Fonts, Graphics, ScriptInfo, Styles};

    match (old, new) {
        (ScriptInfo(old_info), ScriptInfo(new_info)) => {
            // Compare fields, ignoring span
            old_info.fields == new_info.fields
        }
        (Styles(old_styles), Styles(new_styles)) => {
            // Compare styles, ignoring spans
            if old_styles.len() != new_styles.len() {
                return false;
            }

            for (old_style, new_style) in old_styles.iter().zip(new_styles.iter()) {
                if !styles_equal_ignoring_span(old_style, new_style) {
                    return false;
                }
            }
            true
        }
        (Events(old_events), Events(new_events)) => {
            // Compare events, ignoring spans
            if old_events.len() != new_events.len() {
                return false;
            }

            for (old_event, new_event) in old_events.iter().zip(new_events.iter()) {
                if !events_equal_ignoring_span(old_event, new_event) {
                    return false;
                }
            }
            true
        }
        (Fonts(old_fonts), Fonts(new_fonts)) => {
            // Compare fonts, ignoring spans
            if old_fonts.len() != new_fonts.len() {
                return false;
            }

            for (old_font, new_font) in old_fonts.iter().zip(new_fonts.iter()) {
                if !fonts_equal_ignoring_span(old_font, new_font) {
                    return false;
                }
            }
            true
        }
        (Graphics(old_graphics), Graphics(new_graphics)) => {
            // Compare graphics, ignoring spans
            if old_graphics.len() != new_graphics.len() {
                return false;
            }

            for (old_graphic, new_graphic) in old_graphics.iter().zip(new_graphics.iter()) {
                if !graphics_equal_ignoring_span(old_graphic, new_graphic) {
                    return false;
                }
            }
            true
        }
        _ => false, // Different section types
    }
}

/// Compare two styles for equality while ignoring span
fn styles_equal_ignoring_span(old: &Style<'_>, new: &Style<'_>) -> bool {
    old.name == new.name
        && old.parent == new.parent
        && old.fontname == new.fontname
        && old.fontsize == new.fontsize
        && old.primary_colour == new.primary_colour
        && old.secondary_colour == new.secondary_colour
        && old.outline_colour == new.outline_colour
        && old.back_colour == new.back_colour
        && old.bold == new.bold
        && old.italic == new.italic
        && old.underline == new.underline
        && old.strikeout == new.strikeout
        && old.scale_x == new.scale_x
        && old.scale_y == new.scale_y
        && old.spacing == new.spacing
        && old.angle == new.angle
        && old.border_style == new.border_style
        && old.outline == new.outline
        && old.shadow == new.shadow
        && old.alignment == new.alignment
        && old.margin_l == new.margin_l
        && old.margin_r == new.margin_r
        && old.margin_v == new.margin_v
        && old.margin_t == new.margin_t
        && old.margin_b == new.margin_b
        && old.encoding == new.encoding
        && old.relative_to == new.relative_to
    // Note: explicitly NOT comparing span field
}

/// Compare two events for equality while ignoring span
fn events_equal_ignoring_span(old: &Event<'_>, new: &Event<'_>) -> bool {
    old.event_type == new.event_type
        && old.layer == new.layer
        && old.start == new.start
        && old.end == new.end
        && old.style == new.style
        && old.name == new.name
        && old.margin_l == new.margin_l
        && old.margin_r == new.margin_r
        && old.margin_v == new.margin_v
        && old.margin_t == new.margin_t
        && old.margin_b == new.margin_b
        && old.effect == new.effect
        && old.text == new.text
    // Note: explicitly NOT comparing span field
}

/// Compare two fonts for equality while ignoring span
fn fonts_equal_ignoring_span(old: &Font<'_>, new: &Font<'_>) -> bool {
    old.filename == new.filename && old.data_lines == new.data_lines
    // Note: explicitly NOT comparing span field
}

/// Compare two graphics for equality while ignoring span
fn graphics_equal_ignoring_span(old: &Graphic<'_>, new: &Graphic<'_>) -> bool {
    old.filename == new.filename && old.data_lines == new.data_lines
    // Note: explicitly NOT comparing span field
}
