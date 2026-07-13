// App state and GPU state enum for the charts demo.

use std::sync::Arc;

use engine::animation::{Easing, FrameClock, Tween};
use engine::compositor::Compositor;
use engine::gpu::GpuContext;
use engine::gpu::texture_pool::TexturePool;
use engine::text::TextSystem;
use engine::winit::window::Window;

use crate::charts::ChartData;

#[allow(clippy::large_enum_variant)]
pub(crate) enum GpuState {
    Uninitialized,
    Ready {
        gpu: GpuContext,
        text_system: TextSystem,
        pool: TexturePool,
    },
}

pub(crate) struct App {
    pub(crate) window: Option<Arc<Window>>,
    pub(crate) gpu_state: GpuState,
    pub(crate) compositor: Compositor,
    pub(crate) clock: FrameClock,
    pub(crate) frame: u64,
    pub(crate) data: ChartData,
    pub(crate) reveal: Tween<f32>,
}

impl App {
    pub fn new() -> Self {
        let mut reveal = Tween::new(0.0_f32, 1.5, Easing::EaseOutCubic);
        reveal.set_target(1.0);
        Self {
            window: None,
            gpu_state: GpuState::Uninitialized,
            compositor: Compositor::new(),
            clock: FrameClock::new(),
            frame: 0,
            data: ChartData::new(),
            reveal,
        }
    }
}
