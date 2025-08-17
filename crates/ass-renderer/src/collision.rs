//! Collision detection for subtitle positioning

#[cfg(feature = "nostd")]
use alloc::vec::Vec;
#[cfg(not(feature = "nostd"))]
use std::vec::Vec;

/// Trait for collision detection strategies
pub trait CollisionDetector: Send + Sync {
    /// Check if a bounding box collides with existing elements
    fn check_collision(&self, bbox: &BoundingBox, existing: &[BoundingBox]) -> bool;

    /// Find a non-colliding position for a bounding box
    fn find_free_position(
        &self,
        bbox: &BoundingBox,
        existing: &[BoundingBox],
        bounds: &BoundingBox,
    ) -> Option<(f32, f32)>;
}

/// Bounding box for collision detection
#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    /// X coordinate of the box
    pub x: f32,
    /// Y coordinate of the box
    pub y: f32,
    /// Width of the box
    pub width: f32,
    /// Height of the box
    pub height: f32,
}

impl BoundingBox {
    /// Create a new bounding box
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Check if this box intersects with another
    pub fn intersects(&self, other: &BoundingBox) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }

    /// Calculate the overlap area with another box
    pub fn overlap_area(&self, other: &BoundingBox) -> f32 {
        let x_overlap = (self.x + self.width).min(other.x + other.width) - self.x.max(other.x);
        let y_overlap = (self.y + self.height).min(other.y + other.height) - self.y.max(other.y);

        if x_overlap > 0.0 && y_overlap > 0.0 {
            x_overlap * y_overlap
        } else {
            0.0
        }
    }

    /// Expand the box by a margin
    pub fn expand(&self, margin: f32) -> Self {
        Self {
            x: self.x - margin,
            y: self.y - margin,
            width: self.width + margin * 2.0,
            height: self.height + margin * 2.0,
        }
    }

    /// Get the center point
    pub fn center(&self) -> (f32, f32) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
}

/// Positioned subtitle event
#[derive(Debug, Clone)]
pub struct PositionedEvent {
    /// Bounding box of the event
    pub bbox: BoundingBox,
    /// Layer number for z-order
    pub layer: i32,
    /// Vertical margin
    pub margin_v: i32,
    /// Left margin
    pub margin_l: i32,
    /// Right margin
    pub margin_r: i32,
    /// Text alignment (1-9 numpad style)
    pub alignment: u8,
    /// Priority for collision resolution (lower priority events can be moved)
    pub priority: i32,
}

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

    /// Find a non-colliding position for an event
    pub fn find_position(&mut self, mut event: PositionedEvent) -> BoundingBox {
        let original_bbox = event.bbox;
        let expanded_bbox = original_bbox.expand(self.collision_margin);

        // Check if current position has collisions
        let mut has_collision = false;
        for existing in &self.positioned_events {
            if existing.layer == event.layer {
                let existing_expanded = existing.bbox.expand(self.collision_margin);
                if expanded_bbox.intersects(&existing_expanded) {
                    has_collision = true;
                    break;
                }
            }
        }

        if !has_collision {
            self.positioned_events.push(event);
            return original_bbox;
        }

        // Try to find alternative position based on alignment
        let new_bbox = match event.alignment {
            1 | 2 | 3 => self.find_bottom_position(event.clone(), original_bbox),
            4 | 5 | 6 => self.find_middle_position(event.clone(), original_bbox),
            7 | 8 | 9 => self.find_top_position(event.clone(), original_bbox),
            _ => original_bbox,
        };

        event.bbox = new_bbox;
        self.positioned_events.push(event);
        new_bbox
    }

    /// Find position for bottom-aligned text
    fn find_bottom_position(&self, event: PositionedEvent, original: BoundingBox) -> BoundingBox {
        let mut candidates = Vec::new();
        let margin_v = event.margin_v as f32;

        // Try positions moving up from bottom
        let mut y = self.screen_height - original.height - margin_v;
        let step = original.height + self.collision_margin * 2.0;

        while y > margin_v {
            let test_bbox = BoundingBox::new(original.x, y, original.width, original.height);
            if !self.has_collision(&test_bbox, event.layer) {
                candidates.push((test_bbox, (y - original.y).abs()));
            }
            y -= step;
        }

        // Return closest to original position
        candidates
            .into_iter()
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(bbox, _)| bbox)
            .unwrap_or(original)
    }

    /// Find position for middle-aligned text
    fn find_middle_position(&self, event: PositionedEvent, original: BoundingBox) -> BoundingBox {
        let mut candidates = Vec::new();
        let step = original.height + self.collision_margin * 2.0;

        // Try positions above and below
        for direction in &[-1.0, 1.0] {
            let mut offset = step * direction;
            for _ in 0..5 {
                let y = original.y + offset;
                if y >= 0.0 && y + original.height <= self.screen_height {
                    let test_bbox =
                        BoundingBox::new(original.x, y, original.width, original.height);
                    if !self.has_collision(&test_bbox, event.layer) {
                        candidates.push((test_bbox, offset.abs()));
                    }
                }
                offset += step * direction;
            }
        }

        candidates
            .into_iter()
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(bbox, _)| bbox)
            .unwrap_or(original)
    }

    /// Find position for top-aligned text
    fn find_top_position(&self, event: PositionedEvent, original: BoundingBox) -> BoundingBox {
        let mut candidates = Vec::new();
        let margin_v = event.margin_v as f32;

        // Try positions moving down from top
        let mut y = margin_v;
        let step = original.height + self.collision_margin * 2.0;

        while y + original.height < self.screen_height - margin_v {
            let test_bbox = BoundingBox::new(original.x, y, original.width, original.height);
            if !self.has_collision(&test_bbox, event.layer) {
                candidates.push((test_bbox, (y - original.y).abs()));
            }
            y += step;
        }

        candidates
            .into_iter()
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(bbox, _)| bbox)
            .unwrap_or(original)
    }

    /// Check if a bounding box has collision with existing events
    fn has_collision(&self, bbox: &BoundingBox, layer: i32) -> bool {
        let expanded = bbox.expand(self.collision_margin);
        for existing in &self.positioned_events {
            if existing.layer == layer {
                let existing_expanded = existing.bbox.expand(self.collision_margin);
                if expanded.intersects(&existing_expanded) {
                    return true;
                }
            }
        }
        false
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
    fn test_bounding_box_intersection() {
        let box1 = BoundingBox::new(0.0, 0.0, 100.0, 50.0);
        let box2 = BoundingBox::new(50.0, 25.0, 100.0, 50.0);
        let box3 = BoundingBox::new(200.0, 200.0, 50.0, 50.0);

        assert!(box1.intersects(&box2));
        assert!(box2.intersects(&box1));
        assert!(!box1.intersects(&box3));
        assert!(!box3.intersects(&box1));
    }

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
