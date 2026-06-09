//! Benchmarks for large document creation, parsing, and cloning.

use crate::common::generate_large_script;
use ass_editor::core::EditorDocument;
use criterion::{black_box, BenchmarkId, Criterion, Throughput};

/// Benchmark large document creation and parsing
pub fn bench_large_document_ops(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_document_ops");

    for size in [1000, 5000, 10000].iter() {
        let script = generate_large_script(*size, 50);
        let script_size = script.len();

        group.throughput(Throughput::Bytes(script_size as u64));
        group.bench_with_input(BenchmarkId::new("parse", size), size, |b, _| {
            b.iter(|| black_box(EditorDocument::from_content(&script).unwrap()));
        });

        group.bench_with_input(BenchmarkId::new("clone", size), size, |b, _| {
            let doc = EditorDocument::from_content(&script).unwrap();
            b.iter(|| black_box(EditorDocument::from_content(&doc.text()).unwrap()));
        });
    }

    group.finish();
}
