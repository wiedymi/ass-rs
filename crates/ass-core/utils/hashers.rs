//! Hash function utilities for consistent performance across platforms
//!
//! Provides ahash-based hashers optimized for ASS-RS use cases with `DoS` resistance
//! and consistent performance across platforms including WASM.
//!
//! # Features
//!
//! - DoS-resistant hashing via ahash with random seeds
//! - WASM-compatible implementation
//! - `no_std` support when needed
//! - Deterministic hashing for testing when enabled

use ahash::{AHasher, RandomState};
use core::hash::{BuildHasher, Hasher};

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use hashbrown::HashMap;
#[cfg(feature = "std")]
use std::collections::HashMap;
/// Create a new `HashMap` with optimized hasher for ASS-RS use cases
///
/// Uses ahash for consistent performance across platforms with `DoS` resistance.
/// Automatically handles `no_std` vs std `HashMap` selection.
///
/// # Example
///
/// ```rust
/// use ass_core::utils::hashers::create_hash_map;
///
/// let mut map = create_hash_map::<String, i32>();
/// map.insert("key".to_string(), 42);
/// ```
#[must_use]
pub fn create_hash_map<K, V>() -> HashMap<K, V, RandomState> {
    HashMap::with_hasher(RandomState::new())
}

/// Create a new `HashMap` with specific capacity and optimized hasher
///
/// Pre-allocates the specified capacity to avoid rehashing during construction.
/// Useful when the approximate size is known in advance.
///
/// # Example
///
/// ```rust
/// use ass_core::utils::hashers::create_hash_map_with_capacity;
///
/// // Pre-allocate for expected 100 entries
/// let mut map = create_hash_map_with_capacity::<String, i32>(100);
/// ```
#[must_use]
pub fn create_hash_map_with_capacity<K, V>(capacity: usize) -> HashMap<K, V, RandomState> {
    HashMap::with_capacity_and_hasher(capacity, RandomState::new())
}

/// Create a hasher instance for manual hashing operations
///
/// Returns an ahash hasher with random seed for `DoS` resistance.
/// Use this when you need to hash individual values outside of `HashMap`.
///
/// # Example
///
/// ```rust
/// use std::hash::{Hash, Hasher};
/// use ass_core::utils::create_hasher;
///
/// let mut hasher = create_hasher();
/// "some string".hash(&mut hasher);
/// let hash_value = hasher.finish();
/// ```
#[must_use]
pub fn create_hasher() -> AHasher {
    RandomState::new().build_hasher()
}

/// Create a deterministic hasher for testing purposes
///
/// Uses a fixed seed to ensure reproducible hash values across test runs.
/// Should only be used in testing scenarios.
///
/// # Example
///
/// ```rust
/// use std::hash::Hash;
/// use ass_core::utils::create_deterministic_hasher;
///
/// #[cfg(test)]
/// let mut hasher = create_deterministic_hasher();
/// ```
#[cfg(test)]
#[must_use]
pub fn create_deterministic_hasher() -> AHasher {
    use ahash::RandomState;
    RandomState::with_seeds(0x1234_5678_9abc_def0, 0xfedc_ba98_7654_3210, 0, 0).build_hasher()
}

/// Hash a single value using the optimized hasher
///
/// Convenience function for hashing individual values with the same
/// hasher configuration used throughout ASS-RS.
///
/// # Example
///
/// ```rust
/// use ass_core::utils::hashers::hash_value;
///
/// let hash = hash_value(&"test string");
/// let hash2 = hash_value(&42u32);
/// ```
pub fn hash_value<T: core::hash::Hash>(value: &T) -> u64 {
    let mut hasher = create_hasher();
    value.hash(&mut hasher);
    hasher.finish()
}

/// Configuration for hash-related performance tuning
#[derive(Debug, Clone)]
pub struct HashConfig {
    /// Whether to use deterministic hashing (testing only)
    pub deterministic: bool,

    /// Initial capacity hint for `HashMaps`
    pub default_capacity: usize,

    /// Load factor before rehashing (0.0 to 1.0)
    pub load_factor: f32,
}

impl Default for HashConfig {
    fn default() -> Self {
        Self {
            deterministic: false,
            default_capacity: 16,
            load_factor: 0.75,
        }
    }
}

impl HashConfig {
    /// Create configuration for testing with deterministic behavior
    #[cfg(test)]
    #[must_use]
    pub const fn for_testing() -> Self {
        Self {
            deterministic: true,
            default_capacity: 8,
            load_factor: 0.75,
        }
    }

    /// Create `HashMap` using this configuration
    #[must_use]
    pub fn create_map<K, V>(&self) -> HashMap<K, V, RandomState> {
        if self.deterministic {
            HashMap::with_capacity_and_hasher(
                self.default_capacity,
                RandomState::with_seeds(0x1234_5678_9abc_def0, 0xfedc_ba98_7654_3210, 0, 0),
            )
        } else {
            create_hash_map_with_capacity(self.default_capacity)
        }
    }
}

#[cfg(test)]
mod tests {}
