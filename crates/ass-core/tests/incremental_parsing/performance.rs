//! Performance test for incremental parsing of large scripts.

use ass_core::parser::ast::Section;
use ass_core::parser::incremental::TextChange;
use ass_core::parser::Script;

#[test]
fn test_incremental_parsing_performance() {
    use std::fmt::Write;

    // Create a large script
    let mut content =
        String::from("[Script Info]\nTitle: Large Script\n\n[Events]\nFormat: Start, End, Text\n");
    for i in 0..1000 {
        writeln!(
            content,
            "Dialogue: 0:00:{:02}.00,0:00:{:02}.00,Line {}",
            i % 60,
            (i + 5) % 60,
            i
        )
        .unwrap();
    }

    let script = Script::parse(&content).unwrap();

    // Change just one line in the middle
    let target = "Line 500";
    let pos = content.find(target).unwrap();
    let change = TextChange {
        range: pos..pos + target.len(),
        new_text: "Modified 500".to_string(),
        line_range: 505..505,
    };

    let new_content = content.replace(target, "Modified 500");

    // This should be fast since we only reparse the Events section
    let start = std::time::Instant::now();
    let new_script = script.parse_incremental(&new_content, &change).unwrap();
    let elapsed = start.elapsed();

    // Should complete in under 8ms as per performance targets (debug builds are slower)
    assert!(
        elapsed.as_millis() < 8,
        "Incremental parse took {elapsed:?}"
    );

    // Debug: print sections
    println!("Original sections: {}", script.sections().len());
    println!("New sections: {}", new_script.sections().len());
    for (i, section) in new_script.sections().iter().enumerate() {
        match section {
            Section::ScriptInfo(_) => println!("Section {i}: ScriptInfo"),
            Section::Styles(s) => println!("Section {i}: Styles ({})", s.len()),
            Section::Events(e) => println!("Section {i}: Events ({})", e.len()),
            Section::Fonts(f) => println!("Section {i}: Fonts ({})", f.len()),
            Section::Graphics(g) => println!("Section {i}: Graphics ({})", g.len()),
        }
    }

    // Verify the change
    let events_section = new_script
        .sections()
        .iter()
        .find_map(|s| match s {
            Section::Events(events) => Some(events),
            _ => None,
        })
        .expect("Events section should exist");

    // Debug: print first few events to see what's happening
    println!("Total events: {}", events_section.len());
    for (i, event) in events_section.iter().take(5).enumerate() {
        println!("Event {i}: {}", event.text);
    }

    // Look for the modified event - might be "Modified 500" not in text field
    let modified_event = events_section.iter().enumerate().find(|(i, e)| {
        if *i == 500 {
            println!("Event 500 text: '{}'", e.text);
        }
        e.text == "Modified 500"
    });

    assert!(
        modified_event.is_some(),
        "Modified event not found. Expected text 'Modified 500' at index around 500"
    );
}
