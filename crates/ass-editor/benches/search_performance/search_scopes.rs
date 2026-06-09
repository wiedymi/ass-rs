//! Benchmark search with different scopes (document, lines, sections).

use crate::common::generate_search_script;
use ass_editor::{
    core::EditorDocument,
    utils::search::{DocumentSearch, DocumentSearchImpl, SearchOptions, SearchScope},
};
use criterion::{black_box, Criterion};

/// Benchmark search with different scopes
pub fn bench_search_scopes(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_scopes");

    let script = generate_search_script(500);

    // Search entire document
    group.bench_function("scope_all", |b| {
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

                let results = search.search("dialogue", &options).unwrap();
                black_box(results.len())
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // Search specific lines
    group.bench_function("scope_lines", |b| {
        b.iter_batched(
            || EditorDocument::from_content(&script).unwrap(),
            |doc| {
                let mut search = DocumentSearchImpl::new();
                search.build_index(&doc).unwrap();
                let options = SearchOptions {
                    case_sensitive: false,
                    whole_words: false,
                    use_regex: false,
                    scope: SearchScope::Lines {
                        start: 50,
                        end: 150,
                    },
                    max_results: 100,
                };

                let results = search.search("dialogue", &options).unwrap();
                black_box(results.len())
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // Search specific sections
    group.bench_function("scope_sections", |b| {
        b.iter_batched(
            || EditorDocument::from_content(&script).unwrap(),
            |doc| {
                let mut search = DocumentSearchImpl::new();
                search.build_index(&doc).unwrap();
                let options = SearchOptions {
                    case_sensitive: false,
                    whole_words: false,
                    use_regex: false,
                    scope: SearchScope::Sections(vec!["Events".to_string()]),
                    max_results: 100,
                };

                let results = search.search("Style", &options).unwrap();
                black_box(results.len())
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}
