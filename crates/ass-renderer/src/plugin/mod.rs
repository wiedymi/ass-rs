//! Plugin system for custom effects and backends

use crate::utils::RenderError;
use ahash::AHashMap;

#[cfg(feature = "nostd")]
use alloc::{
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
#[cfg(not(feature = "nostd"))]
use std::{
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};

pub mod effects;

/// Effect plugin trait for custom effects
pub trait EffectPlugin: Send + Sync {
    /// Get plugin name
    fn name(&self) -> &str;

    /// Get plugin version
    fn version(&self) -> &str;

    /// Apply effect to pixel data
    fn apply_cpu(
        &self,
        pixels: &mut [u8],
        width: u32,
        height: u32,
        params: &EffectParams,
    ) -> Result<(), RenderError>;

    /// Get shader code for GPU backends
    fn shader_code(&self) -> Option<&str> {
        None
    }

    /// Check if effect supports GPU acceleration
    fn supports_gpu(&self) -> bool {
        self.shader_code().is_some()
    }
}

/// Effect parameters
#[derive(Debug, Clone)]
pub struct EffectParams {
    /// Effect strength (0.0-1.0)
    pub strength: f32,
    /// Custom parameters
    pub custom: AHashMap<String, EffectValue>,
}

/// Effect parameter value
#[derive(Debug, Clone)]
pub enum EffectValue {
    /// Float value
    Float(f32),
    /// Integer value
    Integer(i32),
    /// Color value (RGBA)
    Color([u8; 4]),
    /// Boolean value
    Boolean(bool),
    /// String value
    String(String),
}

impl EffectParams {
    /// Create new effect parameters
    pub fn new(strength: f32) -> Self {
        Self {
            strength,
            custom: AHashMap::new(),
        }
    }

    /// Set a custom parameter
    pub fn set(&mut self, key: impl Into<String>, value: EffectValue) {
        self.custom.insert(key.into(), value);
    }

    /// Get a custom parameter
    pub fn get(&self, key: &str) -> Option<&EffectValue> {
        self.custom.get(key)
    }

    /// Get parameter as float
    pub fn get_float(&self, key: &str) -> Option<f32> {
        self.custom.get(key).and_then(|v| {
            if let EffectValue::Float(f) = v {
                Some(*f)
            } else {
                None
            }
        })
    }

    /// Get parameter as integer
    pub fn get_int(&self, key: &str) -> Option<i32> {
        self.custom.get(key).and_then(|v| {
            if let EffectValue::Integer(i) = v {
                Some(*i)
            } else {
                None
            }
        })
    }

    /// Get parameter as color
    pub fn get_color(&self, key: &str) -> Option<[u8; 4]> {
        self.custom.get(key).and_then(|v| {
            if let EffectValue::Color(c) = v {
                Some(*c)
            } else {
                None
            }
        })
    }
}

/// Plugin registry for managing plugins
pub struct PluginRegistry {
    effects: AHashMap<String, Arc<dyn EffectPlugin>>,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new() -> Self {
        Self {
            effects: AHashMap::new(),
        }
    }

    /// Register an effect plugin
    pub fn register_effect(&mut self, plugin: Arc<dyn EffectPlugin>) {
        self.effects.insert(plugin.name().to_string(), plugin);
    }

    /// Get an effect plugin by name
    pub fn get_effect(&self, name: &str) -> Option<Arc<dyn EffectPlugin>> {
        self.effects.get(name).cloned()
    }

    /// List all registered effects
    pub fn list_effects(&self) -> Vec<String> {
        self.effects.keys().cloned().collect()
    }

    /// Remove an effect plugin
    pub fn unregister_effect(&mut self, name: &str) -> Option<Arc<dyn EffectPlugin>> {
        self.effects.remove(name)
    }

    /// Clear all plugins
    pub fn clear(&mut self) {
        self.effects.clear();
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
