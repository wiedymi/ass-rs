//! Event line reconstruction helper
//!
//! Rebuilds a dialogue/comment line from extracted field data after applying
//! a set of field modifications. Exposed as `pub(super)` for the
//! index-based event editor.

use super::EditorDocument;
use crate::core::errors::{EditorError, Result};

#[cfg(not(feature = "std"))]
use alloc::{format, string::String, vec::Vec};

impl EditorDocument {
    /// Helper to build a modified event line from event data
    pub(super) fn build_modified_event_line_from_data(
        &self,
        event_data: (
            ass_core::parser::ast::EventType,
            String,
            String,
            String,
            String,
            String,
            String,
            String,
            String,
            String,
            String,
        ),
        _original_line: &str,
        modifications: Vec<(&'static str, String)>,
    ) -> Result<String> {
        let (
            event_type,
            layer,
            start,
            end,
            style,
            name,
            margin_l,
            margin_r,
            margin_v,
            effect,
            text,
        ) = event_data;

        // Apply modifications
        let mut layer = layer;
        let mut start = start;
        let mut end = end;
        let mut style = style;
        let mut name = name;
        let mut margin_l = margin_l;
        let mut margin_r = margin_r;
        let mut margin_v = margin_v;
        let mut effect = effect;
        let mut text = text;

        for (field, value) in modifications {
            match field {
                "layer" => layer = value,
                "start" => start = value,
                "end" => end = value,
                "style" => style = value,
                "name" => name = value,
                "margin_l" => margin_l = value,
                "margin_r" => margin_r = value,
                "margin_v" => margin_v = value,
                "effect" => effect = value,
                "text" => text = value,
                _ => {
                    return Err(EditorError::ValidationError {
                        message: format!("Unknown event field: {field}"),
                    });
                }
            }
        }

        // Rebuild the line
        let event_type_str = event_type.as_str();
        Ok(format!("{event_type_str}: {layer},{start},{end},{style},{name},{margin_l},{margin_r},{margin_v},{effect},{text}"))
    }
}
