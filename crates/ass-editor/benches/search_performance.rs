//! Benchmarks for search functionality
//!
//! Tests the performance of:
//! - FST-based indexed search
//! - Regular expression search
//! - Search with different scopes
//! - Index building and updates

use ass_editor::{
    core::{EditorDocument, Position},
    utils::search::{DocumentSearch, DocumentSearchImpl, SearchOptions, SearchScope},
};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

/// Generate a large script with varied content for search testing
fn generate_search_script(events: usize) -> String {
    let mut script = String::from(
        r#"[Script Info]
Title: Search Benchmark Script
ScriptType: v4.00+
WrapStyle: 0
ScaledBorderAndShadow: yes
YCbCr Matrix: None
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: Title,Arial,32,&H00FFFF00,&H000000FF,&H00000000,&H00000000,1,0,0,0,100,100,0,0,1,3,0,2,10,10,10,1
Style: Sign,Arial,18,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,8,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
"#,
    );

    // Add varied events for realistic search scenarios
    let dialogues = [
        "Hello world! This is a test dialogue.",
        "The quick brown fox jumps over the lazy dog.",
        "Advanced SubStation Alpha subtitle format.",
        r"{\pos(960,540)}Positioned text in the center.",
        r"{\fade(255,0,255,0,0,500,1000)}Fading in and out.",
        r"{\k50}Ka{\k30}ra{\k40}o{\k35}ke timing test.",
        r"Multiple {\b1}bold{\b0} and {\i1}italic{\i0} tags.",
        "Sign text appears here with special styling.",
        "Another line with different content for searching.",
        r"Complex effects: {\move(100,100,500,500)}{\c&H00FF00&}Green moving text.",
    ];

    for i in 0..events {
        let style = match i % 10 {
            0..=6 => "Default",
            7..=8 => "Title",
            _ => "Sign",
        };
        let dialogue = &dialogues[i % dialogues.len()];
        let start_time = format!("0:{:02}:{:02}.00", i / 60, i % 60);
        let end_time = format!("0:{:02}:{:02}.00", (i + 5) / 60, (i + 5) % 60);

        script.push_str(&format!(
            "Dialogue: 0,{start_time},{end_time},{style},,0,0,0,,{dialogue}\n"
        ));
    }

    script
}

/// Benchmark simple text search
fn bench_simple_search(c: &mut Criterion) {
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

/// Benchmark regex search
#[cfg(feature = "formats")]
fn bench_regex_search(c: &mut Criterion) {
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

/// Benchmark search with different scopes
fn bench_search_scopes(c: &mut Criterion) {
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

/// Benchmark index building and updates
#[cfg(feature = "search-index")]
fn bench_index_operations(c: &mut Criterion) {
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

/// Benchmark search result retrieval
fn bench_result_retrieval(c: &mut Criterion) {
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

#[cfg(all(feature = "search-index", feature = "formats"))]
criterion_group!(
    benches,
    bench_simple_search,
    bench_regex_search,
    bench_search_scopes,
    bench_index_operations,
    bench_result_retrieval
);

#[cfg(all(feature = "search-index", not(feature = "formats")))]
criterion_group!(
    benches,
    bench_simple_search,
    bench_search_scopes,
    bench_index_operations,
    bench_result_retrieval
);

#[cfg(all(not(feature = "search-index"), feature = "formats"))]
criterion_group!(
    benches,
    bench_simple_search,
    bench_regex_search,
    bench_search_scopes,
    bench_result_retrieval
);

#[cfg(all(not(feature = "search-index"), not(feature = "formats")))]
criterion_group!(
    benches,
    bench_simple_search,
    bench_search_scopes,
    bench_result_retrieval
);

criterion_main!(benches);
