//! Timestamp conversion between SRT (`HH:MM:SS,mmm`) and ASS (`H:MM:SS.cc`).

use super::SrtFormat;
use crate::core::EditorError;

impl SrtFormat {
    /// Parse SRT timestamp (HH:MM:SS,mmm)
    pub(super) fn parse_srt_time(time_str: &str) -> Result<String, EditorError> {
        let time_str = time_str.trim();

        // Convert SRT time format (HH:MM:SS,mmm) to ASS format (H:MM:SS.cc)
        if let Some(comma_pos) = time_str.find(',') {
            let (time_part, ms_part) = time_str.split_at(comma_pos);
            let ms_part = &ms_part[1..]; // Remove comma

            // Parse milliseconds and convert to centiseconds
            let ms: u32 = ms_part.parse().map_err(|_| {
                EditorError::InvalidFormat(format!("Invalid milliseconds: {ms_part}"))
            })?;
            let cs = ms / 10; // Convert to centiseconds

            // Remove leading zero from hours if present for ASS format
            let time_part = if time_part.starts_with("0") && time_part.len() > 1 {
                &time_part[1..]
            } else {
                time_part
            };

            Ok(format!("{time_part}.{cs:02}"))
        } else {
            Err(EditorError::InvalidFormat(format!(
                "Invalid SRT time format: {time_str}"
            )))
        }
    }

    /// Convert ASS timestamp to SRT format
    pub(super) fn format_srt_time(ass_time: &str) -> Result<String, EditorError> {
        let ass_time = ass_time.trim();

        // Convert ASS time format (H:MM:SS.cc) to SRT format (HH:MM:SS,mmm)
        if let Some(dot_pos) = ass_time.find('.') {
            let (time_part, cs_part) = ass_time.split_at(dot_pos);
            let cs_part = &cs_part[1..]; // Remove dot

            // Parse centiseconds and convert to milliseconds
            let cs: u32 = cs_part.parse().map_err(|_| {
                EditorError::InvalidFormat(format!("Invalid centiseconds: {cs_part}"))
            })?;
            let ms = cs * 10; // Convert to milliseconds

            // Ensure hours are zero-padded for SRT format
            let parts: Vec<&str> = time_part.split(':').collect();
            if parts.len() == 3 {
                let hours: u32 = parts[0].parse().map_err(|_| {
                    EditorError::InvalidFormat(format!("Invalid hours: {}", parts[0]))
                })?;
                Ok(format!("{hours:02}:{}:{},{ms:03}", parts[1], parts[2]))
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
