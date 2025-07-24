//! Benchmark utilities for generating synthetic ASS scripts
//!
//! This module provides generators for creating test ASS scripts with varying
//! complexity levels, used primarily for benchmarking parser performance.
//! All generators produce valid ASS format strings that can be parsed by
//! the core parser.

use crate::parser::{ast::EventType, Event};
use std::fmt::Write;

/// Synthetic ASS script generator for benchmarking
pub struct ScriptGenerator {
    /// Script title for metadata
    pub title: String,
    /// Number of styles to generate
    pub styles_count: usize,
    /// Number of events to generate
    pub events_count: usize,
    /// Complexity level for generated content
    pub complexity_level: ComplexityLevel,
}

/// Script complexity levels for testing
#[derive(Debug, Clone, Copy)]
pub enum ComplexityLevel {
    /// Simple text with minimal formatting
    Simple,
    /// Moderate formatting and some animations
    Moderate,
    /// Heavy animations, complex styling, karaoke
    Complex,
    /// Extreme complexity to stress-test parser
    Extreme,
}

impl ScriptGenerator {
    /// Create generator for simple scripts
    #[must_use]
    pub fn simple(events_count: usize) -> Self {
        Self {
            title: "Simple Benchmark Script".to_string(),
            styles_count: 1,
            events_count,
            complexity_level: ComplexityLevel::Simple,
        }
    }

    /// Create generator for moderate complexity scripts
    #[must_use]
    pub fn moderate(events_count: usize) -> Self {
        Self {
            title: "Moderate Benchmark Script".to_string(),
            styles_count: 5,
            events_count,
            complexity_level: ComplexityLevel::Moderate,
        }
    }

    /// Create generator for complex scripts
    #[must_use]
    pub fn complex(events_count: usize) -> Self {
        Self {
            title: "Complex Benchmark Script".to_string(),
            styles_count: 10,
            events_count,
            complexity_level: ComplexityLevel::Complex,
        }
    }

    /// Create generator for extreme complexity scripts
    #[must_use]
    pub fn extreme(events_count: usize) -> Self {
        Self {
            title: "Extreme Benchmark Script".to_string(),
            styles_count: 20,
            events_count,
            complexity_level: ComplexityLevel::Extreme,
        }
    }

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
            let style_name = if i == 0 {
                "Default"
            } else {
                &format!("Style{i}")
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
    fn format_time(centiseconds: u32) -> String {
        let hours = centiseconds / 360_000;
        let minutes = (centiseconds % 360_000) / 6_000;
        let seconds = (centiseconds % 6000) / 100;
        let cs = centiseconds % 100;
        format!("{hours}:{minutes:02}:{seconds:02}.{cs:02}")
    }

    /// Generate dialogue text based on complexity level
    fn generate_dialogue_text(&self, event_index: usize) -> String {
        let base_text = format!("This is dialogue line number {}", event_index + 1);

        match self.complexity_level {
            ComplexityLevel::Simple => base_text,
            ComplexityLevel::Moderate => {
                format!(r"{{\b1}}{base_text}{{\b0}} with {{\i1}}some{{\i0}} formatting")
            }
            ComplexityLevel::Complex => {
                format!(
                    r"{{\pos(100,200)\fad(500,500)\b1\i1\c&H00FF00&}}{base_text}{{\b0\i0\c&HFFFFFF&}} with {{\t(0,1000,\frz360)}}animation{{\t(1000,2000,\frz0)}}"
                )
            }
            ComplexityLevel::Extreme => {
                format!(
                    r"{{\pos(100,200)\move(100,200,500,400)\fad(300,300)\t(0,500,\fscx120\fscy120)\t(500,1000,\fscx100\fscy100)\b1\i1\u1\s1\bord2\shad2\c&H00FF00&\3c&H0000FF&\4c&H000000&\alpha&H00\3a&H80}}{base_text}{{\b0\i0\u0\s0\r}} {{\k50}}with {{\k30}}karaoke {{\k40}}timing {{\k60}}and {{\k45}}complex {{\k35}}animations"
                )
            }
        }
    }
}

/// Create a test event with the given parameters
///
/// This is a convenience function for creating `Event` instances in tests
/// and benchmarks without having to specify all the fields manually.
///
/// # Examples
///
/// ```
/// use ass_core::utils::benchmark_generators::create_test_event;
///
/// let event = create_test_event("0:00:00.00", "0:00:05.00", "Hello world");
/// assert_eq!(event.text, "Hello world");
/// ```
#[must_use]
pub const fn create_test_event<'a>(start: &'a str, end: &'a str, text: &'a str) -> Event<'a> {
    Event {
        event_type: EventType::Dialogue,
        layer: "0",
        start,
        end,
        style: "Default",
        name: "",
        margin_l: "0",
        margin_r: "0",
        margin_v: "0",
        effect: "",
        text,
    }
}

/// Generate script with intentional issues for linting benchmarks
///
/// Creates an ASS script containing various problematic patterns that
/// linting rules should detect, such as empty tags, unknown tags, and
/// performance-heavy animations.
///
/// # Arguments
///
/// * `event_count` - Number of events to generate in the script
///
/// # Returns
///
/// A complete ASS script string with intentional issues for testing
/// linting performance and accuracy.
#[must_use]
pub fn generate_script_with_issues(event_count: usize) -> String {
    let mut script = String::from(
        "[Script Info]\n\
        Title: Test Script\n\n\
        [V4+ Styles]\n\
        Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n\
        Style: Default,Arial,20,&H00FFFFFF&,&H000000FF&,&H00000000&,&H00000000&,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1\n\n\
        [Events]\n\
        Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n"
    );

    for i in 0..event_count {
        let start_time = format!("0:{:02}:{:02}.00", i / 60, i % 60);
        let end_time = format!("0:{:02}:{:02}.50", i / 60, i % 60);

        // Add some problematic content every 10th event
        let text = if i % 10 == 0 {
            r"Text with {\} empty tag and {\invalidtag} unknown tag"
        } else if i % 7 == 0 {
            // Very complex animation that might cause performance issues
            r"{\pos(100,200)\move(100,200,500,400,0,5000)\t(0,1000,\frz360)\t(1000,2000,\fscx200\fscy200)\t(2000,3000,\alpha&HFF&)\t(3000,4000,\alpha&H00&)\t(4000,5000,\c&HFF0000&)}Performance heavy animation"
        } else {
            let line_num = i + 1;
            &format!("Normal dialogue line {line_num}")
        };

        writeln!(
            script,
            "Dialogue: 0,{start_time},{end_time},Default,Speaker,0,0,0,,{text}"
        )
        .unwrap();
    }

    script
}

/// Generate script with overlapping events for timing analysis benchmarks
///
/// Creates an ASS script where events have overlapping time ranges,
/// useful for testing overlap detection algorithms and timing analysis
/// performance.
///
/// # Arguments
///
/// * `event_count` - Number of overlapping events to generate
///
/// # Returns
///
/// A complete ASS script string with overlapping dialogue events.
#[must_use]
pub fn generate_overlapping_script(event_count: usize) -> String {
    let mut script = String::from(
        r"[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,2,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
",
    );

    for i in 0..event_count {
        // Create overlapping events: each event overlaps with several others
        let start_time = i * 2; // 2 second intervals
        let end_time = start_time + 5; // 5 second duration (overlaps next 2-3 events)
        writeln!(
            &mut script,
            "Dialogue: 0,0:{:02}:{:02}.00,0:{:02}:{:02}.00,Default,,0,0,0,,Event {} text",
            start_time / 60,
            start_time % 60,
            end_time / 60,
            end_time % 60,
            i
        )
        .unwrap();
    }

    script
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_generator_simple() {
        let generator = ScriptGenerator::simple(5);
        assert_eq!(generator.events_count, 5);
        assert_eq!(generator.styles_count, 1);
        assert!(matches!(
            generator.complexity_level,
            ComplexityLevel::Simple
        ));

        let script = generator.generate();
        assert!(script.contains("[Script Info]"));
        assert!(script.contains("[V4+ Styles]"));
        assert!(script.contains("[Events]"));
        assert!(script.contains("Simple Benchmark Script"));
    }

    #[test]
    fn script_generator_moderate() {
        let generator = ScriptGenerator::moderate(3);
        assert_eq!(generator.events_count, 3);
        assert_eq!(generator.styles_count, 5);
        assert!(matches!(
            generator.complexity_level,
            ComplexityLevel::Moderate
        ));
    }

    #[test]
    fn script_generator_complex() {
        let generator = ScriptGenerator::complex(2);
        assert_eq!(generator.events_count, 2);
        assert_eq!(generator.styles_count, 10);
        assert!(matches!(
            generator.complexity_level,
            ComplexityLevel::Complex
        ));
    }

    #[test]
    fn script_generator_extreme() {
        let generator = ScriptGenerator::extreme(1);
        assert_eq!(generator.events_count, 1);
        assert_eq!(generator.styles_count, 20);
        assert!(matches!(
            generator.complexity_level,
            ComplexityLevel::Extreme
        ));
    }

    #[test]
    fn format_time_zero() {
        assert_eq!(ScriptGenerator::format_time(0), "0:00:00.00");
    }

    #[test]
    fn format_time_basic() {
        assert_eq!(ScriptGenerator::format_time(6150), "0:01:01.50");
    }

    #[test]
    fn format_time_hours() {
        assert_eq!(ScriptGenerator::format_time(360_000), "1:00:00.00");
    }

    #[test]
    fn create_test_event_basic() {
        let event = create_test_event("0:00:00.00", "0:00:05.00", "Test text");
        assert_eq!(event.start, "0:00:00.00");
        assert_eq!(event.end, "0:00:05.00");
        assert_eq!(event.text, "Test text");
        assert_eq!(event.style, "Default");
        assert!(matches!(event.event_type, EventType::Dialogue));
    }

    #[test]
    fn generate_script_with_issues_basic() {
        let script = generate_script_with_issues(5);
        assert!(script.contains("[Script Info]"));
        assert!(script.contains("[V4+ Styles]"));
        assert!(script.contains("[Events]"));
        assert!(script.contains("Dialogue:"));
    }

    #[test]
    fn generate_script_with_issues_contains_problems() {
        let script = generate_script_with_issues(20);
        // Should contain some problematic content
        assert!(script.lines().count() > 10);
        // At least one event should have issues (every 10th event)
        assert!(script.contains("empty tag") || script.contains("unknown tag"));
    }

    #[test]
    fn generate_overlapping_script_basic() {
        let script = generate_overlapping_script(3);
        assert!(script.contains("[V4+ Styles]"));
        assert!(script.contains("[Events]"));
        assert!(script.contains("Event 0 text"));
        assert!(script.contains("Event 1 text"));
        assert!(script.contains("Event 2 text"));
    }

    #[test]
    fn generate_overlapping_script_timing() {
        let script = generate_overlapping_script(2);
        // First event: 0:00:00.00 to 0:00:05.00
        // Second event: 0:00:02.00 to 0:00:07.00 (overlaps with first)
        assert!(script.contains("0:00:00.00"));
        assert!(script.contains("0:00:05.00"));
        assert!(script.contains("0:00:02.00"));
        assert!(script.contains("0:00:07.00"));
    }

    #[test]
    fn dialogue_text_complexity_simple() {
        let generator = ScriptGenerator::simple(1);
        let text = generator.generate_dialogue_text(0);
        assert_eq!(text, "This is dialogue line number 1");
    }

    #[test]
    fn dialogue_text_complexity_moderate() {
        let generator = ScriptGenerator::moderate(1);
        let text = generator.generate_dialogue_text(0);
        assert!(text.contains(r"{\b1}"));
        assert!(text.contains(r"{\i1}"));
        assert!(text.contains("This is dialogue line number 1"));
    }

    #[test]
    fn dialogue_text_complexity_complex() {
        let generator = ScriptGenerator::complex(1);
        let text = generator.generate_dialogue_text(0);
        assert!(text.contains(r"{\pos("));
        assert!(text.contains(r"{\t("));
        assert!(text.contains("animation"));
    }

    #[test]
    fn dialogue_text_complexity_extreme() {
        let generator = ScriptGenerator::extreme(1);
        let text = generator.generate_dialogue_text(0);
        assert!(text.contains(r"{\k"));
        assert!(text.contains("karaoke"));
        assert!(text.contains("animations"));
    }

    #[test]
    fn script_generator_generate_has_correct_event_count() {
        let generator = ScriptGenerator::simple(3);
        let script = generator.generate();
        assert_eq!(
            script
                .lines()
                .filter(|line| line.starts_with("Dialogue:"))
                .count(),
            3
        );
    }

    #[test]
    fn script_generator_generate_has_correct_style_count() {
        let generator = ScriptGenerator::moderate(1); // 5 styles
        let script = generator.generate();
        assert_eq!(
            script
                .lines()
                .filter(|line| line.starts_with("Style:"))
                .count(),
            5
        );
    }
}
