//! Effect processing pipeline

#[cfg(feature = "nostd")]
use alloc::boxed::Box;
#[cfg(not(feature = "nostd"))]
use std::boxed::Box;

use crate::utils::RenderError;
use smallvec::SmallVec;

mod animation;
mod blur;
mod transform;

pub use animation::{AnimationEffect, InterpolationType};
pub use blur::{BoxBlur, GaussianBlur};
pub use transform::TransformEffect;

/// Effect trait for applying visual effects
pub trait Effect: Send + Sync {
    /// Apply effect to pixel data
    fn apply(&self, pixels: &mut [u8], width: u32, height: u32) -> Result<(), RenderError>;

    /// Get effect name
    fn name(&self) -> &str;

    /// Check if effect can be combined with another
    fn can_combine_with(&self, other: &dyn Effect) -> bool {
        let _ = other;
        false
    }
}

/// Effect chain for applying multiple effects
pub struct EffectChain {
    effects: SmallVec<[Box<dyn Effect>; 8]>,
}

impl EffectChain {
    /// Create a new effect chain
    pub fn new() -> Self {
        Self {
            effects: SmallVec::new(),
        }
    }

    /// Add an effect to the chain
    pub fn add(&mut self, effect: Box<dyn Effect>) {
        self.effects.push(effect);
    }

    /// Apply all effects in the chain
    pub fn apply(&self, pixels: &mut [u8], width: u32, height: u32) -> Result<(), RenderError> {
        for effect in &self.effects {
            effect.apply(pixels, width, height)?;
        }
        Ok(())
    }

    /// Get number of effects in chain
    pub fn len(&self) -> usize {
        self.effects.len()
    }

    /// Check if chain is empty
    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }

    /// Clear all effects
    pub fn clear(&mut self) {
        self.effects.clear();
    }
}

impl Default for EffectChain {
    fn default() -> Self {
        Self::new()
    }
}
