//! Performance metrics and cache statistics for the renderer

#[cfg(not(feature = "nostd"))]
use std::time::Duration;

/// Performance metrics for rendering
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    #[cfg(not(feature = "nostd"))]
    /// Time spent parsing the script
    pub parse_time: Duration,
    #[cfg(not(feature = "nostd"))]
    /// Time spent shaping text
    pub shape_time: Duration,
    #[cfg(not(feature = "nostd"))]
    /// Time spent rendering
    pub render_time: Duration,
    #[cfg(not(feature = "nostd"))]
    /// Total time for the operation
    pub total_time: Duration,
    #[cfg(feature = "nostd")]
    pub parse_time: u64, // milliseconds
    #[cfg(feature = "nostd")]
    pub shape_time: u64, // milliseconds
    #[cfg(feature = "nostd")]
    pub render_time: u64, // milliseconds
    #[cfg(feature = "nostd")]
    pub total_time: u64, // milliseconds
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStatistics {
    /// Number of glyph cache hits
    pub glyph_hits: usize,
    /// Number of font database entries
    pub font_entries: usize,
}
