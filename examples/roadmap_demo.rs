//! Comprehensive demo of the roadmap features
//! 
//! This example demonstrates:
//! 1. Enhanced override-tag registry with animation support
//! 2. Text shaping with rustybuzz
//! 3. Dynamic plugin loading (desktop only)
//! 4. Advanced animation state management

use ass_core::{Script, plugin::*};
use ass_render::{TextShaper, TextDirection, TextLayout};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ASS-RS Roadmap Feature Demo ===\n");

    // Initialize built-in tags
    ass_core::builtins::register_builtins();
    
    // 1. Demonstrate enhanced override-tag registry
    println!("1. Enhanced Override-Tag Registry");
    println!("---------------------------------");
    demo_enhanced_tags()?;

    // 2. Demonstrate text shaping
    println!("\n2. Text Shaping with rustybuzz");
    println!("-------------------------------");
    demo_text_shaping()?;

    // 3. Demonstrate dynamic plugin loading (desktop only)
    #[cfg(all(feature = "dynamic-loading", not(target_family = "wasm")))]
    {
        println!("\n3. Dynamic Plugin Loading");
        println!("--------------------------");
        demo_dynamic_loading()?;
    }

    // 4. Demonstrate animation state management
    println!("\n4. Animation State Management");
    println!("-----------------------------");
    demo_animation_management()?;

    // 5. Performance demonstration
    println!("\n5. Performance Showcase");
    println!("-----------------------");
    demo_performance()?;

    println!("\n=== Demo Complete ===");
    Ok(())
}

fn demo_enhanced_tags() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing enhanced tag parsing and animation...");

    // Test move tag with timing
    if let Some(move_tag) = get_tag("move") {
        let args = b"100,200,300,400,0,2000";
        match move_tag.parse_args(args) {
            Ok(parsed_args) => {
                println!("✓ Move tag parsed {} arguments", parsed_args.len());
                
                // Test animation application
                let mut state = AnimationState::new();
                if move_tag.apply(&parsed_args, &mut state).is_ok() {
                    println!("✓ Move animation applied, {} active animations", state.active_animations.len());
                } else {
                    println!("✗ Failed to apply move animation");
                }
            }
            Err(e) => println!("✗ Move tag parsing failed: {:?}", e),
        }
    } else {
        println!("✗ Move tag not found");
    }

    // Test fade tag
    if let Some(fade_tag) = get_tag("fad") {
        let args = b"500,1000";
        match fade_tag.parse_args(args) {
            Ok(parsed_args) => {
                println!("✓ Fade tag parsed {} arguments", parsed_args.len());
                
                let mut state = AnimationState::new();
                if fade_tag.apply(&parsed_args, &mut state).is_ok() {
                    println!("✓ Fade animation applied");
                } else {
                    println!("✗ Failed to apply fade animation");
                }
            }
            Err(e) => println!("✗ Fade tag parsing failed: {:?}", e),
        }
    }

    // Test karaoke tag
    if let Some(karaoke_tag) = get_tag("k") {
        let args = b"100";
        match karaoke_tag.parse_args(args) {
            Ok(parsed_args) => {
                println!("✓ Karaoke tag parsed {} arguments", parsed_args.len());
                
                let mut state = AnimationState::new();
                if karaoke_tag.apply(&parsed_args, &mut state).is_ok() {
                    println!("✓ Karaoke animation applied");
                }
            }
            Err(e) => println!("✗ Karaoke tag parsing failed: {:?}", e),
        }
    }

    println!("Available tags: {}", get_all_tag_names().join(", "));
    
    Ok(())
}

fn demo_text_shaping() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing text shaping capabilities...");

    let mut shaper = TextShaper::new();
    shaper.set_font_size(24.0);
    shaper.set_dpi(96.0);

    // Test different text directions
    let test_texts = vec![
        ("Hello World", TextDirection::LeftToRight),
        ("مرحبا بالعالم", TextDirection::RightToLeft),
        ("こんにちは世界", TextDirection::LeftToRight),
        ("שלום עולם", TextDirection::RightToLeft),
    ];

    for (text, expected_direction) in test_texts {
        println!("Analyzing text: '{}'", text);
        
        let (detected_script, detected_direction) = shaper.detect_text_properties(text);
        println!("  Detected script: {:?}, direction: {:?}", detected_script, detected_direction);
        
        match shaper.shape_text(text, "default", expected_direction) {
            Ok(shaped) => {
                println!("  ✓ Shaped: {} glyphs, {:.1}px advance, {:.1}px line height", 
                    shaped.glyphs.len(), shaped.total_advance, shaped.line_height);
            }
            Err(e) => {
                println!("  ✗ Shaping failed: {}", e);
            }
        }
    }

    // Test text layout with wrapping
    println!("\nTesting text layout with wrapping...");
    let layout = TextLayout::new(shaper);
    
    let long_text = "This is a very long text that should wrap into multiple lines when the maximum width is constrained. It demonstrates the text layout capabilities.";
    
    match layout.layout_text(long_text, "default", TextDirection::LeftToRight) {
        Ok(lines) => {
            println!("✓ Text layout: {} lines", lines.len());
            for (i, line) in lines.iter().enumerate() {
                println!("  Line {}: {} glyphs, {:.1}px advance", 
                    i + 1, line.glyphs.len(), line.total_advance);
            }
        }
        Err(e) => {
            println!("✗ Text layout failed: {}", e);
        }
    }

    Ok(())
}

#[cfg(all(feature = "dynamic-loading", not(target_family = "wasm")))]
fn demo_dynamic_loading() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing dynamic plugin loading...");

    // In a real scenario, you would compile the wave_plugin and load it
    // For this demo, we'll simulate it by showing how it would work
    
    let plugin_path = Path::new("target/release/libwave_plugin.so"); // Linux
    let plugin_path_win = Path::new("target/release/wave_plugin.dll"); // Windows
    let plugin_path_mac = Path::new("target/release/libwave_plugin.dylib"); // macOS

    let plugin_exists = plugin_path.exists() || plugin_path_win.exists() || plugin_path_mac.exists();
    
    if plugin_exists {
        println!("✓ Plugin binary found");
        
        // Try to load the plugin
        let load_path = if plugin_path.exists() {
            plugin_path
        } else if plugin_path_win.exists() {
            plugin_path_win
        } else {
            plugin_path_mac
        };
        
        match load_plugin(load_path) {
            Ok(()) => {
                println!("✓ Plugin loaded successfully");
                
                // Test if the wave tag is now available
                if let Some(wave_tag) = get_tag("wave") {
                    println!("✓ Wave tag available after dynamic loading");
                    
                    let args = b"10.0,2.0";
                    match wave_tag.parse_args(args) {
                        Ok(parsed_args) => {
                            println!("✓ Wave tag parsed {} arguments", parsed_args.len());
                        }
                        Err(e) => {
                            println!("✗ Wave tag parsing failed: {:?}", e);
                        }
                    }
                } else {
                    println!("✗ Wave tag not found after loading");
                }
            }
            Err(e) => {
                println!("✗ Plugin loading failed: {:?}", e);
            }
        }
    } else {
        println!("⚠ Plugin binary not found. To test dynamic loading:");
        println!("  1. Build the wave plugin: cargo build --release -p wave_plugin");
        println!("  2. Run this demo again");
        
        // Show what tags would be available
        println!("Plugin would register: wave, spiral");
    }

    Ok(())
}

#[cfg(not(all(feature = "dynamic-loading", not(target_family = "wasm"))))]
fn demo_dynamic_loading() -> Result<(), Box<dyn std::error::Error>> {
    println!("Dynamic loading not available on this platform");
    Ok(())
}

fn demo_animation_management() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing animation state management...");

    let mut state = AnimationState::new();
    
    // Add some test animations
    let move_animation = ActiveAnimation {
        tag_name: "pos".to_string(),
        start_time: 0.0,
        end_time: 2.0,
        start_value: TagArgument::Position(100.0, 200.0),
        end_value: TagArgument::Position(300.0, 400.0),
        mode: AnimationMode::Linear,
    };
    
    let fade_animation = ActiveAnimation {
        tag_name: "alpha".to_string(),
        start_time: 0.5,
        end_time: 1.5,
        start_value: TagArgument::Integer(0),
        end_value: TagArgument::Integer(255),
        mode: AnimationMode::Accelerating(2.0),
    };
    
    let rotation_animation = ActiveAnimation {
        tag_name: "rotation".to_string(),
        start_time: 0.0,
        end_time: 3.0,
        start_value: TagArgument::Float(0.0),
        end_value: TagArgument::Float(360.0),
        mode: AnimationMode::Bezier(0.25, 0.1, 0.75, 0.9),
    };

    state.add_animation(move_animation);
    state.add_animation(fade_animation);
    state.add_animation(rotation_animation);
    
    println!("Added {} animations", state.active_animations.len());

    // Test animation updates at different times
    let test_times = vec![0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0];
    
    for time in test_times {
        state.update(time);
        println!("Time {:.1}s: {} interpolated values", time, state.interpolated_values.len());
        
        for (tag_name, value) in &state.interpolated_values {
            match value {
                TagArgument::Position(x, y) => {
                    println!("  {}: position({:.1}, {:.1})", tag_name, x, y);
                }
                TagArgument::Float(f) => {
                    println!("  {}: {:.1}", tag_name, f);
                }
                TagArgument::Integer(i) => {
                    println!("  {}: {}", tag_name, i);
                }
                _ => {
                    println!("  {}: {:?}", tag_name, value);
                }
            }
        }
    }

    Ok(())
}

fn demo_performance() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing performance with complex scripts...");

    let start = std::time::Instant::now();

    // Create a complex ASS script
    let script_content = create_complex_script(1000); // 1000 dialogue lines
    let script = Script::parse(script_content.as_bytes());
    
    let parse_duration = start.elapsed();
    println!("✓ Parsed 1000-line script in {:.2}ms", parse_duration.as_secs_f64() * 1000.0);

    // Test serialization performance
    let start = std::time::Instant::now();
    let serialized = script.serialize();
    let serialize_duration = start.elapsed();
    println!("✓ Serialized script ({} bytes) in {:.2}ms", 
        serialized.len(), serialize_duration.as_secs_f64() * 1000.0);

    // Test tag processing performance
    let start = std::time::Instant::now();
    let mut tag_count = 0;
    
    for tag_name in get_all_tag_names() {
        if let Some(tag) = get_tag(tag_name) {
            tag_count += 1;
            
            // Test basic parsing
            let test_args = b"100,200";
            let _ = tag.parse_args(test_args);
        }
    }
    
    let tag_duration = start.elapsed();
    println!("✓ Processed {} tags in {:.2}ms", 
        tag_count, tag_duration.as_secs_f64() * 1000.0);

    // Test animation performance
    let start = std::time::Instant::now();
    let mut state = AnimationState::new();
    
    // Add many animations
    for i in 0..100 {
        let animation = ActiveAnimation {
            tag_name: format!("test_{}", i),
            start_time: 0.0,
            end_time: 10.0,
            start_value: TagArgument::Float(0.0),
            end_value: TagArgument::Float(100.0),
            mode: AnimationMode::Linear,
        };
        state.add_animation(animation);
    }
    
    // Update animation state
    for time_step in 0..100 {
        state.update(time_step as f64 * 0.1);
    }
    
    let animation_duration = start.elapsed();
    println!("✓ Processed 100 animations x 100 updates in {:.2}ms", 
        animation_duration.as_secs_f64() * 1000.0);

    Ok(())
}

fn create_complex_script(line_count: usize) -> String {
    let mut script = String::from("[Script Info]\nTitle: Performance Test\n\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\nStyle: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,0,0,2,0,0,0,1\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");

    for i in 0..line_count {
        let start_time = format!("0:{:02}:{:02}.{:02}", 
            (i * 2) / 3600, 
            ((i * 2) % 3600) / 60, 
            (i * 2) % 60, 
            ((i * 2) % 100)
        );
        
        let end_time = format!("0:{:02}:{:02}.{:02}", 
            ((i + 1) * 2) / 3600, 
            (((i + 1) * 2) % 3600) / 60, 
            ((i + 1) * 2) % 60, 
            (((i + 1) * 2) % 100)
        );

        let text = format!("{{\\b1\\i1\\c&H00FF00&\\move(100,200,300,400)}}Subtitle line {} with complex formatting", i + 1);
        
        script.push_str(&format!("Dialogue: 0,{},{},Default,,0,0,0,,{}\n", 
            start_time, end_time, text));
    }

    script
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_functions() {
        // Initialize built-in tags for testing
        ass_core::builtins::register_builtins();
        
        assert!(demo_enhanced_tags().is_ok());
        assert!(demo_text_shaping().is_ok());
        assert!(demo_animation_management().is_ok());
        assert!(demo_performance().is_ok());
    }
}