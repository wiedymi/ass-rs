//! Tests for [`EventBuilder`].

use super::*;
#[cfg(not(feature = "std"))]
use alloc::string::ToString;
use ass_core::ScriptVersion;

#[test]
fn event_builder_dialogue() {
    let event = EventBuilder::dialogue()
        .start_time("0:00:05.00")
        .end_time("0:00:10.00")
        .speaker("John")
        .text("Hello world!")
        .build()
        .unwrap();

    assert!(event.contains("Dialogue:"));
    assert!(event.contains("0:00:05.00"));
    assert!(event.contains("Hello world!"));
}

#[test]
fn event_builder_comment() {
    let event = EventBuilder::comment()
        .text("This is a comment")
        .build()
        .unwrap();

    assert!(event.contains("Comment:"));
    assert!(event.contains("This is a comment"));
}

#[test]
fn event_builder_with_margins() {
    let event = EventBuilder::dialogue()
        .start_time("0:00:05.00")
        .end_time("0:00:10.00")
        .margin_left(15)
        .margin_right(20)
        .margin_vertical(25)
        .margin_top(30)
        .margin_bottom(35)
        .text("Testing margins")
        .build()
        .unwrap();

    assert!(event.contains("Dialogue:"));
    assert!(event.contains("15")); // margin_l
    assert!(event.contains("20")); // margin_r
    assert!(event.contains("25")); // margin_v
                                   // Note: margin_t and margin_b are stored but not in V4+ format output yet
}

#[test]
fn event_builder_with_format_v4plus() {
    let event = EventBuilder::dialogue()
        .start_time("0:00:05.00")
        .end_time("0:00:10.00")
        .style("Main")
        .layer(1)
        .text("Test with format")
        .build_with_format(&[
            "Layer", "Start", "End", "Style", "Name", "MarginL", "MarginR", "MarginV", "Effect",
            "Text",
        ])
        .unwrap();

    assert_eq!(
        event,
        "Dialogue: 1,0:00:05.00,0:00:10.00,Main,,0,0,0,,Test with format"
    );
}

#[test]
fn event_builder_with_format_v4plusplus() {
    let event = EventBuilder::dialogue()
        .start_time("0:00:05.00")
        .end_time("0:00:10.00")
        .style("Main")
        .margin_top(5)
        .margin_bottom(10)
        .text("V4++ format")
        .build_with_format(&[
            "Layer", "Start", "End", "Style", "Name", "MarginL", "MarginR", "MarginT", "MarginB",
            "Effect", "Text",
        ])
        .unwrap();

    assert_eq!(
        event,
        "Dialogue: 0,0:00:05.00,0:00:10.00,Main,,0,0,5,10,,V4++ format"
    );
}

#[test]
fn event_builder_with_format_custom() {
    let event = EventBuilder::comment()
        .text("Simple comment")
        .build_with_format(&["Start", "End", "Text"])
        .unwrap();

    assert_eq!(event, "Comment: 0:00:00.00,0:00:05.00,Simple comment");
}

#[test]
fn event_builder_with_format_error() {
    let result = EventBuilder::dialogue()
        .text("Test")
        .build_with_format(&["InvalidField"]);

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Unknown event field"));
}

#[test]
fn event_builder_with_script_version() {
    // Test building with SSA v4 format
    let event_ssa = EventBuilder::dialogue()
        .text("SSA Format")
        .start_time("0:00:01.00")
        .end_time("0:00:03.00")
        .build_with_version(ScriptVersion::SsaV4)
        .unwrap();
    assert!(event_ssa.contains("SSA Format"));

    // Test building with ASS v4+ format
    let event_ass = EventBuilder::dialogue()
        .text("ASS Format")
        .build_with_version(ScriptVersion::AssV4Plus)
        .unwrap();
    assert!(event_ass.contains("ASS Format"));
}
