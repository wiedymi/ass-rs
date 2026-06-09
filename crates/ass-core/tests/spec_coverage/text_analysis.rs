//! Text analysis edge cases: Unicode scripts, bidi text, and escape sequences.
//!
//! Confirms the analysis engine handles empty events, mixed-direction content,
//! long lines, and heavily overridden text without panicking.

#[cfg(feature = "analysis")]
use ass_core::analysis::ScriptAnalysis;
use ass_core::Script;

/// Test text analysis edge cases and Unicode handling
#[test]
fn test_text_analysis_edge_cases() {
    let script_text = r"[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,2,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text

; Empty and whitespace-only events
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,
Dialogue: 0,0:00:05.00,0:00:10.00,Default,,0,0,0,,
Dialogue: 0,0:00:10.00,0:00:15.00,Default,,0,0,0,,{\b1}{\b0}

; Unicode text (various scripts)
Dialogue: 0,0:00:15.00,0:00:20.00,Default,,0,0,0,,English text with unicode: cafe, naive, resume
Dialogue: 0,0:00:20.00,0:00:25.00,Default,,0,0,0,,Cyrillic text sample
Dialogue: 0,0:00:25.00,0:00:30.00,Default,,0,0,0,,Arabic: العربية (bidirectional text)
Dialogue: 0,0:00:30.00,0:00:35.00,Default,,0,0,0,,Hebrew: עברית (right-to-left)
Dialogue: 0,0:00:35.00,0:00:40.00,Default,,0,0,0,,Chinese CJK characters
Dialogue: 0,0:00:40.00,0:00:45.00,Default,,0,0,0,,Japanese hiragana katakana kanji
Dialogue: 0,0:00:45.00,0:00:50.00,Default,,0,0,0,,Korean Hangul script
Dialogue: 0,0:00:50.00,0:00:55.00,Default,,0,0,0,,Unicode emoji test

; Complex bidirectional text mixing
Dialogue: 0,0:00:55.00,0:01:00.00,Default,,0,0,0,,Mixed: English العربية English עברית English

; Line breaks and spacing variations
Dialogue: 0,0:01:00.00,0:01:05.00,Default,,0,0,0,,Line 1\NLine 2\nLine 3\hHard\hSpaces
Dialogue: 0,0:01:05.00,0:01:10.00,Default,,0,0,0,,Multiple\N\NEmpty\n\nLines

; Special Unicode characters
Dialogue: 0,0:01:10.00,0:01:15.00,Default,,0,0,0,,Zero-width joiners and special spacing
Dialogue: 0,0:01:15.00,0:01:20.00,Default,,0,0,0,,Combining diacritical marks
Dialogue: 0,0:01:20.00,0:01:25.00,Default,,0,0,0,,Control characters and direction marks

; Very long text lines
Dialogue: 0,0:01:25.00,0:01:30.00,Default,,0,0,0,,Very long text that exceeds normal subtitle length limits and should test text analysis algorithms for performance and correctness when dealing with extended content that might wrap across multiple lines or cause rendering performance issues in complex subtitle rendering scenarios.

; Malformed Unicode sequences (should be handled gracefully)
Dialogue: 0,0:01:30.00,0:01:35.00,Default,,0,0,0,,Malformed sequences with replacement characters

; Mixed content with overrides
Dialogue: 0,0:01:35.00,0:01:40.00,Default,,0,0,0,,{\b1}Bold English{\b0} {\i1}italic Arabic{\i0} {\u1}underlined Russian{\u0}

; Complex escape sequences
Dialogue: 0,0:01:40.00,0:01:45.00,Default,,0,0,0,,Escaped: \{ \} \\ \n literal braces and backslashes

; Performance stress test with many overrides
Dialogue: 0,0:01:45.00,0:01:50.00,Default,,0,0,0,,{\b1}A{\b0}{\i1}B{\i0}{\u1}C{\u0}{\s1}D{\s0}{\fscx120}E{\fscx100}{\fscy80}F{\fscy100}{\frz10}G{\frz0}H
";

    let script = Script::parse(script_text).expect("Failed to parse Unicode test script");

    #[cfg(feature = "analysis")]
    {
        let analysis = ScriptAnalysis::analyze(&script).expect("Failed to analyze Unicode script");

        // Verify all events were parsed including Unicode content
        assert!(analysis.dialogue_info().len() >= 15);

        // Check that text analysis handles various Unicode scripts
        let has_bidi_content = analysis.dialogue_info().iter().any(|info| {
            info.event().text.contains("العربية") || info.event().text.contains("עברית")
        });
        assert!(has_bidi_content, "Should detect bidirectional text");

        // Verify empty/whitespace events are handled
        let empty_or_whitespace = analysis
            .dialogue_info()
            .iter()
            .filter(|info| {
                info.event().text.trim().is_empty()
                    || info
                        .event()
                        .text
                        .chars()
                        .all(|c| c.is_whitespace() || c == '{' || c == '}')
            })
            .count();
        assert!(
            empty_or_whitespace >= 2,
            "Should handle empty/whitespace events"
        );

        // Check performance with complex content
        let performance = analysis.performance_summary();
        assert!(performance.performance_score > 0);
    }
}
