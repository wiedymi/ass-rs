//! Benchmark simple text search (case sensitivity and whole-word matching).

use crate::common::generate_search_script;
use ass_editor::{
    core::EditorDocument,
    utils::search::{DocumentSearch, DocumentSearchImpl, SearchOptions, SearchScope},
};
use criterion::{black_box, BenchmarkId, Criterion};

/// Benchmark simple text search
pub fn bench_simple_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("simple_search");

    for doc_size in [50, 200, 1000].iter() {
        let script = generate_search_script(*doc_size);

        group.bench_with_input(
            BenchmarkId::new("case_sensitive", doc_size),
            doc_size,
            |b, _| {
                b.iter_batched(
                    || EditorDocument::from_content(&script).unwrap(),
                    |doc| {
                        let mut search = DocumentSearchImpl::new();
                        search.build_index(&doc).unwrap();
                        let options = SearchOptions {
                            case_sensitive: true,
                            whole_words: false,
                            use_regex: false,
                            scope: SearchScope::All,
                            max_results: 100,
                        };

                        let results = search.search("text", &options).unwrap();
                        black_box(results.len())
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );

        group.bench_with_input(
            BenchmarkId::new("case_insensitive", doc_size),
            doc_size,
            |b, _| {
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
                            max_results: 100,
                        };

                        let results = search.search("TEXT", &options).unwrap();
                        black_box(results.len())
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );

        group.bench_with_input(
            BenchmarkId::new("whole_words", doc_size),
            doc_size,
            |b, _| {
                b.iter_batched(
                    || EditorDocument::from_content(&script).unwrap(),
                    |doc| {
                        let mut search = DocumentSearchImpl::new();
                        search.build_index(&doc).unwrap();
                        let options = SearchOptions {
                            case_sensitive: false,
                            whole_words: true,
                            use_regex: false,
                            scope: SearchScope::All,
                            max_results: 100,
                        };

                        let results = search.search("test", &options).unwrap();
                        black_box(results.len())
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}
