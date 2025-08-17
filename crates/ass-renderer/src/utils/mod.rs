//! Utility types and helper functions

mod caches;
mod errors;
mod math;
mod regions;

pub use caches::{GlyphCache, TextureAtlas};
pub use errors::RenderError;
pub use math::{cubic_bezier, lerp, Matrix3x3, Transform2D};
pub use regions::DirtyRegion;
