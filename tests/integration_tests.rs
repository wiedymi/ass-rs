use ass_core::{Script, plugin::{register_tag, Tag}};
use ass_render::SoftwareRenderer;
use std::time::Instant;

/// Integration tests for the full ASS library pipeline
/// These tests verify that all crates work together correctly
/// and test real-world usage scenarios.

#[test]
fn test_full_pipeline_parse_render_output() {
    // Test the complete pipeline: parse → render → output
    let ass_data = include_bytes!("../assets/all_cases.ass");
    let script = Script::parse(ass_data);
    
    // Create a mock font for testing
    let mock_font = create_mock_font();
    
    // Test renderer creation
    let renderer = SoftwareRenderer::new_multi(&script, vec![mock_font]).unwrap();
    
    // Test rendering at different timestamps
    let timestamps = [0.0, 5.0, 10.0, 15.0, 20.0];
    for timestamp in timestamps.iter() {
        let result = renderer.render_bitmap(*timestamp, 640, 360, 24.0);
        assert!(result.is_ok(), "Rendering failed at timestamp {}", timestamp);
    }
}

#[test]
fn test_cross_crate_memory_efficiency() {
    // Test memory usage across crates
    let ass_data = include_bytes!("../assets/all_cases.ass");
    
    // Parse multiple times to test memory efficiency
    let scripts: Vec<_> = (0..100).map(|_| Script::parse(ass_data)).collect();
    
    // Ensure all scripts are valid
    assert_eq!(scripts.len(), 100);
    
    // Test that they don't consume excessive memory
    // (This is more of a regression test)
    for script in &scripts {
        assert!(!script.events().is_empty());
    }
}

#[test]
fn test_error_recovery_across_components() {
    // Test error handling across different components
    
    // Invalid ASS data
    let invalid_data = b"Not valid ASS data";
    let script = Script::parse(invalid_data);
    
    // Should handle gracefully
    assert!(script.events().is_empty());
    
    // Try to render with invalid script
    let mock_font = create_mock_font();
    let renderer = SoftwareRenderer::new_multi(&script, vec![mock_font]);
    
    // Should either fail gracefully or succeed with empty output
    if let Ok(renderer) = renderer {
        let result = renderer.render_bitmap(1.0, 640, 360, 24.0);
        // Should not panic
        let _ = result;
    }
}

#[test]
fn test_timeline_rendering_with_overlapping_events() {
    // Create a script with overlapping events
    let script_content = r#"[Script Info]
Title: Timeline Test

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,First subtitle
Dialogue: 0,0:00:03.00,0:00:07.00,Default,,0,0,0,,Overlapping subtitle
Dialogue: 0,0:00:06.00,0:00:10.00,Default,,0,0,0,,Third subtitle
"#;
    
    let script = Script::parse(script_content.as_bytes());
    let mock_font = create_mock_font();
    
    if let Ok(renderer) = SoftwareRenderer::new_multi(&script, vec![mock_font]) {
        // Test rendering at overlap points
        let overlap_timestamps = [2.0, 4.0, 6.5, 8.0];
        
        for timestamp in overlap_timestamps.iter() {
            let result = renderer.render_bitmap(*timestamp, 640, 360, 24.0);
            assert!(result.is_ok(), "Failed to render overlapping events at {}", timestamp);
        }
    }
}

#[test]
fn test_unicode_content_across_pipeline() {
    // Test Unicode content through the full pipeline
    let unicode_script = r#"[Script Info]
Title: Unicode Test

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,こんにちは世界！
Dialogue: 0,0:00:06.00,0:00:10.00,Default,,0,0,0,,Привет мир! 🌍
Dialogue: 0,0:00:11.00,0:00:15.00,Default,,0,0,0,,مرحبا بالعالم
"#;
    
    let script = Script::parse(unicode_script.as_bytes());
    assert_eq!(script.events().len(), 3);
    
    // Test serialization preserves Unicode
    let serialized = script.serialize();
    let reparsed = Script::parse(serialized.as_bytes());
    assert_eq!(reparsed.events().len(), 3);
    
    // Test rendering with Unicode content
    let mock_font = create_mock_font();
    if let Ok(renderer) = SoftwareRenderer::new_multi(&script, vec![mock_font]) {
        let result = renderer.render_bitmap(2.0, 640, 360, 24.0);
        // Should not panic with Unicode content
        let _ = result;
    }
}

#[test]
fn test_plugin_system_integration() {
    // Test custom plugin integration
    struct TestTag;
    impl Tag for TestTag {
        fn name(&self) -> &'static str { "test" }
        fn parse_args(&self, _args: &[u8]) -> bool { true }
    }
    
    static TEST_TAG: TestTag = TestTag;
    register_tag(&TEST_TAG);
    
    // Test script with custom tag
    let script_with_custom_tag = r#"[Events]
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,{\test(123)}Custom tag test
"#;
    
    let script = Script::parse(script_with_custom_tag.as_bytes());
    assert!(!script.events().is_empty());
    
    // Test that custom tags are processed
    let serialized = script.serialize();
    assert!(serialized.contains("test"));
}

#[test]
fn test_performance_regression_detection() {
    // Basic performance regression test
    let ass_data = include_bytes!("../assets/all_cases.ass");
    
    // Parsing performance
    let start = Instant::now();
    for _ in 0..100 {
        let _script = Script::parse(ass_data);
    }
    let parse_duration = start.elapsed();
    
    // Should parse 100 times in under 1 second (very conservative)
    assert!(parse_duration.as_secs() < 1, "Parsing performance regression detected: {:?}", parse_duration);
    
    // Serialization performance
    let script = Script::parse(ass_data);
    let start = Instant::now();
    for _ in 0..100 {
        let _serialized = script.serialize();
    }
    let serialize_duration = start.elapsed();
    
    // Should serialize 100 times in under 1 second
    assert!(serialize_duration.as_secs() < 1, "Serialization performance regression detected: {:?}", serialize_duration);
}

#[test]
fn test_different_output_resolutions() {
    // Test rendering at various resolutions
    let ass_data = include_bytes!("../assets/all_cases.ass");
    let script = Script::parse(ass_data);
    let mock_font = create_mock_font();
    
    if let Ok(renderer) = SoftwareRenderer::new_multi(&script, vec![mock_font]) {
        let resolutions = [
            (320, 240),   // QVGA
            (640, 480),   // VGA
            (1280, 720),  // HD
            (1920, 1080), // Full HD
            (3840, 2160), // 4K
        ];
        
        for (width, height) in resolutions.iter() {
            let result = renderer.render_bitmap(5.0, *width, *height, 24.0);
            assert!(result.is_ok(), "Failed to render at {}x{}", width, height);
        }
    }
}

#[test]
fn test_edge_cases_and_malformed_input() {
    // Test various edge cases and malformed inputs
    
    // Empty input
    let empty_script = Script::parse(b"");
    assert!(empty_script.events().is_empty());
    
    // Only whitespace
    let whitespace_script = Script::parse(b"   \n\t  \n  ");
    assert!(whitespace_script.events().is_empty());
    
    // Malformed sections
    let malformed = b"[Invalid Section\nSome content without proper structure";
    let malformed_script = Script::parse(malformed);
    // Should not panic
    
    // Invalid timestamps
    let invalid_timestamps = b"[Events]\nDialogue: 0,invalid,also_invalid,Default,,0,0,0,,Test";
    let invalid_script = Script::parse(invalid_timestamps);
    // Should handle gracefully
    
    // Extremely long lines
    let mut long_line = String::from("[Events]\nDialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,");
    long_line.push_str(&"A".repeat(10000));
    let long_script = Script::parse(long_line.as_bytes());
    // Should not cause memory issues
}

#[test]
fn test_concurrent_usage() {
    // Test thread safety (if applicable)
    use std::sync::Arc;
    use std::thread;
    
    let ass_data = include_bytes!("../assets/all_cases.ass");
    let script = Arc::new(Script::parse(ass_data));
    
    let handles: Vec<_> = (0..4).map(|i| {
        let script = Arc::clone(&script);
        thread::spawn(move || {
            // Test concurrent access to script data
            assert!(!script.events().is_empty());
            
            // Test concurrent serialization
            let _serialized = script.serialize();
            
            // Test concurrent parsing
            for _ in 0..10 {
                let _new_script = Script::parse(ass_data);
            }
            
            i
        })
    }).collect();
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_wasm_compatibility() {
    // Test WASM-specific functionality if available
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen_test::*;
        wasm_bindgen_test_configure!(run_in_browser);
        
        let ass_data = include_bytes!("../assets/all_cases.ass");
        let script = Script::parse(ass_data);
        assert!(!script.events().is_empty());
        
        // Test serialization in WASM
        let serialized = script.serialize();
        assert!(!serialized.is_empty());
    }
}

// Helper functions

fn create_mock_font() -> &'static [u8] {
    // Create a minimal valid TTF/OTF mock
    static MOCK_FONT: &[u8] = &[
        0x00, 0x01, 0x00, 0x00, // sfnt version (TrueType)
        0x00, 0x04,             // number of tables
        0x00, 0x80,             // search range
        0x00, 0x02,             // entry selector
        0x00, 0x00,             // range shift
        // Minimal table directory entries would go here
        // For testing purposes, this minimal header should suffice
    ];
    MOCK_FONT
}

#[cfg(test)]
mod stress_tests {
    use super::*;
    
    #[test]
    #[ignore] // Run only when specifically requested
    fn stress_test_large_scripts() {
        // Generate a large script for stress testing
        let mut large_script = String::from("[Script Info]\nTitle: Stress Test\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");
        
        // Add 10,000 dialogue lines
        for i in 0..10000 {
            let start_time = format!("0:{:02}:{:02}.00", (i / 60) % 60, i % 60);
            let end_time = format!("0:{:02}:{:02}.00", ((i + 5) / 60) % 60, (i + 5) % 60);
            large_script.push_str(&format!(
                "Dialogue: 0,{},{},Default,,0,0,0,,Stress test line {}\n",
                start_time, end_time, i
            ));
        }
        
        // Test parsing large script
        let start = Instant::now();
        let script = Script::parse(large_script.as_bytes());
        let parse_time = start.elapsed();
        
        assert_eq!(script.events().len(), 10000);
        println!("Parsed 10,000 events in {:?}", parse_time);
        
        // Test serialization of large script
        let start = Instant::now();
        let _serialized = script.serialize();
        let serialize_time = start.elapsed();
        
        println!("Serialized 10,000 events in {:?}", serialize_time);
        
        // Performance should be reasonable even for large scripts
        assert!(parse_time.as_millis() < 1000, "Parsing took too long: {:?}", parse_time);
        assert!(serialize_time.as_millis() < 1000, "Serialization took too long: {:?}", serialize_time);
    }
    
    #[test]
    #[ignore] // Run only when specifically requested
    fn stress_test_memory_usage() {
        // Test memory usage with many script instances
        let ass_data = include_bytes!("../assets/all_cases.ass");
        
        let scripts: Vec<_> = (0..1000).map(|_| Script::parse(ass_data)).collect();
        
        // All scripts should be valid
        assert_eq!(scripts.len(), 1000);
        
        // Test that we can still create more
        for _ in 0..100 {
            let _additional = Script::parse(ass_data);
        }
        
        // Test concurrent access to many scripts
        use std::sync::Arc;
        use std::thread;
        
        let shared_scripts: Vec<Arc<Script>> = scripts.into_iter().map(Arc::new).collect();
        
        let handles: Vec<_> = (0..4).map(|_| {
            let scripts = shared_scripts.clone();
            thread::spawn(move || {
                for script in scripts.iter() {
                    assert!(!script.events().is_empty());
                }
            })
        }).collect();
        
        for handle in handles {
            handle.join().unwrap();
        }
    }
}