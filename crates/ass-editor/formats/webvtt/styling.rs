//! WebVTT <-> ASS inline styling conversion.
//!
//! Translates WebVTT markup (`<b>`, `<i>`, class/voice/ruby tags, ...) into ASS
//! override tags and back, with the richer tag handling gated behind the
//! `formats` feature.

use super::WebVttFormat;

impl WebVttFormat {
    /// Convert WebVTT styling to ASS override tags
    pub(super) fn convert_vtt_to_ass_styling(text: &str) -> String {
        let mut result = text.to_string();

        // Convert WebVTT tags to ASS override tags
        result = result.replace("<b>", r"{\b1}");
        result = result.replace("</b>", r"{\b0}");
        result = result.replace("<i>", r"{\i1}");
        result = result.replace("</i>", r"{\i0}");
        result = result.replace("<u>", r"{\u1}");
        result = result.replace("</u>", r"{\u0}");

        // Handle WebVTT class-based styling
        #[cfg(feature = "formats")]
        {
            let class_regex = regex::Regex::new(r#"<c\.([^>]+)>([^<]*)</c>"#).unwrap();
            result = class_regex
                .replace_all(&result, r"{\c&H$1&}$2{\c}")
                .to_string();

            // Handle voice tags
            let voice_regex = regex::Regex::new(r#"<v\s+([^>]+)>([^<]*)</v>"#).unwrap();
            result = voice_regex
                .replace_all(&result, r"{\fn$1}$2{\fn}")
                .to_string();

            // Handle ruby text (convert to simple parentheses)
            let ruby_regex = regex::Regex::new(r#"<ruby>([^<]*)<rt>([^<]*)</rt></ruby>"#).unwrap();
            result = ruby_regex.replace_all(&result, "$1($2)").to_string();

            // Handle timestamp tags (cue settings)
            let timestamp_regex = regex::Regex::new(r#"<([0-9:.,]+)>"#).unwrap();
            result = timestamp_regex.replace_all(&result, "").to_string();
        }

        result
    }

    /// Convert ASS override tags to WebVTT styling
    pub(super) fn convert_ass_to_vtt_styling(text: &str) -> String {
        let mut result = text.to_string();

        // Convert ASS override tags to WebVTT tags
        result = result.replace(r"{\b1}", "<b>");
        result = result.replace(r"{\b0}", "</b>");
        result = result.replace(r"{\i1}", "<i>");
        result = result.replace(r"{\i0}", "</i>");
        result = result.replace(r"{\u1}", "<u>");
        result = result.replace(r"{\u0}", "</u>");

        #[cfg(feature = "formats")]
        {
            // Handle color tags
            let color_regex = regex::Regex::new(r"\\c&H([0-9A-Fa-f]{6})&").unwrap();
            result = color_regex.replace_all(&result, r#"<c.$1>"#).to_string();
            result = result.replace(r"{\c}", "</c>");

            // Handle font name tags
            let font_regex = regex::Regex::new(r"\\fn([^}]+)").unwrap();
            result = font_regex.replace_all(&result, r#"<v $1>"#).to_string();
            result = result.replace(r"{\fn}", "</v>");

            // Handle positioning tags (convert to WebVTT cue settings)
            let pos_regex = regex::Regex::new(r"\\pos\(([^,]+),([^)]+)\)").unwrap();
            result = pos_regex.replace_all(&result, "").to_string(); // Will be handled as cue settings

            // Remove any remaining ASS tags
            let cleanup_regex = regex::Regex::new(r"\{[^}]*\}").unwrap();
            result = cleanup_regex.replace_all(&result, "").to_string();
        }

        result
    }
}
