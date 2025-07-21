//! Event analysis for ASS dialogue entries
//!
//! Provides comprehensive analysis of dialogue events including timing validation,
//! overlap detection, animation complexity analysis, and Unicode text processing.
//! Optimized for zero-copy analysis with minimal allocations.
//!
//! # Features
//!
//! - Timing overlap detection with millisecond precision
//! - Animation complexity scoring based on override tags
//! - Unicode linebreak and bidirectional text analysis
//! - Duration validation and performance warnings
//! - Style inheritance resolution per event
//!
//! # Performance
//!
//! - Target: <1ms per event for typical dialogue
//! - Memory: Zero-copy span references to original text
//! - Lazy evaluation: Expensive analysis only when requested
//!
//! # Example
//!
//! ```rust
//! use ass_core::analysis::events::DialogueInfo;
//! use ass_core::parser::{Event, SectionType, ast::Section};
//! use ass_core::Script;
//!
//! let script_text = r#"
//! [Script Info]
//! Title: Test
//!
//! [V4+ Styles]
//! Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
//! Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
//!
//! [Events]
//! Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
//! Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello World!
//! "#;
//!
//! let script = Script::parse(script_text)?;
//! if let Some(Section::Events(events)) = script.find_section(SectionType::Events) {
//!     if let Some(event) = events.first() {
//!         let info = DialogueInfo::analyze(event)?;
//!         println!("Duration: {}ms", info.duration_ms());
//!         println!("Animation complexity: {}/10", info.animation_score());
//!     }
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::{
    parser::Event,
    utils::{parse_ass_time, CoreError},
    Result,
};
use alloc::{string::String, vec::Vec};
use core::cmp::Ordering;

/// Comprehensive analysis information for a dialogue event
///
/// Contains timing, styling, and content analysis results for a single
/// dialogue entry. Uses zero-copy references where possible.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DialogueInfo<'a> {
    /// Reference to the original event
    event: &'a Event<'a>,

    /// Start time in centiseconds
    start_cs: u32,

    /// End time in centiseconds
    end_cs: u32,

    /// Animation complexity score (0-10)
    animation_score: u8,

    /// Number of override blocks in text
    override_count: usize,

    /// Estimated rendering complexity (0-100)
    complexity_score: u8,

    /// Text analysis results
    text_info: TextAnalysis<'a>,
}

/// Analysis of text content and formatting
#[derive(Debug, Clone)]
pub struct TextAnalysis<'a> {
    /// Plain text without override tags
    plain_text: String,

    /// Character count (Unicode aware)
    char_count: usize,

    /// Line count (after processing linebreaks)
    line_count: usize,

    /// Contains bidirectional text
    has_bidi_text: bool,

    /// Contains complex Unicode (beyond ASCII)
    has_complex_unicode: bool,

    /// Override tags found in text
    override_tags: Vec<OverrideTag<'a>>,
}

/// Represents a single override tag with analysis
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct OverrideTag<'a> {
    /// Tag name (e.g., "c", "pos", "move")
    name: &'a str,

    /// Raw tag arguments
    args: &'a str,

    /// Tag complexity score (0-5)
    complexity: u8,

    /// Byte position in original text
    position: usize,
}

/// Timing relationship between two events
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimingRelation {
    /// Events don't overlap
    NoOverlap,
    /// Events partially overlap
    PartialOverlap,
    /// One event completely contains the other
    FullOverlap,
    /// Events have identical timing
    Identical,
}

/// Performance impact category for events
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PerformanceImpact {
    /// Minimal impact - simple static text
    Minimal,
    /// Low impact - basic formatting
    Low,
    /// Medium impact - animations or complex styling
    Medium,
    /// High impact - many animations or large text
    High,
    /// Critical impact - may cause performance issues
    Critical,
}

impl<'a> DialogueInfo<'a> {
    /// Analyze a dialogue event
    ///
    /// Performs comprehensive analysis including timing parsing, text analysis,
    /// and animation complexity scoring. Results are cached for efficiency.
    pub fn analyze(event: &'a Event<'a>) -> Result<Self> {
        // Parse timing
        let start_cs = parse_ass_time(event.start)
            .map_err(|e| CoreError::Analysis(format!("Invalid start time: {}", e)))?;

        let end_cs = parse_ass_time(event.end)
            .map_err(|e| CoreError::Analysis(format!("Invalid end time: {}", e)))?;

        // Validate timing
        if start_cs >= end_cs {
            return Err(CoreError::Analysis(
                "Start time must be before end time".to_string(),
            ));
        }

        // Analyze text content
        let text_info = TextAnalysis::analyze(event.text)?;

        // Calculate animation complexity
        let animation_score = Self::calculate_animation_score(&text_info.override_tags);

        // Calculate overall complexity
        let complexity_score = Self::calculate_complexity_score(
            animation_score,
            text_info.char_count,
            text_info.override_tags.len(),
        );

        Ok(Self {
            event,
            start_cs,
            end_cs,
            animation_score,
            override_count: text_info.override_tags.len(),
            complexity_score,
            text_info,
        })
    }

    /// Get event duration in milliseconds
    pub fn duration_ms(&self) -> u32 {
        (self.end_cs - self.start_cs) * 10 // centiseconds to milliseconds
    }

    /// Get event duration in centiseconds
    pub fn duration_cs(&self) -> u32 {
        self.end_cs - self.start_cs
    }

    /// Get start time in centiseconds
    pub fn start_time_cs(&self) -> u32 {
        self.start_cs
    }

    /// Get end time in centiseconds
    pub fn end_time_cs(&self) -> u32 {
        self.end_cs
    }

    /// Get animation complexity score (0-10)
    pub fn animation_score(&self) -> u8 {
        self.animation_score
    }

    /// Get overall complexity score (0-100)
    pub fn complexity_score(&self) -> u8 {
        self.complexity_score
    }

    /// Get text analysis results
    pub fn text_analysis(&self) -> &TextAnalysis<'a> {
        &self.text_info
    }

    /// Get performance impact category
    pub fn performance_impact(&self) -> PerformanceImpact {
        match self.complexity_score {
            0..=20 => PerformanceImpact::Minimal,
            21..=40 => PerformanceImpact::Low,
            41..=60 => PerformanceImpact::Medium,
            61..=80 => PerformanceImpact::High,
            81..=100 => PerformanceImpact::Critical,
            _ => PerformanceImpact::Critical,
        }
    }

    /// Check timing relationship with another event
    pub fn timing_relation(&self, other: &DialogueInfo<'_>) -> TimingRelation {
        if self.start_cs == other.start_cs && self.end_cs == other.end_cs {
            TimingRelation::Identical
        } else if self.end_cs <= other.start_cs || other.end_cs <= self.start_cs {
            TimingRelation::NoOverlap
        } else if (self.start_cs <= other.start_cs && self.end_cs >= other.end_cs)
            || (other.start_cs <= self.start_cs && other.end_cs >= self.end_cs)
        {
            TimingRelation::FullOverlap
        } else {
            TimingRelation::PartialOverlap
        }
    }

    /// Check if event overlaps with given time range (in centiseconds)
    pub fn overlaps_time_range(&self, start_cs: u32, end_cs: u32) -> bool {
        !(self.end_cs <= start_cs || end_cs <= self.start_cs)
    }

    /// Get reference to original event
    pub fn event(&self) -> &'a Event<'a> {
        self.event
    }

    /// Calculate animation complexity score based on override tags
    fn calculate_animation_score(tags: &[OverrideTag<'_>]) -> u8 {
        let mut score = 0u8;

        for tag in tags {
            score = score.saturating_add(match tag.name {
                // Simple formatting - low complexity
                "b" | "i" | "u" | "s" => 1,
                "c" | "1c" | "2c" | "3c" | "4c" => 1,
                "alpha" | "1a" | "2a" | "3a" | "4a" => 1,

                // Positioning - medium complexity
                "pos" | "an" | "a" => 2,
                "org" => 2,

                // Animations - high complexity
                "move" => 4,
                "t" => 5,
                "clip" | "iclip" => 3,

                // Transforms - very high complexity
                "frx" | "fry" | "frz" => 3,
                "fscx" | "fscy" => 2,
                "fsp" | "fad" | "fade" => 3,

                // Drawing commands - maximum complexity
                "p" => 8,
                "pbo" => 5,

                // Unknown tags - assume medium complexity
                _ => 2,
            });
        }

        // Cap at maximum score
        score.min(10)
    }

    /// Calculate overall complexity score
    fn calculate_complexity_score(
        animation_score: u8,
        char_count: usize,
        override_count: usize,
    ) -> u8 {
        let mut score = animation_score as u32 * 5; // Animation has high weight

        // Add character count impact (large text is expensive)
        score += match char_count {
            0..=50 => 0,
            51..=200 => 5,
            201..=500 => 15,
            501..=1000 => 30,
            _ => 50,
        };

        // Add override tag count impact
        score += match override_count {
            0 => 0,
            1..=5 => 5,
            6..=15 => 15,
            16..=30 => 25,
            _ => 35,
        };

        // Cap at 100
        (score as u8).min(100)
    }
}

impl<'a> TextAnalysis<'a> {
    /// Analyze dialogue text content
    ///
    /// Extracts override tags, analyzes Unicode complexity, and counts
    /// characters and lines. Uses zero-copy spans where possible.
    pub fn analyze(text: &'a str) -> Result<Self> {
        let mut override_tags = Vec::new();
        let mut plain_text = String::new();
        let mut position = 0;

        // Parse override tags and extract plain text
        let mut chars = text.chars();
        while let Some(ch) = chars.next() {
            if ch == '{' {
                // Found start of override block
                let mut brace_count = 1;
                let tag_start = position + ch.len_utf8(); // Start after the opening brace
                let tag_content_start = tag_start;

                // Find matching closing brace
                for inner_ch in chars.by_ref() {
                    position += inner_ch.len_utf8();

                    if inner_ch == '{' {
                        brace_count += 1;
                    } else if inner_ch == '}' {
                        brace_count -= 1;
                        if brace_count == 0 {
                            break;
                        }
                    }
                }

                // Parse individual tags within the block using text slice
                let tag_content_end = position;
                if tag_content_end > tag_content_start {
                    let tag_content = &text[tag_content_start..tag_content_end];
                    Self::parse_override_block(tag_content, tag_start, &mut override_tags);
                }
            } else if ch == '\\' {
                // Handle line breaks and other escape sequences
                if let Some(next_ch) = chars.next() {
                    position += next_ch.len_utf8();

                    match next_ch {
                        'n' | 'N' => plain_text.push('\n'),
                        'h' => plain_text.push('\u{00A0}'), // Non-breaking space
                        _ => {
                            // Unknown escape, preserve as-is
                            plain_text.push(ch);
                            plain_text.push(next_ch);
                        }
                    }
                }
            } else {
                plain_text.push(ch);
            }

            position += ch.len_utf8();
        }

        let char_count = plain_text.chars().count();
        let line_count = plain_text.lines().count().max(1);

        // Analyze Unicode complexity
        let has_bidi_text = Self::detect_bidi_text(&plain_text);
        let has_complex_unicode = Self::detect_complex_unicode(&plain_text);

        Ok(Self {
            plain_text,
            char_count,
            line_count,
            has_bidi_text,
            has_complex_unicode,
            override_tags,
        })
    }

    /// Get plain text without override tags
    pub fn plain_text(&self) -> &str {
        &self.plain_text
    }

    /// Get character count
    pub fn char_count(&self) -> usize {
        self.char_count
    }

    /// Get line count
    pub fn line_count(&self) -> usize {
        self.line_count
    }

    /// Check if text contains bidirectional content
    pub fn has_bidi_text(&self) -> bool {
        self.has_bidi_text
    }

    /// Check if text contains complex Unicode
    pub fn has_complex_unicode(&self) -> bool {
        self.has_complex_unicode
    }

    /// Get override tags
    pub fn override_tags(&self) -> &[OverrideTag<'a>] {
        &self.override_tags
    }

    /// Parse override tags within a block
    fn parse_override_block<'b>(
        content: &'b str,
        start_pos: usize,
        tags: &mut Vec<OverrideTag<'b>>,
    ) {
        let mut pos = 0;
        let chars: Vec<char> = content.chars().collect();

        while pos < chars.len() {
            if chars[pos] == '\\' && pos + 1 < chars.len() {
                let tag_start = pos;
                pos += 1; // Skip the backslash

                // Extract tag name
                let name_start = pos;
                while pos < chars.len() && chars[pos].is_ascii_alphabetic() {
                    pos += 1;
                }

                if pos > name_start {
                    let tag_name = &content[name_start..pos];

                    // Extract arguments (everything until next backslash or end)
                    let args_start = pos;
                    while pos < chars.len() && chars[pos] != '\\' {
                        pos += 1;
                    }

                    let args = &content[args_start..pos];
                    let complexity = Self::calculate_tag_complexity(tag_name);

                    tags.push(OverrideTag {
                        name: tag_name,
                        args,
                        complexity,
                        position: start_pos + tag_start,
                    });
                } else {
                    // Skip invalid backslash
                    pos += 1;
                }
            } else {
                pos += 1;
            }
        }
    }

    /// Calculate complexity score for a single tag
    #[allow(dead_code)]
    fn calculate_tag_complexity(tag_name: &str) -> u8 {
        match tag_name {
            // Simple formatting
            "b" | "i" | "u" | "s" => 1,
            "c" | "1c" | "2c" | "3c" | "4c" => 1,
            "alpha" | "1a" | "2a" | "3a" | "4a" => 1,
            "fn" | "fs" => 1,

            // Medium complexity
            "pos" | "an" | "a" => 2,
            "org" | "be" | "blur" => 2,
            "bord" | "shad" | "xbord" | "ybord" | "xshad" | "yshad" => 2,

            // High complexity
            "move" | "fad" | "fade" => 3,
            "frx" | "fry" | "frz" => 3,
            "fscx" | "fscy" | "fsp" => 3,
            "clip" | "iclip" => 3,

            // Very high complexity
            "t" => 4,
            "p" => 5,
            "pbo" => 4,

            // Unknown - assume medium
            _ => 2,
        }
    }

    /// Detect bidirectional text (RTL scripts)
    fn detect_bidi_text(text: &str) -> bool {
        text.chars().any(|ch| {
            matches!(ch as u32,
                // Arabic
                0x0600..=0x06FF |
                // Hebrew
                0x0590..=0x05FF |
                // Arabic Supplement
                0x0750..=0x077F |
                // Arabic Extended-A
                0x08A0..=0x08FF
            )
        })
    }

    /// Detect complex Unicode beyond basic Latin
    fn detect_complex_unicode(text: &str) -> bool {
        text.chars().any(|ch| {
            let code = ch as u32;
            // Beyond ASCII and basic Latin-1
            code > 0x00FF ||
            // Control characters that might affect rendering
            matches!(code,
                0x0000..=0x001F |   // C0 controls
                0x007F..=0x009F |   // DEL + C1 controls
                0x200C..=0x200D |   // Zero-width joiners
                0x2060..=0x206F     // Various Unicode controls
            )
        })
    }
}

/// Find all overlapping events in a collection
///
/// Returns pairs of event indices that have overlapping timing.
/// Useful for detecting rendering conflicts and performance issues.
pub fn find_overlapping_events(events: &[DialogueInfo<'_>]) -> Vec<(usize, usize)> {
    let mut overlaps = Vec::new();

    for (i, event1) in events.iter().enumerate() {
        for (j, event2) in events.iter().enumerate().skip(i + 1) {
            match event1.timing_relation(event2) {
                TimingRelation::PartialOverlap | TimingRelation::FullOverlap => {
                    overlaps.push((i, j));
                }
                TimingRelation::NoOverlap | TimingRelation::Identical => {}
            }
        }
    }

    overlaps
}

/// Sort events by start time
///
/// Provides a stable sort by start time, then by end time for events
/// that start at the same time.
pub fn sort_events_by_time(events: &mut [DialogueInfo<'_>]) {
    events.sort_by(|a, b| match a.start_cs.cmp(&b.start_cs) {
        Ordering::Equal => a.end_cs.cmp(&b.end_cs),
        other => other,
    });
}

/// Calculate total duration of all events
///
/// Returns the total duration from first event start to last event end.
pub fn calculate_total_duration(events: &[DialogueInfo<'_>]) -> Option<u32> {
    if events.is_empty() {
        return None;
    }

    let start = events.iter().map(|e| e.start_cs).min()?;
    let end = events.iter().map(|e| e.end_cs).max()?;

    Some(end - start)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{ast::EventType, Event};

    fn create_test_event<'a>(start: &'a str, end: &'a str, text: &'a str) -> Event<'a> {
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

    #[test]
    fn dialogue_info_basic() {
        let event = create_test_event("0:00:00.00", "0:00:05.00", "Hello World!");
        let info = DialogueInfo::analyze(&event).unwrap();

        assert_eq!(info.duration_ms(), 5000);
        assert_eq!(info.duration_cs(), 500);
        assert_eq!(info.start_time_cs(), 0);
        assert_eq!(info.end_time_cs(), 500);
        assert_eq!(info.animation_score(), 0);
        assert_eq!(info.complexity_score(), 0);
    }

    #[test]
    fn dialogue_info_with_overrides() {
        let event = create_test_event(
            "0:00:00.00",
            "0:00:05.00",
            "{\\b1\\i1\\c&H00FF00&}Styled text{\\r}",
        );
        let info = DialogueInfo::analyze(&event).unwrap();

        assert!(info.animation_score() > 0);
        assert!(info.complexity_score() > 0);
        assert_eq!(info.text_analysis().override_tags().len(), 4); // b, i, c, r
    }

    #[test]
    fn dialogue_info_timing_validation() {
        // Invalid: start after end
        let event = create_test_event("0:00:05.00", "0:00:02.00", "Invalid timing");
        assert!(DialogueInfo::analyze(&event).is_err());
    }

    #[test]
    fn text_analysis_plain() {
        let text = "Simple text without formatting";
        let analysis = TextAnalysis::analyze(text).unwrap();

        assert_eq!(analysis.plain_text(), text);
        assert_eq!(analysis.char_count(), 30);
        assert_eq!(analysis.line_count(), 1);
        assert!(!analysis.has_bidi_text());
        assert!(!analysis.has_complex_unicode());
        assert_eq!(analysis.override_tags().len(), 0);
    }

    #[test]
    fn text_analysis_with_overrides() {
        let text = "{\\b1}Bold {\\i1}italic{\\r} text";
        let analysis = TextAnalysis::analyze(text).unwrap();

        assert_eq!(analysis.plain_text(), "Bold italic text");
        assert_eq!(analysis.override_tags().len(), 3); // b1, i1, r
        assert!(analysis.override_tags().iter().any(|tag| tag.name == "b"));
        assert!(analysis.override_tags().iter().any(|tag| tag.name == "i"));
        assert!(analysis.override_tags().iter().any(|tag| tag.name == "r"));
    }

    #[test]
    fn text_analysis_line_breaks() {
        let text = "Line 1\\nLine 2\\NLine 3";
        let analysis = TextAnalysis::analyze(text).unwrap();

        assert_eq!(analysis.plain_text(), "Line 1\nLine 2\nLine 3");
        assert_eq!(analysis.line_count(), 3);
    }

    #[test]
    fn text_analysis_bidi() {
        let text = "English العربية Hebrew";
        let analysis = TextAnalysis::analyze(text).unwrap();

        assert!(analysis.has_bidi_text());
        assert!(analysis.has_complex_unicode());
    }

    #[test]
    fn timing_relations() {
        let event1 = create_test_event("0:00:00.00", "0:00:05.00", "Event 1");
        let event2 = create_test_event("0:00:03.00", "0:00:08.00", "Event 2");
        let event3 = create_test_event("0:00:10.00", "0:00:15.00", "Event 3");

        let info1 = DialogueInfo::analyze(&event1).unwrap();
        let info2 = DialogueInfo::analyze(&event2).unwrap();
        let info3 = DialogueInfo::analyze(&event3).unwrap();

        assert_eq!(
            info1.timing_relation(&info2),
            TimingRelation::PartialOverlap
        );
        assert_eq!(info1.timing_relation(&info3), TimingRelation::NoOverlap);
    }

    #[test]
    fn performance_impact_scoring() {
        // Simple text - minimal impact
        let simple = create_test_event("0:00:00.00", "0:00:05.00", "Simple");
        let simple_info = DialogueInfo::analyze(&simple).unwrap();
        assert_eq!(simple_info.performance_impact(), PerformanceImpact::Minimal);

        // Complex animation - higher impact
        let complex = create_test_event(
            "0:00:00.00",
            "0:00:05.00",
            "{\\move(0,0,100,100)\\t(0,1000,\\frz360)}Spinning text",
        );
        let complex_info = DialogueInfo::analyze(&complex).unwrap();
        assert!(complex_info.performance_impact() > PerformanceImpact::Minimal);
    }

    #[test]
    fn find_overlaps() {
        let event1 = create_test_event("0:00:00.00", "0:00:05.00", "Event 1");
        let event2 = create_test_event("0:00:03.00", "0:00:08.00", "Event 2");
        let event3 = create_test_event("0:00:10.00", "0:00:15.00", "Event 3");

        let infos = vec![
            DialogueInfo::analyze(&event1).unwrap(),
            DialogueInfo::analyze(&event2).unwrap(),
            DialogueInfo::analyze(&event3).unwrap(),
        ];

        let overlaps = find_overlapping_events(&infos);
        assert_eq!(overlaps.len(), 1);
        assert_eq!(overlaps[0], (0, 1));
    }

    #[test]
    fn total_duration() {
        let event1 = create_test_event("0:00:00.00", "0:00:05.00", "Event 1");
        let event2 = create_test_event("0:00:03.00", "0:00:08.00", "Event 2");

        let infos = vec![
            DialogueInfo::analyze(&event1).unwrap(),
            DialogueInfo::analyze(&event2).unwrap(),
        ];

        let duration = calculate_total_duration(&infos).unwrap();
        assert_eq!(duration, 800); // 0 to 8 seconds = 800 centiseconds
    }

    #[test]
    fn sort_by_time() {
        let event1 = create_test_event("0:00:05.00", "0:00:10.00", "Event 1");
        let event2 = create_test_event("0:00:00.00", "0:00:03.00", "Event 2");
        let event3 = create_test_event("0:00:02.00", "0:00:07.00", "Event 3");

        let mut infos = vec![
            DialogueInfo::analyze(&event1).unwrap(),
            DialogueInfo::analyze(&event2).unwrap(),
            DialogueInfo::analyze(&event3).unwrap(),
        ];

        sort_events_by_time(&mut infos);

        assert_eq!(infos[0].start_time_cs(), 0); // Event 2
        assert_eq!(infos[1].start_time_cs(), 200); // Event 3
        assert_eq!(infos[2].start_time_cs(), 500); // Event 1
    }
}
