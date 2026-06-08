//! WebVTT <-> ASS timestamp conversion helpers.
//!
//! Provides parsing of WebVTT cue timestamps into ASS time strings and the
//! inverse conversion used when exporting.

use crate::core::EditorError;

use super::WebVttFormat;

impl WebVttFormat {
    /// Parse WebVTT timestamp (HH:MM:SS.mmm or MM:SS.mmm)
    pub(super) fn parse_vtt_time(time_str: &str) -> Result<String, EditorError> {
        let time_str = time_str.trim();

        // WebVTT supports both HH:MM:SS.mmm and MM:SS.mmm formats
        let parts: Vec<&str> = time_str.split('.').collect();
        if parts.len() != 2 {
            return Err(EditorError::InvalidFormat(format!(
                "Invalid WebVTT time format: {time_str}"
            )));
        }

        let time_part = parts[0];
        let ms_part = parts[1];

        // Parse milliseconds and convert to centiseconds
        let ms: u32 = ms_part
            .parse()
            .map_err(|_| EditorError::InvalidFormat(format!("Invalid milliseconds: {ms_part}")))?;
        let cs = ms / 10; // Convert to centiseconds

        // Handle both MM:SS and HH:MM:SS formats
        let time_components: Vec<&str> = time_part.split(':').collect();
        let ass_time = match time_components.len() {
            2 => {
                // MM:SS format - prepend 0 hours
                format!("0:{time_part}.{cs:02}")
            }
            3 => {
                // HH:MM:SS format - remove leading zero from hours if present
                let hours = time_components[0];
                let hours = if hours.starts_with('0') && hours.len() > 1 {
                    &hours[1..]
                } else {
                    hours
                };
                format!(
                    "{hours}:{}:{}.{cs:02}",
                    time_components[1], time_components[2]
                )
            }
            _ => {
                return Err(EditorError::InvalidFormat(format!(
                    "Invalid WebVTT time format: {time_str}"
                )));
            }
        };

        Ok(ass_time)
    }

    /// Convert ASS timestamp to WebVTT format
    pub(super) fn format_vtt_time(ass_time: &str) -> Result<String, EditorError> {
        let ass_time = ass_time.trim();

        // Convert ASS time format (H:MM:SS.cc) to WebVTT format (HH:MM:SS.mmm)
        if let Some(dot_pos) = ass_time.find('.') {
            let (time_part, cs_part) = ass_time.split_at(dot_pos);
            let cs_part = &cs_part[1..]; // Remove dot

            // Parse centiseconds and convert to milliseconds
            let cs: u32 = cs_part.parse().map_err(|_| {
                EditorError::InvalidFormat(format!("Invalid centiseconds: {cs_part}"))
            })?;
            let ms = cs * 10; // Convert to milliseconds

            // Ensure hours are zero-padded for WebVTT format
            let parts: Vec<&str> = time_part.split(':').collect();
            if parts.len() == 3 {
                let hours: u32 = parts[0].parse().map_err(|_| {
                    EditorError::InvalidFormat(format!("Invalid hours: {}", parts[0]))
                })?;
                Ok(format!("{hours:02}:{}:{}.{ms:03}", parts[1], parts[2]))
            } else {
                Err(EditorError::InvalidFormat(format!(
                    "Invalid ASS time format: {ass_time}"
                )))
            }
        } else {
            Err(EditorError::InvalidFormat(format!(
                "Invalid ASS time format: {ass_time}"
            )))
        }
    }
}
