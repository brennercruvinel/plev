//! Frame rendering — build the scene, then hand the frame to plev's shared
//! present path (`engine::window::present_frame`), which resolves the
//! compositor and text, encodes the per-layer draw sequence in scene push
//! order (quads/paths, shadows, SDF rects, images, text and backdrop-blur
//! resolves interleaved as pushed), composites and presents.

use engine::drawing::compositor::Compositor;
use engine::drawing::effects::EffectProcessor;
use engine::drawing::gpu::GpuContext;
use engine::drawing::gpu::texture_pool::TexturePool;
use engine::drawing::text::TextSystem;
use engine::runtime::window::present_frame;
use idestudio::capability::Workbench;

/// Render a single frame: build the scene, resolve layers, encode GPU passes,
/// and present. Called once per `RedrawRequested`.
pub fn render_frame(
    gpu: &mut GpuContext,
    text_system: &mut TextSystem,
    effects: &mut EffectProcessor,
    texture_pool: &mut TexturePool,
    compositor: &mut Compositor,
    bench: &mut Workbench,
) {
    // Build the scene: the workspace owns the whole window.
    compositor.begin_frame();
    bench.render(compositor);

    let clear = bench.workspace().theme().colors.bg.to_linear_array();
    present_frame(
        compositor,
        gpu,
        text_system,
        effects,
        texture_pool,
        clear,
        "ide_frame",
    );
}
