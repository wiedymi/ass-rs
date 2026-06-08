//! WebVTT cue settings parsing and cue-to-dialogue conversion.
//!
//! Parses WebVTT cue setting strings, maps them to ASS positioning tags, and
//! converts complete cues into ASS dialogue lines.

use crate::core::EditorError;
use std::collections::HashMap;

use super::WebVttFormat;

impl WebVttFormat {
    /// Parse WebVTT cue settings for positioning
    pub(super) fn parse_cue_settings(settings: &str) -> HashMap<String, String> {
        let mut cue_settings = HashMap::new();

        for setting in settings.split_whitespace() {
            if let Some(colon_pos) = setting.find(':') {
                let (key, value) = setting.split_at(colon_pos);
                let value = &value[1..]; // Remove colon
                cue_settings.insert(key.to_string(), value.to_string());
            }
        }

        cue_settings
    }

    /// Convert cue settings to ASS positioning tags
    pub(super) fn cue_settings_to_ass_positioning(settings: &HashMap<String, String>) -> String {
        let mut ass_tags = String::new();

        // Handle position settings
        if let (Some(line), Some(position)) = (settings.get("line"), settings.get("position")) {
            // Convert WebVTT positioning to approximate ASS positioning
            if let (Ok(line_val), Ok(pos_val)) = (line.parse::<f32>(), position.parse::<f32>()) {
                let x = (pos_val * 640.0) as u32; // Assume 640x480 resolution
                let y = (line_val * 480.0) as u32;
                ass_tags.push_str(&format!(r"\pos({x},{y})"));
            }
        }

        // Handle alignment
        if let Some(align) = settings.get("align") {
            let alignment = match align.as_str() {
                "start" | "left" => 1,
                "center" | "middle" => 2,
                "end" | "right" => 3,
                _ => 2, // default to center
            };
            ass_tags.push_str(&format!(r"\an{alignment}"));
        }

        if !ass_tags.is_empty() {
            format!("{{{ass_tags}}}")
        } else {
            String::new()
        }
    }

    /// Parse WebVTT cue
    pub(super) fn parse_vtt_cue(
        lines: &[String],
        start_idx: usize,
    ) -> Result<(usize, String), EditorError> {
        if start_idx >= lines.len() {
            return Err(EditorError::InvalidFormat(
                "Unexpected end of file".to_string(),
            ));
        }

        let mut idx = start_idx;

        // Skip empty lines and NOTE blocks
        while idx < lines.len() {
            let line = lines[idx].trim();
            if line.is_empty() || line.starts_with("NOTE") {
                idx += 1;
                continue;
            }
            break;
        }

        if idx >= lines.len() {
            return Err(EditorError::InvalidFormat(
                "Unexpected end of file".to_string(),
            ));
        }

        let current_line = &lines[idx];

        // Check if this line contains timestamp (cue timing)
        if current_line.contains("-->") {
            // Parse timestamp line directly
            let timestamp_line = current_line;
            let parts: Vec<&str> = timestamp_line.split("-->").collect();
            if parts.len() < 2 {
                return Err(EditorError::InvalidFormat(format!(
                    "Invalid timestamp format: {timestamp_line}"
                )));
            }

            let start_time = Self::parse_vtt_time(parts[0])?;
            let end_time_and_settings: Vec<&str> = parts[1].split_whitespace().collect();
            let end_time = Self::parse_vtt_time(end_time_and_settings[0])?;

            // Parse cue settings if present
            let cue_settings = if end_time_and_settings.len() > 1 {
                let settings_str = end_time_and_settings[1..].join(" ");
                Self::parse_cue_settings(&settings_str)
            } else {
                HashMap::new()
            };

            idx += 1;

            // Collect cue text
            let mut text_lines = Vec::new();
            while idx < lines.len() && !lines[idx].trim().is_empty() {
                let styled_text = Self::convert_vtt_to_ass_styling(&lines[idx]);
                text_lines.push(styled_text);
                idx += 1;
            }

            if text_lines.is_empty() {
                return Err(EditorError::InvalidFormat("Empty cue text".to_string()));
            }

            let text = text_lines.join("\\N"); // ASS line break
            let positioning = Self::cue_settings_to_ass_positioning(&cue_settings);
            let dialogue_line =
                format!("Dialogue: 0,{start_time},{end_time},Default,,0,0,0,,{positioning}{text}");

            Ok((idx, dialogue_line))
        } else {
            // This might be a cue identifier, skip to next line for timestamp
            idx += 1;
            if idx < lines.len() && lines[idx].contains("-->") {
                Self::parse_vtt_cue(lines, idx)
            } else {
                Err(EditorError::InvalidFormat(format!(
                    "Expected timestamp line after cue identifier: {current_line}"
                )))
            }
        }
    }
}
