//! GPU memory estimates for layers, feeding the perf monitor. Estimates
//! derive from buffer capacities and texture dimensions; the engine asks
//! the driver for nothing.

use super::{Compositor, Layer};
use crate::gpu_vec::GpuVec;

/// Layer textures use the (4-byte) surface format; MSAA targets hold 4
/// samples per pixel (the engine clamps msaa_samples to 1 or 4).
const TEXTURE_BPP: u64 = 4;
const MSAA_SAMPLES: u64 = 4;

impl Layer {
    /// Estimated resident GPU bytes for this layer: vertex/index buffer
    /// capacities plus the offscreen texture (and its MSAA companion).
    pub fn gpu_memory_bytes(&self) -> u64 {
        let buffers = [
            &self.quad_vb,
            &self.quad_ib,
            &self.sdf_vb,
            &self.sdf_ib,
            &self.shadow_vb,
            &self.shadow_ib,
            &self.image_vb,
            &self.image_ib,
            &self.backdrop_vb,
            &self.backdrop_ib,
            &self.text_vb,
            &self.text_ib,
        ]
        .into_iter()
        .map(|b| b.as_ref().map_or(0, GpuVec::capacity_bytes))
        .sum::<u64>();

        let pixels = u64::from(self.tex_width) * u64::from(self.tex_height);
        let mut textures = 0;
        if self.texture_view.is_some() {
            textures += pixels * TEXTURE_BPP;
        }
        if self.msaa_view.is_some() {
            textures += pixels * TEXTURE_BPP * MSAA_SAMPLES;
        }
        buffers + textures
    }
}

impl Compositor {
    /// Estimated resident GPU bytes across all layers.
    pub fn gpu_memory_bytes(&self) -> u64 {
        self.layers.iter().map(Layer::gpu_memory_bytes).sum()
    }
}

#[cfg(test)]
mod tests {
    use crate::compositor::Compositor;

    #[test]
    fn headless_compositor_reports_zero_gpu_memory() {
        let mut c = Compositor::new();
        c.create_layer(5);
        // No device touched: no buffers, no textures.
        assert_eq!(c.gpu_memory_bytes(), 0);
    }
}
