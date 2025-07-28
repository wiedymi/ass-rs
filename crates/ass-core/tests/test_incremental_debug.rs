//! Debug tests for incremental parsing

#![cfg(feature = "stream")]

use ass_core::parser::incremental::TextChange;
use ass_core::parser::Script;

#[test]
fn test_simple_incremental_parse() {
    let original = "[Script Info]\nTitle: Original\n";
    let script = Script::parse(original).unwrap();

    println!("Original script sections: {}", script.sections().len());

    // Change the title
    let change = TextChange {
        range: 21..29, // "Original"
        new_text: "Modified".to_string(),
        line_range: 2..2,
    };

    let new_content = "[Script Info]\nTitle: Modified\n";

    println!("Affected sections: {:?}", script.affected_sections(&change));

    let new_script = script.parse_incremental(new_content, &change).unwrap();

    println!("New script sections: {}", new_script.sections().len());

    if let ass_core::parser::ast::Section::ScriptInfo(info) = &new_script.sections()[0] {
        println!("ScriptInfo has {} fields", info.fields.len());
        for (k, v) in &info.fields {
            println!("  {k}: {v}");
        }
    }
}

#[test]
fn test_span_calculation() {
    let content = "[Script Info]\nTitle: Test\n\n[Events]\nFormat: Start, End, Text\nDialogue: 0:00:00.00,0:00:05.00,Test";
    let script = Script::parse(content).unwrap();

    for (i, section) in script.sections().iter().enumerate() {
        if let Some(span) = section.span() {
            println!(
                "Section {}: {:?} span: {}..{}",
                i,
                section.section_type(),
                span.start,
                span.end
            );
            println!("  Text: {:?}", &content[span.start..span.end]);
        }
    }
}

#[test]
fn test_affected_sections_detection() {
    let content = "[Script Info]\nTitle: Test\n\n[Events]\nFormat: Start, End, Text\nDialogue: 0:00:00.00,0:00:05.00,Test";
    let script = Script::parse(content).unwrap();

    // Change in ScriptInfo
    let change1 = TextChange {
        range: 14..18, // "Test" in title
        new_text: "Modified".to_string(),
        line_range: 2..2,
    };

    let affected1 = script.affected_sections(&change1);
    println!("Change in ScriptInfo affects: {affected1:?}");
    assert!(affected1.contains(&ass_core::parser::ast::SectionType::ScriptInfo));

    // Change in Events
    let change2 = TextChange {
        range: 92..96, // "Test" in dialogue - correct position
        new_text: "Modified".to_string(),
        line_range: 6..6,
    };

    let affected2 = script.affected_sections(&change2);
    println!("Change in Events affects: {affected2:?}");
    assert!(affected2.contains(&ass_core::parser::ast::SectionType::Events));
}
