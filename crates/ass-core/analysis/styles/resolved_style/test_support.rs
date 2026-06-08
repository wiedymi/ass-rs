//! Shared fixtures for the `ResolvedStyle` unit tests.

use crate::parser::ast::Span;
use crate::parser::Style;

#[cfg(not(feature = "std"))]
pub(super) fn create_test_style() -> Style<'static> {
    Style {
        name: "Test",
        parent: None,
        fontname: "Arial",
        fontsize: "20",
        primary_colour: "&H00FFFFFF",
        secondary_colour: "&H000000FF",
        outline_colour: "&H00000000",
        back_colour: "&H00000000",
        bold: "0",
        italic: "0",
        underline: "0",
        strikeout: "0",
        scale_x: "100",
        scale_y: "100",
        spacing: "0",
        angle: "0",
        border_style: "1",
        outline: "2",
        shadow: "0",
        alignment: "2",
        margin_l: "10",
        margin_r: "10",
        margin_v: "10",
        margin_t: None,
        margin_b: None,
        encoding: "1",
        relative_to: None,
        span: Span::new(0, 0, 0, 0),
    }
}

#[cfg(feature = "std")]
pub(super) fn create_test_style() -> Style<'static> {
    Style {
        name: "Test",
        parent: None,
        fontname: "Arial",
        fontsize: "20",
        primary_colour: "&H00FFFFFF",
        secondary_colour: "&H000000FF",
        outline_colour: "&H00000000",
        back_colour: "&H00000000",
        bold: "0",
        italic: "0",
        underline: "0",
        strikeout: "0",
        scale_x: "100",
        scale_y: "100",
        spacing: "0",
        angle: "0",
        border_style: "1",
        outline: "2",
        shadow: "0",
        alignment: "2",
        margin_l: "10",
        margin_r: "10",
        margin_v: "10",
        margin_t: None,
        margin_b: None,
        encoding: "1",
        relative_to: None,
        span: Span::new(0, 0, 0, 0),
    }
}
