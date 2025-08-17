//! Dirty region tracking for incremental rendering

/// Dirty region for incremental updates
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirtyRegion {
    /// Left coordinate
    pub x: u32,
    /// Top coordinate
    pub y: u32,
    /// Width
    pub width: u32,
    /// Height
    pub height: u32,
}

impl DirtyRegion {
    /// Create a new dirty region
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Create a full screen dirty region
    pub fn full_screen() -> Self {
        Self {
            x: 0,
            y: 0,
            width: u32::MAX,
            height: u32::MAX,
        }
    }

    /// Check if this region intersects with bounds
    pub fn intersects(&self, bounds: (u32, u32, u32, u32)) -> bool {
        let (bx1, by1, bx2, by2) = bounds;
        let rx1 = self.x;
        let ry1 = self.y;
        let rx2 = self.x.saturating_add(self.width);
        let ry2 = self.y.saturating_add(self.height);

        !(rx2 < bx1 || rx1 > bx2 || ry2 < by1 || ry1 > by2)
    }

    /// Merge with another region
    pub fn merge(&self, other: &Self) -> Self {
        let x1 = self.x.min(other.x);
        let y1 = self.y.min(other.y);
        let x2 = (self.x + self.width).max(other.x + other.width);
        let y2 = (self.y + self.height).max(other.y + other.height);

        Self {
            x: x1,
            y: y1,
            width: x2 - x1,
            height: y2 - y1,
        }
    }

    /// Check if region contains point
    pub fn contains_point(&self, x: u32, y: u32) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }

    /// Get area of region
    pub fn area(&self) -> u64 {
        self.width as u64 * self.height as u64
    }
}
