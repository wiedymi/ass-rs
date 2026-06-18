//! Resolved animation state queried by the render pipeline

#[cfg(feature = "nostd")]
use alloc::string::{String, ToString};
#[cfg(not(feature = "nostd"))]
use std::string::String;

use smallvec::SmallVec;

use super::value::AnimatedResult;

/// Current animation state
#[derive(Debug, Clone)]
pub struct AnimationState {
    properties: SmallVec<[(String, AnimatedResult); 16]>,
}

impl AnimationState {
    /// Create new animation state
    pub fn new() -> Self {
        Self {
            properties: SmallVec::new(),
        }
    }

    /// Set a property value
    pub fn set_property(&mut self, name: &str, value: AnimatedResult) {
        if let Some(entry) = self.properties.iter_mut().find(|(n, _)| n == name) {
            entry.1 = value;
        } else {
            self.properties.push((name.to_string(), value));
        }
    }

    /// Get a property value
    pub fn get_property(&self, name: &str) -> Option<&AnimatedResult> {
        self.properties
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v)
    }

    /// Get font size if animated
    pub fn font_size(&self) -> Option<f32> {
        self.get_property("fs").and_then(|v| {
            if let AnimatedResult::Float(size) = v {
                Some(*size)
            } else {
                None
            }
        })
    }

    /// Get scale if animated
    pub fn scale(&self) -> Option<(f32, f32)> {
        let scale_x = self.get_property("fscx").and_then(|v| {
            if let AnimatedResult::Float(scale) = v {
                Some(*scale)
            } else {
                None
            }
        });

        let scale_y = self.get_property("fscy").and_then(|v| {
            if let AnimatedResult::Float(scale) = v {
                Some(*scale)
            } else {
                None
            }
        });

        match (scale_x, scale_y) {
            (Some(x), Some(y)) => Some((x, y)),
            (Some(x), None) => Some((x, 100.0)),
            (None, Some(y)) => Some((100.0, y)),
            _ => None,
        }
    }

    /// Get rotation if animated
    pub fn rotation(&self) -> Option<f32> {
        self.get_property("frz").and_then(|v| {
            if let AnimatedResult::Float(rotation) = v {
                Some(*rotation)
            } else {
                None
            }
        })
    }

    /// Get color if animated
    pub fn color(&self) -> Option<[u8; 4]> {
        self.get_property("c").and_then(|v| {
            if let AnimatedResult::Color(color) = v {
                Some(*color)
            } else {
                None
            }
        })
    }
}

impl Default for AnimationState {
    fn default() -> Self {
        Self::new()
    }
}
