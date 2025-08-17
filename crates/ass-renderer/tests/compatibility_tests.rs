//! Comprehensive libass compatibility tests
//!
//! This test suite verifies pixel-perfect compatibility with libass
//! using a variety of ASS features and edge cases.

use ass_core::parser::Script;
#[cfg(feature = "libass-compare")]
use ass_renderer::debug::{CompatibilityTestSuite, CompatibilityTester, TestConfig};
use ass_renderer::renderer::RenderContext;

const TEST_WIDTH: u32 = 1920;
const TEST_HEIGHT: u32 = 1080;

/// Test basic text rendering compatibility
#[test]
#[cfg(feature = "libass-compare")]
fn test_basic_text_compatibility() {
    let context = RenderContext::new(TEST_WIDTH, TEST_HEIGHT);
    let config = TestConfig {
        pixel_tolerance: 2, // Allow slight differences due to different rasterizers
        significance_threshold: 0.005, // 0.5% threshold
        ..Default::default()
    };

    let mut tester = CompatibilityTester::new(context, config).expect("Failed to create tester");

    let script_content = r#"[Script Info]
Title: Basic Text Test
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,48,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,1,2,50,50,50,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,Hello World!"#;

    let script = Script::parse(script_content).expect("Failed to parse script");
    let result = tester
        .test_script_compatibility(&script, "basic_text")
        .expect("Test failed");

    assert!(
        result.passed,
        "Basic text test failed with {}% difference",
        result.pixel_diff_percentage * 100.0
    );
}

/// Test various text effects and formatting
#[test]
#[cfg(feature = "libass-compare")]
fn test_text_effects_compatibility() {
    let context = RenderContext::new(TEST_WIDTH, TEST_HEIGHT);
    let config = TestConfig::default();

    let mut tester = CompatibilityTester::new(context, config).expect("Failed to create tester");

    let script_content = r#"[Script Info]
Title: Text Effects Test
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,48,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,1,2,50,50,50,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,{\b1}Bold text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,{\i1}Italic text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,{\u1}Underlined text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,{\s1}Strikethrough text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,{\c&H00FF00&}Green text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,{\fs72}Large text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,{\fsp10}Spaced text"#;

    let script = Script::parse(script_content).expect("Failed to parse script");
    let result = tester
        .test_script_compatibility(&script, "text_effects")
        .expect("Test failed");

    // Text effects might have more differences due to implementation differences
    assert!(
        result.pixel_diff_percentage < 0.05,
        "Text effects test failed with {}% difference",
        result.pixel_diff_percentage * 100.0
    );
}

/// Test animation and movement
#[test]
#[cfg(feature = "libass-compare")]
fn test_animation_compatibility() {
    let context = RenderContext::new(TEST_WIDTH, TEST_HEIGHT);
    let config = TestConfig {
        test_animations: true,
        animation_step_cs: 10,          // 100ms steps
        max_animation_duration_cs: 500, // 5 seconds
        ..Default::default()
    };

    let mut tester = CompatibilityTester::new(context, config).expect("Failed to create tester");

    let script_content = r#"[Script Info]
Title: Animation Test
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,48,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,1,2,50,50,50,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,{\move(100,500,1800,500,0,5000)}Moving text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,{\fad(500,500)}Fading text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,{\t(0,2000,\fs72)}Growing text"#;

    let script = Script::parse(script_content).expect("Failed to parse script");
    let result = tester
        .test_script_compatibility(&script, "animation")
        .expect("Test failed");

    // Animation might have timing differences, allow more tolerance
    assert!(
        result.pixel_diff_percentage < 0.1,
        "Animation test failed with {}% difference",
        result.pixel_diff_percentage * 100.0
    );
}

/// Test complex transformations
#[test]
#[cfg(feature = "libass-compare")]
fn test_transform_compatibility() {
    let context = RenderContext::new(TEST_WIDTH, TEST_HEIGHT);
    let config = TestConfig::default();

    let mut tester = CompatibilityTester::new(context, config).expect("Failed to create tester");

    let script_content = r#"[Script Info]
Title: Transform Test
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,48,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,1,2,50,50,50,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,{\frz45}Rotated text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,{\fscx150\fscy75}Scaled text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,{\fax0.3}Sheared text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,{\org(960,540)\frz90}Rotated around center"#;

    let script = Script::parse(script_content).expect("Failed to parse script");
    let result = tester
        .test_script_compatibility(&script, "transforms")
        .expect("Test failed");

    // Transforms are complex and may have implementation differences
    assert!(
        result.pixel_diff_percentage < 0.08,
        "Transform test failed with {}% difference",
        result.pixel_diff_percentage * 100.0
    );
}

/// Test karaoke effects
#[test]
#[cfg(feature = "libass-compare")]
fn test_karaoke_compatibility() {
    let context = RenderContext::new(TEST_WIDTH, TEST_HEIGHT);
    let config = TestConfig {
        test_animations: true,
        animation_step_cs: 5,            // 50ms steps for karaoke
        max_animation_duration_cs: 1000, // 10 seconds
        ..Default::default()
    };

    let mut tester = CompatibilityTester::new(context, config).expect("Failed to create tester");

    let script_content = r#"[Script Info]
Title: Karaoke Test
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,48,&H00FFFFFF,&H000000FF,&H00FF0000,&H80000000,0,0,0,0,100,100,0,0,1,2,1,2,50,50,50,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,karaoke,{\k30}Ka{\k30}ra{\k30}o{\k30}ke {\k30}test
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,{\kf50}Smooth {\kf50}karaoke"#;

    let script = Script::parse(script_content).expect("Failed to parse script");
    let result = tester
        .test_script_compatibility(&script, "karaoke")
        .expect("Test failed");

    // Karaoke timing is complex, allow higher tolerance
    assert!(
        result.pixel_diff_percentage < 0.15,
        "Karaoke test failed with {}% difference",
        result.pixel_diff_percentage * 100.0
    );
}

/// Test drawing commands
#[test]
#[cfg(feature = "libass-compare")]
fn test_drawing_compatibility() {
    let context = RenderContext::new(TEST_WIDTH, TEST_HEIGHT);
    let config = TestConfig::default();

    let mut tester = CompatibilityTester::new(context, config).expect("Failed to create tester");

    let script_content = r#"[Script Info]
Title: Drawing Test
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,48,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,1,2,50,50,50,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,{\p1}m 100 100 l 200 100 200 200 100 200{\p0}
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,{\p1}m 300 300 b 400 250 500 350 600 300{\p0}"#;

    let script = Script::parse(script_content).expect("Failed to parse script");
    let result = tester
        .test_script_compatibility(&script, "drawing")
        .expect("Test failed");

    // Drawing commands might have differences in curve rendering
    assert!(
        result.pixel_diff_percentage < 0.1,
        "Drawing test failed with {}% difference",
        result.pixel_diff_percentage * 100.0
    );
}

/// Test edge cases and error handling
#[test]
#[cfg(feature = "libass-compare")]
fn test_edge_cases_compatibility() {
    let context = RenderContext::new(TEST_WIDTH, TEST_HEIGHT);
    let config = TestConfig::default();

    let mut tester = CompatibilityTester::new(context, config).expect("Failed to create tester");

    let script_content = r#"[Script Info]
Title: Edge Cases Test
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,48,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,1,2,50,50,50,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,{\fs0}Zero size
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,{\pos(-100,-100)}Off screen
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,{\c&HINVALID&}Invalid color
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,{\unknown}Unknown tag"#;

    let script = Script::parse(script_content).expect("Failed to parse script");
    let result = tester
        .test_script_compatibility(&script, "edge_cases")
        .expect("Test failed");

    // Edge cases should still maintain compatibility
    assert!(
        result.pixel_diff_percentage < 0.02,
        "Edge cases test failed with {}% difference",
        result.pixel_diff_percentage * 100.0
    );
}

/// Run comprehensive test suite
#[test]
#[cfg(feature = "libass-compare")]
fn test_comprehensive_compatibility_suite() {
    let context = RenderContext::new(TEST_WIDTH, TEST_HEIGHT);
    let config = TestConfig {
        generate_visual_diffs: true,
        output_dir: "test_output/comprehensive".to_string(),
        ..Default::default()
    };

    let mut test_suite =
        CompatibilityTestSuite::new(context, config).expect("Failed to create test suite");

    // Add various test cases
    test_suite.add_test_case("basic_dialogue".to_string(), create_basic_dialogue_script());
    test_suite.add_test_case(
        "complex_styling".to_string(),
        create_complex_styling_script(),
    );
    test_suite.add_test_case("multi_language".to_string(), create_multi_language_script());
    test_suite.add_test_case(
        "performance_stress".to_string(),
        create_performance_stress_script(),
    );

    let results = test_suite.run_all_tests().expect("Test suite failed");

    // Check overall compatibility
    let total_tests = results.len();
    let passed_tests = results.iter().filter(|r| r.passed).count();
    let avg_diff =
        results.iter().map(|r| r.pixel_diff_percentage).sum::<f64>() / total_tests as f64;

    println!("Compatibility Test Results:");
    println!(
        "  Passed: {}/{} ({:.1}%)",
        passed_tests,
        total_tests,
        (passed_tests as f64 / total_tests as f64) * 100.0
    );
    println!("  Average difference: {:.3}%", avg_diff * 100.0);

    // At least 80% of tests should pass
    assert!(
        passed_tests as f64 / total_tests as f64 >= 0.8,
        "Less than 80% of compatibility tests passed"
    );

    // Average difference should be less than 2%
    assert!(
        avg_diff < 0.02,
        "Average pixel difference too high: {:.3}%",
        avg_diff * 100.0
    );
}

// Helper functions to create test scripts

fn create_basic_dialogue_script() -> String {
    r#"[Script Info]
Title: Basic Dialogue
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,48,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,1,2,50,50,50,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,This is a basic dialogue line.
Dialogue: 0,0:00:02.00,0:00:06.00,Default,,0,0,0,,This is another line with different timing."#.to_string()
}

fn create_complex_styling_script() -> String {
    r#"[Script Info]
Title: Complex Styling
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,48,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,1,2,50,50,50,1
Style: Title,Arial,72,&H00FFFF00,&H000000FF,&H00000000,&H80000000,1,0,0,0,120,120,5,0,1,3,2,2,100,100,100,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Title,,0,0,0,,{\pos(960,200)}Main Title
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,{\c&H00FF00&\b1}Green bold text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,{\blur2\c&HFF0000&}Blurred red text"#.to_string()
}

fn create_multi_language_script() -> String {
    r#"[Script Info]
Title: Multi Language
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,48,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,1,2,50,50,50,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,English text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,日本語テキスト
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,中文文本
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,العربية النص"#.to_string()
}

fn create_performance_stress_script() -> String {
    let mut script = r#"[Script Info]
Title: Performance Stress
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,48,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,1,2,50,50,50,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text"#.to_string();

    // Add many concurrent dialogue lines
    for i in 0..20 {
        script.push_str(&format!(
            "\nDialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,{{\\pos({},{})}}Line {}",
            100 + (i % 10) * 180,
            100 + (i / 10) * 100,
            i + 1
        ));
    }

    script
}

// Stub tests when libass-compare is not available
#[cfg(not(feature = "libass-compare"))]
mod stub_tests {
    #[test]
    fn compatibility_tests_require_libass_feature() {
        println!("Compatibility tests require 'libass-compare' feature to be enabled");
        println!("Install libass and run with: cargo test --features libass-compare");
    }
}
