//! The frame tail every app shares: acquire the surface, resolve the
//! compositor and its text, encode the layer and composite passes, submit,
//! present.
//!
//! Building the scene is the app's business (it owns the view type); what
//! follows is identical everywhere, so it lives here instead of being
//! copied into each app's `renderer.rs`.

use crate::compositor::Compositor;
use crate::effects::EffectProcessor;
use crate::gpu::GpuContext;
use crate::gpu::texture_pool::TexturePool;
use crate::text::TextSystem;

use super::render_passes::{encode_composite_pass, encode_layer_passes, resolve_layer_text};

/// Present one frame of an already-built scene.
///
/// Call it after the view has pushed its scene into `compositor` (which
/// includes `Compositor::begin_frame`). `clear_linear` is the window
/// background in *linear* space -- the sRGB surface re-encodes on write, so
/// feeding it the sRGB tone would show the page ~2.5x too light; pass
/// `color.to_linear_array()`. `label` names the command encoder in GPU
/// captures.
///
/// Returns the number of draw calls issued, or `None` when the frame was
/// skipped because no surface was available (not yet created, lost, or the
/// swapchain needed a reconfigure -- all recoverable on the next redraw).
pub fn present_frame(
    compositor: &mut Compositor,
    gpu: &mut GpuContext,
    text_system: &mut TextSystem,
    effects: &EffectProcessor,
    texture_pool: &mut TexturePool,
    clear_linear: [f32; 4],
    label: &str,
) -> Option<u32> {
    let surface = gpu.surface.as_ref()?;
    let output = match surface.get_current_texture() {
        Ok(t) => t,
        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
            gpu.resize(gpu.surface_config.width, gpu.surface_config.height);
            return None;
        }
        Err(wgpu::SurfaceError::Timeout) => {
            log::warn!("{label}: surface timeout");
            return None;
        }
        Err(e) => {
            log::error!("{label}: surface error: {e:?}");
            return None;
        }
    };
    let surface_view = gpu.surface_render_view(&output);

    compositor.resolve(&crate::compositor::ResolveResources {
        device: &gpu.device,
        queue: &gpu.queue,
        format: gpu.surface_format(),
        width: gpu.surface_config.width,
        height: gpu.surface_config.height,
        msaa_samples: gpu.config.msaa_samples,
        composite_bgl: &gpu.composite_bind_group_layout,
        opacity_bgl: &gpu.opacity_bind_group_layout,
        sampler: &gpu.composite_sampler,
    });

    // Resolve text for each dirty layer (clip-group aware).
    text_system.begin_frame();
    resolve_layer_text(compositor, gpu, text_system);
    text_system.finish_frame();

    // Upload any images loaded while building the scene.
    gpu.prepare_images();

    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some(label) });

    let dirty_layer_ids: Vec<_> = compositor
        .layers()
        .iter()
        .filter(|l| l.visible && l.is_dirty())
        .map(|l| l.id)
        .collect();

    let [r, g, b, a] = clear_linear;
    let clear_color = wgpu::Color {
        r: r as f64,
        g: g as f64,
        b: b as f64,
        a: a as f64,
    };

    let layer_draws = encode_layer_passes(
        compositor,
        gpu,
        text_system,
        effects,
        texture_pool,
        clear_color,
        &dirty_layer_ids,
        &mut encoder,
    );
    for id in &dirty_layer_ids {
        compositor.mark_layer_clean(*id);
    }

    let composite_draws = encode_composite_pass(
        compositor,
        clear_color,
        gpu,
        &surface_view,
        &[],
        &mut encoder,
    );

    gpu.queue.submit(std::iter::once(encoder.finish()));
    output.present();

    Some(layer_draws + composite_draws)
}
