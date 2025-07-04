use ass_core::{tokenizer::AssTokenizer, Script};

#[test]
fn test_script_parse() {
    let input = b"[Script Info]\nTitle: Example\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\nDialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,Hello World!";
    let script = Script::parse(input);
    assert!(!script.sections().is_empty());
}

#[test]
fn test_empty_script() {
    let input = b"";
    let script = Script::parse(input);
    assert!(script.sections().is_empty());
}

#[test]
fn test_script_info_parsing() {
    let input = b"[Script Info]\nTitle: Test Title\nOriginalScript: Test Author\nPlayResX: 1920\nPlayResY: 1080\nWrapStyle: 2\n";
    let script = Script::parse(input);
    assert!(!script.sections().is_empty());
}

#[test]
fn test_v4_styles_parsing() {
    let input = b"[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\nStyle: Default,Arial,32,&H00FFFFFF,&H000000FF,&H00000000,&H64000000,-1,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1\n";
    let script = Script::parse(input);
    assert!(!script.sections().is_empty());
}

#[test]
fn test_events_parsing() {
    let input = b"[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\nDialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,Hello World!\nComment: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,This is a comment\n";
    let script = Script::parse(input);
    assert!(!script.sections().is_empty());
}

#[test]
fn test_multiple_sections() {
    let input = b"[Script Info]\nTitle: Multi Section Test\n\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize\nStyle: Default,Arial,32\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\nDialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,Test\n";
    let script = Script::parse(input);
    assert!(script.sections().len() >= 3);
}

#[test]
fn test_comments_and_empty_lines() {
    let input = b"; This is a comment\n[Script Info]\n; Another comment\nTitle: Test\n\n; Empty line above\n[Events]\nDialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,Test\n";
    let script = Script::parse(input);
    assert!(!script.sections().is_empty());
}

#[test]
fn test_unicode_content() {
    let input = "[Script Info]\nTitle: Unicode Test\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\nDialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,こんにちは世界！\nDialogue: 0,0:00:05.00,0:00:10.00,Default,,0,0,0,,Привет мир!\nDialogue: 0,0:00:10.00,0:00:15.00,Default,,0,0,0,,🌍🚀✨\n".as_bytes();
    let script = Script::parse(input);
    assert!(!script.sections().is_empty());
}

#[test]
fn test_malformed_timestamps() {
    let input = b"[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\nDialogue: 0,invalid,0:00:05.00,Default,,0,0,0,,Test\nDialogue: 0,0:00:01.00,malformed,Default,,0,0,0,,Test2\n";
    let script = Script::parse(input);
    // Should still parse but handle malformed timestamps gracefully
    assert!(!script.sections().is_empty());
}

#[test]
fn test_tokenizer_basic() {
    let text = "Hello {\\b1}Bold{/}\\i0 World";
    let mut tok = AssTokenizer::new(text.as_bytes());
    for tok in &mut tok {
        // iterate through tokens to ensure tokenizer progresses
        let _ = tok;
    }
}

#[test]
fn test_tokenizer_complex_tags() {
    let text =
        "{\\b1\\i1\\u1\\s1}All formatting{\\r}{\\c&HFF0000&}Red text{\\c&H00FF00&}Green{\\r}";
    let mut tok = AssTokenizer::new(text.as_bytes());
    let mut token_count = 0;
    for _tok in &mut tok {
        token_count += 1;
        // Prevent infinite loops in tests
        assert!(
            token_count < 1000,
            "Tokenizer appears to be in infinite loop"
        );
    }
    assert!(token_count > 0, "Should tokenize at least some tokens");
}

#[test]
fn test_tokenizer_nested_tags() {
    let text = "{\\b1}Bold {\\i1}and italic{\\i0} still bold{\\b0}";
    let mut tok = AssTokenizer::new(text.as_bytes());
    let mut token_count = 0;
    for _tok in &mut tok {
        token_count += 1;
        assert!(
            token_count < 1000,
            "Tokenizer appears to be in infinite loop"
        );
    }
    assert!(token_count > 0);
}

#[test]
fn test_tokenizer_positioning_tags() {
    let text = "{\\pos(100,200)}{\\move(100,200,300,400,0,1000)}Moving text";
    let mut tok = AssTokenizer::new(text.as_bytes());
    let mut token_count = 0;
    for _tok in &mut tok {
        token_count += 1;
        assert!(
            token_count < 1000,
            "Tokenizer appears to be in infinite loop"
        );
    }
    assert!(token_count > 0);
}

#[test]
fn test_tokenizer_color_tags() {
    let text = "{\\c&HFF0000&}Red{\\c&H00FF00&}Green{\\c&H0000FF&}Blue{\\c}Default";
    let mut tok = AssTokenizer::new(text.as_bytes());
    let mut token_count = 0;
    for _tok in &mut tok {
        token_count += 1;
        assert!(
            token_count < 1000,
            "Tokenizer appears to be in infinite loop"
        );
    }
    assert!(token_count > 0);
}

#[test]
fn test_tokenizer_animation_tags() {
    let text = "{\\fad(500,1000)}{\\t(0,1000,\\frz360)}Animated text";
    let mut tok = AssTokenizer::new(text.as_bytes());
    let mut token_count = 0;
    for _tok in &mut tok {
        token_count += 1;
        assert!(
            token_count < 1000,
            "Tokenizer appears to be in infinite loop"
        );
    }
    assert!(token_count > 0);
}

#[test]
fn test_tokenizer_escape_sequences() {
    let text = "\\N\\h\\n{\\\\}Literal backslash and braces \\{\\}";
    let mut tok = AssTokenizer::new(text.as_bytes());
    let mut token_count = 0;
    for _tok in &mut tok {
        token_count += 1;
        assert!(
            token_count < 1000,
            "Tokenizer appears to be in infinite loop"
        );
    }
    assert!(token_count > 0);
}

#[test]
fn test_tokenizer_empty_input() {
    let text = "";
    let mut tok = AssTokenizer::new(text.as_bytes());
    assert!(tok.next().is_none());
}

#[test]
fn test_tokenizer_only_text() {
    let text = "Just plain text with no tags";
    let mut tok = AssTokenizer::new(text.as_bytes());
    let mut token_count = 0;
    for _tok in &mut tok {
        token_count += 1;
        assert!(
            token_count < 1000,
            "Tokenizer appears to be in infinite loop"
        );
    }
    assert!(token_count > 0);
}

#[test]
fn test_tokenizer_malformed_tags() {
    let text = "{\\incomplete {\\b1}Good{\\b0} {incomplete_tag";
    let mut tok = AssTokenizer::new(text.as_bytes());
    let mut token_count = 0;
    for _tok in &mut tok {
        token_count += 1;
        assert!(
            token_count < 1000,
            "Tokenizer appears to be in infinite loop"
        );
    }
    // Should handle malformed tags gracefully
    assert!(token_count > 0);
}
