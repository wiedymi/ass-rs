//! Behavioural tests for the [`Graphic`] embedded media AST node.

use super::super::Span;
use super::*;
use alloc::vec::Vec;
#[cfg(not(feature = "std"))]
use alloc::{format, vec};

#[test]
fn graphic_creation() {
    let graphic = Graphic {
        filename: "logo.png",
        data_lines: vec!["data1", "data2", "data3"],
        span: Span::new(0, 0, 0, 0),
    };

    assert_eq!(graphic.filename, "logo.png");
    assert_eq!(graphic.data_lines.len(), 3);
    assert_eq!(graphic.data_lines[0], "data1");
}

#[test]
fn graphic_clone_eq() {
    let graphic = Graphic {
        filename: "test.png",
        data_lines: vec!["data"],
        span: Span::new(0, 0, 0, 0),
    };

    let cloned = graphic.clone();
    assert_eq!(graphic, cloned);
}

#[test]
fn graphic_debug() {
    let graphic = Graphic {
        filename: "debug.png",
        data_lines: vec!["test"],
        span: Span::new(0, 0, 0, 0),
    };

    let debug_str = format!("{graphic:?}");
    assert!(debug_str.contains("Graphic"));
    assert!(debug_str.contains("debug.png"));
}

#[test]
fn empty_data_lines() {
    let font = Font {
        filename: "empty.ttf",
        data_lines: Vec::new(),
        span: Span::new(0, 0, 0, 0),
    };

    let graphic = Graphic {
        filename: "empty.png",
        data_lines: Vec::new(),
        span: Span::new(0, 0, 0, 0),
    };

    assert!(font.data_lines.is_empty());
    assert!(graphic.data_lines.is_empty());
}

#[test]
fn graphic_decode_data_valid() {
    // Valid UU-encoded data for "PNG" (test with known encoding)
    let graphic = Graphic {
        filename: "test.png",
        data_lines: vec!["#4$Y'"],
        span: Span::new(0, 0, 0, 0),
    };
    let decoded = graphic.decode_data().unwrap();
    assert_eq!(decoded, b"PNG");
}

#[test]
fn graphic_decode_data_empty_lines() {
    let graphic = Graphic {
        filename: "test.png",
        data_lines: vec![],
        span: Span::new(0, 0, 0, 0),
    };
    let decoded = graphic.decode_data().unwrap();
    assert!(decoded.is_empty());
}

#[test]
fn graphic_decode_data_with_end_marker() {
    let graphic = Graphic {
        filename: "test.png",
        data_lines: vec!["#4$Y'", "end"],
        span: Span::new(0, 0, 0, 0),
    };
    let decoded = graphic.decode_data().unwrap();
    assert_eq!(decoded, b"PNG");
}

#[test]
fn graphic_decode_data_whitespace_handling() {
    let graphic = Graphic {
        filename: "test.png",
        data_lines: vec!["#4$Y'  ", "\t\n", ""],
        span: Span::new(0, 0, 0, 0),
    };
    let decoded = graphic.decode_data().unwrap();
    assert_eq!(decoded, b"PNG");
}

#[test]
fn graphic_decode_data_handles_malformed_gracefully() {
    // UU decoding should not panic on malformed data but may return unexpected results
    let graphic = Graphic {
        filename: "test.png",
        data_lines: vec!["!@#$%^&*()"],
        span: Span::new(0, 0, 0, 0),
    };
    // Should not panic, result depends on UU decoder implementation
    let _result = graphic.decode_data();
}

#[test]
fn graphic_decode_data_length_validation() {
    // Test that length encoding in first character is respected
    let graphic = Graphic {
        filename: "test.png",
        data_lines: vec!["\"````"], // '"' encodes length 2, provides padding
        span: Span::new(0, 0, 0, 0),
    };
    let decoded = graphic.decode_data().unwrap();
    assert_eq!(decoded.len(), 2); // Should be truncated to declared length
}

#[cfg(debug_assertions)]
#[test]
fn graphic_validate_spans() {
    let source = "filename: logo.png\nimage1\nimage2";
    let graphic = Graphic {
        filename: &source[10..18],                          // "logo.png"
        data_lines: vec![&source[19..25], &source[26..32]], // "image1", "image2"
        span: Span::new(0, 0, 0, 0),
    };

    let source_range = (source.as_ptr() as usize)..(source.as_ptr() as usize + source.len());
    assert!(graphic.validate_spans(&source_range));
}
