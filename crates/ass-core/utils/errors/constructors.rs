//! Convenience constructors for [`CoreError`] variants.
//!
//! Provides ergonomic factory methods on [`CoreError`] that delegate to the
//! specialized error-creation helpers in the sibling submodules.

use super::{encoding, format, resource, CoreError};

impl CoreError {
    /// Create color error from invalid format
    pub fn invalid_color<T: ::core::fmt::Display>(format: T) -> Self {
        format::invalid_color(format)
    }

    /// Create numeric error from parsing failure
    pub fn invalid_numeric<T: ::core::fmt::Display>(value: T, reason: &str) -> Self {
        format::invalid_numeric(value, reason)
    }

    /// Create time error from invalid format
    pub fn invalid_time<T: ::core::fmt::Display>(time: T, reason: &str) -> Self {
        format::invalid_time(time, reason)
    }

    /// Create UTF-8 error with position
    #[must_use]
    pub const fn utf8_error(position: usize, message: alloc::string::String) -> Self {
        encoding::utf8_error(position, message)
    }

    /// Create feature not supported error
    #[must_use]
    pub fn feature_not_supported(feature: &str, required_feature: &str) -> Self {
        resource::feature_not_supported(feature, required_feature)
    }

    /// Create resource limit error
    #[must_use]
    pub fn resource_limit_exceeded(resource: &str, current: usize, limit: usize) -> Self {
        resource::resource_limit_exceeded(resource, current, limit)
    }
}
