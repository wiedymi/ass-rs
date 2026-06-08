//! Tests for section range/offset/boundary queries and change-tracking toggles.

use super::*;
use crate::parser::ast::{Section, SectionType};

#[test]
fn test_section_range() {
    let content = "[Script Info]\nTitle: Test\n\n[V4+ Styles]\nFormat: Name, Fontname\nStyle: Default,Arial\n\n[Events]\nFormat: Layer, Start, End, Style, Text\nDialogue: 0,0:00:00.00,0:00:05.00,Default,Hello";
    let script = Script::parse(content).unwrap();

    // Test existing section
    let script_info_range = script.section_range(SectionType::ScriptInfo);
    assert!(script_info_range.is_some());

    // Test non-existent section
    let fonts_range = script.section_range(SectionType::Fonts);
    assert!(fonts_range.is_none());

    // Verify ranges are reasonable
    if let Some(range) = script.section_range(SectionType::Events) {
        assert!(range.start < range.end);
        assert!(range.end <= content.len());
    }
}

#[test]
fn test_section_at_offset() {
    let content = "[Script Info]\nTitle: Test\n\n[V4+ Styles]\nFormat: Name, Fontname\nStyle: Default,Arial\n\n[Events]\nFormat: Layer, Start, End, Style, Text\nDialogue: 0,0:00:00.00,0:00:05.00,Default,Hello";
    let script = Script::parse(content).unwrap();

    // Find offset in Script Info section
    if let Some(section) = script.section_at_offset(15) {
        assert_eq!(section.section_type(), SectionType::ScriptInfo);
    }

    // Find offset in Events section
    if let Some(events_range) = script.section_range(SectionType::Events) {
        let offset_in_events = events_range.start + 10;
        if let Some(section) = script.section_at_offset(offset_in_events) {
            assert_eq!(section.section_type(), SectionType::Events);
        }
    }

    // Test offset outside any section
    let outside_offset = content.len() + 100;
    assert!(script.section_at_offset(outside_offset).is_none());
}

#[test]
fn test_section_boundaries() {
    let content = "[Script Info]\nTitle: Test\n\n[V4+ Styles]\nFormat: Name, Fontname\nStyle: Default,Arial\n\n[Events]\nFormat: Layer, Start, End, Style, Text\nDialogue: 0,0:00:00.00,0:00:05.00,Default,Hello";
    let script = Script::parse(content).unwrap();

    let boundaries = script.section_boundaries();

    // Should have boundaries for all parsed sections
    assert!(!boundaries.is_empty());

    // Verify each boundary
    for (section_type, range) in &boundaries {
        assert!(range.start < range.end);
        assert!(range.end <= content.len());

        // Verify section type matches
        if let Some(section) = script.find_section(*section_type) {
            if let Some(span) = section.span() {
                assert_eq!(range.start, span.start);
                assert_eq!(range.end, span.end);
            }
        }
    }

    // Check specific sections are present
    let has_script_info = boundaries
        .iter()
        .any(|(t, _)| *t == SectionType::ScriptInfo);
    let has_styles = boundaries.iter().any(|(t, _)| *t == SectionType::Styles);
    let has_events = boundaries.iter().any(|(t, _)| *t == SectionType::Events);

    assert!(has_script_info);
    assert!(has_styles);
    assert!(has_events);
}

#[test]
fn test_boundary_detection_empty_sections() {
    // Test with sections that might have no span
    let content = "[Script Info]\n\n[V4+ Styles]\n\n[Events]\n";
    let script = Script::parse(content).unwrap();

    let boundaries = script.section_boundaries();

    // Empty sections might not have spans
    // This test verifies we handle that gracefully
    for (_, range) in &boundaries {
        assert!(range.start <= range.end);
    }
}

#[test]
fn test_change_tracking_disabled_by_default() {
    let content = "[Script Info]\nTitle: Test";
    let script = Script::parse(content).unwrap();

    // Change tracking should be disabled by default
    assert!(!script.is_change_tracking_enabled());
    assert_eq!(script.change_count(), 0);
}

#[test]
fn test_enable_disable_change_tracking() {
    let content = "[Script Info]\nTitle: Test";
    let mut script = Script::parse(content).unwrap();

    // Enable tracking
    script.enable_change_tracking();
    assert!(script.is_change_tracking_enabled());

    // Disable tracking
    script.disable_change_tracking();
    assert!(!script.is_change_tracking_enabled());
}

#[test]
fn test_change_tracking_update_line() {
    let content = "[Script Info]\nTitle: Test\n\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize\nStyle: Default,Arial,20";
    let mut script = Script::parse(content).unwrap();

    // Enable tracking
    script.enable_change_tracking();

    // Find offset for update
    if let Some(Section::Styles(styles)) = script.find_section(SectionType::Styles) {
        let offset = styles[0].span.start;

        // Update the style
        let result = script.update_line_at_offset(offset, "Style: Default,Helvetica,24", 10);
        assert!(result.is_ok());

        // Check change was recorded
        assert_eq!(script.change_count(), 1);
        let changes = script.changes();
        assert_eq!(changes.len(), 1);

        if let Change::Modified {
            old_content,
            new_content,
            ..
        } = &changes[0]
        {
            if let (LineContent::Style(old_style), LineContent::Style(new_style)) =
                (old_content, new_content)
            {
                assert_eq!(old_style.fontname, "Arial");
                assert_eq!(old_style.fontsize, "20");
                assert_eq!(new_style.fontname, "Helvetica");
                assert_eq!(new_style.fontsize, "24");
            } else {
                panic!("Expected Style line content");
            }
        } else {
            panic!("Expected Modified change");
        }
    }
}
