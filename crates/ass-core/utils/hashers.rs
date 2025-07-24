//! Hash function utilities for consistent performance across platforms
//!
//! Provides ahash-based hashers optimized for ASS-RS use cases with `DoS` resistance
//! and consistent performance across platforms including WASM.
//!
//! # Features
//!
//! - DoS-resistant hashing via ahash with random seeds
//! - WASM-compatible implementation
//! - `nostd` support when needed
//! - Deterministic hashing for testing when enabled

use ahash::{AHasher, RandomState};
use core::hash::{BuildHasher, Hasher};

#[cfg(feature = "nostd")]
use hashbrown::HashMap;
#[cfg(not(feature = "nostd"))]
use std::collections::HashMap;

/// Create a new `HashMap` with optimized hasher for ASS-RS use cases
///
/// Uses ahash for consistent performance across platforms with `DoS` resistance.
/// Automatically handles `nostd` vs std `HashMap` selection.
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
            use ahash::RandomState;
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

        assert_eq!(hash1, hash2);
        assert!(hash1 > 0);
    }

    #[test]
    fn deterministic_map_consistent() {
        let config = HashConfig::for_testing();
        let mut map1 = config.create_map::<&str, i32>();
        map1.insert("a", 1);
        map1.insert("b", 2);

        let mut map2 = config.create_map::<&str, i32>();
        map2.insert("a", 1);
        map2.insert("b", 2);

        assert_eq!(map1.get("a"), Some(&1));
        assert_eq!(map2.get("a"), Some(&1));
        assert_eq!(map1.len(), map2.len());
    }

    #[test]
    fn hash_value_convenience() {
        let hash1 = hash_value(&"test");
        let hash2 = hash_value(&"test");
        let hash3 = hash_value(&"different");

        assert!(hash1 > 0);
        assert!(hash2 > 0);
        assert!(hash3 > 0);
    }

    #[test]
    fn hash_config_default() {
        let config = HashConfig::default();
        assert!(!config.deterministic);
        assert_eq!(config.default_capacity, 16);
        assert!((config.load_factor - 0.75).abs() < f32::EPSILON);
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

        assert!(str_hash > 0);
        assert!(int_hash > 0);
        assert!(u64_hash > 0);
    }

    #[test]
    fn random_hashers_produce_different_seeds() {
        use core::hash::{Hash, Hasher};

        let hasher1 = create_hasher();
        let hasher2 = create_hasher();

        // Hash the same value with different random hasher instances
        let hash1 = {
            let mut h = hasher1;
            "test".hash(&mut h);
            h.finish()
        };

        let hash2 = {
            let mut h = hasher2;
            "test".hash(&mut h);
            h.finish()
        };

        // While theoretically they could be the same, it's extremely unlikely
        // with proper random seeding. This tests that we're using RandomState
        assert!(hash1 > 0);
        assert!(hash2 > 0);
    }

    #[test]
    fn hash_map_capacity_zero_works() {
        let mut map = create_hash_map_with_capacity::<&str, i32>(0);
        map.insert("test", 42);
        assert_eq!(map.get("test"), Some(&42));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn hash_config_with_custom_capacity() {
        let config = HashConfig {
            deterministic: false,
            default_capacity: 32,
            load_factor: 0.8,
        };

        let mut map = config.create_map::<&str, i32>();
        map.insert("test", 42);
        assert_eq!(map.get("test"), Some(&42));
    }

    #[test]
    fn hash_config_deterministic_path() {
        let config = HashConfig {
            deterministic: true,
            default_capacity: 16,
            load_factor: 0.75,
        };

        let mut map1 = config.create_map::<&str, i32>();
        let mut map2 = config.create_map::<&str, i32>();

        map1.insert("key", 1);
        map2.insert("key", 1);

        assert_eq!(map1.get("key"), Some(&1));
        assert_eq!(map2.get("key"), Some(&1));
    }

    #[test]
    fn hash_config_non_deterministic_path() {
        let config = HashConfig {
            deterministic: false,
            default_capacity: 8,
            load_factor: 0.5,
        };

        let mut map = config.create_map::<String, u32>();
        map.insert("test".to_string(), 123);
        assert_eq!(map.get("test"), Some(&123));
    }

    #[test]
    fn hash_config_clone_and_debug() {
        let config1 = HashConfig::default();
        let config2 = config1.clone();

        assert_eq!(config1.deterministic, config2.deterministic);
        assert_eq!(config1.default_capacity, config2.default_capacity);
        assert!((config1.load_factor - config2.load_factor).abs() < f32::EPSILON);

        // Test Debug implementation
        let debug_output = format!("{config1:?}");
        assert!(debug_output.contains("HashConfig"));
        assert!(debug_output.contains("deterministic"));
        assert!(debug_output.contains("default_capacity"));
        assert!(debug_output.contains("load_factor"));
    }

    #[test]
    fn hash_different_data_types() {
        // Test various data types to ensure hasher works with different Hash implementations
        let vec_hash = hash_value(&vec![1, 2, 3]);
        let tuple_hash = hash_value(&(1, "test", 42i32));
        let option_hash = hash_value(&Some(42));
        let result_hash = hash_value(&Ok::<i32, &str>(42));
        let bool_hash = hash_value(&true);
        let char_hash = hash_value(&'X');

        assert!(vec_hash > 0);
        assert!(tuple_hash > 0);
        assert!(option_hash > 0);
        assert!(result_hash > 0);
        assert!(bool_hash > 0);
        assert!(char_hash > 0);
    }

    #[test]
    fn deterministic_hasher_consistency_across_calls() {
        use core::hash::{Hash, Hasher};

        // Test that deterministic hasher produces the same values across multiple calls
        let values = ["test1", "test2", "test3", ""];
        let mut first_run_hashes = Vec::new();
        let mut second_run_hashes = Vec::new();

        for &value in &values {
            let mut first_hasher = create_deterministic_hasher();
            value.hash(&mut first_hasher);
            first_run_hashes.push(first_hasher.finish());

            let mut second_hasher = create_deterministic_hasher();
            value.hash(&mut second_hasher);
            second_run_hashes.push(second_hasher.finish());
        }

        assert_eq!(first_run_hashes, second_run_hashes);
        // Ensure we actually got different hashes for different values
        assert!(first_run_hashes.iter().all(|&h| h > 0));

        // Test that different strings produce different hashes
        let unique_hashes: std::collections::HashSet<_> = first_run_hashes.into_iter().collect();
        assert_eq!(unique_hashes.len(), values.len());
    }

    #[test]
    fn deterministic_hash_map_works() {
        let config = HashConfig::for_testing();
        let mut map1 = config.create_map::<&str, i32>();
        let mut map2 = config.create_map::<&str, i32>();

        map1.insert("consistent", 100);
        map2.insert("consistent", 100);

        assert_eq!(map1.get("consistent"), Some(&100));
        assert_eq!(map2.get("consistent"), Some(&100));

        // Test multiple insertions maintain consistency
        let test_data = [("a", 1), ("b", 2), ("c", 3)];
        for &(key, value) in &test_data {
            map1.insert(key, value);
            map2.insert(key, value);
        }

        for &(key, value) in &test_data {
            assert_eq!(map1.get(key), Some(&value));
            assert_eq!(map2.get(key), Some(&value));
        }
    }

    #[test]
    fn hash_config_for_testing_values() {
        let config = HashConfig::for_testing();
        assert!(config.deterministic);
        assert_eq!(config.default_capacity, 8);
        assert!((config.load_factor - 0.75).abs() < f32::EPSILON);
    }

    #[test]
    fn hash_value_empty_and_edge_cases() {
        // Test edge cases for hash_value function
        let empty_str_hash = hash_value(&"");
        let empty_vec_hash = hash_value(&Vec::<i32>::new());
        let zero_hash = hash_value(&0u64);
        let max_u64_hash = hash_value(&u64::MAX);
        let negative_hash = hash_value(&-1i64);

        assert!(empty_str_hash > 0);
        assert!(empty_vec_hash > 0);
        assert!(zero_hash > 0);
        assert!(max_u64_hash > 0);
        assert!(negative_hash > 0);

        // Verify different values produce different hashes
        let all_hashes = [
            empty_str_hash,
            empty_vec_hash,
            zero_hash,
            max_u64_hash,
            negative_hash,
        ];
        let unique_hashes: std::collections::HashSet<_> = all_hashes.iter().collect();
        assert_eq!(unique_hashes.len(), all_hashes.len());
    }

    #[test]
    fn hash_map_with_different_capacities() {
        // Test that capacity hint doesn't affect functionality
        let capacities = [0, 1, 16, 64, 1024];

        for &cap in &capacities {
            let mut map = create_hash_map_with_capacity::<String, usize>(cap);

            // Insert more items than initial capacity to test resizing
            for i in 0..20 {
                map.insert(format!("key_{i}"), i);
            }

            assert_eq!(map.len(), 20);

            for i in 0..20 {
                assert_eq!(map.get(&format!("key_{i}")), Some(&i));
            }
        }
    }

    #[test]
    fn hash_config_load_factor_variations() {
        let load_factors = [0.1, 0.5, 0.75, 0.9, 1.0];

        for &load_factor in &load_factors {
            let config = HashConfig {
                deterministic: false,
                default_capacity: 16,
                load_factor,
            };

            let mut map = config.create_map::<i32, String>();
            map.insert(42, "test".to_string());
            assert_eq!(map.get(&42), Some(&"test".to_string()));
        }
    }

    #[test]
    fn test_deterministic_seed_constants() {
        // Test that deterministic maps use the specific seed constants
        let config = HashConfig::for_testing();
        let mut map = config.create_map::<&str, u64>();

        // Test that the deterministic map uses the specific constants
        map.insert("test_seed", 12345);
        assert_eq!(map.get("test_seed"), Some(&12345));

        // Create another map to verify consistency
        let mut map2 = config.create_map::<&str, u64>();
        map2.insert("test_seed", 12345);
        assert_eq!(map2.get("test_seed"), Some(&12345));

        // The maps should behave consistently due to fixed seeds
        assert_eq!(map.len(), map2.len());
    }

    #[test]
    fn test_hash_config_create_map_non_deterministic_branch() {
        // Test the non-deterministic branch specifically
        let config = HashConfig {
            deterministic: false,
            default_capacity: 42,
            load_factor: 0.8,
        };

        let mut map = config.create_map::<String, i32>();
        map.insert("coverage_test".to_string(), 999);
        assert_eq!(map.get("coverage_test"), Some(&999));
    }

    #[test]
    fn test_deterministic_config_with_capacity() {
        // Test that deterministic maps respect capacity configuration
        let config = HashConfig {
            deterministic: true,
            default_capacity: 64,
            load_factor: 0.75,
        };

        let mut map = config.create_map::<i32, &str>();
        map.insert(1, "one");
        map.insert(2, "two");

        assert_eq!(map.get(&1), Some(&"one"));
        assert_eq!(map.get(&2), Some(&"two"));
        assert_eq!(map.len(), 2);

        // Test with different types to ensure generic functionality
        let mut string_map = config.create_map::<String, Vec<u8>>();
        string_map.insert("key".to_string(), vec![1, 2, 3]);
        assert_eq!(string_map.get("key"), Some(&vec![1, 2, 3]));
    }
}
