//! Shared script generation helper for search benchmarks.

/// Generate a large script with varied content for search testing
pub fn generate_search_script(events: usize) -> String {
    let mut script = String::from(
        r#"[Script Info]
Title: Search Benchmark Script
ScriptType: v4.00+
WrapStyle: 0
ScaledBorderAndShadow: yes
YCbCr Matrix: None
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: Title,Arial,32,&H00FFFF00,&H000000FF,&H00000000,&H00000000,1,0,0,0,100,100,0,0,1,3,0,2,10,10,10,1
Style: Sign,Arial,18,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,8,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
"#,
    );

    // Add varied events for realistic search scenarios
    let dialogues = [
        "Hello world! This is a test dialogue.",
        "The quick brown fox jumps over the lazy dog.",
        "Advanced SubStation Alpha subtitle format.",
        r"{\pos(960,540)}Positioned text in the center.",
        r"{\fade(255,0,255,0,0,500,1000)}Fading in and out.",
        r"{\k50}Ka{\k30}ra{\k40}o{\k35}ke timing test.",
        r"Multiple {\b1}bold{\b0} and {\i1}italic{\i0} tags.",
        "Sign text appears here with special styling.",
        "Another line with different content for searching.",
        r"Complex effects: {\move(100,100,500,500)}{\c&H00FF00&}Green moving text.",
    ];

    for i in 0..events {
        let style = match i % 10 {
            0..=6 => "Default",
            7..=8 => "Title",
            _ => "Sign",
        };
        let dialogue = &dialogues[i % dialogues.len()];
        let start_time = format!("0:{:02}:{:02}.00", i / 60, i % 60);
        let end_time = format!("0:{:02}:{:02}.00", (i + 5) / 60, (i + 5) % 60);

        script.push_str(&format!(
            "Dialogue: 0,{start_time},{end_time},{style},,0,0,0,,{dialogue}\n"
        ));
    }

    script
}
