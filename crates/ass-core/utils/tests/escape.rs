//! Field-name normalization and escape/unescape tests for the utils module.

use crate::utils::{escape_text, normalize_field_name, parse_escape_sequence, unescape_text};

#[test]
fn field_name_normalization() {
    assert_eq!(normalize_field_name("Title"), "title");
    assert_eq!(normalize_field_name("ScriptType"), "scripttype");
    assert_eq!(normalize_field_name("WrapStyle"), "wrapstyle");
    assert_eq!(normalize_field_name("UPPERCASE"), "uppercase");
    assert_eq!(normalize_field_name("MixedCase"), "mixedcase");
}

#[test]
fn field_name_normalization_whitespace() {
    assert_eq!(normalize_field_name(" Title "), "title");
    assert_eq!(normalize_field_name("\tScriptType\t"), "scripttype");
    assert_eq!(normalize_field_name("\nWrapStyle\n"), "wrapstyle");
}

#[test]
fn escape_sequence_parsing() {
    assert_eq!(parse_escape_sequence("\\n"), Some('\n'));
    assert_eq!(parse_escape_sequence("\\r"), Some('\r'));
    assert_eq!(parse_escape_sequence("\\t"), Some('\t'));
    assert_eq!(parse_escape_sequence("`[Events]`"), Some('\\'));
    assert_eq!(parse_escape_sequence("\\{"), Some('{'));
    assert_eq!(parse_escape_sequence("\\}"), Some('}'));
}

#[test]
fn escape_sequence_parsing_invalid() {
    assert_eq!(parse_escape_sequence("\\x"), None);
    assert_eq!(parse_escape_sequence("\\z"), None);
    assert_eq!(parse_escape_sequence("n"), None);
    assert_eq!(parse_escape_sequence(""), None);
}

#[test]
fn unescape_text_basic() {
    assert_eq!(unescape_text("Hello\\nWorld"), "Hello\nWorld");
    assert_eq!(unescape_text("Tab\\tSeparated"), "Tab\tSeparated");
    assert_eq!(unescape_text("Quote\\\"Test"), "Quote\"Test");
    assert_eq!(unescape_text("Brace\\{Test\\}"), "Brace{Test}");
}

#[test]
fn unescape_text_multiple() {
    assert_eq!(
        unescape_text("Line1\\nLine2\\nLine3"),
        "Line1\nLine2\nLine3"
    );
    assert_eq!(unescape_text("\\t\\r\\n"), "\t\r\n");
}

#[test]
fn unescape_text_no_escapes() {
    assert_eq!(unescape_text("Plain text"), "Plain text");
    assert_eq!(unescape_text("No escapes here"), "No escapes here");
    assert_eq!(unescape_text(""), "");
}

#[test]
fn escape_text_basic() {
    assert_eq!(escape_text("Hello\nWorld"), "Hello\\nWorld");
    assert_eq!(escape_text("Tab\tSeparated"), "Tab\\tSeparated");
    assert_eq!(escape_text("Quote\"Test"), "Quote\\\"Test");
    assert_eq!(escape_text("Brace{Test}"), "Brace\\{Test\\}");
}

#[test]
fn escape_unescape_round_trip() {
    let original = "Hello\nWorld\tWith\"Quotes{And}Braces";
    let escaped = escape_text(original);
    let unescaped = unescape_text(&escaped);
    assert_eq!(original, unescaped);
}
