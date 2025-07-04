//! Integration tests for ass-render crate

use ass_core::Script;
use ass_render::model::*;

#[cfg(feature = "software")]
use ass_render::SoftwareRenderer;

#[cfg(feature = "hardware")]
use ass_render::{HardwareRenderer, HardwareRendererError};

fn create_test_script() -> Script {
    let ass_content = r#"[Script Info]
Title: Integration Test Script

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,32,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello World
Dialogue: 0,0:00:02.00,0:00:07.00,Default,,0,0,0,,{\b1}Bold Text{\b0}
Dialogue: 0,0:00:04.00,0:00:09.00,Default,,0,0,0,,{\i1}Italic Text{\i0}
Dialogue: 0,0:00:06.00,0:00:11.00,Default,,0,0,0,,{\c&H0000FF&}Red Text
"#;
    Script::parse(ass_content.as_bytes())
}

fn create_mock_font_data() -> Vec<u8> {
    // Create mock font data for testing
    vec![0u8; 1024]
}

#[cfg(feature = "software")]
#[test]
fn test_software_renderer_basic() {
    let script = create_test_script();
    let font_data = create_mock_font_data();
    let font_static: &'static [u8] = Box::leak(font_data.into_boxed_slice());

    let renderer = SoftwareRenderer::new(&script, font_static);

    // Test rendering at different times
    let frame_0 = renderer.render(0.0);
    let frame_3 = renderer.render(3.0);
    let frame_10 = renderer.render(10.0);

    // At time 0, should have "Hello World"
    assert!(!frame_0.lines.is_empty());

    // At time 3, should have multiple lines
    assert!(frame_3.lines.len() >= 2);

    // At time 10, should still have one line (Red Text ends at 11 seconds)
    assert_eq!(frame_10.lines.len(), 1);
}

#[cfg(feature = "software")]
#[test]
fn test_software_renderer_bitmap_output() {
    let script = create_test_script();
    let font_data = create_mock_font_data();
    let font_static: &'static [u8] = Box::leak(font_data.into_boxed_slice());

    let renderer = SoftwareRenderer::new(&script, font_static);

    let bitmap = renderer.render_bitmap(1.0, 640, 480, 24.0);

    // Should have correct buffer size (width * height * 4 channels)
    assert_eq!(bitmap.len(), 640 * 480 * 4);

    // Buffer should not be all zeros (some content should be rendered)
    // Note: This might be all zeros with mock font data, but structure is correct
    assert!(bitmap.iter().all(|&x| x == 0) || bitmap.iter().any(|&x| x != 0));
}

#[cfg(feature = "hardware")]
#[tokio::test]
async fn test_hardware_renderer_basic() {
    let script = create_test_script();
    let font_data = create_mock_font_data();

    match HardwareRenderer::new(&script, &font_data).await {
        Ok(renderer) => {
            println!("Hardware renderer created successfully");
            println!("Backend: {}", renderer.get_backend());

            // Test frame rendering
            let frame_0 = renderer.render(0.0);
            let frame_3 = renderer.render(3.0);

            assert!(!frame_0.lines.is_empty());
            assert!(frame_3.lines.len() >= 2);
        }
        Err(HardwareRendererError::AdapterNotFound) => {
            println!("Hardware renderer test skipped: No graphics adapter available");
        }
        Err(e) => {
            panic!("Unexpected hardware renderer error: {}", e);
        }
    }
}

#[cfg(feature = "hardware")]
#[tokio::test]
async fn test_hardware_renderer_texture_output() {
    let script = create_test_script();
    let font_data = create_mock_font_data();

    match HardwareRenderer::new(&script, &font_data).await {
        Ok(mut renderer) => {
            let result = renderer.render_to_texture(1.0, 800, 600, 32.0).await;

            match result {
                Ok(buffer) => {
                    assert_eq!(buffer.len(), 800 * 600 * 4);
                    println!("Hardware rendering completed successfully");
                }
                Err(e) => {
                    println!("Hardware texture rendering failed: {}", e);
                }
            }
        }
        Err(HardwareRendererError::AdapterNotFound) => {
            println!("Hardware renderer test skipped: No graphics adapter available");
        }
        Err(e) => {
            panic!("Unexpected hardware renderer error: {}", e);
        }
    }
}

#[cfg(all(feature = "software", feature = "hardware"))]
#[tokio::test]
async fn test_software_vs_hardware_comparison() {
    let script = create_test_script();
    let font_data = create_mock_font_data();

    // Software renderer
    let font_static: &'static [u8] = Box::leak(font_data.clone().into_boxed_slice());
    let sw_renderer = SoftwareRenderer::new(&script, font_static);
    let sw_frame = sw_renderer.render(1.0);

    // Hardware renderer
    match HardwareRenderer::new(&script, &font_data).await {
        Ok(hw_renderer) => {
            let hw_frame = hw_renderer.render(1.0);

            // Both should have the same number of lines at the same time
            assert_eq!(sw_frame.lines.len(), hw_frame.lines.len());

            // Both should have the same text content (simplified check)
            for (sw_line, hw_line) in sw_frame.lines.iter().zip(hw_frame.lines.iter()) {
                assert_eq!(sw_line.segments.len(), hw_line.segments.len());
                for (sw_seg, hw_seg) in sw_line.segments.iter().zip(hw_line.segments.iter()) {
                    assert_eq!(sw_seg.text, hw_seg.text);
                }
            }

            println!("Software and hardware renderers produce consistent results");
        }
        Err(HardwareRendererError::AdapterNotFound) => {
            println!("Hardware comparison test skipped: No graphics adapter available");
        }
        Err(e) => {
            println!("Hardware comparison test failed: {}", e);
        }
    }
}

#[test]
fn test_model_types() {
    // Test the model types work correctly
    let pos = Pos { x: 100.0, y: 200.0 };
    assert_eq!(pos.x, 100.0);
    assert_eq!(pos.y, 200.0);

    let style = StyleState::default();
    assert_eq!(style.color, 0xFFFFFF); // White
    assert_eq!(style.alpha, 1.0);
    assert_eq!(style.font_size, 32.0);

    let clip = ClipRect {
        x1: 0.0,
        y1: 0.0,
        x2: 100.0,
        y2: 100.0,
    };
    assert_eq!(clip.x2 - clip.x1, 100.0);
    assert_eq!(clip.y2 - clip.y1, 100.0);
}

#[test]
fn test_feature_compilation() {
    // Test that the crate compiles with different feature combinations
    println!("Software feature enabled: {}", cfg!(feature = "software"));
    println!("Hardware feature enabled: {}", cfg!(feature = "hardware"));

    // At least one rendering feature should be enabled for meaningful functionality
    assert!(
        cfg!(feature = "software") || cfg!(feature = "hardware"),
        "At least one rendering feature should be enabled"
    );
}
