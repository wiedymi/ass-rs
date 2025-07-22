//! AST (Abstract Syntax Tree) definitions for ASS scripts
//!
//! Provides zero-copy AST nodes using lifetime-generic design for maximum performance.
//! All nodes reference spans in the original source text to avoid allocations.
//!
//! # Thread Safety
//!
//! All AST nodes are immutable after construction and implement `Send + Sync`
//! for safe multi-threaded access.
//!
//! # Performance
//!
//! - Zero allocations via `&'a str` spans
//! - Memory usage ~1.1x input size
//! - Validation via pointer arithmetic for span checking
//!
//! # Examples
//!
//! ```rust
//! use ass_core::parser::ast::{Section, ScriptInfo, Event, EventType};
//!
//! // Create script info
//! let info = ScriptInfo { fields: vec![("Title", "Test")] };
//! let section = Section::ScriptInfo(info);
//!
//! // Create dialogue event
//! let event = Event {
//!     event_type: EventType::Dialogue,
//!     start: "0:00:05.00",
//!     end: "0:00:10.00",
//!     text: "Hello World!",
//!     ..Event::default()
//! };
//! ```

mod event;
mod media;
mod script_info;
mod section;
mod style;

// Re-export all public types to maintain API compatibility
pub use event::{Event, EventType};
pub use media::{Font, Graphic};
pub use script_info::ScriptInfo;
pub use section::{Section, SectionType};
pub use style::Style;

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn ast_integration_script_info() {
        let fields = vec![("Title", "Integration Test"), ("ScriptType", "v4.00+")];
        let info = ScriptInfo { fields };
        let section = Section::ScriptInfo(info);

        assert_eq!(section.section_type(), SectionType::ScriptInfo);
    }

    #[test]
    fn ast_integration_events() {
        let event = Event {
            event_type: EventType::Dialogue,
            start: "0:00:05.00",
            end: "0:00:10.00",
            style: "Default",
            text: "Test dialogue",
            ..Event::default()
        };

        let events = vec![event];
        let section = Section::Events(events);

        assert_eq!(section.section_type(), SectionType::Events);
    }

    #[test]
    fn ast_integration_styles() {
        let style = Style {
            name: "TestStyle",
            fontname: "Arial",
            fontsize: "20",
            ..Style::default()
        };

        let styles = vec![style];
        let section = Section::Styles(styles);

        assert_eq!(section.section_type(), SectionType::Styles);
    }

    #[test]
    fn ast_integration_fonts() {
        let font = Font {
            filename: "test.ttf",
            data_lines: vec!["encoded data line 1", "encoded data line 2"],
        };

        let fonts = vec![font];
        let section = Section::Fonts(fonts);

        assert_eq!(section.section_type(), SectionType::Fonts);
    }

    #[test]
    fn ast_integration_graphics() {
        let graphic = Graphic {
            filename: "logo.png",
            data_lines: vec!["encoded image data"],
        };

        let graphics = vec![graphic];
        let section = Section::Graphics(graphics);

        assert_eq!(section.section_type(), SectionType::Graphics);
    }

    #[test]
    fn event_type_round_trip() {
        let types = [
            EventType::Dialogue,
            EventType::Comment,
            EventType::Picture,
            EventType::Sound,
            EventType::Movie,
            EventType::Command,
        ];

        for event_type in types {
            let str_repr = event_type.as_str();
            let parsed = EventType::parse_type(str_repr);
            assert_eq!(parsed, Some(event_type));
        }
    }

    #[test]
    fn section_type_properties() {
        assert!(SectionType::ScriptInfo.is_required());
        assert!(SectionType::Events.is_required());
        assert!(!SectionType::Styles.is_required());

        assert!(SectionType::Events.is_timed());
        assert!(!SectionType::ScriptInfo.is_timed());

        assert_eq!(SectionType::ScriptInfo.header_name(), "Script Info");
        assert_eq!(SectionType::Events.header_name(), "Events");
    }
}
