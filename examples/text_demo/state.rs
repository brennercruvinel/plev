// App state for the text demo.

use std::sync::Arc;

use plev::compositor::Compositor;
use plev::gpu::GpuContext;
use plev::text::TextSystem;
use plev::winit::window::Window;

// Ready is ~2320 B vs 0 for Uninitialized; one instance lives for the
// whole process, so boxing would only add indirection on the render path.
#[allow(clippy::large_enum_variant)]
pub(crate) enum State {
    Uninitialized,
    Ready {
        gpu: GpuContext,
        text_system: TextSystem,
    },
}

pub(crate) struct TextDemoApp {
    pub(crate) window: Option<Arc<Window>>,
    pub(crate) state: State,
    pub(crate) compositor: Compositor,
}

impl TextDemoApp {
    pub(crate) fn new() -> Self {
        Self {
            window: None,
            state: State::Uninitialized,
            compositor: Compositor::new(),
        }
    }
}
