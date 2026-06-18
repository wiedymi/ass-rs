//! Parsed timing index used by the event selector for incremental rendering.

use ass_core::parser::{ast::EventType, Event};

#[cfg(feature = "nostd")]
use alloc::vec::Vec;

/// Parsed, start-sorted timing index for a script's events.
///
/// Keyed by the events `Vec`'s address, length, and the comment-rendering flag,
/// so it is reused across frames and rebuilt only when the script (or that flag)
/// changes. Each entry is `(start_cs, end_cs, original_index)`; the original
/// index preserves file-order rendering when active events are emitted.
#[derive(Debug, Clone)]
pub(super) struct TimeIndex {
    pub(super) key: (usize, usize, bool),
    pub(super) by_start: Vec<(u32, u32, usize)>,
}

impl TimeIndex {
    /// Build the parsed time index for `events`.
    ///
    /// Each included event's start/end time string is parsed once here, and the
    /// resulting entries are sorted by start time so the selector can bound its
    /// per-frame scan with `partition_point`. `Dialogue` events are always
    /// included; `Comment` events only when `render_comments` is set.
    pub(super) fn build(events: &[Event], render_comments: bool) -> Self {
        let key = (events.as_ptr() as usize, events.len(), render_comments);

        let mut by_start = Vec::new();
        for (idx, event) in events.iter().enumerate() {
            let should_include = match event.event_type {
                EventType::Dialogue => true,
                EventType::Comment => render_comments,
                _ => false,
            };
            if should_include {
                let start = event.start_time_cs().unwrap_or(0);
                let end = event.end_time_cs().unwrap_or(0);
                by_start.push((start, end, idx));
            }
        }
        by_start.sort_unstable_by_key(|&(start, _, _)| start);

        Self { key, by_start }
    }
}
