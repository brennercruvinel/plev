//! Frame rendering: resolve the compositor scene graph and encode GPU passes
//! through plev's shared encoders (the per-layer draw sequence in push order,
//! then the composite into the surface).

use engine::compositor::Compositor;
use engine::effects::EffectProcessor;
use engine::gpu::GpuContext;
use engine::gpu::texture_pool::TexturePool;
use engine::text::TextSystem;
use engine::window::{encode_composite_pass, encode_layer_passes, resolve_layer_text};

use crate::view::SpellbookView;

pub fn render_frame(
    gpu: &mut GpuContext,
    text_system: &mut TextSystem,
    effects: &EffectProcessor,
    texture_pool: &mut TexturePool,
    compositor: &mut Compositor,
    view: &mut SpellbookView,
) {
    // Build the scene (includes compositor.begin_frame()).
    view.render(compositor);

    let Some(surface) = gpu.surface.as_ref() else {
        return;
    };
    let output = match surface.get_current_texture() {
        Ok(t) => t,
        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
            gpu.resize(gpu.surface_config.width, gpu.surface_config.height);
            return;
        }
        Err(_) => return,
    };
    // Only through surface_render_view: a default create_view skips the gamma
    // encode, which shows up as a washed-out window.
    let surface_view = gpu.surface_render_view(&output);

    compositor.resolve(&engine::compositor::ResolveResources {
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

    text_system.begin_frame();
    resolve_layer_text(compositor, gpu, text_system);
    text_system.finish_frame();

    // Upload any card art loaded while building the scene.
    gpu.prepare_images();

    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("spellbook_frame"),
        });

    let dirty_layer_ids: Vec<_> = compositor
        .layers()
        .iter()
        .filter(|l| l.visible && l.is_dirty())
        .map(|l| l.id)
        .collect();

    // Linear clear value: the sRGB surface re-encodes on write, so feed it the
    // linearized theme color (else the page background shows ~2.5x too light).
    let [cr, cg, cb, ca] = view.theme.colors.bg.to_linear_array();
    let clear_color = wgpu::Color {
        r: cr as f64,
        g: cg as f64,
        b: cb as f64,
        a: ca as f64,
    };

    encode_layer_passes(
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

    encode_composite_pass(
        compositor,
        clear_color,
        gpu,
        &surface_view,
        &[],
        &mut encoder,
    );

    gpu.queue.submit(std::iter::once(encoder.finish()));
    output.present();
}
