//! Frame rendering — resolves the compositor scene graph to GPU through
//! plev's shared encoders: each layer's draw sequence in scene push order
//! (quads/paths, shadows, SDF rects, images, text and backdrop-blur
//! resolves interleaved as pushed), then composite.
//!
//! The shared helpers honor `SceneNode::PushClip` draw ranges and the
//! shadow/image passes; the previous hand-rolled encoder silently dropped
//! both, so HOFF shadows never rendered and scrolled rows painted over the
//! panel heads.

use crate::views::workspace::WorkspaceView;
use plev::compositor::Compositor;
use plev::effects::EffectProcessor;
use plev::gpu::GpuContext;
use plev::text::TextSystem;
use plev::texture_pool::TexturePool;
use plev::window::{encode_composite_pass, encode_layer_passes, resolve_layer_text};

/// Render a single frame: build the scene, resolve layers, encode GPU passes,
/// and present. Called once per `RedrawRequested`.
pub fn render_frame(
    gpu: &mut GpuContext,
    text_system: &mut TextSystem,
    effects: &EffectProcessor,
    texture_pool: &mut TexturePool,
    compositor: &mut Compositor,
    workspace: &mut WorkspaceView,
) {
    // Build scene (includes compositor.begin_frame() inside)
    workspace.render(compositor);

    // Get surface texture
    let surface = match gpu.surface.as_ref() {
        Some(s) => s,
        None => return,
    };
    let output = match surface.get_current_texture() {
        Ok(t) => t,
        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
            gpu.resize(gpu.surface_config.width, gpu.surface_config.height);
            return;
        }
        Err(_) => return,
    };
    let surface_view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    // Resolve compositor (upload dirty layers to GPU)
    compositor.resolve(&plev::compositor::ResolveResources {
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

    // Encode render passes
    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("basicIDE_plev_frame"),
        });

    let dirty_layer_ids: Vec<_> = compositor
        .layers()
        .iter()
        .filter(|l| l.visible && l.is_dirty())
        .map(|l| l.id)
        .collect();

    let theme = workspace.theme();
    let [cr, cg, cb, ca] = theme.bg_body.to_array();
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
