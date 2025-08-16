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
}

/// A region that needs re-rendering
#[derive(Debug, Clone)]
pub struct DirtyRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
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
            render_comments: true, // Enable by default for compatibility with sign rendering
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

        // Find all active events
        if let Some(events_section) = script.sections().iter().find_map(|section| {
            if let Section::Events(events) = section {
                Some(events)
            } else {
                None
            }
        }) {
            #[cfg(all(debug_assertions, not(feature = "nostd")))]
            eprintln!("EventSelector: Checking {} events at time {}", events_section.iter().count(), time_cs);
            for (idx, event) in events_section.iter().enumerate() {
                // Include both Dialogue and optionally Comment events
                let should_include = match event.event_type {
                    EventType::Dialogue => true,
                    EventType::Comment => self.render_comments,
                    _ => false,
                };

                if should_include {
                    let start = event.start_time_cs().unwrap_or(0);
                    let end = event.end_time_cs().unwrap_or(0);
                    
                    // Debug output for fade events
                    #[cfg(all(debug_assertions, not(feature = "nostd")))]
                    if event.text.contains("\\fad") {
                        eprintln!("EventSelector: Fade event - start={}cs, end={}cs, time_cs={}cs, active={}", 
                            start, end, time_cs, start <= time_cs && end >= time_cs);
                    }
                    
                    // Debug output for the specific dialogue we're looking for
                    #[cfg(all(debug_assertions, not(feature = "nostd")))]
                    if event.text.contains("Чысценькая") {
                        eprintln!("Found target dialogue: start={}, end={}, current_time={}, active={}", 
                            start, end, time_cs, start <= time_cs && end >= time_cs);
                    }
                    
                    // Events are active from start time (inclusive) to end time (inclusive)
                    // This matches ASS/SSA specification and libass behavior
                    if start <= time_cs && end >= time_cs {
                        active_events.push(event);
                        current_active.insert(idx);
                    }
                }
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
                .map_or(true, |last| (time_cs as i32 - last as i32).abs() > 100);

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
pub fn select_active_events<'a>(script: &'a Script<'a>, time_cs: u32) -> Vec<&'a Event<'a>> {
    let mut selector = EventSelector::new();
    selector
        .select_active(script, time_cs)
        .map(|active| active.events)
        .unwrap_or_default()
}
