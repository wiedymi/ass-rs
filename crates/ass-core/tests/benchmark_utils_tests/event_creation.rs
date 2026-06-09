//! Benchmark test-event construction helper and associated tests.
//!
//! Exercises the synthetic `Event` builder mirroring the benchmark
//! function, covering formatting, Unicode, defaults, and edge cases.

// Helper function for creating test events (mirrors benchmark function)
const fn create_benchmark_test_event(
    start: &'static str,
    end: &'static str,
    text: &'static str,
) -> ass_core::parser::ast::Event<'static> {
    use ass_core::parser::ast::{Event, EventType, Span};
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

#[test]
fn test_create_test_event_basic() {
    use ass_core::parser::ast::EventType;

    let simple_event = create_benchmark_test_event("0:00:00.00", "0:00:05.00", "Simple text");
    assert_eq!(simple_event.event_type, EventType::Dialogue);
    assert_eq!(simple_event.start, "0:00:00.00");
    assert_eq!(simple_event.end, "0:00:05.00");
    assert_eq!(simple_event.text, "Simple text");
    assert_eq!(simple_event.style, "Default");
    assert_eq!(simple_event.layer, "0");
    assert_eq!(simple_event.name, "");
    assert_eq!(simple_event.margin_l, "0");
    assert_eq!(simple_event.margin_r, "0");
    assert_eq!(simple_event.margin_v, "0");
    assert_eq!(simple_event.effect, "");
}

#[test]
fn test_create_test_event_formatting() {
    let complex_event = create_benchmark_test_event(
        "0:00:05.00",
        "0:00:10.00",
        r"Text with {\b1}formatting{\b0} and {\c&H0000FF&}colors{\c}",
    );
    assert_eq!(complex_event.start, "0:00:05.00");
    assert_eq!(complex_event.end, "0:00:10.00");
    assert!(complex_event.text.contains(r"{\b1}"));
    assert!(complex_event.text.contains(r"{\c&H0000FF&}"));
}

#[test]
fn test_create_test_event_unicode() {
    let unicode_event = create_benchmark_test_event(
        "0:00:10.00",
        "0:00:15.00",
        "Hello, 世界! 🌍 Здравствуй мир! 🚀",
    );
    assert_eq!(unicode_event.text, "Hello, 世界! 🌍 Здравствуй мир! 🚀");
}

#[test]
fn test_create_test_event_defaults() {
    use ass_core::parser::ast::EventType;

    let events = vec![
        create_benchmark_test_event("0:00:00.00", "0:00:05.00", "Event 1"),
        create_benchmark_test_event("0:00:05.00", "0:00:10.00", "Event 2"),
        create_benchmark_test_event("0:00:10.00", "0:00:15.00", "Event 3"),
    ];

    for event in &events {
        assert_eq!(event.event_type, EventType::Dialogue);
        assert_eq!(event.layer, "0");
        assert_eq!(event.style, "Default");
        assert_eq!(event.name, "");
        assert_eq!(event.margin_l, "0");
        assert_eq!(event.margin_r, "0");
        assert_eq!(event.margin_v, "0");
        assert_eq!(event.effect, "");
    }

    // Verify unique text and timing
    assert_eq!(events[0].text, "Event 1");
    assert_eq!(events[1].text, "Event 2");
    assert_eq!(events[2].text, "Event 3");
}

#[test]
fn test_benchmark_event_creation_edge_cases() {
    // Test with karaoke timing
    let karaoke_event = create_benchmark_test_event(
        "0:00:00.00",
        "0:00:10.00",
        r"{\k50}Ka{\k30}ra{\k40}o{\k35}ke {\k45}text",
    );
    assert!(karaoke_event.text.contains(r"{\k"));

    // Test with animation tags
    let animation_event = create_benchmark_test_event(
        "0:00:00.00",
        "0:00:05.00",
        r"{\move(0,0,100,100)\t(0,1000,\fscx120)}Animated text",
    );
    assert!(animation_event.text.contains(r"{\move"));
    assert!(animation_event.text.contains(r"\t("));

    // Test with empty text
    let empty_event = create_benchmark_test_event("0:00:00.00", "0:00:01.00", "");
    assert_eq!(empty_event.text, "");

    // Test with special characters
    let special_event =
        create_benchmark_test_event("0:00:00.00", "0:00:05.00", "Special: \n\r\t{}[]():,;");
    assert!(special_event.text.contains("{}"));
    assert!(special_event.text.contains("[]"));
}
