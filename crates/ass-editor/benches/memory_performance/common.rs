//! Shared fixtures for the memory performance benchmarks.
//!
//! Provides a generator for large ASS scripts used across the benchmark
//! submodules.

/// Generate a very large ASS script
pub fn generate_large_script(events: usize, styles: usize) -> String {
    let mut script = String::from(
        r#"[Script Info]
Title: Large Memory Test Script
ScriptType: v4.00+
WrapStyle: 0
ScaledBorderAndShadow: yes
YCbCr Matrix: TV.601
PlayResX: 1920
PlayResY: 1080
Video Zoom Percent: 1.000000
Video Position: 0

[Aegisub Project Garbage]
Last Style Storage: Default
Video File: test.mp4
Video AR Value: 1.777778
Video Zoom Percent: 0.500000
Scroll Position: 0
Active Line: 0
Video Position: 0

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
"#,
    );

    // Add many styles
    for i in 0..styles {
        let color1 = format!("&H{:02X}FFFFFF", (i * 10) % 256);
        let color2 = format!("&H{:02X}000000", (i * 20) % 256);
        script.push_str(&format!(
            "Style: Style{i},Arial,{},{color1},{color2},&H00000000,&H00000000,{},{},0,0,100,100,0,{},1,2,0,{},10,10,10,1\n",
            20 + (i % 10),
            i % 2,
            (i + 1) % 2,
            i % 360,
            (i % 9) + 1
        ));
    }

    script.push_str("\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");

    // Add many events with complex content
    let complex_texts = [
        "Simple dialogue line with no formatting",
        "{\\pos(960,540)\\fade(255,0,255,0,0,500,1000)}Positioned and fading text",
        "{\\k20}Ka{\\k30}ra{\\k25}o{\\k35}ke {\\k40}ti{\\k30}ming {\\k45}test",
        "{\\move(100,100,1820,980)\\c&HFF0000&\\3c&H0000FF&\\bord3}Moving colored text",
        "{\\be1\\blur5\\fscx120\\fscy120}Blurred and scaled text",
        "{\\clip(100,100,500,500)}Clipped text region example",
        "{\\org(960,540)\\frz45\\t(\\frz0)}Rotating text animation",
        "Multi-line\\Ndialogue\\Nwith\\Nline breaks",
        "{\\an5\\q2}Center aligned wrapped text that should span multiple lines when rendered",
        "{\\p1}m 0 0 l 100 0 100 100 0 100{\\p0} Drawing command",
    ];

    for i in 0..events {
        let layer = i % 3;
        let style = format!("Style{}", i % styles);
        let start_seconds = i * 2;
        let start_time = format!(
            "0:{:02}:{:02}.{:02}",
            start_seconds / 3600,
            (start_seconds % 3600) / 60,
            (start_seconds % 60)
        );
        let end_time = format!(
            "0:{:02}:{:02}.{:02}",
            (start_seconds + 5) / 3600,
            ((start_seconds + 5) % 3600) / 60,
            ((start_seconds + 5) % 60)
        );
        let text = &complex_texts[i % complex_texts.len()];
        let actor = if i % 5 == 0 {
            format!("Actor{}", i % 10)
        } else {
            String::new()
        };

        script.push_str(&format!(
            "Dialogue: {layer},{start_time},{end_time},{style},{actor},0,0,0,,{text} [Event #{i}]\n"
        ));
    }

    script
}
