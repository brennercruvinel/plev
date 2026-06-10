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
