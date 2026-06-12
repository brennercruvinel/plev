mod events;
mod hot_reload;
mod lifecycle;
mod render;
mod render_passes;
pub(crate) mod state;

pub use render_passes::{encode_composite_pass, encode_layer_passes, resolve_layer_text};

#[cfg(target_arch = "wasm32")]
pub use lifecycle::setup_wasm_canvas;

use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
#[cfg(any(target_arch = "wasm32", target_os = "android"))]
use winit::event_loop::EventLoopProxy;
use winit::window::{Window, WindowId};

use crate::animation::{AnimationTick, FrameClock};
use crate::compositor::Compositor;
use crate::effects::EffectProcessor;
use crate::gpu::GpuContext;
use crate::gpu::texture_pool::TexturePool;
use crate::input::{InputState, TouchInputState, TouchPointerSynth};
use crate::perf::{PerfHud, PerfMonitor};
use crate::platform::SafeAreaInsets;
use crate::platform::ime::ImeState;
use crate::platform::lifecycle::LifecycleManager;
use crate::signal::{ReadSignal, WriteSignal, create_signal};
use crate::text::TextSystem;

use events::BufferedEvent;
use state::GpuState;

pub enum AppEvent {
    GpuReady {
        gpu: GpuContext,
        text_system: TextSystem,
        effect_processor: EffectProcessor,
        texture_pool: TexturePool,
    },
}

pub struct App {
    window: Option<Arc<Window>>,
    pub(crate) state: GpuState,
    compositor: Compositor,
    input_state: InputState,
    #[allow(dead_code)]
    touch_input: TouchInputState,
    /// Synthesizes mouse-equivalent pointer events from the primary touch,
    /// so touch input drives the same widget path as the mouse.
    touch_pointer: TouchPointerSynth,
    theme: crate::theme::Theme,
    frame_read: ReadSignal<u64>,
    frame_write: WriteSignal<u64>,
    frame_clock: FrameClock,
    #[allow(dead_code)]
    animation_tick: AnimationTick,
    lifecycle: LifecycleManager,
    ime_state: ImeState,
    safe_area: SafeAreaInsets,
    scale_factor: f64,
    is_animating: bool,
    perf: PerfMonitor,
    perf_hud: PerfHud,
    pub(crate) event_buffer: Vec<BufferedEvent>,
    #[cfg(any(target_arch = "wasm32", target_os = "android"))]
    pub(crate) event_loop_proxy: Option<EventLoopProxy<AppEvent>>,
    #[cfg(feature = "accessibility")]
    a11y_state: crate::accessibility::AccessibilityState,
    #[cfg(feature = "accessibility")]
    a11y_adapter: Option<accesskit_winit::Adapter>,
    #[cfg(feature = "hot-reload")]
    shader_watcher: Option<crate::hot_reload::ShaderWatcher>,
    #[cfg(feature = "hot-reload")]
    narrate_watcher: Option<crate::hot_reload::NarrateWatcher>,
}

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    #[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
    pub fn new() -> Self {
        let compositor = Compositor::new();
        let (frame_read, frame_write) = create_signal(0u64);
        Self {
            window: None,
            state: GpuState::Uninitialized,
            compositor,
            input_state: InputState::new(),
            touch_input: TouchInputState::new(),
            touch_pointer: TouchPointerSynth::new(),
            theme: crate::theme::Theme::default(),
            frame_read,
            frame_write,
            frame_clock: FrameClock::new(),
            animation_tick: AnimationTick {
                dt: 0.0,
                elapsed: 0.0,
            },
            lifecycle: LifecycleManager::new(),
            ime_state: ImeState::new(),
            safe_area: SafeAreaInsets::default(),
            scale_factor: 1.0,
            is_animating: false,
            perf: PerfMonitor::new(),
            perf_hud: PerfHud::new(),
            event_buffer: Vec::with_capacity(64),
            #[cfg(feature = "accessibility")]
            a11y_state: crate::accessibility::AccessibilityState::new(),
            #[cfg(feature = "accessibility")]
            a11y_adapter: None,
            #[cfg(feature = "hot-reload")]
            shader_watcher: {
                let dir = crate::hot_reload::shader_dir();
                match crate::hot_reload::ShaderWatcher::new(&dir) {
                    Ok(w) => {
                        log::info!("Shader hot reload enabled -- watching {}", dir.display());
                        Some(w)
                    }
                    Err(e) => {
                        log::error!("Failed to start shader watcher: {}", e);
                        None
                    }
                }
            },
            #[cfg(feature = "hot-reload")]
            narrate_watcher: {
                let root = crate::hot_reload::project_root();
                let src = root.join("src");
                let examples = root.join("examples");
                let dirs: Vec<&std::path::Path> = [src.as_path(), examples.as_path()]
                    .into_iter()
                    .filter(|d| d.exists())
                    .collect();
                match crate::hot_reload::NarrateWatcher::new(&dirs) {
                    Ok(w) => {
                        log::info!("Narrate hot reload enabled -- watching src/ + examples/");
                        Some(w)
                    }
                    Err(e) => {
                        log::error!("Failed to start narrate watcher: {}", e);
                        None
                    }
                }
            },
        }
    }

    /// Keep rendering frames continuously while `animating` is true.
    /// When false (default), frames are only rendered on demand: input,
    /// resize, or explicit `Compositor::invalidate` calls.
    pub fn set_animating(&mut self, animating: bool) {
        self.is_animating = animating;
        if animating && let Some(ref window) = self.window {
            window.request_redraw();
        }
    }

    pub fn is_animating(&self) -> bool {
        self.is_animating
    }

    /// Whether the perf HUD forces continuous frames. The overlay text
    /// changes every frame, so while it is on the render loop cannot go
    /// idle: that invalidation cost is the price of live measurement.
    pub(crate) fn perf_overlay_active(&self) -> bool {
        matches!(&self.state, GpuState::Ready { gpu, .. } if gpu.config.perf_hud)
    }

    #[cfg(any(target_arch = "wasm32", target_os = "android"))]
    pub fn new_with_proxy(proxy: EventLoopProxy<AppEvent>) -> Self {
        let compositor = Compositor::new();
        let (frame_read, frame_write) = create_signal(0u64);
        Self {
            window: None,
            state: GpuState::Uninitialized,
            compositor,
            input_state: InputState::new(),
            touch_input: TouchInputState::new(),
            touch_pointer: TouchPointerSynth::new(),
            theme: crate::theme::Theme::default(),
            frame_read,
            frame_write,
            frame_clock: FrameClock::new(),
            animation_tick: AnimationTick {
                dt: 0.0,
                elapsed: 0.0,
            },
            lifecycle: LifecycleManager::new(),
            ime_state: ImeState::new(),
            safe_area: SafeAreaInsets::default(),
            scale_factor: 1.0,
            is_animating: false,
            perf: PerfMonitor::new(),
            perf_hud: PerfHud::new(),
            event_buffer: Vec::with_capacity(64),
            event_loop_proxy: Some(proxy),
            #[cfg(feature = "accessibility")]
            a11y_state: crate::accessibility::AccessibilityState::new(),
            #[cfg(feature = "accessibility")]
            a11y_adapter: None,
        }
    }
}

// -- Accessibility helpers (cfg-gated) ------------------------------------

#[cfg(feature = "accessibility")]
pub(crate) struct PlevActivationHandler;

#[cfg(feature = "accessibility")]
impl accesskit::ActivationHandler for PlevActivationHandler {
    fn request_initial_tree(&mut self) -> Option<accesskit::TreeUpdate> {
        let mut root = accesskit::Node::new(accesskit::Role::Window);
        root.set_label("plev Showcase");
        Some(accesskit::TreeUpdate {
            nodes: vec![(accesskit::NodeId(u64::MAX), root)],
            tree: Some(accesskit::Tree::new(accesskit::NodeId(u64::MAX))),
            tree_id: accesskit::TreeId::ROOT,
            focus: accesskit::NodeId(u64::MAX),
        })
    }
}

#[cfg(feature = "accessibility")]
pub(crate) struct PlevActionHandler;

#[cfg(feature = "accessibility")]
impl accesskit::ActionHandler for PlevActionHandler {
    fn do_action(&mut self, _request: accesskit::ActionRequest) {
        log::debug!("AccessKit action: {:?}", _request);
    }
}

#[cfg(feature = "accessibility")]
pub(crate) struct PlevDeactivationHandler;

#[cfg(feature = "accessibility")]
impl accesskit::DeactivationHandler for PlevDeactivationHandler {
    fn deactivate_accessibility(&mut self) {
        log::info!("AccessKit: screen reader disconnected");
    }
}

// -- ApplicationHandler dispatch ------------------------------------------

impl ApplicationHandler<AppEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.handle_resumed(event_loop);
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        self.handle_suspended(event_loop);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: AppEvent) {
        self.handle_user_event(event_loop, event);
    }

    fn memory_warning(&mut self, event_loop: &ActiveEventLoop) {
        self.handle_memory_warning(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        self.handle_window_event(event_loop, window_id, event);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.handle_about_to_wait(event_loop);
    }
}
