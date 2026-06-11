#[cfg(test)]
mod tests {
    use crate::color::Color;
    use crate::theme::Oklch;

    // -- OKLCH round-trip --

    fn assert_round_trip(color: Color, name: &str) {
        let oklch = color.to_oklch();
        let back = Color::from_oklch(oklch);
        for i in 0..3 {
            assert!(
                (color.0[i] - back.0[i]).abs() < 0.02,
                "{} channel {}: {} vs {}",
                name,
                i,
                color.0[i],
                back.0[i],
            );
        }
    }

    #[test]
    fn oklch_round_trip_white() {
        assert_round_trip(Color::WHITE, "WHITE");
    }

    #[test]
    fn oklch_round_trip_black() {
        assert_round_trip(Color::BLACK, "BLACK");
    }

    #[test]
    fn oklch_round_trip_red() {
        assert_round_trip(Color::RED, "RED");
    }

    #[test]
    fn oklch_round_trip_green() {
        assert_round_trip(Color::GREEN, "GREEN");
    }

    #[test]
    fn oklch_round_trip_blue() {
        assert_round_trip(Color::BLUE, "BLUE");
    }

    #[test]
    fn oklch_round_trip_accent() {
        assert_round_trip(Color::rgb(0.30, 0.55, 1.0), "ACCENT");
    }

    #[test]
    fn oklch_lightness_ordering() {
        let black_l = Color::BLACK.to_oklch().l;
        let gray_l = Color::GRAY.to_oklch().l;
        let white_l = Color::WHITE.to_oklch().l;
        assert!(black_l < gray_l, "black L={} < gray L={}", black_l, gray_l);
        assert!(gray_l < white_l, "gray L={} < white L={}", gray_l, white_l);
    }

    #[test]
    fn oklch_red_hue_range() {
        let oklch = Color::RED.to_oklch();
        assert!(oklch.h > 15.0 && oklch.h < 45.0, "red hue={}", oklch.h);
        assert!(oklch.c > 0.2, "red chroma={}", oklch.c);
    }

    #[test]
    fn oklch_achromatic_zero_chroma() {
        let gray = Color::GRAY.to_oklch();
        assert!(
            gray.c < 0.01,
            "gray should be achromatic, chroma={}",
            gray.c
        );
    }

    #[test]
    fn oklch_shift_lightness() {
        let base = Color::rgb(0.5, 0.5, 0.5);
        let lighter = base.oklch_shift(0.1, 0.0, 0.0);
        let darker = base.oklch_shift(-0.1, 0.0, 0.0);
        assert!(lighter.to_oklch().l > base.to_oklch().l);
        assert!(darker.to_oklch().l < base.to_oklch().l);
    }

    #[test]
    fn oklch_shift_hue() {
        // Use a mid-saturation color to avoid sRGB gamut clipping distortion
        let base = Color::rgb(0.4, 0.5, 0.7);
        let shifted = base.oklch_shift(0.0, 0.0, 120.0);
        let base_h = base.to_oklch().h;
        let shifted_h = shifted.to_oklch().h;
        let expected = (base_h + 120.0).rem_euclid(360.0);
        assert!(
            (shifted_h - expected).abs() < 5.0,
            "shifted hue={} expected ~{}",
            shifted_h,
            expected,
        );
    }

    #[test]
    fn oklch_shift_chroma() {
        let base = Color::rgb(0.5, 0.3, 0.7);
        let more_vivid = base.oklch_shift(0.0, 0.05, 0.0);
        assert!(more_vivid.to_oklch().c > base.to_oklch().c);
    }

    #[test]
    fn oklch_perceptual_uniformity() {
        // Equal lightness steps should produce visually similar deltas
        let steps: Vec<f32> = (0..=10).map(|i| i as f32 * 0.1).collect();
        let colors: Vec<Color> = steps
            .iter()
            .map(|l| {
                Color::from_oklch(Oklch {
                    l: *l,
                    c: 0.0,
                    h: 0.0,
                })
            })
            .collect();
        // Each step should produce a valid sRGB color
        for (i, c) in colors.iter().enumerate() {
            for ch in 0..3 {
                assert!(
                    c.0[ch] >= 0.0 && c.0[ch] <= 1.0,
                    "step {} channel {} out of range: {}",
                    i,
                    ch,
                    c.0[ch],
                );
            }
        }
        // Lightness should be monotonically increasing
        for i in 1..colors.len() {
            let l_prev = colors[i - 1].to_oklch().l;
            let l_curr = colors[i].to_oklch().l;
            assert!(
                l_curr >= l_prev,
                "step {}: L={} should be >= L={}",
                i,
                l_curr,
                l_prev
            );
        }
    }
}
