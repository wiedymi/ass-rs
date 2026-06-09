//! Benchmark search result retrieval with varying result limits.

use crate::common::generate_search_script;
use ass_editor::{
    core::EditorDocument,
    utils::search::{DocumentSearch, DocumentSearchImpl, SearchOptions, SearchScope},
};
use criterion::{black_box, Criterion};

/// Benchmark search result retrieval
pub fn bench_result_retrieval(c: &mut Criterion) {
    let mut group = c.benchmark_group("result_retrieval");

    let script = generate_search_script(1000);

    // Limited results
    group.bench_function("max_10_results", |b| {
        b.iter_batched(
            || EditorDocument::from_content(&script).unwrap(),
            |doc| {
                let mut search = DocumentSearchImpl::new();
                search.build_index(&doc).unwrap();
                let options = SearchOptions {
                    case_sensitive: false,
                    whole_words: false,
                    use_regex: false,
                    scope: SearchScope::All,
                    max_results: 10,
                };

                let results = search.search("the", &options).unwrap();
                black_box(results.len())
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // Many results
    group.bench_function("max_1000_results", |b| {
        b.iter_batched(
            || EditorDocument::from_content(&script).unwrap(),
            |doc| {
                let mut search = DocumentSearchImpl::new();
                search.build_index(&doc).unwrap();
                let options = SearchOptions {
                    case_sensitive: false,
                    whole_words: false,
                    use_regex: false,
                    scope: SearchScope::All,
                    max_results: 1000,
                };

                let results = search.search("e", &options).unwrap();
                black_box(results.len())
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}
