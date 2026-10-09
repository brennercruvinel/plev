//! What one probe costs, off and on: a tight loop of the two probe kinds
//! (`span` with items, then its drop; `count`), the best of a few rounds
//! so a busy machine reads as close to the floor as it can.

use std::fmt::Write as _;
use std::hint::black_box;

use engine::perf::stages::{self, Stage};
use web_time::Instant;

/// Probes per round.
const LOOPS: u64 = 10_000_000;

/// Rounds; the best one is reported.
const ROUNDS: usize = 7;

/// Best nanoseconds per call of `probe` over the rounds.
fn best(probe: impl Fn(u64)) -> f64 {
    (0..ROUNDS)
        .map(|_| {
            let start = Instant::now();
            for i in 0..LOOPS {
                probe(black_box(i));
            }
            start.elapsed().as_nanos() as f64 / LOOPS as f64
        })
        .fold(f64::INFINITY, f64::min)
}

fn span(i: u64) {
    drop(stages::span(Stage::Layout).items(i));
}

fn count(i: u64) {
    stages::count(Stage::Bind, i);
}

/// The loop body alone, to subtract.
fn empty(i: u64) {
    black_box(i);
}

pub fn markdown() -> String {
    stages::enable(false);
    let floor = best(empty);
    let span_off = best(span);
    let count_off = best(count);
    stages::enable(true);
    let span_on = best(|i| {
        span(i);
        if i % 4096 == 0 {
            let _ = stages::take();
        }
    });
    let count_on = best(count);
    stages::enable(false);
    let mut md = String::new();
    let _ = writeln!(
        md,
        "one loop iteration, best of {ROUNDS} rounds of {LOOPS} iterations, in ns; the \
         empty loop is the floor:\n"
    );
    let _ = writeln!(md, "| loop body | stages off | stages on |");
    let _ = writeln!(md, "| --- | ---: | ---: |");
    let _ = writeln!(md, "| nothing (the floor) | {floor:.2} | {floor:.2} |");
    let _ = writeln!(
        md,
        "| `span(stage).items(n)` and its drop | {span_off:.2} | {span_on:.2} |"
    );
    let _ = writeln!(md, "| `count(stage, n)` | {count_off:.2} | {count_on:.2} |");
    md
}
