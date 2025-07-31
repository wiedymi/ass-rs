//! Format conversion utilities for importing/exporting subtitles
//!
//! Provides conversion between ASS and other subtitle formats like SRT and WebVTT.
//! Supports both import and export operations with format auto-detection.

use crate::core::{EditorDocument, Result};
use crate::core::errors::EditorError;
use ass_core::parser::ast::EventType;

#[cfg(not(feature = "std"))]
use alloc::{format, string::{String, ToString}, vec::Vec};


/// Supported subtitle formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubtitleFormat {
    /// Advanced SubStation Alpha (.ass)
    ASS,
    /// SubStation Alpha (.ssa)
    SSA,
    /// SubRip Text (.srt)
    SRT,
    /// WebVTT (.vtt)
    WebVTT,
    /// Plain text
    PlainText,
}

impl SubtitleFormat {
    /// Detect format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "ass" => Some(Self::ASS),
            "ssa" => Some(Self::SSA),
            "srt" => Some(Self::SRT),
            "vtt" | "webvtt" => Some(Self::WebVTT),
            "txt" => Some(Self::PlainText),
            _ => None,
        }
    }

    /// Detect format from content
    pub fn from_content(content: &str) -> Self {
        if content.contains("[Script Info]") || content.contains("[Events]") {
            Self::ASS
        } else if content.starts_with("WEBVTT") {
            Self::WebVTT
        } else if content.contains("-->") && !content.starts_with("WEBVTT") {
            Self::SRT
        } else {
            Self::PlainText
        }
    }

    /// Get the standard file extension for this format
    pub const fn extension(&self) -> &'static str {
        match self {
            Self::ASS => "ass",
            Self::SSA => "ssa",
            Self::SRT => "srt",
            Self::WebVTT => "vtt",
            Self::PlainText => "txt",
        }
    }
}

/// Options for format conversion
#[derive(Debug, Clone)]
pub struct ConversionOptions {
    /// Preserve styling information when possible
    pub preserve_styling: bool,
    
    /// Preserve positioning information when possible
    pub preserve_positioning: bool,
    
    /// Convert karaoke timing to inline format
    pub inline_karaoke: bool,
    
    /// Strip all formatting tags
    pub strip_formatting: bool,
    
    /// Target format-specific options
    pub format_options: FormatOptions,
}

impl Default for ConversionOptions {
    fn default() -> Self {
        Self {
            preserve_styling: true,
            preserve_positioning: true,
            inline_karaoke: false,
            strip_formatting: false,
            format_options: FormatOptions::default(),
        }
    }
}

/// Format-specific conversion options
#[derive(Debug, Clone)]
pub enum FormatOptions {
    /// No format-specific options
    None,
    
    /// SRT-specific options
    SRT {
        /// Include sequential numbering
        include_numbers: bool,
        /// Use millisecond precision (3 digits)
        millisecond_precision: bool,
    },
    
    /// WebVTT-specific options
    WebVTT {
        /// Include STYLE block for CSS
        include_style_block: bool,
        /// Include NOTE comments
        include_notes: bool,
        /// Use cue settings for positioning
        use_cue_settings: bool,
    },
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self::None
    }
}

/// Format converter for subtitle import/export
pub struct FormatConverter;

impl FormatConverter {
    /// Import subtitle content from various formats into ASS
    pub fn import(content: &str, format: Option<SubtitleFormat>) -> Result<String> {
        let detected_format = format.unwrap_or_else(|| SubtitleFormat::from_content(content));
        
        match detected_format {
            SubtitleFormat::ASS | SubtitleFormat::SSA => {
                // Already in ASS/SSA format, just return
                Ok(content.to_string())
            }
            SubtitleFormat::SRT => Self::import_srt(content),
            SubtitleFormat::WebVTT => Self::import_webvtt(content),
            SubtitleFormat::PlainText => Self::import_plain_text(content),
        }
    }
    
    /// Export ASS content to another subtitle format
    pub fn export(
        document: &EditorDocument,
        format: SubtitleFormat,
        options: &ConversionOptions,
    ) -> Result<String> {
        match format {
            SubtitleFormat::ASS => Ok(document.text()),
            SubtitleFormat::SSA => Self::export_ssa(document, options),
            SubtitleFormat::SRT => Self::export_srt(document, options),
            SubtitleFormat::WebVTT => Self::export_webvtt(document, options),
            SubtitleFormat::PlainText => Self::export_plain_text(document, options),
        }
    }
    
    /// Import SRT format
    fn import_srt(content: &str) -> Result<String> {
        let mut output = String::new();
        
        // Add ASS header
        output.push_str("[Script Info]\n");
        output.push_str("Title: Imported from SRT\n");
        output.push_str("ScriptType: v4.00+\n");
        output.push_str("WrapStyle: 0\n");
        output.push_str("PlayResX: 640\n");
        output.push_str("PlayResY: 480\n");
        output.push_str("ScaledBorderAndShadow: yes\n\n");
        
        // Add default style
        output.push_str("[V4+ Styles]\n");
        output.push_str("Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n");
        output.push_str("Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1\n\n");
        
        // Add events section
        output.push_str("[Events]\n");
        output.push_str("Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");
        
        // Parse SRT entries
        let entries = Self::parse_srt_entries(content)?;
        for entry in entries {
            output.push_str(&format!(
                "Dialogue: 0,{},{},Default,,0,0,0,,{}\n",
                entry.start, entry.end, entry.text
            ));
        }
        
        Ok(output)
    }
    
    /// Parse SRT entries
    fn parse_srt_entries(content: &str) -> Result<Vec<SrtEntry>> {
        let mut entries = Vec::new();
        let mut current_entry: Option<SrtEntry> = None;
        let mut in_text = false;
        
        for line in content.lines() {
            let line = line.trim();
            
            if line.is_empty() {
                if let Some(entry) = current_entry.take() {
                    entries.push(entry);
                }
                in_text = false;
                continue;
            }
            
            // Check if it's a number (subtitle index)
            if line.chars().all(|c| c.is_ascii_digit()) && !in_text {
                // Start new entry
                current_entry = Some(SrtEntry::default());
                continue;
            }
            
            // Check if it's a timestamp line
            if line.contains("-->") {
                if let Some(ref mut entry) = current_entry {
                    let parts: Vec<&str> = line.split("-->").collect();
                    if parts.len() == 2 {
                        entry.start = Self::parse_srt_time(parts[0].trim())?;
                        entry.end = Self::parse_srt_time(parts[1].trim())?;
                        in_text = true;
                    }
                }
                continue;
            }
            
            // Otherwise it's subtitle text
            if in_text {
                if let Some(ref mut entry) = current_entry {
                    if !entry.text.is_empty() {
                        entry.text.push_str("\\N");
                    }
                    // Convert basic HTML-like tags to ASS
                    let converted_text = Self::convert_srt_formatting(line);
                    entry.text.push_str(&converted_text);
                }
            }
        }
        
        // Don't forget the last entry
        if let Some(entry) = current_entry {
            entries.push(entry);
        }
        
        Ok(entries)
    }
    
    /// Parse SRT timestamp to ASS format
    fn parse_srt_time(time: &str) -> Result<String> {
        // SRT format: 00:00:00,000
        // ASS format: 0:00:00.00
        
        let time = time.replace(',', ".");
        let parts: Vec<&str> = time.split(':').collect();
        
        if parts.len() != 3 {
            return Err(EditorError::ValidationError {
                message: format!("Invalid SRT timestamp: {time}"),
            });
        }
        
        let hours: u32 = parts[0].parse().map_err(|_| EditorError::ValidationError {
            message: format!("Invalid hours in timestamp: {}", parts[0]),
        })?;
        
        let minutes: u32 = parts[1].parse().map_err(|_| EditorError::ValidationError {
            message: format!("Invalid minutes in timestamp: {}", parts[1]),
        })?;
        
        let seconds_parts: Vec<&str> = parts[2].split('.').collect();
        let seconds: u32 = seconds_parts[0].parse().map_err(|_| EditorError::ValidationError {
            message: format!("Invalid seconds in timestamp: {}", seconds_parts[0]),
        })?;
        
        let centiseconds = if seconds_parts.len() > 1 {
            // Convert milliseconds to centiseconds
            let millis: u32 = seconds_parts[1].parse().unwrap_or(0);
            millis / 10
        } else {
            0
        };
        
        Ok(format!("{hours}:{minutes:02}:{seconds:02}.{centiseconds:02}"))
    }
    
    /// Convert SRT formatting to ASS
    fn convert_srt_formatting(text: &str) -> String {
        let mut result = text.to_string();
        
        // Convert basic HTML-like tags
        result = result.replace("<i>", "{\\i1}");
        result = result.replace("</i>", "{\\i0}");
        result = result.replace("<b>", "{\\b1}");
        result = result.replace("</b>", "{\\b0}");
        result = result.replace("<u>", "{\\u1}");
        result = result.replace("</u>", "{\\u0}");
        
        // Remove any other HTML tags
        #[cfg(feature = "formats")]
        {
            result = regex::Regex::new(r"<[^>]+>")
                .unwrap()
                .replace_all(&result, "")
                .to_string();
        }
        
        result
    }
    
    /// Import WebVTT format
    fn import_webvtt(content: &str) -> Result<String> {
        let mut output = String::new();
        
        // Add ASS header
        output.push_str("[Script Info]\n");
        output.push_str("Title: Imported from WebVTT\n");
        output.push_str("ScriptType: v4.00+\n");
        output.push_str("WrapStyle: 0\n");
        output.push_str("PlayResX: 640\n");
        output.push_str("PlayResY: 480\n");
        output.push_str("ScaledBorderAndShadow: yes\n\n");
        
        // Add default style
        output.push_str("[V4+ Styles]\n");
        output.push_str("Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n");
        output.push_str("Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1\n\n");
        
        // Add events section
        output.push_str("[Events]\n");
        output.push_str("Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");
        
        // Parse WebVTT cues
        let cues = Self::parse_webvtt_cues(content)?;
        for cue in cues {
            output.push_str(&format!(
                "Dialogue: 0,{},{},Default,,0,0,0,,{}\n",
                cue.start, cue.end, cue.text
            ));
        }
        
        Ok(output)
    }
    
    /// Parse WebVTT cues
    fn parse_webvtt_cues(content: &str) -> Result<Vec<WebVttCue>> {
        let mut cues = Vec::new();
        let mut current_cue: Option<WebVttCue> = None;
        let mut in_cue = false;
        
        for line in content.lines() {
            let line = line.trim();
            
            // Skip WEBVTT header and empty lines
            if line.starts_with("WEBVTT") || line.starts_with("NOTE") || line.is_empty() {
                if let Some(cue) = current_cue.take() {
                    cues.push(cue);
                }
                in_cue = false;
                continue;
            }
            
            // Check if it's a timestamp line
            if line.contains("-->") {
                current_cue = Some(WebVttCue::default());
                if let Some(ref mut cue) = current_cue {
                    let parts: Vec<&str> = line.split("-->").collect();
                    if parts.len() >= 2 {
                        cue.start = Self::parse_webvtt_time(parts[0].trim())?;
                        cue.end = Self::parse_webvtt_time(parts[1].trim())?;
                        in_cue = true;
                    }
                }
                continue;
            }
            
            // Otherwise it's cue text
            if in_cue {
                if let Some(ref mut cue) = current_cue {
                    if !cue.text.is_empty() {
                        cue.text.push_str("\\N");
                    }
                    let converted_text = Self::convert_webvtt_formatting(line);
                    cue.text.push_str(&converted_text);
                }
            }
        }
        
        // Don't forget the last cue
        if let Some(cue) = current_cue {
            cues.push(cue);
        }
        
        Ok(cues)
    }
    
    /// Parse WebVTT timestamp to ASS format
    fn parse_webvtt_time(time: &str) -> Result<String> {
        // WebVTT format: 00:00:00.000 or 00:00.000
        // ASS format: 0:00:00.00
        
        let parts: Vec<&str> = time.split(':').collect();
        
        let (hours, minutes, seconds_str) = if parts.len() == 3 {
            // HH:MM:SS.mmm
            (parts[0].parse::<u32>().unwrap_or(0), parts[1], parts[2])
        } else if parts.len() == 2 {
            // MM:SS.mmm
            (0, parts[0], parts[1])
        } else {
            return Err(EditorError::ValidationError {
                message: format!("Invalid WebVTT timestamp: {time}"),
            });
        };
        
        let minutes: u32 = minutes.parse().map_err(|_| EditorError::ValidationError {
            message: format!("Invalid minutes in timestamp: {minutes}"),
        })?;
        
        let seconds_parts: Vec<&str> = seconds_str.split('.').collect();
        let seconds: u32 = seconds_parts[0].parse().map_err(|_| EditorError::ValidationError {
            message: format!("Invalid seconds in timestamp: {}", seconds_parts[0]),
        })?;
        
        let centiseconds = if seconds_parts.len() > 1 {
            // Convert milliseconds to centiseconds
            let millis: u32 = seconds_parts[1].parse().unwrap_or(0);
            millis / 10
        } else {
            0
        };
        
        Ok(format!("{hours}:{minutes:02}:{seconds:02}.{centiseconds:02}"))
    }
    
    /// Convert WebVTT formatting to ASS
    fn convert_webvtt_formatting(text: &str) -> String {
        let mut result = text.to_string();
        
        // Convert WebVTT tags
        result = result.replace("<i>", "{\\i1}");
        result = result.replace("</i>", "{\\i0}");
        result = result.replace("<b>", "{\\b1}");
        result = result.replace("</b>", "{\\b0}");
        result = result.replace("<u>", "{\\u1}");
        result = result.replace("</u>", "{\\u0}");
        
        // Convert voice spans
        result = regex::Regex::new(r"<v\s+([^>]+)>")
            .unwrap()
            .replace_all(&result, "")
            .to_string();
        result = result.replace("</v>", "");
        
        // Remove any other tags
        result = regex::Regex::new(r"<[^>]+>")
            .unwrap()
            .replace_all(&result, "")
            .to_string();
        
        result
    }
    
    /// Import plain text
    fn import_plain_text(content: &str) -> Result<String> {
        let mut output = String::new();
        
        // Add minimal ASS header
        output.push_str("[Script Info]\n");
        output.push_str("Title: Imported from Plain Text\n");
        output.push_str("ScriptType: v4.00+\n\n");
        
        output.push_str("[V4+ Styles]\n");
        output.push_str("Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n");
        output.push_str("Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1\n\n");
        
        output.push_str("[Events]\n");
        output.push_str("Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");
        
        // Create a single dialogue line with all text
        let text = content.lines().collect::<Vec<_>>().join("\\N");
        output.push_str(&format!(
            "Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,{text}\n"
        ));
        
        Ok(output)
    }
    
    /// Export to SSA format
    fn export_ssa(document: &EditorDocument, _options: &ConversionOptions) -> Result<String> {
        // SSA is very similar to ASS, just with slightly different headers
        let content = document.text();
        let mut output = content.replace("[V4+ Styles]", "[V4 Styles]");
        output = output.replace("ScriptType: v4.00+", "ScriptType: v4.00");
        Ok(output)
    }
    
    /// Export to SRT format
    fn export_srt(document: &EditorDocument, options: &ConversionOptions) -> Result<String> {
        let mut output = String::new();
        let mut index = 1;
        
        document.parse_script_with(|script| {
            for section in script.sections() {
                if let ass_core::parser::ast::Section::Events(events) = section {
                    for event in events {
                        if event.event_type == EventType::Dialogue {
                            // Add index
                            output.push_str(&format!("{index}\n"));
                            index += 1;
                            
                            // Add timestamps
                            let start = Self::ass_time_to_srt(event.start);
                            let end = Self::ass_time_to_srt(event.end);
                            output.push_str(&format!("{start} --> {end}\n"));
                            
                            // Add text
                            let text = if options.strip_formatting {
                                Self::strip_ass_tags(event.text)
                            } else {
                                Self::convert_ass_to_srt_formatting(event.text)
                            };
                            output.push_str(&text.replace("\\N", "\n"));
                            output.push_str("\n\n");
                        }
                    }
                }
            }
        })?;
        
        Ok(output)
    }
    
    /// Convert ASS time to SRT format
    fn ass_time_to_srt(time: &str) -> String {
        // ASS format: 0:00:00.00
        // SRT format: 00:00:00,000
        
        let parts: Vec<&str> = time.split(':').collect();
        if parts.len() != 3 {
            return time.to_string();
        }
        
        let hours = format!("{:02}", parts[0].parse::<u32>().unwrap_or(0));
        let minutes = parts[1];
        
        let seconds_parts: Vec<&str> = parts[2].split('.').collect();
        let seconds = seconds_parts[0];
        let centiseconds = seconds_parts.get(1).unwrap_or(&"00");
        
        // Convert centiseconds to milliseconds
        let millis = centiseconds.parse::<u32>().unwrap_or(0) * 10;
        
        format!("{hours}:{minutes}:{seconds},{millis:03}")
    }
    
    /// Convert ASS formatting to SRT
    fn convert_ass_to_srt_formatting(text: &str) -> String {
        let mut result = text.to_string();
        
        // Convert basic formatting
        result = result.replace("{\\i1}", "<i>");
        result = result.replace("{\\i0}", "</i>");
        result = result.replace("{\\b1}", "<b>");
        result = result.replace("{\\b0}", "</b>");
        result = result.replace("{\\u1}", "<u>");
        result = result.replace("{\\u0}", "</u>");
        
        // Remove all other ASS tags
        while let Some(start) = result.find('{') {
            if let Some(end) = result[start..].find('}') {
                result.replace_range(start..start + end + 1, "");
            } else {
                break;
            }
        }
        
        result
    }
    
    /// Strip all ASS tags from text
    fn strip_ass_tags(text: &str) -> String {
        let mut result = text.to_string();
        while let Some(start) = result.find('{') {
            if let Some(end) = result[start..].find('}') {
                result.replace_range(start..start + end + 1, "");
            } else {
                break;
            }
        }
        result
    }
    
    /// Export to WebVTT format
    fn export_webvtt(document: &EditorDocument, options: &ConversionOptions) -> Result<String> {
        let mut output = String::new();
        
        // Add WebVTT header
        output.push_str("WEBVTT\n\n");
        
        // Add style block if requested
        if let FormatOptions::WebVTT { include_style_block: true, .. } = &options.format_options {
            output.push_str("STYLE\n");
            output.push_str("::cue {\n");
            output.push_str("  background-image: linear-gradient(to bottom, dimgray, lightgray);\n");
            output.push_str("  color: papayawhip;\n");
            output.push_str("}\n\n");
        }
        
        document.parse_script_with(|script| {
            for section in script.sections() {
                if let ass_core::parser::ast::Section::Events(events) = section {
                    for event in events {
                        if event.event_type == EventType::Dialogue {
                            // Add timestamps
                            let start = Self::ass_time_to_webvtt(event.start);
                            let end = Self::ass_time_to_webvtt(event.end);
                            output.push_str(&format!("{start} --> {end}"));
                            
                            // Add cue settings if requested
                            if let FormatOptions::WebVTT { use_cue_settings: true, .. } = &options.format_options {
                                // Parse margins as integers for positioning
                                let margin_v: i32 = event.margin_v.parse().unwrap_or(0);
                                if margin_v != 0 {
                                    output.push_str(&format!(" line:{}", 100 - margin_v));
                                }
                            }
                            
                            output.push('\n');
                            
                            // Add text
                            let text = if options.strip_formatting {
                                Self::strip_ass_tags(event.text)
                            } else {
                                Self::convert_ass_to_webvtt_formatting(event.text)
                            };
                            output.push_str(&text.replace("\\N", "\n"));
                            output.push_str("\n\n");
                        }
                    }
                }
            }
        })?;
        
        Ok(output)
    }
    
    /// Convert ASS time to WebVTT format
    fn ass_time_to_webvtt(time: &str) -> String {
        // ASS format: 0:00:00.00
        // WebVTT format: 00:00:00.000
        
        let parts: Vec<&str> = time.split(':').collect();
        if parts.len() != 3 {
            return time.to_string();
        }
        
        let hours = format!("{:02}", parts[0].parse::<u32>().unwrap_or(0));
        let minutes = parts[1];
        
        let seconds_parts: Vec<&str> = parts[2].split('.').collect();
        let seconds = seconds_parts[0];
        let centiseconds = seconds_parts.get(1).unwrap_or(&"00");
        
        // Convert centiseconds to milliseconds
        let millis = centiseconds.parse::<u32>().unwrap_or(0) * 10;
        
        format!("{hours}:{minutes}:{seconds}.{millis:03}")
    }
    
    /// Convert ASS formatting to WebVTT
    fn convert_ass_to_webvtt_formatting(text: &str) -> String {
        let mut result = text.to_string();
        
        // Convert basic formatting
        result = result.replace("{\\i1}", "<i>");
        result = result.replace("{\\i0}", "</i>");
        result = result.replace("{\\b1}", "<b>");
        result = result.replace("{\\b0}", "</b>");
        result = result.replace("{\\u1}", "<u>");
        result = result.replace("{\\u0}", "</u>");
        
        // Remove all other ASS tags
        while let Some(start) = result.find('{') {
            if let Some(end) = result[start..].find('}') {
                result.replace_range(start..start + end + 1, "");
            } else {
                break;
            }
        }
        
        result
    }
    
    /// Export to plain text
    fn export_plain_text(document: &EditorDocument, options: &ConversionOptions) -> Result<String> {
        let mut output = String::new();
        
        document.parse_script_with(|script| {
            for section in script.sections() {
                if let ass_core::parser::ast::Section::Events(events) = section {
                    for event in events {
                        if event.event_type == EventType::Dialogue {
                            let text = if options.strip_formatting {
                                Self::strip_ass_tags(event.text)
                            } else {
                                event.text.to_string()
                            };
                            output.push_str(&text.replace("\\N", "\n"));
                            output.push('\n');
                        }
                    }
                }
            }
        })?;
        
        Ok(output)
    }
}

/// Helper struct for SRT entries
#[derive(Default)]
struct SrtEntry {
    start: String,
    end: String,
    text: String,
}

/// Helper struct for WebVTT cues
#[derive(Default)]
struct WebVttCue {
    start: String,
    end: String,
    text: String,
}

/// Import content from a file path
#[cfg(feature = "std")]
pub fn import_from_file(path: &str) -> Result<EditorDocument> {
    use std::fs;
    
    let content = fs::read_to_string(path)
        .map_err(|e| EditorError::IoError(e.to_string()))?;
    
    let format = path.rfind('.')
        .and_then(|pos| SubtitleFormat::from_extension(&path[pos + 1..]));
    
    let ass_content = FormatConverter::import(&content, format)?;
    EditorDocument::from_content(&ass_content)
}

/// Export document to a file
#[cfg(feature = "std")]
pub fn export_to_file(
    document: &EditorDocument,
    path: &str,
    format: Option<SubtitleFormat>,
    options: &ConversionOptions,
) -> Result<()> {
    use std::fs;
    
    let detected_format = format.or_else(|| {
        path.rfind('.')
            .and_then(|pos| SubtitleFormat::from_extension(&path[pos + 1..]))
    }).unwrap_or(SubtitleFormat::ASS);
    
    let content = FormatConverter::export(document, detected_format, options)?;
    
    fs::write(path, content)
        .map_err(|e| EditorError::IoError(e.to_string()))?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_detection() {
        assert_eq!(SubtitleFormat::from_extension("ass"), Some(SubtitleFormat::ASS));
        assert_eq!(SubtitleFormat::from_extension("srt"), Some(SubtitleFormat::SRT));
        assert_eq!(SubtitleFormat::from_extension("vtt"), Some(SubtitleFormat::WebVTT));
        assert_eq!(SubtitleFormat::from_extension("unknown"), None);
        
        assert_eq!(
            SubtitleFormat::from_content("[Script Info]\nTitle: Test"),
            SubtitleFormat::ASS
        );
        assert_eq!(
            SubtitleFormat::from_content("WEBVTT\n\n00:00.000 --> 00:05.000"),
            SubtitleFormat::WebVTT
        );
        assert_eq!(
            SubtitleFormat::from_content("1\n00:00:00,000 --> 00:00:05,000\nHello"),
            SubtitleFormat::SRT
        );
    }
    
    #[test]
    fn test_srt_import() {
        let srt_content = r#"1
00:00:00,000 --> 00:00:05,000
Hello <i>world</i>!

2
00:00:05,000 --> 00:00:10,000
This is a <b>test</b>."#;
        
        let result = FormatConverter::import(srt_content, Some(SubtitleFormat::SRT)).unwrap();
        
        assert!(result.contains("[Script Info]"));
        assert!(result.contains("[Events]"));
        assert!(result.contains("Hello {\\i1}world{\\i0}!"));
        assert!(result.contains("This is a {\\b1}test{\\b0}."));
    }
    
    #[test]
    fn test_webvtt_import() {
        let webvtt_content = r#"WEBVTT

00:00:00.000 --> 00:00:05.000
Hello <i>world</i>!

00:00:05.000 --> 00:00:10.000
This is a test."#;
        
        let result = FormatConverter::import(webvtt_content, Some(SubtitleFormat::WebVTT)).unwrap();
        
        assert!(result.contains("[Script Info]"));
        assert!(result.contains("[Events]"));
        assert!(result.contains("Hello {\\i1}world{\\i0}!"));
    }
    
    #[test]
    fn test_export_srt() {
        let doc = EditorDocument::from_content(
            r#"[Script Info]
Title: Test

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello {\i1}world{\i0}!
Dialogue: 0,0:00:05.00,0:00:10.00,Default,,0,0,0,,Test line\NSecond line"#
        ).unwrap();
        
        let options = ConversionOptions::default();
        let result = FormatConverter::export(&doc, SubtitleFormat::SRT, &options).unwrap();
        
        assert!(result.contains("1\n00:00:00,000 --> 00:00:05,000"));
        assert!(result.contains("Hello <i>world</i>!"));
        assert!(result.contains("Test line\nSecond line"));
    }
    
    #[test]
    fn test_export_webvtt() {
        let doc = EditorDocument::from_content(
            r#"[Script Info]
Title: Test

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello world!"#
        ).unwrap();
        
        let options = ConversionOptions::default();
        let result = FormatConverter::export(&doc, SubtitleFormat::WebVTT, &options).unwrap();
        
        assert!(result.starts_with("WEBVTT"));
        assert!(result.contains("00:00:00.000 --> 00:00:05.000"));
        assert!(result.contains("Hello world!"));
    }
    
    #[test]
    fn test_strip_formatting() {
        let doc = EditorDocument::from_content(
            r#"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,{\i1}Hello{\i0} {\b1}world{\b0}!"#
        ).unwrap();
        
        let options = ConversionOptions {
            strip_formatting: true,
            ..Default::default()
        };
        
        let result = FormatConverter::export(&doc, SubtitleFormat::SRT, &options).unwrap();
        assert!(result.contains("Hello world!"));
        assert!(!result.contains("<i>"));
        assert!(!result.contains("<b>"));
    }
}