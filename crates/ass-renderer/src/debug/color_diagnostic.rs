//! Color diagnostic tool for debugging rendering issues

use ass_core::Script;
use ass_core::parser::ast::{Section, SectionType};

/// Diagnose color issues in ASS scripts
pub struct ColorDiagnostic;

impl ColorDiagnostic {
    /// Analyze colors in a script and report potential issues
    pub fn analyze_script(script: &Script) -> ColorReport {
        let mut report = ColorReport::default();
        
        // Check styles section
        if let Some(Section::Styles(styles)) = script.find_section(SectionType::Styles) {
            for style in styles {
                let name = style.name;
                let primary = style.primary_colour;
                let secondary = style.secondary_colour;
                let outline = style.outline_colour;
                let back = style.back_colour;
                
                report.styles.push(StyleColors {
                    name: name.to_string(),
                    primary_raw: primary.to_string(),
                    primary_parsed: parse_color_debug(primary),
                    secondary_raw: secondary.to_string(),
                    outline_raw: outline.to_string(),
                    back_raw: back.to_string(),
                });
            }
        }
        
        // Check events for color overrides
        if let Some(Section::Events(events)) = script.find_section(SectionType::Events) {
            for event in events {
                if event.text.contains("\\c") || event.text.contains("\\1c") {
                    report.has_color_overrides = true;
                }
            }
        }
        
        report
    }
    
    /// Create a test script with white text
    pub fn create_white_text_test() -> String {
        r#"[Script Info]
Title: White Text Test
PlayResX: 640
PlayResY: 480

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: WhiteText,Arial,50,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,WhiteText,,0,0,0,,This should be WHITE text"#.to_string()
    }
    
    /// Create a color reference test with multiple colors
    pub fn create_color_reference_test() -> String {
        r#"[Script Info]
Title: Color Reference Test
PlayResX: 640
PlayResY: 480

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: White,Arial,40,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,-1,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: Red,Arial,40,&H000000FF,&H000000FF,&H00000000,&H00000000,-1,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: Green,Arial,40,&H0000FF00,&H000000FF,&H00000000,&H00000000,-1,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: Blue,Arial,40,&H00FF0000,&H000000FF,&H00000000,&H00000000,-1,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: Yellow,Arial,40,&H0000FFFF,&H000000FF,&H00000000,&H00000000,-1,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: Cyan,Arial,40,&H00FFFF00,&H000000FF,&H00000000,&H00000000,-1,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: Magenta,Arial,40,&H00FF00FF,&H000000FF,&H00000000,&H00000000,-1,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:10.00,White,,0,0,0,,WHITE
Dialogue: 0,0:00:00.00,0:00:10.00,Red,,0,0,60,,RED
Dialogue: 0,0:00:00.00,0:00:10.00,Green,,0,0,120,,GREEN
Dialogue: 0,0:00:00.00,0:00:10.00,Blue,,0,0,180,,BLUE
Dialogue: 0,0:00:00.00,0:00:10.00,Yellow,,0,0,240,,YELLOW
Dialogue: 0,0:00:00.00,0:00:10.00,Cyan,,0,0,300,,CYAN
Dialogue: 0,0:00:00.00,0:00:10.00,Magenta,,0,0,360,,MAGENTA"#.to_string()
    }
}

#[derive(Debug, Default)]
pub struct ColorReport {
    pub styles: Vec<StyleColors>,
    pub has_color_overrides: bool,
}

#[derive(Debug)]
pub struct StyleColors {
    pub name: String,
    pub primary_raw: String,
    pub primary_parsed: ColorDebugInfo,
    pub secondary_raw: String,
    pub outline_raw: String,
    pub back_raw: String,
}

#[derive(Debug)]
pub struct ColorDebugInfo {
    pub hex_value: String,
    pub has_alpha: bool,
    pub alpha: u8,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub expected_color: String,
}

fn parse_color_debug(color: &str) -> ColorDebugInfo {
    let color_trimmed = color.trim_end_matches('&');
    
    if let Some(hex) = color_trimmed.strip_prefix("&H") {
        let has_alpha = hex.len() >= 8;
        
        if let Ok(value) = u32::from_str_radix(hex, 16) {
            let (alpha, bgr_value) = if has_alpha {
                let alpha = ((value >> 24) & 0xFF) as u8;
                (alpha, value & 0xFFFFFF)
            } else {
                (0x00, value) // Default to opaque
            };
            
            let r = (bgr_value & 0xFF) as u8;
            let g = ((bgr_value >> 8) & 0xFF) as u8;
            let b = ((bgr_value >> 16) & 0xFF) as u8;
            
            let expected = match (r, g, b) {
                (255, 255, 255) => "WHITE",
                (255, 0, 0) => "RED",
                (0, 255, 0) => "GREEN",
                (0, 0, 255) => "BLUE",
                (255, 255, 0) => "YELLOW",
                (0, 255, 255) => "CYAN",
                (255, 0, 255) => "MAGENTA",
                (0, 0, 0) => "BLACK",
                _ => "CUSTOM",
            };
            
            return ColorDebugInfo {
                hex_value: hex.to_string(),
                has_alpha,
                alpha,
                red: r,
                green: g,
                blue: b,
                expected_color: expected.to_string(),
            };
        }
    }
    
    ColorDebugInfo {
        hex_value: "INVALID".to_string(),
        has_alpha: false,
        alpha: 0,
        red: 0,
        green: 0,
        blue: 0,
        expected_color: "ERROR".to_string(),
    }
}

impl ColorReport {
    pub fn print_diagnostic(&self) {
        println!("=== ASS Color Diagnostic Report ===\n");
        
        for style in &self.styles {
            println!("Style: {}", style.name);
            println!("  Primary Color: {}", style.primary_raw);
            println!("    Hex: {}", style.primary_parsed.hex_value);
            println!("    RGBA: ({}, {}, {}, {})", 
                style.primary_parsed.red,
                style.primary_parsed.green,
                style.primary_parsed.blue,
                if style.primary_parsed.has_alpha { 
                    format!("{}", 255 - style.primary_parsed.alpha) // Convert ASS to RGBA alpha
                } else { 
                    "255".to_string() 
                }
            );
            println!("    Expected: {}", style.primary_parsed.expected_color);
            
            // Diagnostic for common issues
            if style.primary_parsed.expected_color == "YELLOW" && style.name.contains("White") {
                println!("    ⚠️  WARNING: Style named '{}' is YELLOW but should probably be WHITE!", style.name);
                println!("       Correct white: &H00FFFFFF");
                println!("       Current value might be: &H0000FFFF (yellow)");
            }
            println!();
        }
        
        if self.has_color_overrides {
            println!("Note: Script contains color override tags (\\c or \\1c) in events");
        }
    }
}