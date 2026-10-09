//! the prime coherence matrix: an N x N table of pairwise affinities in
//! [0, 1], precomputed from the primes under a `CohMode`. faithful to the
//! demo's `buildCoherenceMatrix`: the diagonal is 1.0; each off-diagonal raw
//! value is clamped to [0, 1] then raised to the 4th power to sharpen
//! affinity. raw values compute in f64 (the demo's Math space) and store f32
//! (its Float32Array).

use crate::sim::params::CohMode;

/// a row-major N x N matrix of coherence values. `get(i, j)` is the affinity
/// between the i-th and j-th primes; the matrix is symmetric.
pub struct CoherenceMatrix {
    n: usize,
    values: Vec<f32>,
}

impl CoherenceMatrix {
    /// build the matrix for `primes` (length N) under `mode`. `modulus` is
    /// read only by `Modular`; the other modes ignore it.
    pub fn build(primes: &[u32], mode: CohMode, modulus: u32) -> Self {
        let n = primes.len();
        let mut values = vec![0.0f32; n * n];
        for (i, &p) in primes.iter().enumerate() {
            for (j, &q) in primes.iter().enumerate() {
                values[i * n + j] = if i == j {
                    1.0
                } else {
                    pair_coherence(p, q, mode, modulus).clamp(0.0, 1.0).powi(4) as f32
                };
            }
        }
        Self { n, values }
    }

    /// the matrix dimension N (number of primes).
    pub fn n(&self) -> usize {
        self.n
    }

    /// coherence between prime indices `i` and `j` (row-major lookup).
    pub fn get(&self, i: usize, j: usize) -> f32 {
        self.values[i * self.n + j]
    }
}

/// the raw, pre-clamp, pre-pow coherence of a prime pair under `mode`.
fn pair_coherence(p: u32, q: u32, mode: CohMode, modulus: u32) -> f64 {
    match mode {
        CohMode::Modular => {
            let m = modulus as f64;
            let mut d = ((p % modulus) as f64 - (q % modulus) as f64).abs();
            if d > m / 2.0 {
                d = m - d;
            }
            1.0 - d / (m / 2.0)
        }
        CohMode::Harmonic => {
            let ratio = ((p as f64).ln() - (q as f64).ln()).abs();
            (-ratio * ratio * 5.0).exp()
        }
        CohMode::Bitwise => {
            let or = (p | q) as f64;
            if or == 0.0 { 0.0 } else { (p & q) as f64 / or }
        }
        CohMode::Proximity => 1.0 - (p as f64 - q as f64).abs() / (p + q) as f64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::params::DEFAULT_MODULUS;

    const PRIMES: [u32; 5] = [2, 3, 5, 7, 11];

    fn finite_in_unit(m: &CoherenceMatrix) {
        for i in 0..m.n() {
            for j in 0..m.n() {
                let v = m.get(i, j);
                assert!(v.is_finite() && (0.0..=1.0).contains(&v), "({i},{j})={v}");
            }
        }
    }

    #[test]
    fn diagonal_is_one_and_values_stay_in_unit() {
        for mode in [
            CohMode::Proximity,
            CohMode::Modular,
            CohMode::Harmonic,
            CohMode::Bitwise,
        ] {
            let m = CoherenceMatrix::build(&PRIMES, mode, DEFAULT_MODULUS);
            for i in 0..m.n() {
                assert_eq!(m.get(i, i), 1.0, "diagonal under {mode:?}");
            }
            finite_in_unit(&m);
        }
    }

    #[test]
    fn matrix_is_symmetric() {
        for mode in [
            CohMode::Proximity,
            CohMode::Modular,
            CohMode::Harmonic,
            CohMode::Bitwise,
        ] {
            let m = CoherenceMatrix::build(&PRIMES, mode, DEFAULT_MODULUS);
            for i in 0..m.n() {
                for j in 0..m.n() {
                    assert!(
                        (m.get(i, j) - m.get(j, i)).abs() < 1e-6,
                        "asym under {mode:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn proximity_matches_the_demo_formula() {
        // (1 - |2-3|/(2+3))^4 = 0.8^4 = 0.4096.
        let m = CoherenceMatrix::build(&PRIMES, CohMode::Proximity, DEFAULT_MODULUS);
        assert!((m.get(0, 1) - 0.4096).abs() < 1e-5, "got {}", m.get(0, 1));
    }

    #[test]
    fn bitwise_matches_the_demo_formula() {
        // 2 & 3 = 2, 2 | 3 = 3, (2/3)^4 = 0.197530...
        let m = CoherenceMatrix::build(&PRIMES, CohMode::Bitwise, DEFAULT_MODULUS);
        let expected = (2.0_f64 / 3.0).powi(4) as f32;
        assert!((m.get(0, 1) - expected).abs() < 1e-6, "got {}", m.get(0, 1));
    }

    #[test]
    fn empty_primes_gives_empty_matrix() {
        let m = CoherenceMatrix::build(&[], CohMode::Proximity, DEFAULT_MODULUS);
        assert_eq!(m.n(), 0);
    }
}
