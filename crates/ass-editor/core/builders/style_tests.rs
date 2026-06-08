//! Tests for [`StyleBuilder`].

use super::*;
use ass_core::ScriptVersion;

#[test]
fn style_builder_default() {
    let style = StyleBuilder::default_style()
        .name("TestStyle")
        .font("Comic Sans MS")
        .size(24)
        .bold(true)
        .build()
        .unwrap();

    assert!(style.contains("Style: TestStyle"));
    assert!(style.contains("Comic Sans MS"));
    assert!(style.contains("24"));
    assert!(style.contains("-1")); // Bold = true
}

#[test]
fn style_builder_minimal() {
    let style = StyleBuilder::new().name("Minimal").build().unwrap();

    assert!(style.contains("Style: Minimal"));
    assert!(style.contains("Arial")); // Default font
}

#[test]
fn style_builder_all_fields() {
    let style = StyleBuilder::new()
        .name("Complete")
        .font("Helvetica")
        .size(18)
        .color("&Hffffff")
        .secondary_color("&H00ff00")
        .outline_color("&H0000ff")
        .back_color("&H808080")
        .bold(true)
        .italic(false)
        .underline(true)
        .strikeout(false)
        .scale_x(95.5)
        .scale_y(105.0)
        .spacing(1.5)
        .angle(15.0)
        .border_style(3)
        .outline(2.5)
        .shadow(1.0)
        .align(7)
        .margin_left(5)
        .margin_right(15)
        .margin_vertical(20)
        .margin_top(25)
        .margin_bottom(30)
        .encoding(0)
        .relative_to("video")
        .build()
        .unwrap();

    assert!(style.contains("Style: Complete"));
    assert!(style.contains("Helvetica"));
    assert!(style.contains("18"));
    assert!(style.contains("&Hffffff"));
    assert!(style.contains("&H00ff00"));
    assert!(style.contains("&H0000ff"));
    assert!(style.contains("&H808080"));
    assert!(style.contains("-1")); // bold = true
    assert!(style.contains("95.5"));
    assert!(style.contains("105"));
    assert!(style.contains("1.5"));
    assert!(style.contains("15")); // angle
    assert!(style.contains("3")); // border_style
    assert!(style.contains("2.5")); // outline
    assert!(style.contains("7")); // alignment
                                  // Note: margin_t, margin_b, and relative_to are stored but not in V4+ format output yet
}

#[test]
fn style_builder_with_format_v4plus() {
    let style = StyleBuilder::new()
        .name("TestStyle")
        .font("Arial")
        .size(20)
        .build_with_format(&[
            "Name",
            "Fontname",
            "Fontsize",
            "PrimaryColour",
            "SecondaryColour",
            "OutlineColour",
            "BackColour",
            "Bold",
            "Italic",
            "Underline",
            "StrikeOut",
            "ScaleX",
            "ScaleY",
            "Spacing",
            "Angle",
            "BorderStyle",
            "Outline",
            "Shadow",
            "Alignment",
            "MarginL",
            "MarginR",
            "MarginV",
            "Encoding",
        ])
        .unwrap();

    assert_eq!(style, "Style: TestStyle,Arial,20,&Hffffff,&Hff0000,&H0,&H0,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1");
}

#[test]
fn style_builder_with_format_v4plusplus() {
    let style = StyleBuilder::new()
        .name("V4++Style")
        .margin_top(15)
        .margin_bottom(20)
        .relative_to("video")
        .build_with_format(&[
            "Name",
            "Fontname",
            "Fontsize",
            "PrimaryColour",
            "SecondaryColour",
            "OutlineColour",
            "BackColour",
            "Bold",
            "Italic",
            "Underline",
            "StrikeOut",
            "ScaleX",
            "ScaleY",
            "Spacing",
            "Angle",
            "BorderStyle",
            "Outline",
            "Shadow",
            "Alignment",
            "MarginL",
            "MarginR",
            "MarginT",
            "MarginB",
            "Encoding",
            "RelativeTo",
        ])
        .unwrap();

    assert!(style.contains("V4++Style"));
    assert!(style.contains("15")); // MarginT
    assert!(style.contains("20")); // MarginB
    assert!(style.contains("video")); // RelativeTo
}

#[test]
fn style_builder_with_format_minimal() {
    let style = StyleBuilder::new()
        .name("MinimalStyle")
        .build_with_format(&["Name", "Fontname", "Fontsize"])
        .unwrap();

    assert_eq!(style, "Style: MinimalStyle,Arial,20");
}

#[test]
fn style_builder_with_script_version() {
    // Test building style with SSA v4 format
    let style_ssa = StyleBuilder::new()
        .name("TestSSA")
        .font("Arial")
        .size(18)
        .build_with_version(ScriptVersion::SsaV4)
        .unwrap();
    // SSA v4 has TertiaryColour instead of OutlineColour
    assert!(style_ssa.contains("TestSSA"));
    assert!(style_ssa.contains("Arial"));

    // Test building style with ASS v4+ format
    let style_ass = StyleBuilder::new()
        .name("TestASS")
        .font("Verdana")
        .size(20)
        .build_with_version(ScriptVersion::AssV4Plus)
        .unwrap();
    assert!(style_ass.contains("TestASS"));
    assert!(style_ass.contains("Verdana"));
}
