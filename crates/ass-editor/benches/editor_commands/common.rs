//! Shared script generation helpers for editor command benchmarks.

/// Generate a test script with styles and events
pub fn generate_complex_script(styles: usize, events: usize) -> String {
    let mut script = String::from(
        r#"[Script Info]
Title: Command Benchmark Script
ScriptType: v4.00+
WrapStyle: 0
ScaledBorderAndShadow: yes
YCbCr Matrix: None
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
"#,
    );

    // Add additional styles
    for i in 1..styles {
        script.push_str(&format!(
            "Style: Style{i},Arial,24,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1\n"
        ));
    }

    script.push_str("\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");

    // Add events
    for i in 0..events {
        let style = if i % 3 == 0 {
            "Default"
        } else {
            &format!("Style{}", (i % styles).max(1))
        };
        let start_time = format!("0:{:02}:{:02}.00", i / 60, i % 60);
        let end_time = format!("0:{:02}:{:02}.00", (i + 5) / 60, (i + 5) % 60);
        script.push_str(&format!(
            "Dialogue: 0,{start_time},{end_time},{style},,0,0,0,,Event {i} with {{\\pos(960,540)}}some {{\\b1}}bold{{\\b0}} text\n"
        ));
    }

    script
}
