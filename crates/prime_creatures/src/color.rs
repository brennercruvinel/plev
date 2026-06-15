//! particle color: a prime maps to a hue per `CohMode`, then `hsl(hue, 85%,
//! 60%)` resolves to sRGB. faithful to the demo's `updateColor` and its
//! `hsl(h, 85%, 60%)` swatch.
//!
//! returns sRGB in [0, 1], NOT linear. vertex colors are linearized once in
//! the shader (`srgb_to_linear`), so the host must hand the gpu sRGB. the
//! near-black clear is the opposite case and is linearized on the cpu via
//! `plev::color::Color::to_linear_array` in the bin.

use crate::sim::params::{CohMode, MAX_PRIME_INDEX};

/// the base saturation and lightness of every particle swatch (demo: 85%, 60%).
const SATURATION: f32 = 0.85;
const LIGHTNESS: f32 = 0.60;

/// hue in degrees [0, 360) for a prime under `mode`, mirroring `updateColor`.
pub fn hue_for(mode: CohMode, prime_index: usize, prime_value: u32, modulus: u32) -> f32 {
    match mode {
        CohMode::Modular => (prime_value % modulus) as f32 / modulus as f32 * 360.0,
        CohMode::Harmonic => ((prime_value as f32).ln() * 50.0).rem_euclid(360.0),
        CohMode::Bitwise => (prime_value % 256) as f32 * 1.4,
        CohMode::Proximity => prime_index as f32 / MAX_PRIME_INDEX as f32 * 360.0,
    }
}

/// a particle's base color as sRGB [r, g, b, a] in [0, 1].
pub fn particle_color(
    mode: CohMode,
    prime_index: usize,
    prime_value: u32,
    modulus: u32,
) -> [f32; 4] {
    let hue = hue_for(mode, prime_index, prime_value, modulus);
    hsl_to_srgb(hue, SATURATION, LIGHTNESS)
}

/// hsl (h in degrees, s and l in [0, 1]) to sRGB [r, g, b, a] in [0, 1], with
/// alpha 1. the standard css hsl conversion; the result is sRGB, the space css
/// colors are defined in.
pub fn hsl_to_srgb(h_deg: f32, s: f32, l: f32) -> [f32; 4] {
    let h = h_deg.rem_euclid(360.0) / 360.0;
    if s <= 0.0 {
        return [l, l, l, 1.0];
    }
    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    [
        hue_to_channel(p, q, h + 1.0 / 3.0),
        hue_to_channel(p, q, h),
        hue_to_channel(p, q, h - 1.0 / 3.0),
        1.0,
    ]
}

fn hue_to_channel(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        p + (q - p) * 6.0 * t
    } else if t < 1.0 / 2.0 {
        q
    } else if t < 2.0 / 3.0 {
        p + (q - p) * (2.0 / 3.0 - t) * 6.0
    } else {
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::params::DEFAULT_MODULUS;

    fn close(a: [f32; 4], b: [f32; 4]) -> bool {
        a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-3)
    }

    #[test]
    fn primary_hues_resolve() {
        assert!(close(hsl_to_srgb(0.0, 1.0, 0.5), [1.0, 0.0, 0.0, 1.0]));
        assert!(close(hsl_to_srgb(120.0, 1.0, 0.5), [0.0, 1.0, 0.0, 1.0]));
        assert!(close(hsl_to_srgb(240.0, 1.0, 0.5), [0.0, 0.0, 1.0, 1.0]));
        // hue wraps: 360 == 0.
        assert!(close(
            hsl_to_srgb(360.0, 1.0, 0.5),
            hsl_to_srgb(0.0, 1.0, 0.5)
        ));
    }

    #[test]
    fn zero_saturation_is_gray() {
        assert!(close(hsl_to_srgb(123.0, 0.0, 0.5), [0.5, 0.5, 0.5, 1.0]));
    }

    #[test]
    fn proximity_hue_spans_the_wheel() {
        // index 0 -> 0 degrees; halfway -> 180 degrees.
        assert!((hue_for(CohMode::Proximity, 0, 999, 432)).abs() < 1e-4);
        let half = MAX_PRIME_INDEX / 2;
        assert!((hue_for(CohMode::Proximity, half, 999, 432) - 180.0).abs() < 1.0);
    }

    #[test]
    fn every_mode_yields_a_valid_hue_and_color() {
        for mode in [
            CohMode::Proximity,
            CohMode::Modular,
            CohMode::Harmonic,
            CohMode::Bitwise,
        ] {
            let h = hue_for(mode, 7, 1583, DEFAULT_MODULUS);
            assert!((0.0..360.0).contains(&h), "{mode:?} hue {h} out of range");
            let c = particle_color(mode, 7, 1583, DEFAULT_MODULUS);
            assert!(
                c.iter().all(|x| x.is_finite() && (0.0..=1.0).contains(x)),
                "{mode:?} color {c:?} out of range"
            );
            assert_eq!(c[3], 1.0);
        }
    }
}
