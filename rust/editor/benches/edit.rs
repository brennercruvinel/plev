//! rope edit hot paths: building a document from a large body, and a
//! bounded insert/delete roundtrip that keeps the rope size stable across
//! iterations (so the bench measures edit cost, not unbounded growth).

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use rope::{Document, Transaction};

fn big_body(lines: usize) -> String {
    (0..lines)
        .map(|i| format!("line {i}: the quick brown fox jumps over the lazy dog\n"))
        .collect()
}

fn doc_with(lines: usize) -> Document {
    let mut d = Document::new();
    d.apply(Transaction::insert(0, &big_body(lines)));
    d
}

fn bench_rope(c: &mut Criterion) {
    c.bench_function("rope_build_5k_lines", |b| {
        b.iter(|| black_box(doc_with(5_000)));
    });

    c.bench_function("rope_insert_delete_roundtrip", |b| {
        let mut d = doc_with(1_000);
        b.iter(|| {
            d.apply(Transaction::insert(10, black_box("abc")));
            d.apply(Transaction::delete(10..13));
        });
    });
}

criterion_group!(benches, bench_rope);
criterion_main!(benches);
