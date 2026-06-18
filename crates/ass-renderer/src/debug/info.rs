//! Debug information data structures for rendered frames.
//!
//! Holds the per-frame debug metadata produced by [`DebugRenderer`](crate::debug::DebugRenderer)
//! together with the result of comparing two captured frames, plus their
//! optional `serde` serialization implementations.

#[cfg(feature = "nostd")]
extern crate alloc;
#[cfg(feature = "nostd")]
use alloc::{format, string::String, vec::Vec};

/// Debug information for a rendered frame
#[derive(Debug, Clone)]
pub struct FrameDebugInfo {
    /// Frame timestamp in milliseconds
    pub timestamp_ms: u32,
    /// Number of active subtitle events
    pub active_events: usize,
    /// Dirty regions that need re-rendering
    pub dirty_regions: Vec<DirtyRegionInfo>,
    /// Time taken to render this frame in milliseconds
    pub render_time_ms: f64,
    /// Memory used by frame data in bytes
    pub memory_usage_bytes: usize,
    /// Number of cache hits during rendering
    pub cache_hits: usize,
    /// Number of cache misses during rendering
    pub cache_misses: usize,
    /// Name of the rendering backend used
    pub backend_type: String,
    /// Checksum of the frame data for comparison
    pub frame_checksum: u64,
    /// Number of non-transparent pixels
    pub non_transparent_pixels: usize,
    /// Bounding box of rendered content
    pub bounding_box: Option<BoundingBoxInfo>,
}

/// Information about a dirty region that needs re-rendering
#[derive(Debug, Clone)]
pub struct DirtyRegionInfo {
    /// X coordinate of the region
    pub x: u32,
    /// Y coordinate of the region
    pub y: u32,
    /// Width of the region
    pub width: u32,
    /// Height of the region
    pub height: u32,
    /// Reason why this region is dirty
    pub reason: String,
}

/// Bounding box information for rendered content
#[derive(Debug, Clone, PartialEq)]
pub struct BoundingBoxInfo {
    /// Minimum X coordinate
    pub min_x: u32,
    /// Minimum Y coordinate
    pub min_y: u32,
    /// Maximum X coordinate
    pub max_x: u32,
    /// Maximum Y coordinate
    pub max_y: u32,
}

/// Result of comparing two frames
#[derive(Debug)]
pub struct FrameComparison {
    /// Whether the frame checksums match
    pub checksum_match: bool,
    /// Difference in non-transparent pixel count
    pub pixel_diff: u32,
    /// Difference in render time (milliseconds)
    pub render_time_diff: f64,
    /// Whether the bounding boxes differ
    pub bbox_changed: bool,
}

// Make FrameDebugInfo serializable
#[cfg(feature = "serde")]
impl serde::Serialize for FrameDebugInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("FrameDebugInfo", 11)?;
        state.serialize_field("timestamp_ms", &self.timestamp_ms)?;
        state.serialize_field("active_events", &self.active_events)?;
        state.serialize_field("dirty_regions", &self.dirty_regions)?;
        state.serialize_field("render_time_ms", &self.render_time_ms)?;
        state.serialize_field("memory_usage_bytes", &self.memory_usage_bytes)?;
        state.serialize_field("cache_hits", &self.cache_hits)?;
        state.serialize_field("cache_misses", &self.cache_misses)?;
        state.serialize_field("backend_type", &self.backend_type)?;
        state.serialize_field("frame_checksum", &format!("0x{:016x}", self.frame_checksum))?;
        state.serialize_field("non_transparent_pixels", &self.non_transparent_pixels)?;
        state.serialize_field("bounding_box", &self.bounding_box)?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for DirtyRegionInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("DirtyRegionInfo", 5)?;
        state.serialize_field("x", &self.x)?;
        state.serialize_field("y", &self.y)?;
        state.serialize_field("width", &self.width)?;
        state.serialize_field("height", &self.height)?;
        state.serialize_field("reason", &self.reason)?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for BoundingBoxInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("BoundingBoxInfo", 4)?;
        state.serialize_field("min_x", &self.min_x)?;
        state.serialize_field("min_y", &self.min_y)?;
        state.serialize_field("max_x", &self.max_x)?;
        state.serialize_field("max_y", &self.max_y)?;
        state.end()
    }
}
