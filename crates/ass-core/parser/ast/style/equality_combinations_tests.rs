//! Per-field inequality combination tests for the `Style` AST node.

use super::*;

#[test]
fn style_equality_all_combinations() {
    let style1 = Style::default();
    let mut style2 = Style::default();

    // Should be equal initially
    assert_eq!(style1, style2);

    // Test each field for inequality
    style2.name = "Different";
    assert_ne!(style1, style2);
    style2 = Style::default();

    style2.fontname = "Different";
    assert_ne!(style1, style2);
    style2 = Style::default();

    style2.fontsize = "Different";
    assert_ne!(style1, style2);
    style2 = Style::default();

    style2.primary_colour = "Different";
    assert_ne!(style1, style2);
    style2 = Style::default();

    style2.secondary_colour = "Different";
    assert_ne!(style1, style2);
    style2 = Style::default();

    style2.outline_colour = "Different";
    assert_ne!(style1, style2);
    style2 = Style::default();

    style2.back_colour = "Different";
    assert_ne!(style1, style2);
    style2 = Style::default();

    style2.bold = "Different";
    assert_ne!(style1, style2);
    style2 = Style::default();

    style2.italic = "Different";
    assert_ne!(style1, style2);
    style2 = Style::default();

    style2.underline = "Different";
    assert_ne!(style1, style2);
    style2 = Style::default();

    style2.strikeout = "Different";
    assert_ne!(style1, style2);
    style2 = Style::default();

    style2.scale_x = "Different";
    assert_ne!(style1, style2);
    style2 = Style::default();

    style2.scale_y = "Different";
    assert_ne!(style1, style2);
    style2 = Style::default();

    style2.spacing = "Different";
    assert_ne!(style1, style2);
    style2 = Style::default();

    style2.angle = "Different";
    assert_ne!(style1, style2);
    style2 = Style::default();

    style2.border_style = "Different";
    assert_ne!(style1, style2);
    style2 = Style::default();

    style2.outline = "Different";
    assert_ne!(style1, style2);
    style2 = Style::default();

    style2.shadow = "Different";
    assert_ne!(style1, style2);
    style2 = Style::default();

    style2.alignment = "Different";
    assert_ne!(style1, style2);
    style2 = Style::default();

    style2.margin_l = "Different";
    assert_ne!(style1, style2);
    style2 = Style::default();

    style2.margin_r = "Different";
    assert_ne!(style1, style2);
    style2 = Style::default();

    style2.margin_v = "Different";
    assert_ne!(style1, style2);
    style2 = Style::default();

    style2.encoding = "Different";
    assert_ne!(style1, style2);
}
