// App state for the mobile input demo.

use std::sync::Arc;

use plev::compositor::Compositor;
use plev::gpu::GpuContext;
use plev::ime::ImeState;
use plev::lifecycle::LifecycleManager;
use plev::platform::SafeAreaInsets;
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

pub(crate) struct MobileInputApp {
    pub(crate) window: Option<Arc<Window>>,
    pub(crate) state: State,
    pub(crate) compositor: Compositor,
    pub(crate) lifecycle: LifecycleManager,
    pub(crate) ime_state: ImeState,
    pub(crate) safe_area: SafeAreaInsets,
    pub(crate) scale_factor: f64,
    pub(crate) input_text: String,
}

impl MobileInputApp {
    pub fn new() -> Self {
        Self {
            window: None,
            state: State::Uninitialized,
            compositor: Compositor::new(),
            lifecycle: LifecycleManager::new(),
            ime_state: ImeState::new(),
            safe_area: SafeAreaInsets::default(),
            scale_factor: 1.0,
            input_text: String::new(),
        }
    }
}
