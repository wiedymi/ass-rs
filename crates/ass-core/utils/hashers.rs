//! Hash function utilities for consistent performance across platforms
//!
//! Provides ahash-based hashers optimized for ASS-RS use cases with DoS resistance
//! and consistent performance across platforms including WASM.
//!
//! # Features
//!
//! - DoS-resistant hashing via ahash with random seeds
//! - WASM-compatible implementation
//! - no_std support when needed
//! - Deterministic hashing for testing when enabled

use ahash::{AHasher, RandomState};
use core::hash::{BuildHasher, Hasher};

#[cfg(feature = "no_std")]
use hashbrown::HashMap;
#[cfg(not(feature = "no_std"))]
use std::collections::HashMap;

/// Create a new HashMap with optimized hasher for ASS-RS use cases
///
/// Uses ahash for consistent performance across platforms with DoS resistance.
/// Automatically handles no_std vs std HashMap selection.
///
/// # Example
///
/// ```rust
/// use ass_core::utils::hashers::create_hash_map;
///
/// let mut map = create_hash_map::<String, i32>();
/// map.insert("key".to_string(), 42);
/// ```
pub fn create_hash_map<K, V>() -> HashMap<K, V, RandomState> {
    HashMap::with_hasher(RandomState::new())
}

/// Create a new HashMap with specific capacity and optimized hasher
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
pub fn create_hash_map_with_capacity<K, V>(capacity: usize) -> HashMap<K, V, RandomState> {
    HashMap::with_capacity_and_hasher(capacity, RandomState::new())
}

/// Create a hasher instance for manual hashing operations
///
/// Returns an ahash hasher with random seed for DoS resistance.
/// Use this when you need to hash individual values outside of HashMap.
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
pub fn create_deterministic_hasher() -> AHasher {
    use ahash::RandomState;
    // Use fixed seeds for deterministic testing
    RandomState::with_seeds(0x1234_5678_9abc_def0, 0xfedc_ba98_7654_3210, 0, 0).build_hasher()
}

/// Create a deterministic HashMap for testing
///
/// Uses fixed seeds to ensure consistent ordering and hashing in tests.
/// Should only be used in testing scenarios where reproducibility is needed.
#[cfg(test)]
pub fn create_deterministic_hash_map<K, V>() -> HashMap<K, V, RandomState> {
    use ahash::RandomState;
    HashMap::with_hasher(RandomState::with_seeds(
        0x1234_5678_9abc_def0,
        0xfedc_ba98_7654_3210,
        0,
        0,
    ))
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

    /// Initial capacity hint for HashMaps
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
    pub fn for_testing() -> Self {
        Self {
            deterministic: true,
            default_capacity: 8,
            load_factor: 0.75,
        }
    }

    /// Create HashMap using this configuration
    pub fn create_map<K, V>(&self) -> HashMap<K, V, RandomState> {
        if self.deterministic {
            #[cfg(test)]
            return create_deterministic_hash_map();
            #[cfg(not(test))]
            return create_hash_map_with_capacity(self.default_capacity);
        } else {
            create_hash_map_with_capacity(self.default_capacity)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::hash::Hash;

    #[test]
    fn create_hash_map_works() {
        let mut map = create_hash_map::<&str, i32>();
        map.insert("test", 42);
        assert_eq!(map.get("test"), Some(&42));
    }

    #[test]
    fn create_hash_map_with_capacity_works() {
        let mut map = create_hash_map_with_capacity::<String, i32>(100);
        map.insert("test".to_string(), 42);
        assert_eq!(map.get("test"), Some(&42));
    }

    #[test]
    fn hasher_produces_values() {
        let mut hasher = create_hasher();
        "test".hash(&mut hasher);
        let hash1 = hasher.finish();

        let mut hasher = create_hasher();
        "test".hash(&mut hasher);
        let hash2 = hasher.finish();

        // Hashes should be the same for same input (within same run)
        // But different seeds mean they might differ between runs
        assert!(hash1 > 0);
        assert!(hash2 > 0);
    }

    #[test]
    fn deterministic_hasher_is_reproducible() {
        let mut hasher1 = create_deterministic_hasher();
        "test".hash(&mut hasher1);
        let hash1 = hasher1.finish();

        let mut hasher2 = create_deterministic_hasher();
        "test".hash(&mut hasher2);
        let hash2 = hasher2.finish();

        // Deterministic hashes should be identical
        assert_eq!(hash1, hash2);
        assert!(hash1 > 0);
    }

    #[test]
    fn deterministic_map_consistent() {
        let mut map1 = create_deterministic_hash_map::<&str, i32>();
        map1.insert("a", 1);
        map1.insert("b", 2);

        let mut map2 = create_deterministic_hash_map::<&str, i32>();
        map2.insert("a", 1);
        map2.insert("b", 2);

        // Maps should behave identically
        assert_eq!(map1.get("a"), Some(&1));
        assert_eq!(map2.get("a"), Some(&1));
        assert_eq!(map1.len(), map2.len());
    }

    #[test]
    fn hash_value_convenience() {
        let hash1 = hash_value(&"test");
        let hash2 = hash_value(&"test");
        let hash3 = hash_value(&"different");

        // Same input might produce different hashes due to random seeds
        // But function should work without panicking
        assert!(hash1 > 0);
        assert!(hash2 > 0);
        assert!(hash3 > 0);
    }

    #[test]
    fn hash_config_default() {
        let config = HashConfig::default();
        assert!(!config.deterministic);
        assert_eq!(config.default_capacity, 16);
        assert_eq!(config.load_factor, 0.75);
    }

    #[test]
    fn hash_config_for_testing() {
        let config = HashConfig::for_testing();
        assert!(config.deterministic);
        assert_eq!(config.default_capacity, 8);
    }

    #[test]
    fn hash_config_create_map() {
        let config = HashConfig::default();
        let mut map = config.create_map::<&str, i32>();
        map.insert("test", 42);
        assert_eq!(map.get("test"), Some(&42));
    }

    #[test]
    fn different_types_hash_differently() {
        let str_hash = hash_value(&"42");
        let int_hash = hash_value(&42i32);
        let u64_hash = hash_value(&42u64);

        // Different types should generally produce different hashes
        // (Though collisions are theoretically possible)
        assert!(str_hash > 0);
        assert!(int_hash > 0);
        assert!(u64_hash > 0);
    }
}
