//! Collision detection for subtitle positioning

mod resolver;

pub use resolver::{CollisionResolver, SmartCollisionResolver};

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
}
