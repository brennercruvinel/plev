//! Benchmark math for the Stats screen: latency percentiles, deterministic
//! pseudo-random query vectors, and ANN-vs-exact recall — a minimal port of
//! `nest benchmark`'s methodology (`nest-cli/src/cmd/benchmark.rs`):
//! N random normalized queries, per-query timing, sorted percentile pick
//! (`(len-1)*q` rounded), recall@k as hit-set overlap. The worker owns the
//! timing loop; everything here is pure and unit tested.
//!
//! Two deliberate deltas from the CLI: queries come from a fixed-seed
//! splitmix64 stream (no `rand` dependency, and runs are reproducible),
//! and the madvise-cold pass is not ported (the CLI itself documents it as
//! a hint, not a guarantee).

use super::graph::splitmix64;

/// Fixed seed for query generation: two runs of the same benchmark see
/// the same query set, so timings are comparable across runs.
const QUERY_SEED: u64 = 42;

/// Latency summary over sorted per-query times (ms).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LatencyStats {
    pub mean: f64,
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
    pub min: f64,
    pub max: f64,
}

/// Percentiles from an UNSORTED sample (sorting happens here). Mirrors
/// the CLI's nearest-index pick. Empty input yields zeros (the caller
/// never benchmarks zero queries; tests cover the guard).
pub fn latency_stats(times: &[f64]) -> LatencyStats {
    if times.is_empty() {
        return LatencyStats {
            mean: 0.0,
            p50: 0.0,
            p95: 0.0,
            p99: 0.0,
            min: 0.0,
            max: 0.0,
        };
    }
    let mut sorted = times.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let pick = |q: f64| sorted[((sorted.len() as f64 - 1.0) * q).round() as usize];
    LatencyStats {
        mean: sorted.iter().sum::<f64>() / sorted.len() as f64,
        p50: pick(0.50),
        p95: pick(0.95),
        p99: pick(0.99),
        min: sorted[0],
        max: sorted[sorted.len() - 1],
    }
}

/// `n` deterministic L2-normalized pseudo-random query vectors of
/// dimension `dim` (the CLI draws `rand::random::<f32>()` per component;
/// we draw from a splitmix64 stream so runs are reproducible).
pub fn gen_queries(dim: usize, n: usize) -> Vec<Vec<f32>> {
    (0..n)
        .map(|qi| {
            let mut q: Vec<f32> = (0..dim)
                .map(|i| {
                    // splitmix64 bits → [-1, 1): use the top 24 bits as a
                    // uniform [0,1) f32, centered.
                    let bits = splitmix64(QUERY_SEED.wrapping_add((qi * dim + i) as u64));
                    ((bits >> 40) as f32 / (1u64 << 24) as f32) * 2.0 - 1.0
                })
                .collect();
            let norm = q.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 0.0 {
                for x in &mut q {
                    *x /= norm;
                }
            }
            q
        })
        .collect()
}

/// recall@k: fraction of the approximate result set that also appears in
/// the exact result set (the CLI's hit-overlap definition). `k` is the
/// requested count (the denominator, like the CLI).
pub fn recall_at_k(exact_ids: &[&str], approx_ids: &[&str], k: usize) -> f32 {
    if k == 0 {
        return 0.0;
    }
    let exact: std::collections::HashSet<&str> = exact_ids.iter().copied().collect();
    let overlap = approx_ids.iter().filter(|id| exact.contains(**id)).count();
    overlap as f32 / k as f32
}

/// What the worker ships back: exact latency, optional ANN latency and
/// recall when the file has an HNSW section.
#[derive(Clone, Debug)]
pub struct BenchmarkView {
    pub n_queries: usize,
    pub k: i32,
    pub dim: usize,
    pub dtype: String,
    pub simd_backend: String,
    pub exact: LatencyStats,
    pub ann: Option<LatencyStats>,
    pub recall_at_k: Option<f32>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latency_stats_pick_nearest_percentile() {
        // 101 samples 0..=100: p50=50, p95=95, p99=99.
        let times: Vec<f64> = (0..=100).rev().map(|i| i as f64).collect();
        let s = latency_stats(&times);
        assert_eq!(s.p50, 50.0);
        assert_eq!(s.p95, 95.0);
        assert_eq!(s.p99, 99.0);
        assert_eq!(s.min, 0.0);
        assert_eq!(s.max, 100.0);
        assert!((s.mean - 50.0).abs() < 1e-9);
    }

    #[test]
    fn latency_stats_guard_empty() {
        assert_eq!(latency_stats(&[]).p50, 0.0);
    }

    #[test]
    fn queries_are_deterministic_normalized_and_distinct() {
        let a = gen_queries(8, 4);
        let b = gen_queries(8, 4);
        assert_eq!(a, b);
        assert_eq!(a.len(), 4);
        for q in &a {
            assert_eq!(q.len(), 8);
            let norm = q.iter().map(|x| x * x).sum::<f32>().sqrt();
            assert!((norm - 1.0).abs() < 1e-5, "query is L2-normalized");
        }
        assert_ne!(a[0], a[1], "queries differ");
    }

    #[test]
    fn recall_is_hit_overlap_over_k() {
        let exact = ["a", "b", "c", "d"];
        let approx = ["a", "b", "x"];
        // 2 of 3 approx hits are in the exact set; denominator is k=4.
        assert!((recall_at_k(&exact, &approx, 4) - 0.5).abs() < 1e-6);
        assert_eq!(recall_at_k(&exact, &exact, 4), 1.0);
        assert_eq!(recall_at_k(&exact, &approx, 0), 0.0);
    }
}
