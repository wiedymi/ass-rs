//! Public validation API and cache management for `LazyValidator`.
//!
//! Implements the on-demand validation entry points along with cache lookup,
//! invalidation, and content hashing used to avoid redundant work.

use super::{LazyValidator, ValidationResult};
use crate::core::{errors::EditorError, EditorDocument, Result};

#[cfg(feature = "std")]
use std::time::Instant;

impl LazyValidator {
    /// Validate document using ass-core's ScriptAnalysis
    pub fn validate(&mut self, document: &EditorDocument) -> Result<&ValidationResult> {
        let content = document.text();
        let content_hash = self.calculate_hash(&content);

        // Check if we can use cached result
        if self.should_use_cache(content_hash) {
            return self.cached_result.as_ref().ok_or_else(|| {
                EditorError::command_failed(
                    "Cache validation inconsistency: cached result expected but not found",
                )
            });
        }

        #[cfg(feature = "std")]
        let start_time = Instant::now();

        // Perform validation using ass-core
        let issues = self.validate_with_core(&content, document)?;

        // Update cache
        #[cfg(feature = "std")]
        let mut result = ValidationResult::new(issues);
        #[cfg(not(feature = "std"))]
        let result = ValidationResult::new(issues);

        #[cfg(feature = "std")]
        {
            result.validation_time_us = start_time.elapsed().as_micros() as u64;
        }

        self.cached_result = Some(result);
        self.content_hash = content_hash;

        #[cfg(feature = "std")]
        {
            self.last_validation = Some(Instant::now());
        }

        self.cached_result.as_ref().ok_or_else(|| {
            EditorError::command_failed("Validation completed but cached result is missing")
        })
    }

    /// Force validation even if cached result exists
    pub fn force_validate(&mut self, document: &EditorDocument) -> Result<&ValidationResult> {
        self.cached_result = None; // Clear cache
        self.validate(document)
    }

    /// Check if document is valid (quick check using cache if available)
    pub fn is_valid(&mut self, document: &EditorDocument) -> Result<bool> {
        Ok(self.validate(document)?.is_valid)
    }

    /// Get cached validation result without revalidating
    pub fn cached_result(&self) -> Option<&ValidationResult> {
        self.cached_result.as_ref()
    }

    /// Clear validation cache
    pub fn clear_cache(&mut self) {
        self.cached_result = None;
        self.content_hash = 0;
        #[cfg(feature = "std")]
        {
            self.last_validation = None;
        }
    }

    /// Check if cached result can be used
    fn should_use_cache(&self, content_hash: u64) -> bool {
        if self.cached_result.is_none() || self.content_hash != content_hash {
            return false;
        }

        #[cfg(feature = "std")]
        {
            if let Some(last_validation) = self.last_validation {
                return last_validation.elapsed() < self.config.min_validation_interval;
            }
        }

        true
    }

    /// Calculate hash of content for cache invalidation
    fn calculate_hash(&self, content: &str) -> u64 {
        // Simple FNV hash
        let mut hash = 0xcbf29ce484222325u64;
        for byte in content.bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash
    }
}
