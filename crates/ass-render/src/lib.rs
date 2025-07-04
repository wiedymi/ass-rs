//! Rendering engine.

#![cfg_attr(not(feature = "std"), no_std)]

pub mod model;
pub mod text_shaping;

#[cfg(feature = "software")]
pub mod software;

#[cfg(feature = "hardware")]
pub mod hardware;

#[cfg(feature = "software")]
pub use software::SoftwareRenderer;

#[cfg(feature = "hardware")]
pub use hardware::{HardwareRenderer, HardwareRendererError};

pub use text_shaping::{ShapedText, TextDirection, TextShaper, TextShapingError};
