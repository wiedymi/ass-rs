//! Error types for rendering

#[cfg(feature = "nostd")]
use alloc::{fmt, string::String};
#[cfg(not(feature = "nostd"))]
use std::{fmt, string::String};

#[cfg(not(feature = "nostd"))]
use thiserror::Error;

/// Rendering error types
#[cfg_attr(not(feature = "nostd"), derive(Error))]
#[derive(Debug)]
pub enum RenderError {
    /// Invalid dimensions provided
    #[cfg_attr(
        not(feature = "nostd"),
        error("Invalid dimensions: dimensions must be positive and non-zero")
    )]
    InvalidDimensions,

    /// Invalid buffer size
    #[cfg_attr(
        not(feature = "nostd"),
        error("Invalid buffer size: expected {expected} bytes, got {actual}")
    )]
    InvalidBufferSize {
        /// Expected size
        expected: usize,
        /// Actual size
        actual: usize,
    },

    /// Invalid pixmap creation
    #[cfg_attr(not(feature = "nostd"), error("Failed to create pixmap"))]
    InvalidPixmap,

    /// No backend available
    #[cfg_attr(not(feature = "nostd"), error("No rendering backend available"))]
    NoBackendAvailable,

    /// Unsupported backend
    #[cfg_attr(not(feature = "nostd"), error("Unsupported backend: {0}"))]
    UnsupportedBackend(&'static str),

    /// Backend initialization failed
    #[cfg_attr(not(feature = "nostd"), error("Backend initialization failed: {0}"))]
    BackendInitFailed(String),

    /// Generic backend error
    #[cfg_attr(not(feature = "nostd"), error("Backend error: {0}"))]
    BackendError(String),

    /// Pipeline error
    #[cfg_attr(not(feature = "nostd"), error("Pipeline error: {0}"))]
    PipelineError(String),

    /// Shaping error
    #[cfg_attr(not(feature = "nostd"), error("Text shaping failed: {0}"))]
    ShapingError(String),

    /// Drawing error
    #[cfg_attr(not(feature = "nostd"), error("Drawing failed: {0}"))]
    DrawingError(String),

    /// Invalid draw command
    #[cfg_attr(not(feature = "nostd"), error("Invalid draw command: {0}"))]
    InvalidDrawCommand(String),

    /// Effect error
    #[cfg_attr(not(feature = "nostd"), error("Effect application failed: {0}"))]
    EffectError(String),

    /// Compositing error
    #[cfg_attr(not(feature = "nostd"), error("Compositing failed: {0}"))]
    CompositingError(String),

    /// Font error
    #[cfg_attr(not(feature = "nostd"), error("Font error: {0}"))]
    FontError(String),

    /// GPU error
    #[cfg_attr(not(feature = "nostd"), error("GPU error: {0}"))]
    GpuError(String),

    /// WASM error
    #[cfg(target_arch = "wasm32")]
    #[cfg_attr(not(feature = "nostd"), error("WASM error: {0}"))]
    WasmError(String),

    /// Resource limit exceeded
    #[cfg_attr(not(feature = "nostd"), error("Resource limit exceeded: {0}"))]
    ResourceLimitExceeded(String),

    /// Invalid script
    #[cfg_attr(not(feature = "nostd"), error("Invalid script: {0}"))]
    InvalidScript(String),

    /// Parse error
    #[cfg_attr(not(feature = "nostd"), error("Parse error: {0}"))]
    ParseError(String),

    /// Invalid input
    #[cfg_attr(not(feature = "nostd"), error("Invalid input: {0}"))]
    InvalidInput(String),

    /// Core error from ass-core
    #[cfg_attr(not(feature = "nostd"), error("Core error: {0}"))]
    CoreError(#[cfg_attr(not(feature = "nostd"), from)] ass_core::utils::CoreError),

    /// Initialization error
    #[cfg_attr(not(feature = "nostd"), error("Initialization error: {0}"))]
    InitializationError(String),

    /// IO error
    #[cfg_attr(not(feature = "nostd"), error("IO error: {0}"))]
    IOError(String),

    /// Invalid state
    #[cfg_attr(not(feature = "nostd"), error("Invalid state: {0}"))]
    InvalidState(String),

    /// Unsupported operation
    #[cfg_attr(not(feature = "nostd"), error("Unsupported operation: {0}"))]
    UnsupportedOperation(String),
}

impl RenderError {
    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::ShapingError(_)
                | Self::DrawingError(_)
                | Self::EffectError(_)
                | Self::FontError(_)
        )
    }

    /// Check if error indicates missing resources
    pub fn is_resource_error(&self) -> bool {
        matches!(self, Self::FontError(_) | Self::ResourceLimitExceeded(_))
    }
}

// Manual Display implementation for nostd
#[cfg(feature = "nostd")]
impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDimensions => write!(
                f,
                "Invalid dimensions: dimensions must be positive and non-zero"
            ),
            Self::InvalidBufferSize { expected, actual } => {
                write!(
                    f,
                    "Invalid buffer size: expected {} bytes, got {}",
                    expected, actual
                )
            }
            Self::InvalidPixmap => write!(f, "Failed to create pixmap"),
            Self::NoBackendAvailable => write!(f, "No rendering backend available"),
            Self::UnsupportedBackend(s) => write!(f, "Unsupported backend: {}", s),
            Self::BackendInitFailed(s) => write!(f, "Backend initialization failed: {}", s),
            Self::BackendError(s) => write!(f, "Backend error: {}", s),
            Self::PipelineError(s) => write!(f, "Pipeline error: {}", s),
            Self::ShapingError(s) => write!(f, "Text shaping failed: {}", s),
            Self::DrawingError(s) => write!(f, "Drawing failed: {}", s),
            Self::InvalidDrawCommand(s) => write!(f, "Invalid draw command: {}", s),
            Self::EffectError(s) => write!(f, "Effect application failed: {}", s),
            Self::CompositingError(s) => write!(f, "Compositing failed: {}", s),
            Self::FontError(s) => write!(f, "Font error: {}", s),
            Self::GpuError(s) => write!(f, "GPU error: {}", s),
            #[cfg(target_arch = "wasm32")]
            Self::WasmError(s) => write!(f, "WASM error: {}", s),
            Self::ResourceLimitExceeded(s) => write!(f, "Resource limit exceeded: {}", s),
            Self::InvalidScript(s) => write!(f, "Invalid script: {}", s),
            Self::ParseError(s) => write!(f, "Parse error: {}", s),
            Self::InvalidInput(s) => write!(f, "Invalid input: {}", s),
            Self::CoreError(e) => write!(f, "Core error: {}", e),
            Self::InitializationError(s) => write!(f, "Initialization error: {}", s),
            Self::IOError(s) => write!(f, "IO error: {}", s),
            Self::InvalidState(s) => write!(f, "Invalid state: {}", s),
            Self::UnsupportedOperation(s) => write!(f, "Unsupported operation: {}", s),
        }
    }
}

// Manual Error implementation for nostd
#[cfg(feature = "nostd")]
impl core::error::Error for RenderError {}
