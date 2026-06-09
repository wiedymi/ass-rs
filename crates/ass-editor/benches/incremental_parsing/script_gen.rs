//! Test-script generation helper for the incremental-parsing benchmarks.

/// Generate a realistic ASS script for benchmarking
pub fn generate_test_script(events: usize) -> String {
    let mut script = String::from(
        r#"[Script Info]
Title: Benchmark Script
ScriptType: v4.00+
WrapStyle: 0
ScaledBorderAndShadow: yes
YCbCr Matrix: None
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
"#,
    );

    // Add events
    for i in 0..events {
        let start_time = format!("0:{:02}:{:02}.00", i / 60, i % 60);
        let end_time = format!("0:{:02}:{:02}.00", (i + 5) / 60, (i + 5) % 60);
        script.push_str(&format!(
            "Dialogue: 0,{start_time},{end_time},Default,,0,0,0,,Event {i} - Some dialogue text with {{\\pos(960,540)}}positioning\n"
        ));
    }

    script
}
