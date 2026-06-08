//! Tests for `ScriptGenerator` construction, time formatting, and dialogue
//! text generation across the core complexity levels.

use super::*;

#[test]
fn script_generator_simple() {
    let generator = ScriptGenerator::simple(5);
    assert_eq!(generator.events_count, 5);
    assert_eq!(generator.styles_count, 1);
    assert!(matches!(
        generator.complexity_level,
        ComplexityLevel::Simple
    ));

    let script = generator.generate();
    assert!(script.contains("[Script Info]"));
    assert!(script.contains("[V4+ Styles]"));
    assert!(script.contains("[Events]"));
    assert!(script.contains("Simple Benchmark Script"));
}

#[test]
fn script_generator_moderate() {
    let generator = ScriptGenerator::moderate(3);
    assert_eq!(generator.events_count, 3);
    assert_eq!(generator.styles_count, 5);
    assert!(matches!(
        generator.complexity_level,
        ComplexityLevel::Moderate
    ));
}

#[test]
fn script_generator_complex() {
    let generator = ScriptGenerator::complex(2);
    assert_eq!(generator.events_count, 2);
    assert_eq!(generator.styles_count, 10);
    assert!(matches!(
        generator.complexity_level,
        ComplexityLevel::Complex
    ));
}

#[test]
fn script_generator_extreme() {
    let generator = ScriptGenerator::extreme(1);
    assert_eq!(generator.events_count, 1);
    assert_eq!(generator.styles_count, 20);
    assert!(matches!(
        generator.complexity_level,
        ComplexityLevel::Extreme
    ));
}

#[test]
fn format_time_zero() {
    assert_eq!(ScriptGenerator::format_time(0), "0:00:00.00");
}

#[test]
fn format_time_basic() {
    assert_eq!(ScriptGenerator::format_time(6150), "0:01:01.50");
}

#[test]
fn format_time_hours() {
    assert_eq!(ScriptGenerator::format_time(360_000), "1:00:00.00");
}

#[test]
fn dialogue_text_complexity_simple() {
    let generator = ScriptGenerator::simple(1);
    let text = generator.generate_dialogue_text(0);
    assert_eq!(text, "This is dialogue line number 1");
}

#[test]
fn dialogue_text_complexity_moderate() {
    let generator = ScriptGenerator::moderate(1);
    let text = generator.generate_dialogue_text(0);
    assert!(text.contains(r"{\b1}"));
    assert!(text.contains(r"{\i1}"));
    assert!(text.contains("This is dialogue line number 1"));
}

#[test]
fn dialogue_text_complexity_complex() {
    let generator = ScriptGenerator::complex(1);
    let text = generator.generate_dialogue_text(0);
    assert!(text.contains(r"{\pos("));
    assert!(text.contains(r"{\t("));
    assert!(text.contains("animation"));
}

#[test]
fn dialogue_text_complexity_extreme() {
    let generator = ScriptGenerator::extreme(1);
    let text = generator.generate_dialogue_text(0);
    assert!(text.contains(r"{\k"));
    assert!(text.contains("karaoke"));
    assert!(text.contains("animations"));
}

#[test]
fn script_generator_generate_has_correct_event_count() {
    let generator = ScriptGenerator::simple(3);
    let script = generator.generate();
    assert_eq!(
        script
            .lines()
            .filter(|line| line.starts_with("Dialogue:"))
            .count(),
        3
    );
}

#[test]
fn script_generator_generate_has_correct_style_count() {
    let generator = ScriptGenerator::moderate(1); // 5 styles
    let script = generator.generate();
    assert_eq!(
        script
            .lines()
            .filter(|line| line.starts_with("Style:"))
            .count(),
        5
    );
}
