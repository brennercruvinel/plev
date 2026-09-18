//! Frame rendering — build the scene, then hand the frame to plev's shared
//! present path (`engine::window::present_frame`), which resolves the
//! compositor and text, encodes the per-layer draw sequence in push order
//! (including backdrop-blur resolves), composites and presents.

use crate::view::UrnauiView;
use engine::compositor::Compositor;
use engine::effects::EffectProcessor;
use engine::gpu::GpuContext;
use engine::gpu::texture_pool::TexturePool;
use engine::text::TextSystem;
use engine::window::present_frame;

pub fn render_frame(
    gpu: &mut GpuContext,
    text_system: &mut TextSystem,
    effects: &EffectProcessor,
    texture_pool: &mut TexturePool,
    compositor: &mut Compositor,
    view: &mut UrnauiView,
) {
    // Build the scene (includes compositor.begin_frame()).
    view.render(compositor);

    let clear = view.theme.colors.bg.to_linear_array();
    present_frame(
        compositor,
        gpu,
        text_system,
        effects,
        texture_pool,
        clear,
        "urnaui_frame",
    );
}
