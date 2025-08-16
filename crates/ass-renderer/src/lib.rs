//! ASS subtitle renderer with modular backend support
//!
//! `ass-renderer` provides high-performance subtitle rendering with support for
//! multiple backends including software (CPU), hardware (Vulkan/Metal), and web
//! (WebGPU/WebGL).

#![cfg_attr(feature = "nostd", no_std)]
#![deny(unsafe_code)] // Changed from forbid to allow overrides in FFI modules
#![warn(missing_docs)]

#[cfg(feature = "nostd")]
extern crate alloc;

#[cfg(feature = "nostd")]
use alloc::{boxed::Box, string::String, vec::Vec};

pub mod animation;
pub mod backends;
pub mod cache;
pub mod collision;
#[cfg(not(feature = "nostd"))]
pub mod debug;
pub mod layout;
pub mod pipeline;
pub mod plugin;
pub mod renderer;
pub mod utils;

pub use backends::{BackendType, RenderBackend};
#[cfg(all(not(feature = "nostd"), feature = "libass-compare"))]
pub use debug::LibassRenderer;
#[cfg(not(feature = "nostd"))]
pub use debug::{DebugPlayer, FrameAnalyzer, FrameInspector, PlayerFrame};
pub use pipeline::{Pipeline, PipelineStage};
pub use plugin::{EffectPlugin, PluginRegistry};
pub use renderer::{Frame, RenderContext, Renderer};
pub use utils::RenderError;

#[cfg(feature = "backend-metrics")]
pub use backends::BackendMetrics;

#[cfg(feature = "analysis-integration")]
pub use ass_core::analysis::styles::ResolvedStyle;
pub use ass_core::parser::ast::EventType;
/// Re-export commonly used types from ass-core
pub use ass_core::parser::{Event, Script, Section, Style};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
