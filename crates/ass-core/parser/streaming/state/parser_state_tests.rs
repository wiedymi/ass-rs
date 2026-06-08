//! Unit tests for [`ParserState`] transitions and trait implementations.

use super::*;
#[cfg(not(feature = "std"))]
use alloc::format;

#[test]
fn parser_state_transitions() {
    let mut state = ParserState::ExpectingSection;
    assert!(!state.is_in_section());
    assert_eq!(state.current_section(), None);

    state.enter_section(SectionKind::Events);
    assert!(state.is_in_section());
    assert_eq!(state.current_section(), Some(SectionKind::Events));

    state.enter_event(SectionKind::Events);
    assert!(state.is_in_section());
    assert_eq!(state.current_section(), Some(SectionKind::Events));

    state.exit_section();
    assert!(!state.is_in_section());
    assert_eq!(state.current_section(), None);
}

#[test]
fn parser_state_debug_and_clone() {
    let state = ParserState::ExpectingSection;
    let debug_str = format!("{state:?}");
    assert!(debug_str.contains("ExpectingSection"));

    let cloned = state.clone();
    assert_eq!(state, cloned);

    let section_state = ParserState::InSection(SectionKind::Events);
    let section_debug = format!("{section_state:?}");
    assert!(section_debug.contains("InSection"));
    assert!(section_debug.contains("Events"));

    let event_state = ParserState::InEvent {
        section: SectionKind::Events,
        fields_seen: 3,
    };
    let event_debug = format!("{event_state:?}");
    assert!(event_debug.contains("InEvent"));
    assert!(event_debug.contains("fields_seen"));
}

#[test]
fn parser_state_equality() {
    let state1 = ParserState::ExpectingSection;
    let state2 = ParserState::ExpectingSection;
    assert_eq!(state1, state2);

    let state3 = ParserState::InSection(SectionKind::Events);
    let state4 = ParserState::InSection(SectionKind::Events);
    assert_eq!(state3, state4);

    let state5 = ParserState::InEvent {
        section: SectionKind::Events,
        fields_seen: 2,
    };
    let state6 = ParserState::InEvent {
        section: SectionKind::Events,
        fields_seen: 2,
    };
    assert_eq!(state5, state6);

    // Test inequality
    assert_ne!(state1, state3);
    assert_ne!(state3, state5);

    let state7 = ParserState::InEvent {
        section: SectionKind::Events,
        fields_seen: 3,
    };
    assert_ne!(state5, state7);
}

#[test]
fn parser_state_all_variants() {
    // Test ExpectingSection
    let expecting = ParserState::ExpectingSection;
    assert!(!expecting.is_in_section());
    assert_eq!(expecting.current_section(), None);

    // Test InSection for all section kinds
    for &kind in &[
        SectionKind::ScriptInfo,
        SectionKind::Styles,
        SectionKind::Events,
        SectionKind::Fonts,
        SectionKind::Graphics,
        SectionKind::Unknown,
    ] {
        let in_section = ParserState::InSection(kind);
        assert!(in_section.is_in_section());
        assert_eq!(in_section.current_section(), Some(kind));
    }

    // Test InEvent
    let in_event = ParserState::InEvent {
        section: SectionKind::Events,
        fields_seen: 5,
    };
    assert!(in_event.is_in_section());
    assert_eq!(in_event.current_section(), Some(SectionKind::Events));
}

#[test]
fn parser_state_transition_sequences() {
    let mut state = ParserState::ExpectingSection;

    // Test complete transition sequence
    state.enter_section(SectionKind::Events);
    assert!(state.is_in_section());
    assert_eq!(state.current_section(), Some(SectionKind::Events));

    state.enter_event(SectionKind::Events);
    assert!(state.is_in_section());
    assert_eq!(state.current_section(), Some(SectionKind::Events));

    state.exit_section();
    assert!(!state.is_in_section());
    assert_eq!(state.current_section(), None);

    // Test direct event entry
    state.enter_event(SectionKind::Styles);
    assert!(state.is_in_section());
    assert_eq!(state.current_section(), Some(SectionKind::Styles));
}
