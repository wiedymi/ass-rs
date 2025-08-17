//! Caching utilities for glyphs and textures

use ahash::AHashMap;

#[cfg(feature = "nostd")]
use alloc::{string::String, vec, vec::Vec};
#[cfg(not(feature = "nostd"))]
use std::{string::String, vec::Vec};

/// Glyph cache for rendered glyphs
pub struct GlyphCache {
    cache: AHashMap<GlyphKey, CachedGlyph>,
    max_entries: usize,
}

/// Key for glyph cache lookup
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct GlyphKey {
    /// Font family
    pub font_family: String,
    /// Glyph ID
    pub glyph_id: u16,
    /// Font size
    pub size: u32,
    /// Style flags (bold, italic, etc.)
    pub style_flags: u32,
}

/// Cached glyph data
pub struct CachedGlyph {
    /// Glyph bitmap data
    pub bitmap: Vec<u8>,
    /// Bitmap width
    pub width: u32,
    /// Bitmap height
    pub height: u32,
    /// Horizontal advance
    pub advance: f32,
    /// Bearing X
    pub bearing_x: f32,
    /// Bearing Y
    pub bearing_y: f32,
}

impl GlyphCache {
    /// Create a new glyph cache
    pub fn new(max_entries: usize) -> Self {
        Self {
            cache: AHashMap::new(),
            max_entries,
        }
    }

    /// Get a cached glyph
    pub fn get(&self, key: &GlyphKey) -> Option<&CachedGlyph> {
        self.cache.get(key)
    }

    /// Insert a glyph into the cache
    pub fn insert(&mut self, key: GlyphKey, glyph: CachedGlyph) {
        if self.cache.len() >= self.max_entries {
            // Simple eviction: remove first entry
            if let Some(first_key) = self.cache.keys().next().cloned() {
                self.cache.remove(&first_key);
            }
        }
        self.cache.insert(key, glyph);
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// Get cache size
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

/// Texture atlas for GPU backends
pub struct TextureAtlas {
    width: u32,
    height: u32,
    data: Vec<u8>,
    allocations: Vec<AtlasRegion>,
    next_x: u32,
    next_y: u32,
    row_height: u32,
}

/// Region in texture atlas
#[derive(Debug, Clone)]
pub struct AtlasRegion {
    /// Region ID
    pub id: u32,
    /// X coordinate in atlas
    pub x: u32,
    /// Y coordinate in atlas
    pub y: u32,
    /// Region width
    pub width: u32,
    /// Region height
    pub height: u32,
}

impl TextureAtlas {
    /// Create a new texture atlas
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![0; (width * height * 4) as usize],
            allocations: Vec::new(),
            next_x: 0,
            next_y: 0,
            row_height: 0,
        }
    }

    /// Allocate a region in the atlas
    pub fn allocate(&mut self, width: u32, height: u32) -> Option<AtlasRegion> {
        // Simple row-based packing
        if self.next_x + width > self.width {
            // Move to next row
            self.next_x = 0;
            self.next_y += self.row_height;
            self.row_height = 0;
        }

        if self.next_y + height > self.height {
            // Atlas is full
            return None;
        }

        let region = AtlasRegion {
            id: self.allocations.len() as u32,
            x: self.next_x,
            y: self.next_y,
            width,
            height,
        };

        self.next_x += width;
        self.row_height = self.row_height.max(height);
        self.allocations.push(region.clone());

        Some(region)
    }

    /// Write data to a region
    pub fn write_region(&mut self, region: &AtlasRegion, data: &[u8]) {
        let stride = self.width * 4;
        for y in 0..region.height {
            let src_offset = (y * region.width * 4) as usize;
            let dst_offset = ((region.y + y) * stride + region.x * 4) as usize;
            let src_end = src_offset + (region.width * 4) as usize;
            let dst_end = dst_offset + (region.width * 4) as usize;

            if src_end <= data.len() && dst_end <= self.data.len() {
                self.data[dst_offset..dst_end].copy_from_slice(&data[src_offset..src_end]);
            }
        }
    }

    /// Get atlas data
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Clear the atlas
    pub fn clear(&mut self) {
        self.data.fill(0);
        self.allocations.clear();
        self.next_x = 0;
        self.next_y = 0;
        self.row_height = 0;
    }
}
