//! The frame tail every app shares: acquire the surface, resolve the
//! compositor and its text, encode the layer and composite passes, submit,
//! present.
//!
//! Building the scene is the app's business (it owns the view type); what
//! follows is identical everywhere, so it lives here instead of being
//! copied into each app's `renderer.rs`.
//!
//! Drawing ([`draw_frame`]) is separate from presenting ([`present_frame`]):
//! a window draws into its surface texture, `engine::headless` draws into
//! an offscreen texture, and both run the same function.

use crate::compositor::Compositor;
use crate::effects::EffectProcessor;
use crate::gpu::GpuContext;
use crate::gpu::texture_pool::TexturePool;
use crate::perf::gpu_time::FrameStamps;
use crate::perf::stages::{self, Stage};
use crate::text::TextSystem;

use super::render_passes::{
    encode_composite_pass_timed, encode_layer_passes_timed, resolve_layer_text,
};

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
    effects: &mut EffectProcessor,
    texture_pool: &mut TexturePool,
    clear_linear: [f32; 4],
    label: &str,
) -> Option<u32> {
    // A shader saved on disk since the last frame: swap its pipeline and
    // redraw every layer, clean ones still hold the old shader's output.
    #[cfg(hot_reload)]
    if gpu.poll_shader_reload(effects) {
        compositor.mark_all_dirty();
    }

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
    let draws = draw_frame(
        compositor,
        gpu,
        text_system,
        effects,
        texture_pool,
        &surface_view,
        clear_linear,
        label,
    );
    output.present();

    Some(draws)
}

/// Draw one frame of an already-built scene into `target` and submit it.
///
/// This is the whole frame after the scene is built: compositor resolve,
/// text resolve, image upload, the layer passes, the composite pass into
/// `target`, submit. [`present_frame`] calls it with the surface view and
/// then presents; `engine::headless` calls it with an offscreen view and
/// then reads the pixels back.
///
/// `target` must be a view of a texture sized like `gpu.surface_config`
/// (width x height) in `gpu.surface_format()`, created through the sRGB
/// view path (`gpu.surface_render_view` or `gpu.texture_render_view`):
/// layer textures, scissors and the backdrop resolve all take their size
/// and format from the context, not from the view. `clear_linear` and
/// `label` are as in [`present_frame`].
///
/// With `perf::stages` on it records each stage it runs; on a context
/// built with `RenderConfig::gpu_timestamps` the layer and composite
/// passes also write GPU timestamps, read back by a later frame without
/// waiting (see `perf::gpu_time`). Neither changes a pixel.
///
/// Returns the number of draw calls issued.
#[allow(clippy::too_many_arguments)] // the frame's resources, one each
pub fn draw_frame(
    compositor: &mut Compositor,
    gpu: &mut GpuContext,
    text_system: &mut TextSystem,
    effects: &mut EffectProcessor,
    texture_pool: &mut TexturePool,
    target: &wgpu::TextureView,
    clear_linear: [f32; 4],
    label: &str,
) -> u32 {
    let draw = stages::span(Stage::Draw);
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
    let text = stages::span(Stage::TextResolve);
    text_system.begin_frame();
    resolve_layer_text(compositor, gpu, text_system);
    text_system.finish_frame();
    text.end(|| dirty_glyphs(compositor));

    // Upload any images loaded while building the scene.
    gpu.prepare_images();

    let encode = stages::span(Stage::Encode);
    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some(label) });

    let dirty_layer_ids: Vec<_> = compositor
        .layers()
        .iter()
        .filter(|l| l.visible && l.is_dirty())
        .map(|l| l.id)
        .collect();

    #[cfg(hot_reload)]
    let clear_linear = crate::hot_reload::literals::linear(clear_linear);
    let [r, g, b, a] = clear_linear;
    let clear_color = wgpu::Color {
        r: r as f64,
        g: g as f64,
        b: b as f64,
        a: a as f64,
    };

    // GPU timestamps, only on a context built with
    // `RenderConfig::gpu_timestamps` (see `perf::gpu_time`): the finished
    // frames' queries are resolved first, then this frame's passes write
    // theirs while stages are on.
    if let Some(timestamps) = &gpu.timestamps {
        timestamps.resolve_finished(&mut encoder);
    }
    let mut stamps = FrameStamps::begin(gpu.timestamps.as_ref());

    let layer_draws = encode_layer_passes_timed(
        compositor,
        gpu,
        text_system,
        effects,
        texture_pool,
        clear_color,
        &dirty_layer_ids,
        &mut encoder,
        stamps.as_mut(),
    );
    for id in &dirty_layer_ids {
        compositor.mark_layer_clean(*id);
    }

    let composite_draws = encode_composite_pass_timed(
        compositor,
        clear_color,
        gpu,
        target,
        &[],
        &mut encoder,
        stamps.as_ref(),
    );

    gpu.queue.submit(std::iter::once(encoder.finish()));
    if let Some(stamps) = stamps {
        stamps.submitted(&gpu.queue);
    }
    encode.end(|| u64::from(layer_draws + composite_draws));
    drop(draw);
    stages::wait_for_gpu(&gpu.device);
    if let Some(timestamps) = &gpu.timestamps {
        timestamps.collect();
    }

    layer_draws + composite_draws
}

/// Glyphs in the layers whose text was resolved this frame (the dirty
/// ones), for the text stage's items.
fn dirty_glyphs(compositor: &Compositor) -> u64 {
    compositor
        .layers()
        .iter()
        .filter(|l| l.is_dirty())
        .map(|l| u64::from(l.glyph_count()))
        .sum()
}
