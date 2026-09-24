// ---------------------------------------------------------------------------
// LayerEffect -- describes an effect to apply to a rendered layer
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub enum LayerEffect {
    Blur {
        sigma: f32,
    },
    DropShadow {
        offset_x: f32,
        offset_y: f32,
        sigma: f32,
        color: [f32; 4],
    },
    Opacity {
        alpha: f32,
    },
}

// ---------------------------------------------------------------------------
// Uniform structs (must match WGSL layout)
// ---------------------------------------------------------------------------

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BlurUniforms {
    pub direction: [f32; 2],
    pub texel_size: [f32; 2],
    pub weights: [f32; 16], // 13 weights + 3 padding (vec4 aligned)
}

/// The blur of a scale other than 1 (`fs_scaled` in `blur.wgsl`): `taps`
/// per side (in `taps[0]`), their weights from the center out.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ScaledBlurUniforms {
    pub direction: [f32; 2],
    pub texel_size: [f32; 2],
    pub taps: [f32; 4],
    pub weights: [f32; SCALED_WEIGHTS],
}

/// Weights the scaled blur's uniform holds: the center and up to
/// `SCALED_WEIGHTS - 1` taps per side, enough for a scale near 9.7.
pub const SCALED_WEIGHTS: usize = 64;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CompositeUniforms {
    pub alpha: f32,
    pub _padding: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShadowUniforms {
    pub color: [f32; 4],
}

// ---------------------------------------------------------------------------
// Gaussian weight computation
// ---------------------------------------------------------------------------

/// Compute 13-tap symmetric Gaussian weights for the given sigma.
/// Returns [center, w1, w2, ..., w6, 0, 0, ...] (16 floats).
pub fn gaussian_weights(sigma: f32) -> [f32; 16] {
    let mut weights = [0.0f32; 16];
    if sigma <= 0.0 {
        weights[0] = 1.0;
        return weights;
    }

    let s2 = 2.0 * sigma * sigma;
    let mut sum = 0.0f32;

    weights[0] = 1.0;
    sum += weights[0];

    for (i, weight) in weights.iter_mut().enumerate().skip(1).take(6) {
        let w = (-((i * i) as f32) / s2).exp();
        *weight = w;
        sum += 2.0 * w;
    }

    let inv = 1.0 / sum;
    for w in weights.iter_mut().take(7) {
        *w *= inv;
    }

    weights
}

/// The kernel of scale 1 (`gaussian_weights(sigma)`, 13 taps) laid over
/// the physical texels of an axis at `scale` physical px per logical px:
/// 1x's tap k stands for the logical px [k - 0.5, k + 0.5], and physical
/// tap j takes, from each, the part inside [(j - 0.5) / scale,
/// (j + 0.5) / scale]. The same weights and the same cut (6 logical px
/// either side of the center) at every scale; the sum stays 1.
///
/// Returns the taps per side and the weights from the center out. A
/// sigma of 0 or less is no blur: one tap.
pub fn scaled_weights(sigma: f32, scale: f32) -> (u32, [f32; SCALED_WEIGHTS]) {
    let mut out = [0.0f32; SCALED_WEIGHTS];
    if sigma <= 0.0 {
        out[0] = 1.0;
        return (0, out);
    }
    let one = gaussian_weights(sigma);
    let s = f64::from(scale);
    let taps = ((6.5 * s + 0.5).ceil() as usize)
        .saturating_sub(1)
        .min(SCALED_WEIGHTS - 1);
    for (j, weight) in out.iter_mut().enumerate().take(taps + 1) {
        let (a, b) = ((j as f64 - 0.5) / s, (j as f64 + 0.5) / s);
        let mut sum = 0.0;
        for k in -6i32..=6 {
            let (lo, hi) = (f64::from(k) - 0.5, f64::from(k) + 0.5);
            let overlap = (b.min(hi) - a.max(lo)).max(0.0);
            sum += overlap * f64::from(one[k.unsigned_abs() as usize]);
        }
        *weight = sum as f32;
    }
    (taps as u32, out)
}
