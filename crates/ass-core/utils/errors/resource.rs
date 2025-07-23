//! Resource management error utilities for ASS-RS
//!
//! Provides specialized error creation and validation functions for resource
//! limits including memory allocation, processing limits, and other system
//! resource constraints. Focuses on preventing resource exhaustion attacks.

use super::CoreError;
use alloc::format;
use core::fmt;

/// Create memory allocation error
///
/// Generates a `CoreError::OutOfMemory` with descriptive context about
/// the failed memory allocation attempt.
///
/// # Arguments
///
/// * `context` - Description of what allocation failed
///
/// # Examples
///
/// ```rust
/// use ass_core::utils::errors::{out_of_memory, CoreError};
///
/// let error = out_of_memory("Failed to allocate parser buffer");
/// assert!(matches!(error, CoreError::OutOfMemory(_)));
/// ```
pub fn out_of_memory<T: fmt::Display>(context: T) -> CoreError {
    CoreError::OutOfMemory(format!("{context}"))
}

/// Create resource limit exceeded error
///
/// Generates a `CoreError::ResourceLimitExceeded` with detailed information
/// about which resource was exceeded and by how much.
///
/// # Arguments
///
/// * `resource` - Name of the resource that was exceeded
/// * `current` - Current usage that triggered the limit
/// * `limit` - Maximum allowed usage
#[must_use]
pub fn resource_limit_exceeded(resource: &str, current: usize, limit: usize) -> CoreError {
    CoreError::ResourceLimitExceeded {
        resource: resource.to_string(),
        current,
        limit,
    }
}

/// Create feature not supported error
///
/// Generates a `CoreError::FeatureNotSupported` when functionality requires
/// a feature that is not enabled in the current build configuration.
///
/// # Arguments
///
/// * `feature` - The feature that was requested
/// * `required_feature` - The Cargo feature flag needed to enable it
#[must_use]
pub fn feature_not_supported(feature: &str, required_feature: &str) -> CoreError {
    CoreError::FeatureNotSupported {
        feature: feature.to_string(),
        required_feature: required_feature.to_string(),
    }
}

/// Check memory usage against limit
///
/// Validates current memory usage against a configured limit and returns
/// an error if the limit would be exceeded. Used to prevent OOM conditions.
///
/// # Arguments
///
/// * `current_bytes` - Current memory usage in bytes
/// * `additional_bytes` - Additional bytes that would be allocated
/// * `limit_bytes` - Maximum allowed memory usage
pub fn check_memory_limit(
    current_bytes: usize,
    additional_bytes: usize,
    limit_bytes: usize,
) -> Result<(), CoreError> {
    let total = current_bytes
        .checked_add(additional_bytes)
        .ok_or_else(|| out_of_memory("Integer overflow calculating memory usage"))?;

    if total > limit_bytes {
        return Err(resource_limit_exceeded("memory", total, limit_bytes));
    }

    Ok(())
}

/// Check processing time limit
///
/// Validates processing duration against a configured time limit to prevent
/// infinite loops or excessive processing time attacks.
///
/// # Arguments
///
/// * `elapsed_ms` - Time elapsed so far in milliseconds
/// * `limit_ms` - Maximum allowed processing time
#[allow(dead_code)]
pub fn check_time_limit(elapsed_ms: u64, limit_ms: u64) -> Result<(), CoreError> {
    if elapsed_ms > limit_ms {
        return Err(resource_limit_exceeded(
            "processing_time",
            elapsed_ms as usize,
            limit_ms as usize,
        ));
    }
    Ok(())
}

/// Check input size limit
///
/// Validates input size against configured limits to prevent processing
/// of excessively large inputs that could cause resource exhaustion.
///
/// # Arguments
///
/// * `input_size` - Size of input data in bytes
/// * `max_size` - Maximum allowed input size
#[allow(dead_code)]
pub fn check_input_size_limit(input_size: usize, max_size: usize) -> Result<(), CoreError> {
    if input_size > max_size {
        return Err(resource_limit_exceeded("input_size", input_size, max_size));
    }
    Ok(())
}

/// Check nesting depth limit
///
/// Prevents stack overflow from deeply nested structures by validating
/// nesting depth against a configured maximum.
///
/// # Arguments
///
/// * `current_depth` - Current nesting depth
/// * `max_depth` - Maximum allowed nesting depth
#[allow(dead_code)]
pub fn check_depth_limit(current_depth: usize, max_depth: usize) -> Result<(), CoreError> {
    if current_depth > max_depth {
        return Err(resource_limit_exceeded(
            "nesting_depth",
            current_depth,
            max_depth,
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn out_of_memory_creation() {
        let error = out_of_memory("Buffer allocation failed");
        assert!(matches!(error, CoreError::OutOfMemory(_)));
    }

    #[test]
    fn resource_limit_creation() {
        let error = resource_limit_exceeded("memory", 1000, 500);
        assert!(matches!(error, CoreError::ResourceLimitExceeded { .. }));
    }

    #[test]
    fn feature_not_supported_creation() {
        let error = feature_not_supported("simd", "simd");
        assert!(matches!(error, CoreError::FeatureNotSupported { .. }));
    }

    #[test]
    fn memory_limit_within_bounds() {
        assert!(check_memory_limit(100, 50, 200).is_ok());
        assert!(check_memory_limit(0, 100, 100).is_ok());
    }

    #[test]
    fn memory_limit_exceeded() {
        assert!(check_memory_limit(100, 150, 200).is_err());
        assert!(check_memory_limit(150, 100, 200).is_err());
    }

    #[test]
    fn memory_limit_overflow() {
        let result = check_memory_limit(usize::MAX, 1, usize::MAX);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CoreError::OutOfMemory(_)));
    }

    #[test]
    fn time_limit_check() {
        assert!(check_time_limit(50, 100).is_ok());
        assert!(check_time_limit(150, 100).is_err());
    }

    #[test]
    fn input_size_limit_check() {
        assert!(check_input_size_limit(500, 1000).is_ok());
        assert!(check_input_size_limit(1500, 1000).is_err());
    }

    #[test]
    fn depth_limit_check() {
        assert!(check_depth_limit(5, 10).is_ok());
        assert!(check_depth_limit(15, 10).is_err());
    }
}
