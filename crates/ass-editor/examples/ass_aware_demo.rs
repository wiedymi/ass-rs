//! Demonstration of ASS-aware editor APIs
//!
//! Shows the transformation from manual parsing to direct ASS access

use ass_editor::{EditorDocument, EventBuilder, Position, StyleBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎬 ASS-Editor Refactor Demo: From Manual Parsing to Direct Access\n");

    // Create sample ASS content
    let ass_content = r#"[Script Info]
Title: Demo Movie
ScriptType: v4.00+

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&Hffffff,&Hff0000,&H0,&H0,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: Title,Arial,24,&Hffff00,&Hff0000,&H0,&H0,-1,0,0,0,100,100,0,0,1,3,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:05.00,0:00:10.00,Default,John,0,0,0,,Hello, world!
Dialogue: 0,0:00:12.00,0:00:15.00,Default,Jane,0,0,0,,How are you?
Comment: 0,0:00:20.00,0:00:25.00,Default,,0,0,0,,This is a comment
"#;

    // === OLD APPROACH (Manual Parsing) ===
    println!("❌ OLD APPROACH: Manual parsing required");
    println!("let document = EditorDocument::from_content(content)?;");
    println!("let script = Script::parse(document.text())?; // Manual parsing!");
    println!("let events = script.find_section(\"Events\")?;  // Manual section search!");
    println!("for event in events {{ ... }}                // Manual iteration!");

    // === NEW APPROACH (Direct Access) ===
    println!("\n✅ NEW APPROACH: Direct ASS access");
    let mut document = EditorDocument::from_content(ass_content)?;

    // Direct access to ASS structures - no manual parsing!
    println!("let mut doc = EditorDocument::from_content(content)?;");

    let events_count = document.events()?;
    println!("let events_count = doc.events()?;             // {events_count} events found!");

    let styles_count = document.styles_count()?;
    println!("let styles_count = doc.styles_count()?;       // {styles_count} styles found!");

    let has_events = document.has_events()?;
    println!("let has_events = doc.has_events()?;           // Has events: {has_events}");

    let script_fields = document.script_info_fields()?;
    println!("let script_fields = doc.script_info_fields()?; // Fields: {script_fields:?}");

    // ASS-aware search
    let hello_events = document.find_event_text("Hello")?;
    println!("let hello_events = doc.find_event_text(\"Hello\")?; // Found: {hello_events:?}");

    // === FLUENT API DEMO ===
    println!("\n🔗 FLUENT API: Position-based operations");

    // Find where to insert new content
    let content = document.text();
    let insert_pos = content.find("[Events]").unwrap()
        + "[Events]\n".len()
        + content[content.find("[Events]").unwrap()..]
            .find('\n')
            .unwrap()
        + 1;

    println!("doc.at(position).insert_text(\"new content\")?; // Fluent API");
    document
        .at(Position::new(insert_pos))
        .insert_text("Comment: 0,0:00:01.00,0:00:02.00,Default,,0,0,0,,Added via fluent API\n")?;

    // === BUILDER PATTERN DEMO ===
    println!("\n🏗️  BUILDER PATTERNS: Ergonomic ASS creation");

    // Create new event using builder
    let new_event = EventBuilder::dialogue()
        .start_time("0:00:30.00")
        .end_time("0:00:35.00")
        .speaker("Builder")
        .text("Created with EventBuilder!")
        .layer(1)
        .style("Default")
        .build()?;

    println!("let event = EventBuilder::dialogue()");
    println!("    .start_time(\"0:00:30.00\")");
    println!("    .speaker(\"Builder\")");
    println!("    .text(\"Created with EventBuilder!\")");
    println!("    .build()?;");
    println!("// Result: {new_event}");

    // Create new style using builder
    let new_style = StyleBuilder::default_style()
        .name("CustomStyle")
        .font("Comic Sans MS")
        .size(28)
        .color("&Hff00ff")
        .bold(true)
        .italic(true)
        .align(8)
        .build()?;

    println!("\nlet style = StyleBuilder::default_style()");
    println!("    .name(\"CustomStyle\")");
    println!("    .font(\"Comic Sans MS\")");
    println!("    .size(28)");
    println!("    .bold(true)");
    println!("    .build()?;");
    println!("// Result: {new_style}");

    // === ASS-AWARE EDITING DEMO ===
    println!("\n✏️  ASS-AWARE EDITING: Direct manipulation");

    // Edit event text directly
    document.edit_event_text("Hello, world!", "Hello, ASS-RS!")?;
    println!("doc.edit_event_text(\"Hello, world!\", \"Hello, ASS-RS!\")?;");

    // Set script info field
    document.set_script_info_field("Author", "ASS-RS Team")?;
    println!("doc.set_script_info_field(\"Author\", \"ASS-RS Team\")?;");

    // Get updated info
    let author = document.get_script_info_field("Author")?;
    println!("let author = doc.get_script_info_field(\"Author\")?; // {author:?}");

    // === SUMMARY ===
    println!("\n📊 TRANSFORMATION SUMMARY:");
    println!("✅ Direct ASS access - no manual Script::parse()");
    println!("✅ Fluent position API - doc.at(pos).insert_text()");
    println!("✅ Builder patterns - EventBuilder, StyleBuilder");
    println!("✅ ASS-aware editing - edit_event_text(), set_script_info_field()");
    println!("✅ Type safety - ass-core types re-exported as first-class");

    println!("\n🎉 ASS-Editor is now a proper ASS manipulation library!");

    Ok(())
}
