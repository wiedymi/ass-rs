//! Internal stateful style-line parsing for the section driver.
//!
//! Implements [`StylesParser::parse_style_line_internal`], which records issues
//! on the parser while materializing a [`Style`] during a full-section parse.

use super::StylesParser;
use crate::parser::{
    ast::Style,
    errors::{IssueCategory, IssueSeverity, ParseIssue},
    position_tracker::PositionTracker,
};
use alloc::{format, vec::Vec};

impl<'a> StylesParser<'a> {
    /// Parse single style definition line
    pub(super) fn parse_style_line_internal(
        &mut self,
        line: &'a str,
        line_start: &PositionTracker<'a>,
    ) -> Option<Style<'a>> {
        // First check if this is an inheritance style
        let (adjusted_line, parent_style) = if line.trim_start().starts_with('*') {
            // Find the first comma after the asterisk to extract parent style
            line.find(',').map_or((line, None), |first_comma| {
                let parent_part = &line[0..first_comma];
                let parent_name = parent_part.trim_start().trim_start_matches('*').trim();
                let remaining = &line[first_comma + 1..];
                (remaining, Some(parent_name))
            })
        } else {
            (line, None)
        };

        let parts: Vec<&str> = adjusted_line.split(',').collect();

        let format = self.format.as_deref().unwrap_or(&[
            "Name",
            "Fontname",
            "Fontsize",
            "PrimaryColour",
            "SecondaryColour",
            "OutlineColour",
            "BackColour",
            "Bold",
            "Italic",
            "Underline",
            "StrikeOut",
            "ScaleX",
            "ScaleY",
            "Spacing",
            "Angle",
            "BorderStyle",
            "Outline",
            "Shadow",
            "Alignment",
            "MarginL",
            "MarginR",
            "MarginV",
            "Encoding",
        ]);

        if parts.len() != format.len() {
            self.issues.push(ParseIssue::new(
                IssueSeverity::Warning,
                IssueCategory::Format,
                format!(
                    "Style line has {} fields, expected {}",
                    parts.len(),
                    format.len()
                ),
                line_start.line() as usize,
            ));
            if parts.len() < format.len() {
                return None;
            }
        }

        let get_field = |name: &str| -> &'a str {
            format
                .iter()
                .position(|&field| field.eq_ignore_ascii_case(name))
                .and_then(|idx| parts.get(idx))
                .map_or("", |s| s.trim())
        };

        // Calculate span for this style line
        let full_line = self.current_line();
        let span = line_start.span_for(full_line.len());

        Some(Style {
            name: get_field("Name"),
            parent: parent_style,
            fontname: get_field("Fontname"),
            fontsize: get_field("Fontsize"),
            primary_colour: get_field("PrimaryColour"),
            secondary_colour: get_field("SecondaryColour"),
            outline_colour: get_field("OutlineColour"),
            back_colour: get_field("BackColour"),
            bold: get_field("Bold"),
            italic: get_field("Italic"),
            underline: get_field("Underline"),
            strikeout: get_field("StrikeOut"),
            scale_x: get_field("ScaleX"),
            scale_y: get_field("ScaleY"),
            spacing: get_field("Spacing"),
            angle: get_field("Angle"),
            border_style: get_field("BorderStyle"),
            outline: get_field("Outline"),
            shadow: get_field("Shadow"),
            alignment: get_field("Alignment"),
            margin_l: get_field("MarginL"),
            margin_r: get_field("MarginR"),
            margin_v: get_field("MarginV"),
            margin_t: format
                .iter()
                .any(|&f| f.eq_ignore_ascii_case("MarginT"))
                .then(|| get_field("MarginT")),
            margin_b: format
                .iter()
                .any(|&f| f.eq_ignore_ascii_case("MarginB"))
                .then(|| get_field("MarginB")),
            encoding: get_field("Encoding"),
            relative_to: format
                .iter()
                .any(|&f| f.eq_ignore_ascii_case("RelativeTo"))
                .then(|| get_field("RelativeTo")),
            span,
        })
    }
}
