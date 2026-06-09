//! Benchmark regex-based search patterns.

use crate::common::generate_search_script;
use ass_editor::{
    core::EditorDocument,
    utils::search::{DocumentSearch, DocumentSearchImpl, SearchOptions, SearchScope},
};
use criterion::{black_box, Criterion};

/// Benchmark regex search
pub fn bench_regex_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("regex_search");

    let script = generate_search_script(500);

    // Simple regex
    group.bench_function("simple_pattern", |b| {
        b.iter_batched(
            || EditorDocument::from_content(&script).unwrap(),
            |doc| {
                let mut search = DocumentSearchImpl::new();
                search.build_index(&doc).unwrap();
                let options = SearchOptions {
                    case_sensitive: true,
                    whole_words: false,
                    use_regex: true,
                    scope: SearchScope::All,
                    max_results: 100,
                };

                let results = search.search(r"\b\w{5}\b", &options).unwrap();
                black_box(results.len())
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // Complex regex
    group.bench_function("complex_pattern", |b| {
        b.iter_batched(
            || EditorDocument::from_content(&script).unwrap(),
            |doc| {
                let mut search = DocumentSearchImpl::new();
                search.build_index(&doc).unwrap();
                let options = SearchOptions {
                    case_sensitive: true,
                    whole_words: false,
                    use_regex: true,
                    scope: SearchScope::All,
                    max_results: 100,
                };

                // Match ASS tags
                let results = search.search(r"\{\\[^}]+\}", &options).unwrap();
                black_box(results.len())
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}
