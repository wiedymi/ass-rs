//! Tests for `calculate_delta` section diffing.

use ass_core::parser::ast::Section;
use ass_core::parser::{calculate_delta, Script};

#[test]
fn test_calculate_delta_section_added() {
    let original = r"[Script Info]
Title: Test

[Events]
Format: Start, End, Text
Dialogue: 0:00:00.00,0:00:05.00,Test";

    let modified = r"[Script Info]
Title: Test

[V4+ Styles]
Format: Name, Fontname
Style: Default,Arial

[Events]
Format: Start, End, Text
Dialogue: 0:00:00.00,0:00:05.00,Test";

    let script1 = Script::parse(original).unwrap();
    let script2 = Script::parse(modified).unwrap();

    let delta = calculate_delta(&script1, &script2);

    println!("Script1 sections: {}", script1.sections().len());
    for (i, s) in script1.sections().iter().enumerate() {
        println!("  {}: {:?}", i, s.section_type());
    }
    println!("Script2 sections: {}", script2.sections().len());
    for (i, s) in script2.sections().iter().enumerate() {
        println!("  {}: {:?}", i, s.section_type());
    }
    println!(
        "Delta: added={}, modified={}, removed={}",
        delta.added.len(),
        delta.modified.len(),
        delta.removed.len()
    );

    assert_eq!(delta.added.len(), 1);
    assert!(matches!(delta.added[0], Section::Styles(_)));
    assert_eq!(delta.modified.len(), 0);
    assert_eq!(delta.removed.len(), 0);
}

#[test]
fn test_calculate_delta_section_modified() {
    let original = r"[Script Info]
Title: Original

[Events]
Format: Start, End, Text
Dialogue: 0:00:00.00,0:00:05.00,Original";

    let modified = r"[Script Info]
Title: Modified

[Events]
Format: Start, End, Text
Dialogue: 0:00:00.00,0:00:05.00,Modified";

    let script1 = Script::parse(original).unwrap();
    let script2 = Script::parse(modified).unwrap();

    let delta = calculate_delta(&script1, &script2);

    assert_eq!(delta.added.len(), 0);
    assert_eq!(delta.modified.len(), 2); // Both ScriptInfo and Events changed
    assert_eq!(delta.removed.len(), 0);
}

#[test]
fn test_calculate_delta_section_removed() {
    let original = r"[Script Info]
Title: Test

[V4+ Styles]
Format: Name, Fontname
Style: Default,Arial

[Events]
Format: Start, End, Text
Dialogue: 0:00:00.00,0:00:05.00,Test";

    let modified = r"[Script Info]
Title: Test

[Events]
Format: Start, End, Text
Dialogue: 0:00:00.00,0:00:05.00,Test";

    let script1 = Script::parse(original).unwrap();
    let script2 = Script::parse(modified).unwrap();

    let delta = calculate_delta(&script1, &script2);

    assert_eq!(delta.added.len(), 0);
    assert_eq!(delta.modified.len(), 0);
    assert_eq!(delta.removed.len(), 1);
    assert_eq!(delta.removed[0], 1); // Index of Styles section
}
