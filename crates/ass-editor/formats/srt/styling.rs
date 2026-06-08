//! Conversion between SRT inline styling and ASS override tags.

use super::SrtFormat;

impl SrtFormat {
    /// Convert SRT styling to ASS override tags
    pub(super) fn convert_srt_to_ass_styling(text: &str) -> String {
        let mut result = text.to_string();

        // Convert HTML-like tags to ASS override tags
        result = result.replace("<b>", r"{\b1}");
        result = result.replace("</b>", r"{\b0}");
        result = result.replace("<i>", r"{\i1}");
        result = result.replace("</i>", r"{\i0}");
        result = result.replace("<u>", r"{\u1}");
        result = result.replace("</u>", r"{\u0}");
        result = result.replace("<s>", r"{\s1}");
        result = result.replace("</s>", r"{\s0}");

        #[cfg(feature = "formats")]
        {
            // Handle font color tags
            let color_regex = regex::Regex::new(r#"<font color="?#?([0-9A-Fa-f]{6})"?>"#).unwrap();
            result = color_regex.replace_all(&result, r"{\c&H$1&}").to_string();
            result = result.replace("</font>", r"{\c}");

            // Handle font face tags
            let font_regex = regex::Regex::new(r#"<font face="([^"]+)">"#).unwrap();
            result = font_regex.replace_all(&result, r"{\fn$1}").to_string();
        }

        result
    }

    /// Convert ASS override tags to SRT styling
    pub(super) fn convert_ass_to_srt_styling(text: &str) -> String {
        let mut result = text.to_string();

        // Convert ASS override tags to HTML-like tags
        result = result.replace(r"{\b1}", "<b>");
        result = result.replace(r"{\b0}", "</b>");
        result = result.replace(r"{\i1}", "<i>");
        result = result.replace(r"{\i0}", "</i>");
        result = result.replace(r"{\u1}", "<u>");
        result = result.replace(r"{\u0}", "</u>");
        result = result.replace(r"{\s1}", "<s>");
        result = result.replace(r"{\s0}", "</s>");

        #[cfg(feature = "formats")]
        {
            // Handle color tags
            let color_regex = regex::Regex::new(r"\\c&H([0-9A-Fa-f]{6})&").unwrap();
            result = color_regex
                .replace_all(&result, "<font color=\"#$1\">")
                .to_string();
            result = result.replace(r"{\c}", "</font>");

            // Handle font name tags
            let font_regex = regex::Regex::new(r"\\fn([^}]+)").unwrap();
            result = font_regex
                .replace_all(&result, "<font face=\"$1\">")
                .to_string();

            // Remove any remaining ASS tags
            let cleanup_regex = regex::Regex::new(r"\{[^}]*\}").unwrap();
            result = cleanup_regex.replace_all(&result, "").to_string();
        }

        result
    }
}
