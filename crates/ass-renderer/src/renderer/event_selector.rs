//! Event selection and dirty region tracking for incremental rendering

use crate::utils::RenderError;
use ass_core::parser::{ast::EventType, Event, Script, Section};

#[cfg(feature = "nostd")]
use alloc::{collections::BTreeSet, vec::Vec};
#[cfg(not(feature = "nostd"))]
use std::collections::HashSet;

/// Tracks active events and dirty regions for optimized rendering
#[derive(Debug, Clone)]
pub struct EventSelector {
    /// Cache of previously active event indices
    #[cfg(not(feature = "nostd"))]
    previous_active: HashSet<usize>,
    #[cfg(feature = "nostd")]
    previous_active: BTreeSet<usize>,

    /// Last rendered timestamp
    last_timestamp: Option<u32>,

    /// Dirty regions that need re-rendering
    dirty_regions: Vec<DirtyRegion>,

    /// Whether to render comment events (for signs and complex effects)
    render_comments: bool,

    /// Cached, parsed time index for the current script. Rebuilt only when the
    /// events change. Avoids re-parsing every event's start/end time string on
    /// every frame (the dominant per-frame cost for large scripts).
    time_index: Option<TimeIndex>,
}

/// Parsed, start-sorted timing index for a script's events.
///
/// Keyed by the events `Vec`'s address, length, and the comment-rendering flag,
/// so it is reused across frames and rebuilt only when the script (or that flag)
/// changes. Each entry is `(start_cs, end_cs, original_index)`; the original
/// index preserves file-order rendering when active events are emitted.
#[derive(Debug, Clone)]
struct TimeIndex {
    key: (usize, usize, bool),
    by_start: Vec<(u32, u32, usize)>,
}

/// A region that needs re-rendering
#[derive(Debug, Clone)]
pub struct DirtyRegion {
    /// X coordinate of the region
    pub x: u32,
    /// Y coordinate of the region
    pub y: u32,
    /// Width of the region
    pub width: u32,
    /// Height of the region
    pub height: u32,
}

/// Result of event selection with dirty tracking
#[derive(Debug)]
pub struct ActiveEvents<'a> {
    /// Currently active events
    pub events: Vec<&'a Event<'a>>,
    /// Indices of newly activated events
    pub newly_active: Vec<usize>,
    /// Indices of newly deactivated events
    pub newly_inactive: Vec<usize>,
    /// Whether the frame needs re-rendering
    pub is_dirty: bool,
}

impl EventSelector {
    /// Create a new event selector
    pub fn new() -> Self {
        Self {
            #[cfg(not(feature = "nostd"))]
            previous_active: HashSet::new(),
            #[cfg(feature = "nostd")]
            previous_active: BTreeSet::new(),
            last_timestamp: None,
            dirty_regions: Vec::new(),
            // libass never renders `Comment` events, so default to matching it.
            // `Comment` lines in real scripts hold source text, karaoke templates
            // and disabled alternates — rendering them duplicates/overlaps the
            // real `Dialogue` lines. Opt in via `set_render_comments(true)`.
            render_comments: false,
            time_index: None,
        }
    }

    /// Set whether to render comment events
    pub fn set_render_comments(&mut self, render: bool) {
        self.render_comments = render;
    }

    /// Select active events and track changes for incremental rendering
    pub fn select_active<'a>(
        &mut self,
        script: &'a Script<'a>,
        time_cs: u32,
    ) -> Result<ActiveEvents<'a>, RenderError> {
        let mut active_events = Vec::new();
        #[cfg(not(feature = "nostd"))]
        let mut current_active = HashSet::new();
        #[cfg(feature = "nostd")]
        let mut current_active = BTreeSet::new();

        // Find the events section, then answer the query from the cached, parsed
        // time index. Active events satisfy `start <= t <= end`; entries are
        // start-sorted, so `partition_point` bounds the scan to events that have
        // already started, and the original index restores file order.
        if let Some(events_section) = script.sections().iter().find_map(|section| {
            if let Section::Events(events) = section {
                Some(events)
            } else {
                None
            }
        }) {
            self.ensure_index(events_section);
            let index = self
                .time_index
                .as_ref()
                .expect("time index built by ensure_index");
            let hi = index
                .by_start
                .partition_point(|&(start, _, _)| start <= time_cs);
            let mut active_idx: Vec<usize> = index.by_start[..hi]
                .iter()
                .filter(|&&(_, end, _)| end >= time_cs)
                .map(|&(_, _, idx)| idx)
                .collect();
            active_idx.sort_unstable();
            for idx in active_idx {
                active_events.push(&events_section[idx]);
                current_active.insert(idx);
            }
        }

        // Track changes for incremental rendering
        let newly_active: Vec<usize> = current_active
            .iter()
            .filter(|idx| !self.previous_active.contains(idx))
            .cloned()
            .collect();

        let newly_inactive: Vec<usize> = self
            .previous_active
            .iter()
            .filter(|idx| !current_active.contains(idx))
            .cloned()
            .collect();

        // Check if re-render is needed
        let is_dirty = !newly_active.is_empty()
            || !newly_inactive.is_empty()
            || self.has_animated_events(&active_events, time_cs)
            || self
                .last_timestamp
                .is_none_or(|last| (time_cs as i32 - last as i32).abs() > 100);

        // Update state
        self.previous_active = current_active;
        self.last_timestamp = Some(time_cs);

        Ok(ActiveEvents {
            events: active_events,
            newly_active,
            newly_inactive,
            is_dirty,
        })
    }

    /// Build (or reuse) the parsed time index for `events`.
    ///
    /// The index is keyed by the events slice's address, length, and the current
    /// comment-rendering flag; when that key is unchanged the existing index is
    /// kept, so each event's start/end time string is parsed only once per script
    /// rather than on every frame.
    fn ensure_index(&mut self, events: &[Event]) {
        let key = (events.as_ptr() as usize, events.len(), self.render_comments);
        if self
            .time_index
            .as_ref()
            .is_some_and(|index| index.key == key)
        {
            return;
        }

        let mut by_start = Vec::new();
        for (idx, event) in events.iter().enumerate() {
            let should_include = match event.event_type {
                EventType::Dialogue => true,
                EventType::Comment => self.render_comments,
                _ => false,
            };
            if should_include {
                let start = event.start_time_cs().unwrap_or(0);
                let end = event.end_time_cs().unwrap_or(0);
                by_start.push((start, end, idx));
            }
        }
        by_start.sort_unstable_by_key(|&(start, _, _)| start);

        self.time_index = Some(TimeIndex { key, by_start });
    }

    /// Check if any events have active animations
    fn has_animated_events(&self, events: &[&Event], time_cs: u32) -> bool {
        for event in events {
            let text = event.text;
            // Check for animation tags
            if text.contains(r"\t(")
                || text.contains(r"\move(")
                || text.contains(r"\fade(")
                || text.contains(r"\fad(")
            {
                return true;
            }
            // Check for karaoke
            if text.contains(r"\k") || text.contains(r"\K") {
                if let Ok(start) = event.start_time_cs() {
                    if time_cs > start {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Add a dirty region for partial re-rendering
    pub fn add_dirty_region(&mut self, x: u32, y: u32, width: u32, height: u32) {
        // Merge overlapping regions for efficiency
        for region in &mut self.dirty_regions {
            if Self::regions_overlap(region, x, y, width, height) {
                // Expand existing region
                let min_x = region.x.min(x);
                let min_y = region.y.min(y);
                let max_x = (region.x + region.width).max(x + width);
                let max_y = (region.y + region.height).max(y + height);
                region.x = min_x;
                region.y = min_y;
                region.width = max_x - min_x;
                region.height = max_y - min_y;
                return;
            }
        }

        self.dirty_regions.push(DirtyRegion {
            x,
            y,
            width,
            height,
        });
    }

    /// Check if two regions overlap
    fn regions_overlap(region: &DirtyRegion, x: u32, y: u32, width: u32, height: u32) -> bool {
        !(x >= region.x + region.width
            || x + width <= region.x
            || y >= region.y + region.height
            || y + height <= region.y)
    }

    /// Get current dirty regions
    pub fn dirty_regions(&self) -> &[DirtyRegion] {
        &self.dirty_regions
    }

    /// Clear dirty regions after rendering
    pub fn clear_dirty_regions(&mut self) {
        self.dirty_regions.clear();
    }

    /// Reset selector state
    pub fn reset(&mut self) {
        self.previous_active.clear();
        self.last_timestamp = None;
        self.dirty_regions.clear();
    }
}

impl Default for EventSelector {
    fn default() -> Self {
        Self::new()
    }
}

/// Legacy function for backward compatibility
#[allow(dead_code)] // Kept for backward compatibility
pub fn select_active_events<'a>(script: &'a Script<'a>, time_cs: u32) -> Vec<&'a Event<'a>> {
    let mut selector = EventSelector::new();
    selector
        .select_active(script, time_cs)
        .map(|active| active.events)
        .unwrap_or_default()
}
