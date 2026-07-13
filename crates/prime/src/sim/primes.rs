//! prime generation by trial division, faithful to the demo's `isPrime` and
//! `generatePrimes`: the first `count` primes, ascending, starting at 2.

/// true when `n` has no divisor in [2, sqrt(n)]. 0 and 1 are not prime; 2 and
/// 3 are. uses `i <= n / i` to test `i*i <= n` without overflow.
pub fn is_prime(n: u32) -> bool {
    if n < 2 {
        return false;
    }
    let mut i = 2u32;
    while i <= n / i {
        if n % i == 0 {
            return false;
        }
        i += 1;
    }
    true
}

/// the first `count` primes, ascending, starting at 2 (the demo's
/// `generatePrimes`). `count == 0` yields an empty vec.
pub fn generate_primes(count: usize) -> Vec<u32> {
    let mut out = Vec::with_capacity(count);
    let mut n = 2u32;
    while out.len() < count {
        if is_prime(n) {
            out.push(n);
        }
        n += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_primes_and_composites() {
        for p in [2u32, 3, 5, 7, 11, 13, 17, 19, 23, 1583] {
            assert!(is_prime(p), "{p} should be prime");
        }
        for c in [0u32, 1, 4, 6, 8, 9, 15, 21, 25, 1584] {
            assert!(!is_prime(c), "{c} should not be prime");
        }
    }

    #[test]
    fn generates_the_first_primes_in_order() {
        assert_eq!(generate_primes(5), [2, 3, 5, 7, 11]);
    }

    #[test]
    fn zero_count_is_empty() {
        assert!(generate_primes(0).is_empty());
    }

    #[test]
    fn the_250th_prime_is_1583() {
        // edge: the demo's MAX_PRIME_INDEX = 250; the last generated prime
        // anchors every downstream coherence and hue value.
        let primes = generate_primes(250);
        assert_eq!(primes.len(), 250);
        assert_eq!(*primes.last().unwrap(), 1583);
        // strictly ascending, all prime.
        for pair in primes.windows(2) {
            assert!(pair[0] < pair[1]);
        }
        assert!(primes.iter().all(|&p| is_prime(p)));
    }
}
