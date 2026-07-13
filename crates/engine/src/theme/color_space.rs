// ============================================================================
// OKLCH -- perceptual color space utilities
//
// OKLCH (Oklab Lightness-Chroma-Hue) provides perceptually uniform color
// manipulation. Shifting lightness by 0.1 looks like the same visual step
// regardless of hue. No existing Rust UI framework exposes OKLCH as a
// first-class primitive for design token derivation.
//
// sRGB -> linear RGB -> LMS (cone response) -> OKLab -> OKLCH
// ============================================================================

use crate::color::Color;

/// OKLCH color: Lightness [0,1], Chroma [0,~0.4], Hue [0,360).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Oklch {
    pub l: f32,
    pub c: f32,
    pub h: f32,
}

fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

impl Color {
    /// Convert sRGB Color to OKLCH.
    pub fn to_oklch(&self) -> Oklch {
        let [r, g, b, _] = self.0;
        let lr = srgb_to_linear(r);
        let lg = srgb_to_linear(g);
        let lb = srgb_to_linear(b);

        let l = (0.412_221_5 * lr + 0.536_332_5 * lg + 0.051_446_0 * lb)
            .max(0.0)
            .cbrt();
        let m = (0.211_903_5 * lr + 0.680_699_5 * lg + 0.107_396_96 * lb)
            .max(0.0)
            .cbrt();
        let s = (0.088_302_46 * lr + 0.281_718_84 * lg + 0.629_978_7 * lb)
            .max(0.0)
            .cbrt();

        let lab_l = 0.210_454_26 * l + 0.793_617_8 * m - 0.004_072_047 * s;
        let lab_a = 1.977_998_5 * l - 2.428_592_2 * m + 0.450_593_7 * s;
        let lab_b = 0.025_904_037 * l + 0.782_771_77 * m - 0.808_675_77 * s;

        let c = (lab_a * lab_a + lab_b * lab_b).sqrt();
        let h = if c < 1e-6 {
            0.0
        } else {
            lab_b.atan2(lab_a).to_degrees().rem_euclid(360.0)
        };

        Oklch { l: lab_l, c, h }
    }

    /// Create sRGB Color from OKLCH values.
    pub fn from_oklch(oklch: Oklch) -> Self {
        let h_rad = oklch.h.to_radians();
        let lab_a = oklch.c * h_rad.cos();
        let lab_b = oklch.c * h_rad.sin();

        let l = oklch.l + 0.396_337_78 * lab_a + 0.215_803_76 * lab_b;
        let m = oklch.l - 0.105_561_35 * lab_a - 0.063_854_17 * lab_b;
        let s = oklch.l - 0.089_484_18 * lab_a - 1.291_485_5 * lab_b;

        let l = l * l * l;
        let m = m * m * m;
        let s = s * s * s;

        let lr = 4.076_741_7 * l - 3.307_711_6 * m + 0.230_969_94 * s;
        let lg = -1.268_438 * l + 2.609_757_4 * m - 0.341_319_38 * s;
        let lb = -0.004_196_086 * l - 0.703_418_6 * m + 1.707_614_7 * s;

        Color::rgb(
            linear_to_srgb(lr.clamp(0.0, 1.0)),
            linear_to_srgb(lg.clamp(0.0, 1.0)),
            linear_to_srgb(lb.clamp(0.0, 1.0)),
        )
    }

    /// Derive a color variant by adjusting OKLCH parameters.
    /// `dl`: lightness delta, `dc`: chroma delta, `dh`: hue shift in degrees.
    pub fn oklch_shift(&self, dl: f32, dc: f32, dh: f32) -> Self {
        let mut oklch = self.to_oklch();
        oklch.l = (oklch.l + dl).clamp(0.0, 1.0);
        oklch.c = (oklch.c + dc).max(0.0);
        oklch.h = (oklch.h + dh).rem_euclid(360.0);
        Color::from_oklch(oklch)
    }
}
