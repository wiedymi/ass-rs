//! Behavioural tests for the [`Font`] embedded media AST node.

use super::super::Span;
use super::*;
#[cfg(not(feature = "std"))]
use alloc::{format, vec};

#[test]
fn font_creation() {
    let font = Font {
        filename: "test.ttf",
        data_lines: vec!["line1", "line2"],
        span: Span::new(0, 0, 0, 0),
    };

    assert_eq!(font.filename, "test.ttf");
    assert_eq!(font.data_lines.len(), 2);
    assert_eq!(font.data_lines[0], "line1");
    assert_eq!(font.data_lines[1], "line2");
}

#[test]
fn font_clone_eq() {
    let font = Font {
        filename: "test.ttf",
        data_lines: vec!["data"],
        span: Span::new(0, 0, 0, 0),
    };

    let cloned = font.clone();
    assert_eq!(font, cloned);
}

#[test]
fn font_debug() {
    let font = Font {
        filename: "debug.ttf",
        data_lines: vec!["test"],
        span: Span::new(0, 0, 0, 0),
    };

    let debug_str = format!("{font:?}");
    assert!(debug_str.contains("Font"));
    assert!(debug_str.contains("debug.ttf"));
}

#[test]
fn media_inequality() {
    let font1 = Font {
        filename: "font1.ttf",
        data_lines: vec!["data"],
        span: Span::new(0, 0, 0, 0),
    };

    let font2 = Font {
        filename: "font2.ttf",
        data_lines: vec!["data"],
        span: Span::new(0, 0, 0, 0),
    };

    assert_ne!(font1, font2);
}

#[test]
fn font_decode_data_valid() {
    // Valid UU-encoded data for "Cat" (test with known encoding)
    let font = Font {
        filename: "test.ttf",
        data_lines: vec!["#0V%T", "`"],
        span: Span::new(0, 0, 0, 0),
    };
    let decoded = font.decode_data().unwrap();
    assert_eq!(decoded, b"Cat");
}

#[test]
fn font_decode_data_empty_lines() {
    let font = Font {
        filename: "test.ttf",
        data_lines: vec![],
        span: Span::new(0, 0, 0, 0),
    };
    let decoded = font.decode_data().unwrap();
    assert!(decoded.is_empty());
}

#[test]
fn font_decode_data_whitespace_lines() {
    let font = Font {
        filename: "test.ttf",
        data_lines: vec!["   ", "\t\n", ""],
        span: Span::new(0, 0, 0, 0),
    };
    let decoded = font.decode_data().unwrap();
    assert!(decoded.is_empty());
}

#[test]
fn font_decode_data_with_end_marker() {
    let font = Font {
        filename: "test.ttf",
        data_lines: vec!["#0V%T", "end"],
        span: Span::new(0, 0, 0, 0),
    };
    let decoded = font.decode_data().unwrap();
    assert_eq!(decoded, b"Cat");
}

#[test]
fn font_decode_data_zero_length_line() {
    let font = Font {
        filename: "test.ttf",
        data_lines: vec!["#0V%T", " "],
        span: Span::new(0, 0, 0, 0),
    };
    let decoded = font.decode_data().unwrap();
    assert_eq!(decoded, b"Cat");
}

#[test]
fn font_decode_data_multiline() {
    // Multi-line UU-encoded data
    let font = Font {
        filename: "test.ttf",
        data_lines: vec!["$4F3\"", "$4F3\""],
        span: Span::new(0, 0, 0, 0),
    };
    let decoded = font.decode_data().unwrap();
    // Should decode both lines and concatenate results
    assert_eq!(decoded.len(), 6); // 3 bytes per line
}

#[test]
fn font_decode_data_handles_malformed_gracefully() {
    // UU decoding should not panic on malformed data but may return unexpected results
    let font = Font {
        filename: "test.ttf",
        data_lines: vec!["invalid-characters-here"],
        span: Span::new(0, 0, 0, 0),
    };
    // Should not panic, result depends on UU decoder implementation
    let _result = font.decode_data();
}

#[test]
fn font_decode_data_length_validation() {
    // Test that length encoding in first character is respected
    let font = Font {
        filename: "test.ttf",
        data_lines: vec!["!    "], // '!' encodes length 1, but provides more data
        span: Span::new(0, 0, 0, 0),
    };
    let decoded = font.decode_data().unwrap();
    assert_eq!(decoded.len(), 1); // Should be truncated to declared length
}

#[cfg(debug_assertions)]
#[test]
fn font_validate_spans() {
    let source = "fontname: test.ttf\ndata1\ndata2";
    let font = Font {
        filename: &source[10..18],                          // "test.ttf"
        data_lines: vec![&source[19..24], &source[25..30]], // "data1", "data2"
        span: Span::new(0, 0, 0, 0),
    };

    let source_range = (source.as_ptr() as usize)..(source.as_ptr() as usize + source.len());
    assert!(font.validate_spans(&source_range));
}

#[cfg(debug_assertions)]
#[test]
fn font_validate_spans_invalid() {
    let source1 = "fontname: test.ttf";
    let source2 = "different source";

    let font = Font {
        filename: &source1[10..18],       // "test.ttf" from source1
        data_lines: vec![&source2[0..9]], // "different" from source2
        span: Span::new(0, 0, 0, 0),
    };

    let source1_range = (source1.as_ptr() as usize)..(source1.as_ptr() as usize + source1.len());
    assert!(!font.validate_spans(&source1_range)); // Should fail because data_lines reference different source
}
