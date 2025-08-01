//! Demo test file showcasing the new fluent event filtering and sorting API

use ass_editor::{EditorDocument, EventType, EventFilter, EventSortCriteria, EventSortOptions};

const SAMPLE_ASS_CONTENT: &str = r#"[Script Info]
Title: Sample Subtitle
ScriptType: v4.00+

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: Title,Arial,24,&H00FF0000,&H000000FF,&H00000000,&H00000000,1,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Title,Narrator,0,0,0,,Welcome to the show!
Dialogue: 0,0:00:05.50,0:00:08.00,Default,Alice,0,0,0,,Hello everyone
Comment: 0,0:00:08.00,0:00:08.50,Default,,0,0,0,,Note: Alice enters
Dialogue: 0,0:00:08.50,0:00:12.00,Default,Bob,0,0,0,,Hi Alice, how are you?
Dialogue: 0,0:00:12.50,0:00:15.00,Default,Alice,0,0,0,,I'm doing great, thanks!
Comment: 0,0:00:15.00,0:00:15.50,Default,,0,0,0,,Note: Scene transition
Dialogue: 0,0:00:16.00,0:00:20.00,Title,Narrator,0,0,0,,End of scene one
Dialogue: 0,0:00:20.50,0:00:25.00,Default,Charlie,0,0,0,,Now I enter the conversation
"#;

#[test]
fn demo_basic_event_access() {
    let mut doc = EditorDocument::from_content(SAMPLE_ASS_CONTENT).unwrap();
    
    println!("\n=== Basic Event Access Demo ===");
    
    // Get total event count
    let count = doc.events().count().unwrap();
    println!("Total events in document: {count}");
    assert_eq!(count, 8);
    
    // Access specific event by index
    let first_event = doc.events().get(0).unwrap().unwrap();
    println!("First event (index 0): '{}'", first_event.event.text);
    println!("  Type: {:?}", first_event.event.event_type);
    println!("  Style: {}", first_event.event.style);
    println!("  Speaker: {}", first_event.event.name);
    
    // Use fluent accessor
    let third_event_text = doc.events().event(2).text().unwrap().unwrap();
    println!("Third event text: '{third_event_text}'");
    
    // Check if an event exists
    let event_10_exists = doc.events().event(10).exists().unwrap();
    println!("Event at index 10 exists: {event_10_exists}");
    assert!(!event_10_exists);
}

#[test]
fn demo_filtering_operations() {
    let mut doc = EditorDocument::from_content(SAMPLE_ASS_CONTENT).unwrap();
    
    println!("\n=== Event Filtering Demo ===");
    
    // Filter by event type - get all dialogues
    let dialogues = doc.events().dialogues().execute().unwrap();
    println!("Dialogue events found: {}", dialogues.len());
    for (i, dialogue) in dialogues.iter().enumerate() {
        println!("  {}: '{}' (by {})", i+1, dialogue.event.text, dialogue.event.name);
    }
    assert_eq!(dialogues.len(), 6);
    
    // Filter by event type - get all comments  
    let comments = doc.events().comments().execute().unwrap();
    println!("\nComment events found: {}", comments.len());
    for (i, comment) in comments.iter().enumerate() {
        println!("  {}: '{}'", i+1, comment.event.text);
    }
    assert_eq!(comments.len(), 2);
    
    // Filter by speaker
    let alice_lines = doc.events()
        .query()
        .filter_by_speaker("Alice")
        .execute()
        .unwrap();
    println!("\nAlice's lines: {}", alice_lines.len());
    for line in &alice_lines {
        println!("  '{}' at {}", line.event.text, line.event.start);
    }
    assert_eq!(alice_lines.len(), 2);
    
    // Filter by style
    let title_events = doc.events()
        .query()
        .filter_by_style("Title")
        .execute()
        .unwrap();
    println!("\nTitle style events: {}", title_events.len());
    for event in &title_events {
        println!("  '{}' ({})", event.event.text, event.event.name);
    }
    assert_eq!(title_events.len(), 2);
    
    // Filter by text content (case insensitive)
    let greeting_events = doc.events()
        .query()
        .filter_by_text("hello")
        .case_sensitive(false)
        .execute()
        .unwrap();
    println!("\nEvents containing 'hello' (case insensitive): {}", greeting_events.len());
    for event in &greeting_events {
        println!("  '{}' by {}", event.event.text, event.event.name);
    }
    assert_eq!(greeting_events.len(), 1);
}

#[test]
fn demo_sorting_operations() {
    let mut doc = EditorDocument::from_content(SAMPLE_ASS_CONTENT).unwrap();
    
    println!("\n=== Event Sorting Demo ===");
    
    // Sort by time (default)
    let by_time = doc.events().by_time().execute().unwrap();
    println!("Events sorted by start time:");
    for (i, event) in by_time.iter().enumerate() {
        println!("  {}: {} - '{}' ({})", 
                 i+1, 
                 event.event.start, 
                 event.event.text,
                 event.event.name);
    }
    
    // Sort by speaker name
    let by_speaker = doc.events()
        .query()
        .filter_by_type(EventType::Dialogue)
        .sort(EventSortCriteria::Speaker)
        .execute()
        .unwrap();
    println!("\nDialogue events sorted by speaker:");
    for event in &by_speaker {
        println!("  {}: '{}'", event.event.name, event.event.text);
    }
    
    // Sort by style, then by time
    let by_style_then_time = doc.events()
        .query()
        .sort_by(EventSortOptions {
            criteria: EventSortCriteria::Style,
            secondary: Some(EventSortCriteria::StartTime),
            ascending: true,
        })
        .execute()
        .unwrap();
    println!("\nEvents sorted by style, then by time:");
    for event in &by_style_then_time {
        println!("  {} ({}): '{}'", 
                 event.event.style,
                 event.event.start, 
                 event.event.text);
    }
}

#[test]
fn demo_complex_queries() {
    let mut doc = EditorDocument::from_content(SAMPLE_ASS_CONTENT).unwrap();
    
    println!("\n=== Complex Query Demo ===");
    
    // Complex filter: Dialogue events by specific speakers, sorted by time, limited to 3
    let character_dialogue = doc.events()
        .query()
        .filter_by_type(EventType::Dialogue)
        .filter_by_speaker("Alice")
        .sort_by_time()
        .limit(3)
        .execute()
        .unwrap();
    
    println!("Alice's dialogue (max 3, sorted by time):");
    for event in &character_dialogue {
        println!("  {}: '{}'", event.event.start, event.event.text);
    }
    
    // Get just the indices for further processing
    let title_indices = doc.events()
        .query()
        .filter_by_style("Title")
        .sort_by_time()
        .indices()
        .unwrap();
    
    println!("\nTitle event indices (sorted by time): {title_indices:?}");
    
    // Custom filter with multiple criteria
    let custom_filter = EventFilter {
        event_type: Some(EventType::Dialogue),
        style_pattern: Some("Default".to_string()),
        speaker_pattern: None,
        text_pattern: None,
        time_range: Some((500, 1500)), // 5-15 seconds in centiseconds
        layer: None,
        effect_pattern: None,
        use_regex: false,
        case_sensitive: false,
    };
    
    let filtered_events = doc.events()
        .query()
        .filter(custom_filter)
        .sort_by_time()
        .execute()
        .unwrap();
    
    println!("\nDefault style dialogue between 5-15 seconds:");
    for event in &filtered_events {
        println!("  {} by {}: '{}'", 
                 event.event.start,
                 event.event.name, 
                 event.event.text);
    }
    
    // Get first matching event
    let first_long_text = doc.events()
        .query()
        .filter_by_text("conversation")
        .first()
        .unwrap();
    
    if let Some(event) = first_long_text {
        println!("\nFirst event containing 'conversation': '{}'", event.event.text);
    }
    
    // Count matching events
    let narrator_count = doc.events()
        .query()
        .filter_by_speaker("Narrator")
        .count()
        .unwrap();
    
    println!("Narrator events count: {narrator_count}");
    assert_eq!(narrator_count, 2);
}

#[test]
fn demo_event_properties_access() {
    let mut doc = EditorDocument::from_content(SAMPLE_ASS_CONTENT).unwrap();
    
    println!("\n=== Event Properties Access Demo ===");
    
    // Access all properties of a specific event
    println!("Event 0 properties:");
    println!("  Text: {:?}", doc.events().event(0).text().unwrap());
    println!("  Style: {:?}", doc.events().event(0).style().unwrap());
    println!("  Speaker: {:?}", doc.events().event(0).speaker().unwrap());
    println!("  Timing: {:?}", doc.events().event(0).timing().unwrap());
    println!("  Layer: {:?}", doc.events().event(0).layer().unwrap());
    println!("  Effect: {:?}", doc.events().event(0).effect().unwrap());
    println!("  Event Type: {:?}", doc.events().event(0).event_type().unwrap());
    println!("  Margins: {:?}", doc.events().event(0).margins().unwrap());
    
    // Check individual properties
    let event_3_start = doc.events().event(3).start_time().unwrap();
    let event_3_end = doc.events().event(3).end_time().unwrap();
    println!("\nEvent 3 timing:");
    println!("  Start: {event_3_start:?}");
    println!("  End: {event_3_end:?}");
}

/// Demonstrates the main features requested in the original proposal:
/// - Direct event access: `doc.events().event(2)` for getting specific events
/// - Fluent querying: `doc.events().filter(options).sort(options).execute()`
/// - Individual property access: `doc.events().event(2).text()`
/// - Filtering and sorting with multiple criteria
#[test]
fn demo_main_requested_features() {
    let mut doc = EditorDocument::from_content(SAMPLE_ASS_CONTENT).unwrap();
    
    println!("\n=== Main Requested Features Demo ===");
    
    // ✅ Direct event access (the main feature you requested!)
    println!("✅ DIRECT EVENT ACCESS:");
    let event_2 = doc.events().event(2).text().unwrap();
    println!("  doc.events().event(2).text() = {event_2:?}");
    
    let event_1_style = doc.events().event(1).style().unwrap();
    println!("  doc.events().event(1).style() = {event_1_style:?}");
    
    let event_0_timing = doc.events().event(0).timing().unwrap();
    println!("  doc.events().event(0).timing() = {event_0_timing:?}");
    
    // ✅ Check if event exists
    let exists = doc.events().event(10).exists().unwrap();
    println!("  doc.events().event(10).exists() = {exists}");
    
    // ✅ Get event count
    let count = doc.events().count().unwrap();
    println!("  doc.events().count() = {count}");
    
    // ✅ Get all events
    let all_events = doc.events().all().unwrap();
    println!("  doc.events().all().len() = {}", all_events.len());
    
    // ✅ Fluent querying with filter, sort, and execute
    println!("\n✅ FLUENT QUERYING:");
    let filtered_sorted = doc.events()
        .query()
        .filter_by_type(EventType::Dialogue)
        .filter_by_text("Alice")
        .sort_by_time()
        .execute()
        .unwrap();
    
    println!("  doc.events().query().filter(...).sort(...).execute():");
    for event in &filtered_sorted {
        println!("    {} - '{}' by {}", event.event.start, event.event.text, event.event.name);
    }
    
    // ✅ Convenience methods
    println!("\n✅ CONVENIENCE METHODS:");
    let dialogues = doc.events().dialogues().execute().unwrap();
    println!("  doc.events().dialogues() found {} events", dialogues.len());
    
    let by_time = doc.events().by_time().execute().unwrap();
    println!("  doc.events().by_time() sorted {} events chronologically", by_time.len());
    
    let with_alice = doc.events().containing("Alice").execute().unwrap();
    println!("  doc.events().containing('Alice') found {} events", with_alice.len());
    
    println!("\n🎉 All main requested features are working!");
}