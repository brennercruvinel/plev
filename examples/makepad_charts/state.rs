// App state and GPU state enum for the charts demo.

use std::sync::Arc;

use plev::animation::{Easing, FrameClock, Tween};
use plev::compositor::Compositor;
use plev::gpu::GpuContext;
use plev::text::TextSystem;
use plev::texture_pool::TexturePool;
use plev::winit::window::Window;

use crate::charts::ChartData;

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
