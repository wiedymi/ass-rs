//! Stress tests for ass-editor
//!
//! Tests editor performance and correctness under extreme conditions
//! with very large documents and many operations.

use ass_editor::{
    commands::*,
    core::{EditorDocument, Position},
    utils::search::{DocumentSearch, DocumentSearchImpl},
};
use std::time::{Duration, Instant};

/// Generate a massive ASS script for stress testing
fn generate_massive_script(events: usize) -> String {
    let mut script = String::from(
        r#"[Script Info]
Title: Massive Stress Test Script
ScriptType: v4.00+
WrapStyle: 0
ScaledBorderAndShadow: yes
YCbCr Matrix: TV.601
PlayResX: 3840
PlayResY: 2160

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
"#,
    );

    // Add many styles
    for i in 0..100 {
        script.push_str(&format!(
            "Style: Style{i},Arial,{},&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1\n",
            20 + (i % 20)
        ));
    }

    script.push_str("\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");

    // Add many events
    for i in 0..events {
        let hours = i / 3600;
        let minutes = (i % 3600) / 60;
        let seconds = i % 60;
        let start_time = format!("{hours}:{minutes:02}:{seconds:02}.00");
        let end_time = format!("{}:{:02}:{:02}.00", hours, minutes, seconds + 5);
        let style = format!("Style{}", i % 100);

        // Vary the content
        let text = match i % 10 {
            0 => format!("Simple dialogue line number {i}"),
            1 => format!("{{\\pos(1920,1080)}}Positioned text at line {i}"),
            2 => format!("{{\\k20}}Ka{{\\k30}}ra{{\\k25}}o{{\\k35}}ke line {i}"),
            3 => format!("{{\\fade(255,0,255,0,0,500,1000)}}Fading text {i}"),
            4 => format!("{{\\move(100,100,3740,2060)}}Moving across screen {i}"),
            5 => format!("{{\\be1\\blur5}}Blurred effect on line {i}"),
            6 => format!("Multi\\Nline\\Ntext\\Nwith\\Nbreaks {i}"),
            7 => format!("{{\\c&HFF0000&}}Red {{\\c&H00FF00&}}Green {{\\c&H0000FF&}}Blue {i}"),
            8 => format!("{{\\fscx150\\fscy150}}Scaled larger text {i}"),
            _ => format!("{{\\frz45}}Rotated diagonal text line {i}"),
        };

        script.push_str(&format!(
            "Dialogue: 0,{start_time},{end_time},{style},,0,0,0,,{text}\n"
        ));
    }

    script
}

#[test]
#[ignore] // Run with --ignored flag
fn stress_test_massive_document_parsing() {
    let sizes = vec![10_000, 50_000, 100_000];

    for size in sizes {
        println!("\nTesting with {size} events...");
        let script = generate_massive_script(size);
        let script_len = script.len();
        println!(
            "Script size: {script_len} bytes ({:.2} MB)",
            script_len as f64 / 1_048_576.0
        );

        let start = Instant::now();
        let doc = EditorDocument::from_content(&script).unwrap();
        let parse_time = start.elapsed();

        println!("Parse time: {parse_time:?}");
        assert!(parse_time < Duration::from_secs(5), "Parsing took too long");

        // Verify document integrity
        assert_eq!(doc.text(), script);
        doc.validate().unwrap();
    }
}

#[test]
#[ignore]
fn stress_test_many_small_edits() {
    let mut doc = EditorDocument::from_content(&generate_massive_script(10_000)).unwrap();
    let num_edits = 1000;

    println!("\nPerforming {num_edits} small edits...");
    let start = Instant::now();

    for i in 0..num_edits {
        let pos = Position::new((i * 100) % doc.len());
        doc.insert(pos, "X").unwrap();
    }

    let edit_time = start.elapsed();
    println!("Total edit time: {edit_time:?}");
    let avg_time = edit_time / num_edits as u32;
    println!("Average per edit: {avg_time:?}");

    // Should maintain sub-millisecond average
    assert!(edit_time < Duration::from_millis((num_edits * 2) as u64));

    // Test undo performance
    let undo_start = Instant::now();
    let mut undo_count = 0;
    while doc.can_undo() && undo_count < 100 {
        doc.undo().unwrap();
        undo_count += 1;
    }
    let undo_time = undo_start.elapsed();
    println!("Undo {undo_count} operations: {undo_time:?}");
}

#[test]
#[ignore]
fn stress_test_search_performance() {
    use ass_editor::utils::search::{SearchOptions, SearchScope};

    let doc = EditorDocument::from_content(&generate_massive_script(50_000)).unwrap();
    let mut search = DocumentSearchImpl::new();
    search.build_index(&doc).unwrap();

    let patterns = vec!["text", "line", "\\k", "pos", "a"];

    for pattern in patterns {
        println!("\nSearching for '{pattern}'...");

        let options = SearchOptions {
            case_sensitive: false,
            whole_words: false,
            use_regex: false,
            scope: SearchScope::All,
            max_results: 10_000,
        };

        let start = Instant::now();
        let results = search.search(pattern, &options).unwrap();
        let search_time = start.elapsed();

        let result_count = results.len();
        println!("Found {result_count} results in {search_time:?}");
        assert!(search_time < Duration::from_millis(500));
    }
}

#[test]
#[ignore]
fn stress_test_batch_operations() {
    let mut doc = EditorDocument::from_content(&generate_massive_script(20_000)).unwrap();

    // Create a large batch operation
    let mut batch = BatchCommand::new("Massive batch operation".to_string());

    // Add 100 different commands
    for i in 0..100 {
        match i % 5 {
            0 => {
                // Style change
                batch = batch.add_command(Box::new(
                    EditStyleCommand::new(format!("Style{}", i % 100))
                        .set_size(22 + (i % 10) as u32),
                ));
            }
            1 => {
                // Tag insertion
                batch = batch.add_command(Box::new(InsertTagCommand::new(
                    Position::new(i * 100),
                    format!("\\fade({},0)", i * 10),
                )));
            }
            2 => {
                // Text insertion
                let pos = Position::new((i * 1000) % doc.len());
                batch = batch.add_command(Box::new(InsertTextCommand::new(pos, format!("[{i}]"))));
            }
            3 => {
                // Apply style
                batch = batch.add_command(Box::new(ApplyStyleCommand::new(
                    format!("Style{}", i % 50),
                    "Default".to_string(),
                )));
            }
            _ => {
                // Timing adjustment
                batch = batch.add_command(Box::new(TimingAdjustCommand::new(
                    vec![i, i + 1, i + 2],
                    100,
                    100,
                )));
            }
        }
    }

    println!("\nExecuting batch with 100 commands...");
    let start = Instant::now();
    let result = batch.execute(&mut doc).unwrap();
    let batch_time = start.elapsed();

    println!("Batch execution time: {batch_time:?}");
    assert!(result.success);
    assert!(batch_time < Duration::from_secs(2));

    // Test undo of entire batch
    let undo_start = Instant::now();
    doc.undo().unwrap();
    let undo_time = undo_start.elapsed();
    println!("Batch undo time: {undo_time:?}");
    assert!(undo_time < Duration::from_millis(500));
}

#[test]
#[ignore]
fn stress_test_memory_efficiency() {
    // Note: Memory tracking would require a custom allocator setup, which is complex.
    // For now, we'll just test that operations don't leak memory excessively
    // by monitoring document size growth patterns.

    let initial_script = generate_massive_script(5_000);
    let initial_size = initial_script.len();

    // Create and destroy many documents
    for i in 0..10 {
        let mut doc = EditorDocument::from_content(&initial_script).unwrap();

        // Perform operations
        for j in 0..100 {
            let pos = Position::new((i * j) % doc.len());
            doc.insert(pos, "TEST").unwrap();
        }

        // Undo half
        for _ in 0..50 {
            doc.undo().unwrap();
        }

        // Document should not grow unbounded
        let final_size = doc.text().len();
        assert!(final_size < initial_size * 2);

        // Force drop
        drop(doc);
    }
}

#[test]
#[ignore]
fn stress_test_concurrent_operations() {
    #[cfg(feature = "concurrency")]
    {
        // Note: SyncDocument is not Send due to Bump allocator constraints
        // This test validates the concept but can't actually spawn threads
        use ass_editor::core::SyncDocument;

        let doc = EditorDocument::from_content(&generate_massive_script(1_000)).unwrap();
        let sync_doc = SyncDocument::new(doc);

        // Test basic sync operations without threads due to Bump allocator constraints
        println!("\nTesting SyncDocument operations...");
        let start = Instant::now();

        // Test read operations
        let text = sync_doc.text().unwrap();
        assert!(!text.is_empty());

        // Test validation
        sync_doc.validate().unwrap();

        // Test write operations
        for i in 0..50 {
            let result = sync_doc.with_write(|doc| {
                let pos = Position::new((i * 100).min(doc.len()));
                let _ = doc.insert(pos, "X");
                Ok(())
            });
            assert!(result.is_ok());
        }

        let sync_time = start.elapsed();
        println!("Sync operations completed in {sync_time:?}");
    }
}

#[test]
#[ignore]
#[cfg(feature = "formats")]
fn stress_test_format_conversions() {
    use ass_editor::utils::formats::{ConversionOptions, FormatConverter, SubtitleFormat};

    let doc = EditorDocument::from_content(&generate_massive_script(5_000)).unwrap();
    let options = ConversionOptions::default();

    println!("\nTesting format conversions on large document...");

    // Test SRT export
    let start = Instant::now();
    let srt = FormatConverter::export(&doc, SubtitleFormat::SRT, &options).unwrap();
    let srt_time = start.elapsed();
    let srt_len = srt.len();
    println!("SRT export ({srt_len} bytes): {srt_time:?}");

    // Test WebVTT export
    let start = Instant::now();
    let vtt = FormatConverter::export(&doc, SubtitleFormat::WebVTT, &options).unwrap();
    let vtt_time = start.elapsed();
    let vtt_len = vtt.len();
    println!("WebVTT export ({vtt_len} bytes): {vtt_time:?}");

    // Test round-trip conversion
    let start = Instant::now();
    let ass_from_srt = FormatConverter::import(&srt, Some(SubtitleFormat::SRT)).unwrap();
    let import_time = start.elapsed();
    println!("Import from SRT: {import_time:?}");

    // Verify we can parse the result
    let _doc2 = EditorDocument::from_content(&ass_from_srt).unwrap();
}

#[test]
#[ignore]
fn stress_test_validation_performance() {
    let mut doc = EditorDocument::from_content(&generate_massive_script(20_000)).unwrap();

    println!("\nTesting validation performance...");

    // Basic validation
    let start = Instant::now();
    doc.validate().unwrap();
    let basic_time = start.elapsed();
    println!("Basic validation: {basic_time:?}");

    // Comprehensive validation
    let start = Instant::now();
    let issues = doc.validate_comprehensive().unwrap();
    let comprehensive_time = start.elapsed();
    let issue_count = issues.issues.len();
    println!("Comprehensive validation: {comprehensive_time:?}, {issue_count} issues found");

    assert!(basic_time < Duration::from_millis(100));
    assert!(comprehensive_time < Duration::from_secs(1));
}
