//! Layout system for proper subtitle positioning

pub mod alignment;
pub mod context;
pub mod metrics;
pub mod multiline;
pub mod positioning;

pub use alignment::Alignment;
pub use context::LayoutContext;
pub use metrics::TextMetrics;
pub use multiline::{LineLayout, MultiLineLayout};
pub use positioning::{convert_ssa_alignment, scale_coordinates, BoundingBox, PositionInfo};
