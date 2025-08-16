//! Caching system for expensive operations

use crate::pipeline::shaping::ShapedText;
use tiny_skia::Path;

#[cfg(not(feature = "nostd"))]
use std::collections::HashMap;
#[cfg(not(feature = "nostd"))]
use std::sync::Arc;

#[cfg(feature = "nostd")]
use alloc::collections::BTreeMap as HashMap;
#[cfg(feature = "nostd")]
use alloc::{string::String, sync::Arc};

/// Cache key for shaped text
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct TextCacheKey {
    pub text: String,
    pub font_family: String,
    pub font_size: u32, // Rounded to avoid float comparison issues
    pub bold: bool,
    pub italic: bool,
}

/// Cache key for drawing paths
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct DrawingCacheKey {
    pub commands: String,
}

/// Render cache for expensive operations
pub struct RenderCache {
    /// Cache for shaped text
    shaped_text_cache: HashMap<TextCacheKey, Arc<ShapedText>>,
    max_shaped_entries: usize,

    /// Cache for drawing paths
    drawing_path_cache: HashMap<DrawingCacheKey, Option<Path>>,
    max_drawing_entries: usize,

    /// Cache statistics
    pub stats: CacheStats,
}

/// Cache statistics for monitoring
#[derive(Debug, Default, Clone)]
pub struct CacheStats {
    pub text_hits: usize,
    pub text_misses: usize,
    pub drawing_hits: usize,
    pub drawing_misses: usize,
    pub evictions: usize,
}

impl RenderCache {
    /// Create a new render cache
    pub fn new() -> Self {
        Self {
            shaped_text_cache: HashMap::new(),
            max_shaped_entries: 1000,
            drawing_path_cache: HashMap::new(),
            max_drawing_entries: 500,
            stats: CacheStats::default(),
        }
    }

    /// Create with custom limits
    pub fn with_limits(max_shaped: usize, max_drawing: usize) -> Self {
        Self {
            shaped_text_cache: HashMap::new(),
            max_shaped_entries: max_shaped,
            drawing_path_cache: HashMap::new(),
            max_drawing_entries: max_drawing,
            stats: CacheStats::default(),
        }
    }

    /// Get shaped text from cache
    pub fn get_shaped_text(&mut self, key: &TextCacheKey) -> Option<Arc<ShapedText>> {
        if let Some(shaped) = self.shaped_text_cache.get(key) {
            self.stats.text_hits += 1;
            Some(Arc::clone(shaped))
        } else {
            self.stats.text_misses += 1;
            None
        }
    }

    /// Store shaped text in cache
    pub fn store_shaped_text(&mut self, key: TextCacheKey, shaped: ShapedText) -> Arc<ShapedText> {
        // Evict if at capacity
        if self.shaped_text_cache.len() >= self.max_shaped_entries {
            // Simple LRU: remove first item (not ideal but simple)
            if let Some(first_key) = self.shaped_text_cache.keys().next().cloned() {
                self.shaped_text_cache.remove(&first_key);
                self.stats.evictions += 1;
            }
        }

        let arc_shaped = Arc::new(shaped);
        self.shaped_text_cache.insert(key, Arc::clone(&arc_shaped));
        arc_shaped
    }

    /// Get drawing path from cache
    pub fn get_drawing_path(&mut self, key: &DrawingCacheKey) -> Option<Option<Path>> {
        if let Some(path) = self.drawing_path_cache.get(key) {
            self.stats.drawing_hits += 1;
            Some(path.clone())
        } else {
            self.stats.drawing_misses += 1;
            None
        }
    }

    /// Store drawing path in cache
    pub fn store_drawing_path(&mut self, key: DrawingCacheKey, path: Option<Path>) {
        // Evict if at capacity
        if self.drawing_path_cache.len() >= self.max_drawing_entries {
            if let Some(first_key) = self.drawing_path_cache.keys().next().cloned() {
                self.drawing_path_cache.remove(&first_key);
                self.stats.evictions += 1;
            }
        }

        self.drawing_path_cache.insert(key, path);
    }

    /// Clear all caches
    pub fn clear(&mut self) {
        self.shaped_text_cache.clear();
        self.drawing_path_cache.clear();
        self.stats = CacheStats::default();
    }

    /// Get cache statistics
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }

    /// Print cache statistics
    pub fn print_stats(&self) {
        let text_ratio = if self.stats.text_hits + self.stats.text_misses > 0 {
            self.stats.text_hits as f64 / (self.stats.text_hits + self.stats.text_misses) as f64
        } else {
            0.0
        };

        let drawing_ratio = if self.stats.drawing_hits + self.stats.drawing_misses > 0 {
            self.stats.drawing_hits as f64
                / (self.stats.drawing_hits + self.stats.drawing_misses) as f64
        } else {
            0.0
        };

        #[cfg(not(feature = "nostd"))]
        eprintln!("=== Cache Statistics ===");
        #[cfg(not(feature = "nostd"))]
        eprintln!(
            "Text Cache: {} entries, {:.1}% hit rate ({}/{} hits)",
            self.shaped_text_cache.len(),
            text_ratio * 100.0,
            self.stats.text_hits,
            self.stats.text_hits + self.stats.text_misses
        );
        #[cfg(not(feature = "nostd"))]
        eprintln!(
            "Drawing Cache: {} entries, {:.1}% hit rate ({}/{} hits)",
            self.drawing_path_cache.len(),
            drawing_ratio * 100.0,
            self.stats.drawing_hits,
            self.stats.drawing_hits + self.stats.drawing_misses
        );
        #[cfg(not(feature = "nostd"))]
        eprintln!("Total Evictions: {}", self.stats.evictions);
    }
}

impl Default for RenderCache {
    fn default() -> Self {
        Self::new()
    }
}
