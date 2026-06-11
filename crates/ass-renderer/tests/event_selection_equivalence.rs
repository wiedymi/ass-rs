//! Equivalence guard for the start-sorted time index in `EventSelector`.
//!
//! The index replaced an O(n) per-frame scan that re-parsed every event's
//! start/end time string. These tests pin the index to the exact semantics of
//! that brute-force scan (`start <= t <= end`, file-order output, Dialogue
//! always / Comment gated by `render_comments`) across a dense sweep of
//! timestamps and an adversarial event layout: out-of-order starts, overlaps,
//! inclusive boundaries, zero-duration events, millisecond and centisecond
//! precision, and interleaved comments.

use ass_core::parser::ast::EventType;
use ass_core::parser::{Event, Script, Section};
use ass_renderer::renderer::EventSelector;

/// An adversarial script: events are deliberately NOT in start-time order, so a
/// correct index must sort by start yet restore file order on output.
const ADVERSARIAL: &str = "\
[Script Info]
ScriptType: v4.00+

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,40,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:03.00,Default,,0,0,0,,A overlaps
Dialogue: 0,0:00:00.50,0:00:02.00,Default,,0,0,0,,B starts first listed second
Comment: 0,0:00:01.50,0:00:04.00,Default,,0,0,0,,C comment sign
Dialogue: 0,0:00:03.00,0:00:03.00,Default,,0,0,0,,D zero duration at boundary
Dialogue: 0,0:00:02.000,0:00:05.000,Default,,0,0,0,,E millisecond precision
Dialogue: 0,0:00:00.00,0:00:00.00,Default,,0,0,0,,F only at zero
Dialogue: 0,0:00:04.00,0:00:04.50,Default,,0,0,0,,G late
";

/// Brute-force reference: the exact logic the index replaced.
fn brute_force<'a>(script: &'a Script<'a>, t: u32, render_comments: bool) -> Vec<*const Event<'a>> {
    let mut out = Vec::new();
    if let Some(Section::Events(events)) = script
        .sections()
        .iter()
        .find(|s| matches!(s, Section::Events(_)))
    {
        for event in events.iter() {
            let include = match event.event_type {
                EventType::Dialogue => true,
                EventType::Comment => render_comments,
                _ => false,
            };
            if include {
                let start = event.start_time_cs().unwrap_or(0);
                let end = event.end_time_cs().unwrap_or(0);
                if start <= t && end >= t {
                    out.push(core::ptr::from_ref(event));
                }
            }
        }
    }
    out
}

fn selected<'a>(
    selector: &mut EventSelector,
    script: &'a Script<'a>,
    t: u32,
) -> Vec<*const Event<'a>> {
    selector
        .select_active(script, t)
        .expect("select_active")
        .events
        .iter()
        .map(|e| core::ptr::from_ref::<Event>(e))
        .collect()
}

#[test]
fn index_matches_brute_force_across_sweep() {
    let script = Script::parse(ADVERSARIAL).expect("parse");
    let mut selector = EventSelector::new(); // render_comments = true by default

    // Dense sweep past the last end time, hitting every inclusive boundary.
    for t in 0..=520u32 {
        let expected = brute_force(&script, t, true);
        let got = selected(&mut selector, &script, t);
        assert_eq!(
            got, expected,
            "active set mismatch at t={t}cs (comments on)"
        );
    }
}

#[test]
fn index_respects_render_comments_toggle() {
    let script = Script::parse(ADVERSARIAL).expect("parse");
    let mut selector = EventSelector::new();
    selector.set_render_comments(false);

    // Toggling the flag must rebuild the index (it is part of the cache key), so
    // the comment event C must now be excluded at every timestamp.
    for t in 0..=520u32 {
        let expected = brute_force(&script, t, false);
        let got = selected(&mut selector, &script, t);
        assert_eq!(
            got, expected,
            "active set mismatch at t={t}cs (comments off)"
        );
    }
}

#[test]
fn boundaries_are_inclusive_on_both_ends() {
    let script = Script::parse(ADVERSARIAL).expect("parse");
    let mut selector = EventSelector::new();

    // Event A is 100..=300cs. It must be active at both endpoints and inactive
    // one unit outside each.
    let text_at = |sel: &mut EventSelector, t: u32| -> Vec<String> {
        sel.select_active(&script, t)
            .unwrap()
            .events
            .iter()
            .map(|e| e.text.to_string())
            .collect()
    };

    assert!(text_at(&mut selector, 99)
        .iter()
        .all(|s| !s.starts_with("A ")));
    assert!(text_at(&mut selector, 100)
        .iter()
        .any(|s| s.starts_with("A ")));
    assert!(text_at(&mut selector, 300)
        .iter()
        .any(|s| s.starts_with("A ")));
    assert!(text_at(&mut selector, 301)
        .iter()
        .all(|s| !s.starts_with("A ")));

    // Zero-duration event D (300..=300) and only-at-zero event F (0..=0).
    assert!(text_at(&mut selector, 300)
        .iter()
        .any(|s| s.starts_with("D ")));
    assert!(text_at(&mut selector, 0)
        .iter()
        .any(|s| s.starts_with("F ")));
    assert!(text_at(&mut selector, 1)
        .iter()
        .all(|s| !s.starts_with("F ")));
}
