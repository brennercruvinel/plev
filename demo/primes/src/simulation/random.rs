//! a small seeded deterministic rng (xorshift64*). the demo used
//! `Math.random()`; determinism is a prerequisite for the pixel-measured
//! validation this repo requires, so every random draw flows through here
//! from a fixed seed.

/// xorshift64* generator. cheap, deterministic, good enough for particle
/// placement and phase init; it is not cryptographic.
pub struct Rng {
    state: u64,
}

impl Rng {
    /// new generator from a seed. 0 is remapped (xorshift cannot leave the
    /// all-zero state), so `new(0)` still produces a non-degenerate stream.
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 {
                0x9E37_79B9_7F4A_7C15
            } else {
                seed
            },
        }
    }

    /// next raw 64-bit value.
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// uniform f32 in [0, 1) with 24-bit precision (the demo's Math.random).
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u32 << 24) as f32
    }

    /// uniform f32 in [min, max).
    pub fn range(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.next_f32()
    }

    /// uniform integer in [0, n) for n > 0 (n == 0 is treated as 1).
    pub fn below(&mut self, n: usize) -> usize {
        (self.next_u64() >> 33) as usize % n.max(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sequence(seed: u64, len: usize) -> Vec<u64> {
        let mut r = Rng::new(seed);
        (0..len).map(|_| r.next_u64()).collect()
    }

    #[test]
    fn same_seed_is_reproducible() {
        assert_eq!(sequence(42, 16), sequence(42, 16));
    }

    #[test]
    fn different_seeds_diverge() {
        assert_ne!(sequence(1, 16), sequence(2, 16));
    }

    #[test]
    fn next_f32_stays_in_unit() {
        let mut r = Rng::new(7);
        for _ in 0..10_000 {
            let v = r.next_f32();
            assert!((0.0..1.0).contains(&v), "out of range: {v}");
        }
    }

    #[test]
    fn range_respects_bounds() {
        let mut r = Rng::new(99);
        for _ in 0..10_000 {
            let v = r.range(-3.0, 5.0);
            assert!((-3.0..5.0).contains(&v), "out of range: {v}");
        }
    }

    #[test]
    fn below_is_bounded_and_seed_zero_is_not_degenerate() {
        // edge: seed 0 must not collapse to a zero stream.
        let mut r = Rng::new(0);
        let mut seen_nonzero = false;
        for _ in 0..1000 {
            let i = r.below(250);
            assert!(i < 250);
            seen_nonzero |= i != 0;
        }
        assert!(seen_nonzero, "below(250) only ever returned 0");
    }
}
