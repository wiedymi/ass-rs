//! Benchmark index building and incremental update operations.

use crate::common::generate_search_script;
use ass_editor::{
    core::{EditorDocument, Position},
    utils::search::{DocumentSearch, DocumentSearchImpl, SearchOptions, SearchScope},
};
use criterion::{black_box, BenchmarkId, Criterion};

/// Benchmark index building and updates
pub fn bench_index_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("index_operations");

    // Initial index build
    for size in [100, 500, 1000].iter() {
        let script = generate_search_script(*size);

        group.bench_with_input(BenchmarkId::new("build_index", size), size, |b, _| {
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
                    // Trigger index use
                    let results = search.search("test", &options).unwrap();
                    black_box(results.len())
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    // Incremental updates - search after edits
    group.bench_function("search_after_edit", |b| {
        b.iter_batched(
            || EditorDocument::from_content(&generate_search_script(500)).unwrap(),
            |mut doc| {
                // Make an edit
                let pos = Position::new(1000);
                doc.insert(pos, "NEW TEXT").unwrap();

                // Search to test incremental update
                let options = SearchOptions {
                    case_sensitive: false,
                    whole_words: false,
                    use_regex: false,
                    scope: SearchScope::All,
                    max_results: 100,
                };

                let mut search = DocumentSearchImpl::new();
                search.build_index(&doc).unwrap();
                let results = search.search("NEW", &options).unwrap();
                black_box(results.len())
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}
