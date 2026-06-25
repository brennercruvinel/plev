//! parser hot path: transpile the studied gpui separator widget end to end
//! (parse -> resolve -> emit). uses the committed fixture so the bench runs
//! anywhere the crate builds, no external sources needed.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use parser::transpile_gpui;

const SEPARATOR: &str = include_str!("../fixtures/gpui/separator.rs");

fn bench_transpile(c: &mut Criterion) {
    c.bench_function("parser_transpile_gpui_separator", |b| {
        b.iter(|| transpile_gpui(("separator.rs", black_box(SEPARATOR))).expect("transpile"));
    });
}

criterion_group!(benches, bench_transpile);
criterion_main!(benches);
