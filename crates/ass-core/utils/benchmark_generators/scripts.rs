//! Standalone synthetic script builders for benchmarking and tests.
//!
//! Provides a convenience constructor for `Event` values plus generators that
//! emit scripts with intentional lint issues or overlapping event timings.

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::{fmt::Write, format, string::String};
#[cfg(feature = "std")]
use std::fmt::Write;

use crate::parser::{
    ast::{EventType, Span},
    Event,
};

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
        margin_t: None,
        margin_b: None,
        effect: "",
        text,
        span: Span::new(0, 0, 0, 0),
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
        let text_string;
        let text = if i % 10 == 0 {
            r"Text with {\} empty tag and {\invalidtag} unknown tag"
        } else if i % 7 == 0 {
            // Very complex animation that might cause performance issues
            r"{\pos(100,200)\move(100,200,500,400,0,5000)\t(0,1000,\frz360)\t(1000,2000,\fscx200\fscy200)\t(2000,3000,\alpha&HFF&)\t(3000,4000,\alpha&H00&)\t(4000,5000,\c&HFF0000&)}Performance heavy animation"
        } else {
            let line_num = i + 1;
            text_string = format!("Normal dialogue line {line_num}");
            &text_string
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
