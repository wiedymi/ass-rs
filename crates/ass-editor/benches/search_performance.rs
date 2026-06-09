//! Benchmarks for search functionality
//!
//! Tests the performance of:
//! - FST-based indexed search
//! - Regular expression search
//! - Search with different scopes
//! - Index building and updates

use criterion::{criterion_group, criterion_main};

#[path = "search_performance/common.rs"]
mod common;

#[path = "search_performance/simple_search.rs"]
mod simple_search;

#[cfg(feature = "formats")]
#[path = "search_performance/regex_search.rs"]
mod regex_search;

#[path = "search_performance/search_scopes.rs"]
mod search_scopes;

#[cfg(feature = "search-index")]
#[path = "search_performance/index_operations.rs"]
mod index_operations;

#[path = "search_performance/result_retrieval.rs"]
mod result_retrieval;

#[cfg(all(feature = "search-index", feature = "formats"))]
criterion_group!(
    benches,
    simple_search::bench_simple_search,
    regex_search::bench_regex_search,
    search_scopes::bench_search_scopes,
    index_operations::bench_index_operations,
    result_retrieval::bench_result_retrieval
);

#[cfg(all(feature = "search-index", not(feature = "formats")))]
criterion_group!(
    benches,
    simple_search::bench_simple_search,
    search_scopes::bench_search_scopes,
    index_operations::bench_index_operations,
    result_retrieval::bench_result_retrieval
);

#[cfg(all(not(feature = "search-index"), feature = "formats"))]
criterion_group!(
    benches,
    simple_search::bench_simple_search,
    regex_search::bench_regex_search,
    search_scopes::bench_search_scopes,
    result_retrieval::bench_result_retrieval
);

#[cfg(all(not(feature = "search-index"), not(feature = "formats")))]
criterion_group!(
    benches,
    simple_search::bench_simple_search,
    search_scopes::bench_search_scopes,
    result_retrieval::bench_result_retrieval
);

criterion_main!(benches);
