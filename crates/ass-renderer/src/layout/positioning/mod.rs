//! Advanced positioning and alignment calculations based on libass
//!
//! This module provides positioning calculations that match libass behavior,
//! including proper anchor point handling, rotation origins, and alignment.

mod coords;
mod info;

pub use coords::{convert_ssa_alignment, scale_coordinates};
pub use info::PositionInfo;

/// Bounding box for text
#[derive(Debug, Clone)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Configuration for position calculation
pub struct PositionConfig {
    pub screen_width: f32,
    pub screen_height: f32,
    pub margin_left: f32,
    pub margin_right: f32,
    pub margin_vertical: f32,
    pub default_alignment: u8,
}
