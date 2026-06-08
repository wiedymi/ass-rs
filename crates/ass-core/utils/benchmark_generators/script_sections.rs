//! Script section builders for the synthetic ASS generator.
//!
//! Implements the high-level `generate` entry point along with the
//! `[Script Info]`, `[V4+ Styles]`, and `[Events]` section builders plus the
//! ASS time formatting helper.

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::{
    fmt::Write,
    format,
    string::{String, ToString},
};
#[cfg(feature = "std")]
use std::fmt::Write;

use super::ScriptGenerator;

impl ScriptGenerator {
    /// Generate complete ASS script as string
    #[must_use]
    pub fn generate(&self) -> String {
        let mut script =
            String::with_capacity(1000 + (self.styles_count * 200) + (self.events_count * 150));

        // Script Info section
        script.push_str(&self.generate_script_info());
        script.push('\n');

        // V4+ Styles section
        script.push_str(&self.generate_styles());
        script.push('\n');

        // Events section
        script.push_str(&self.generate_events());

        script
    }

    /// Generate Script Info section
    fn generate_script_info(&self) -> String {
        format!(
            r"[Script Info]
Title: {}
ScriptType: v4.00+
WrapStyle: 0
ScaledBorderAndShadow: yes
PlayResX: 1920
PlayResY: 1080",
            self.title
        )
    }

    /// Generate V4+ Styles section
    fn generate_styles(&self) -> String {
        let mut styles = String::from(
            "[V4+ Styles]\n\
            Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n"
        );

        for i in 0..self.styles_count {
            let style_name_string;
            let style_name = if i == 0 {
                "Default"
            } else {
                style_name_string = format!("Style{i}");
                &style_name_string
            };
            let fontsize = 20 + (i * 2);
            let color = format!("&H00{:06X}&", i * 0x0011_1111);

            writeln!(
                styles,
                "Style: {style_name},Arial,{fontsize},{color},{color},{color},&H00000000&,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1"
            ).unwrap();
        }

        styles
    }

    /// Generate Events section
    fn generate_events(&self) -> String {
        let mut events = String::from(
            "[Events]\n\
            Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n",
        );

        for i in 0..self.events_count {
            let start_cs = u32::try_from(i * 3000).unwrap_or(u32::MAX);
            let end_cs = u32::try_from(i * 3000 + 2500).unwrap_or(u32::MAX);
            let start_time = Self::format_time(start_cs); // 3 seconds apart
            let end_time = Self::format_time(end_cs); // 2.5 second duration
            let style = if self.styles_count > 1 {
                format!("Style{}", i % self.styles_count)
            } else {
                "Default".to_string()
            };
            let text = self.generate_dialogue_text(i);

            writeln!(
                events,
                "Dialogue: 0,{start_time},{end_time},{style},Speaker,0,0,0,,{text}"
            )
            .unwrap();
        }

        events
    }

    /// Format time in ASS format (H:MM:SS.cc)
    pub(super) fn format_time(centiseconds: u32) -> String {
        let hours = centiseconds / 360_000;
        let minutes = (centiseconds % 360_000) / 6_000;
        let seconds = (centiseconds % 6000) / 100;
        let cs = centiseconds % 100;
        format!("{hours}:{minutes:02}:{seconds:02}.{cs:02}")
    }
}
