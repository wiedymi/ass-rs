//! Collision resolvers for subtitle positioning

#[cfg(feature = "nostd")]
use alloc::vec::Vec;
#[cfg(not(feature = "nostd"))]
use std::vec::Vec;

use super::{BoundingBox, CollisionDetector, PositionedEvent};

/// Collision resolver for subtitle positioning
pub struct CollisionResolver {
    #[allow(dead_code)] // Used in constructor, may be needed for future collision algorithms
    screen_width: f32,
    screen_height: f32,
    positioned_events: Vec<PositionedEvent>,
    collision_margin: f32,
}

impl CollisionResolver {
    /// Create a new collision resolver
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        Self {
            screen_width,
            screen_height,
            positioned_events: Vec::new(),
            collision_margin: 2.0, // Default 2px margin between subtitles
        }
    }

    /// Clear all positioned events
    pub fn clear(&mut self) {
        self.positioned_events.clear();
    }

    /// Set collision margin
    pub fn set_collision_margin(&mut self, margin: f32) {
        self.collision_margin = margin;
    }

    /// Add a positioned event that won't be moved
    pub fn add_fixed(&mut self, event: PositionedEvent) {
        self.positioned_events.push(event);
    }

    /// Resolve collisions by stacking the event away from its alignment margin:
    /// bottom-aligned events (1-3) move up, top/middle (4-9) move down, until the
    /// event no longer overlaps any already-placed same-layer event. This matches
    /// libass "Normal" collisions, where earlier events keep the margin position
    /// and later events stack past them. The resolved box is recorded so that
    /// subsequent events stack against it.
    pub fn find_position(&mut self, mut event: PositionedEvent) -> BoundingBox {
        let margin = self.collision_margin;
        let push_down = !matches!(event.alignment, 1..=3);
        let mut bbox = event.bbox;

        // Each iteration clears at least one overlap, so the placed-event count
        // bounds the number of repositions needed.
        for _ in 0..=self.positioned_events.len() {
            let overlapping: Vec<BoundingBox> = self
                .positioned_events
                .iter()
                .filter(|e| {
                    e.layer == event.layer && e.bbox.expand(margin).intersects(&bbox.expand(margin))
                })
                .map(|e| e.bbox)
                .collect();
            if overlapping.is_empty() {
                break;
            }
            bbox.y = if push_down {
                overlapping
                    .iter()
                    .map(|b| b.y + b.height)
                    .fold(f32::MIN, f32::max)
                    + margin * 2.0
            } else {
                overlapping.iter().map(|b| b.y).fold(f32::MAX, f32::min)
                    - bbox.height
                    - margin * 2.0
            };
        }

        // Keep the event on-screen if the stack would overflow the frame edge.
        let max_y = (self.screen_height - bbox.height).max(0.0);
        bbox.y = bbox.y.clamp(0.0, max_y);

        event.bbox = bbox;
        self.positioned_events.push(event);
        bbox
    }

    /// Get all positioned events
    pub fn positioned_events(&self) -> &[PositionedEvent] {
        &self.positioned_events
    }
}

/// Smart collision system with priority-based resolution
pub struct SmartCollisionResolver {
    resolver: CollisionResolver,
    priority_threshold: i32,
}

impl SmartCollisionResolver {
    /// Create a new smart collision resolver
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        Self {
            resolver: CollisionResolver::new(screen_width, screen_height),
            priority_threshold: 100,
        }
    }

    /// Process events with smart collision resolution
    pub fn process_events(&mut self, events: &mut [PositionedEvent]) {
        // Sort by priority (lower priority = can be moved)
        events.sort_by_key(|e| -e.priority);

        self.resolver.clear();

        for event in events.iter_mut() {
            if event.priority >= self.priority_threshold {
                // High priority - don't move
                self.resolver.add_fixed(event.clone());
            } else {
                // Low priority - can be repositioned
                let new_bbox = self.resolver.find_position(event.clone());
                event.bbox = new_bbox;
            }
        }
    }

    /// Set the priority threshold
    pub fn set_priority_threshold(&mut self, threshold: i32) {
        self.priority_threshold = threshold;
    }
}

impl CollisionDetector for SmartCollisionResolver {
    fn check_collision(&self, bbox: &BoundingBox, existing: &[BoundingBox]) -> bool {
        existing.iter().any(|e| bbox.intersects(e))
    }

    fn find_free_position(
        &self,
        bbox: &BoundingBox,
        existing: &[BoundingBox],
        bounds: &BoundingBox,
    ) -> Option<(f32, f32)> {
        // Try multiple positions to find a free spot
        let step = 10.0; // Search step size

        // First try moving down
        for y_offset in (0..20).map(|i| i as f32 * step) {
            let test_y = bbox.y + y_offset;
            if test_y + bbox.height <= bounds.height {
                let test_bbox = BoundingBox::new(bbox.x, test_y, bbox.width, bbox.height);
                if !self.check_collision(&test_bbox, existing) {
                    return Some((bbox.x, test_y));
                }
            }
        }

        // Then try moving up
        for y_offset in (1..20).map(|i| i as f32 * step) {
            let test_y = bbox.y - y_offset;
            if test_y >= 0.0 {
                let test_bbox = BoundingBox::new(bbox.x, test_y, bbox.width, bbox.height);
                if !self.check_collision(&test_bbox, existing) {
                    return Some((bbox.x, test_y));
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collision_resolver() {
        let mut resolver = CollisionResolver::new(1920.0, 1080.0);

        let event1 = PositionedEvent {
            bbox: BoundingBox::new(100.0, 900.0, 200.0, 50.0),
            layer: 0,
            margin_v: 10,
            margin_l: 10,
            margin_r: 10,
            alignment: 2,
            priority: 100,
        };

        let event2 = PositionedEvent {
            bbox: BoundingBox::new(100.0, 900.0, 200.0, 50.0),
            layer: 0,
            margin_v: 10,
            margin_l: 10,
            margin_r: 10,
            alignment: 2,
            priority: 50,
        };

        resolver.add_fixed(event1);
        let new_pos = resolver.find_position(event2);

        // Second event should be moved due to collision
        assert_ne!(new_pos.y, 900.0);
    }
}
