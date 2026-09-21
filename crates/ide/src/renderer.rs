//! Frame rendering — build the scene, then hand the frame to plev's shared
//! present path (`engine::window::present_frame`), which resolves the
//! compositor and text, encodes the per-layer draw sequence in scene push
//! order (quads/paths, shadows, SDF rects, images, text and backdrop-blur
//! resolves interleaved as pushed), composites and presents.

use engine::compositor::Compositor;
use engine::effects::EffectProcessor;
use engine::gpu::GpuContext;
use engine::gpu::texture_pool::TexturePool;
use engine::text::TextSystem;
use engine::window::present_frame;
use ide::views::workspace::WorkspaceView;

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
    // Build the scene: the workspace owns the whole window.
    compositor.begin_frame();
    workspace.render(compositor);

    let clear = workspace.theme().colors.bg.to_linear_array();
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
