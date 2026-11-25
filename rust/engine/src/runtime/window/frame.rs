//! The frame tail every app shares: acquire the surface, resolve the
//! compositor and its text, encode the layer and composite passes, submit,
//! present.
//!
//! Building the scene is the app's business (it owns the view type); what
//! follows is identical everywhere, so it lives here instead of being
//! copied into each app's `renderer.rs`.
//!
//! Drawing ([`draw_frame`]) is separate from presenting ([`present_frame`]):
//! a window draws into its surface texture, `engine::drawing::headless` draws into
//! an offscreen texture, and both run the same function.

use crate::drawing::compositor::Compositor;
use crate::drawing::effects::EffectProcessor;
use crate::drawing::gpu::GpuContext;
use crate::drawing::gpu::texture_pool::TexturePool;
use crate::drawing::text::TextSystem;
use crate::runtime::perf::gpu_time::FrameStamps;
use crate::runtime::perf::stages::{self, Stage};

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
/// swapchain needed a reconfigure -- all recoverable on the next redraw),
/// or because the GPU device was lost and no adapter answers yet (logged
/// once; the next requested frame tries again).
///
/// A lost device (a driver reset, a GPU gone, a sleep the driver did not
/// survive) is recovered here, in place, before the surface is acquired:
/// everything on the old device is rebuilt on a new one and this frame is
/// drawn on it (`gpu::recover`).
pub fn present_frame(
    compositor: &mut Compositor,
    gpu: &mut GpuContext,
    text_system: &mut TextSystem,
    effects: &mut EffectProcessor,
    texture_pool: &mut TexturePool,
    clear_linear: [f32; 4],
    label: &str,
) -> Option<u32> {
    crate::zone!("present_frame");
    // A shader saved on disk since the last frame: swap its pipeline and
    // redraw every layer, clean ones still hold the old shader's output.
    #[cfg(hot_reload)]
    if gpu.poll_shader_reload(effects) {
        compositor.mark_all_dirty();
    }

    // The device may be gone since the last frame; the poll also delivers
    // a loss wgpu only reports on a poll. Recover and draw on the new
    // device, or skip the frame.
    if gpu.poll_device_loss()
        && !crate::drawing::gpu::recover_or_skip(
            compositor,
            gpu,
            text_system,
            effects,
            texture_pool,
            label,
        )
    {
        return None;
    }

    let surface = gpu.surface.as_ref()?;
    let acquired = {
        crate::zone!("acquire surface");
        surface.get_current_texture()
    };
    let output = match acquired {
        wgpu::CurrentSurfaceTexture::Success(t) | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
        wgpu::CurrentSurfaceTexture::Lost | wgpu::CurrentSurfaceTexture::Outdated => {
            // A lost device loses its surface too: ask for another frame,
            // which recovers before acquiring. A lost swapchain alone is
            // reconfigured.
            if gpu.poll_device_loss() {
                compositor.invalidate();
            } else {
                gpu.resize(gpu.surface_config.width, gpu.surface_config.height);
            }
            return None;
        }
        wgpu::CurrentSurfaceTexture::Timeout => {
            log::warn!("{label}: surface timeout");
            return None;
        }
        // A covered window: its `WindowEvent::Occluded(false)` draws the
        // frame when it shows again.
        wgpu::CurrentSurfaceTexture::Occluded => return None,
        wgpu::CurrentSurfaceTexture::Validation => {
            log::error!("{label}: surface validation error");
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
    {
        crate::zone!("present");
        output.present();
    }

    Some(draws)
}

/// Draw one frame of an already-built scene into `target` and submit it.
///
/// This is the whole frame after the scene is built: compositor resolve,
/// text resolve, image upload, the layer passes, the composite pass into
/// `target`, submit. [`present_frame`] calls it with the surface view and
/// then presents; `engine::drawing::headless` calls it with an offscreen view and
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
/// waiting (see `perf::gpu_time`), tagged with
/// `Compositor::frame_index`. Neither changes a pixel. With
/// `perf::zones` on it records a zone per step.
///
/// It leaves in the compositor what the frame drew: the encode stats
/// (`Compositor::stats`) and the layers it redrew with their draw calls
/// (`Compositor::drawn_layers`).
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
    crate::zone!("draw_frame");
    let draw = stages::span(Stage::Draw);
    let resolve = crate::runtime::perf::zones::enter("resolve");
    compositor.resolve(&crate::drawing::compositor::ResolveResources {
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
    drop(resolve);

    // Resolve text for each dirty layer (clip-group aware).
    {
        crate::zone!("text resolve");
        let text = stages::span(Stage::TextResolve);
        text_system.begin_frame();
        resolve_layer_text(compositor, gpu, text_system);
        text_system.finish_frame();
        text.end(|| dirty_glyphs(compositor));
    }

    // Upload any images loaded while building the scene.
    {
        crate::zone!("prepare images");
        gpu.prepare_images();
    }

    let encode = stages::span(Stage::Encode);
    let encode_start = web_time::Instant::now();
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
    let clear_linear = crate::runtime::hot_reload::literals::linear(clear_linear);
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
    let mut stamps = FrameStamps::begin(gpu.timestamps.as_ref(), compositor.frame_index());

    let mut drawn = std::mem::take(&mut compositor.drawn);
    drawn.clear();
    let layer_draws = {
        crate::zone!("encode layers");
        encode_layer_passes_timed(
            compositor,
            gpu,
            text_system,
            effects,
            texture_pool,
            clear_color,
            &dirty_layer_ids,
            &mut encoder,
            stamps.as_mut(),
            Some(&mut drawn),
        )
    };
    compositor.drawn = drawn;
    for id in &dirty_layer_ids {
        compositor.mark_layer_clean(*id);
    }

    let composite_draws = {
        crate::zone!("encode composite");
        encode_composite_pass_timed(
            compositor,
            clear_color,
            gpu,
            target,
            &[],
            &mut encoder,
            stamps.as_ref(),
        )
    };

    {
        crate::zone!("submit");
        gpu.queue.submit(std::iter::once(encoder.finish()));
    }
    if let Some(stamps) = stamps {
        stamps.submitted(&gpu.queue);
    }
    let draws = layer_draws + composite_draws;
    encode.end(|| u64::from(draws));
    drop(draw);
    let glyphs = compositor.layers().iter().map(|l| l.glyph_count()).sum();
    let encode_micros = u64::try_from(encode_start.elapsed().as_micros()).unwrap_or(u64::MAX);
    compositor.record_encode_stats(draws, glyphs, encode_micros);
    stages::wait_for_gpu(&gpu.device);
    if let Some(timestamps) = &gpu.timestamps {
        timestamps.collect();
    }

    draws
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
