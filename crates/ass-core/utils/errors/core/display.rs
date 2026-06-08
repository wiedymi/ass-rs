//! `Display` and `Error` trait implementations for `CoreError`.
//!
//! Provides feature-gated formatting so the error type renders identically
//! in both `std` and `no_std` builds.

use super::CoreError;
use core::fmt;

/// nostd compatible Display implementation
#[cfg(not(feature = "std"))]
impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(parse_err) => write!(f, "Parse error: {parse_err}"),
            Self::Tokenization(msg) => write!(f, "Tokenization error: {msg}"),
            Self::Analysis(msg) => write!(f, "Analysis error: {msg}"),
            Self::Plugin(msg) => write!(f, "Plugin error: {msg}"),
            Self::InvalidColor(msg) => write!(f, "Invalid color format: {msg}"),
            Self::InvalidNumeric(msg) => write!(f, "Invalid numeric value: {msg}"),
            Self::InvalidTime(msg) => write!(f, "Invalid time format: {msg}"),
            Self::Utf8Error { position, message } => {
                write!(f, "UTF-8 encoding error at position {position}: {message}")
            }
            Self::Io(msg) => write!(f, "I/O error: {msg}"),
            Self::OutOfMemory(msg) => write!(f, "Memory allocation failed: {msg}"),
            Self::Config(msg) => write!(f, "Configuration error: {msg}"),
            Self::Validation(msg) => write!(f, "Validation error: {msg}"),
            Self::FeatureNotSupported {
                feature,
                required_feature,
            } => {
                write!(
                    f,
                    "Feature not supported: {feature} (requires feature '{required_feature}')"
                )
            }
            Self::VersionIncompatible { message } => {
                write!(f, "Version incompatibility: {message}")
            }
            Self::ResourceLimitExceeded {
                resource,
                current,
                limit,
            } => {
                write!(f, "Resource limit exceeded: {resource} ({current}/{limit})")
            }
            Self::SecurityViolation(msg) => write!(f, "Security policy violation: {msg}"),
            Self::Internal(msg) => {
                write!(f, "Internal error: {msg} (this is a bug, please report)")
            }
        }
    }
}
/// nostd compatible Error implementation
#[cfg(not(feature = "std"))]
impl core::error::Error for CoreError {}
/// std compatible Display implementation
#[cfg(feature = "std")]
impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(parse_err) => write!(f, "Parse error: {parse_err}"),
            Self::Tokenization(msg) => write!(f, "Tokenization error: {msg}"),
            Self::Analysis(msg) => write!(f, "Analysis error: {msg}"),
            Self::Plugin(msg) => write!(f, "Plugin error: {msg}"),
            Self::InvalidColor(msg) => write!(f, "Invalid color format: {msg}"),
            Self::InvalidNumeric(msg) => write!(f, "Invalid numeric value: {msg}"),
            Self::InvalidTime(msg) => write!(f, "Invalid time format: {msg}"),
            Self::Utf8Error { position, message } => {
                write!(f, "UTF-8 encoding error at position {position}: {message}")
            }
            Self::Io(msg) => write!(f, "I/O error: {msg}"),
            Self::OutOfMemory(msg) => write!(f, "Memory allocation failed: {msg}"),
            Self::Config(msg) => write!(f, "Configuration error: {msg}"),
            Self::Validation(msg) => write!(f, "Validation error: {msg}"),
            Self::FeatureNotSupported {
                feature,
                required_feature,
            } => {
                write!(
                    f,
                    "Feature not supported: {feature} (requires feature '{required_feature}')"
                )
            }
            Self::VersionIncompatible { message } => {
                write!(f, "Version incompatibility: {message}")
            }
            Self::ResourceLimitExceeded {
                resource,
                current,
                limit,
            } => {
                write!(f, "Resource limit exceeded: {resource} ({current}/{limit})")
            }
            Self::SecurityViolation(msg) => write!(f, "Security policy violation: {msg}"),
            Self::Internal(msg) => {
                write!(f, "Internal error: {msg} (this is a bug, please report)")
            }
        }
    }
}
