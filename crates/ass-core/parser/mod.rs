//! ASS script parser module
//!
//! Provides zero-copy parsing of ASS subtitle scripts with lifetime-generic AST nodes.
//! Supports full ASS v4+, SSA v4 compatibility, and libass 0.17.4+ extensions.
//!
//! # Performance
//!
//! - Target: <5ms parsing for typical 1KB scripts
//! - Memory: ~1.1x input size via zero-copy spans
//! - Incremental updates: <2ms for single-event changes
//!
//! # Example
//!
//! ```rust
//! use ass_core::parser::Script;
//!
//! let script_text = r#"
//! [Script Info]
//! Title: Example
//! ScriptType: v4.00+
//!
//! [Events\]
//! Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
//! Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello World!
//! "#;
//!
//! let script = Script::parse(script_text)?;
//! assert_eq!(script.sections().len(), 2);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod ast;
pub mod binary_data;
pub mod errors;
pub mod incremental;
pub mod main;
pub mod position_tracker;
pub mod script;
pub mod sections;

#[cfg(feature = "stream")]
pub mod streaming;

// Re-export public API
pub use ast::{Event, ScriptInfo, Section, SectionType, Style};
pub use errors::{IssueCategory, IssueSeverity, ParseError, ParseIssue, ParseResult};
pub use script::Script;
#[cfg(feature = "stream")]
pub use script::{calculate_delta, ScriptDelta, ScriptDeltaOwned};

#[cfg(feature = "stream")]
pub use streaming::build_modified_source;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_unknown_section() {
        let script =
            Script::parse("[Script Info]\nTitle: Test\n[Unknown Section]\nSomething: here")
                .unwrap();
        assert_eq!(script.sections().len(), 1);
        assert_eq!(script.issues().len(), 1);
        assert_eq!(script.issues()[0].severity, IssueSeverity::Warning);
    }

    #[test]
    fn parse_with_custom_format() {
        let script_text = r"[Script Info]
Title: Format Test
ScriptType: v4.00+

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Custom,Arial,20,&H00FF0000&,&H000000FF&,&H00000000&,&H00000000&,1,0,0,0,100,100,0,0,1,2,0,2,15,15,15,1

[Events\]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Custom,,0,0,0,,Custom format test
";

        let script = Script::parse(script_text).unwrap();
        assert_eq!(script.sections().len(), 3);

        if let Some(Section::Styles(styles)) = script
            .sections()
            .iter()
            .find(|s| matches!(s, Section::Styles(_)))
        {
            assert_eq!(styles.len(), 1);
            let style = &styles[0];
            assert_eq!(style.name, "Custom");
            assert_eq!(style.fontname, "Arial");
            assert_eq!(style.fontsize, "20");
        } else {
            panic!("Should have found styles section");
        }

        if let Some(Section::Events(events)) = script
            .sections()
            .iter()
            .find(|s| matches!(s, Section::Events(_)))
        {
            assert_eq!(events.len(), 1);
            let event = &events[0];
            assert_eq!(event.start, "0:00:00.00");
            assert_eq!(event.layer, "0");
            assert_eq!(event.end, "0:00:05.00");
            assert_eq!(event.style, "Custom");
            assert_eq!(event.text, "Custom format test");
        } else {
            panic!("Should have found events section");
        }
    }
}
