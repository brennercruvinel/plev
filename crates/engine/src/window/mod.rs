mod events;
mod frame;
mod host;
mod hot_reload;
mod lifecycle;
mod render;
mod render_passes;
mod windows;

pub use frame::{draw_frame, present_frame};
pub use host::WindowHost;
pub use render_passes::{encode_composite_pass, encode_layer_passes, resolve_layer_text};
pub use windows::{Routed, Windows};

#[cfg(target_arch = "wasm32")]
pub use lifecycle::setup_wasm_canvas;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
#[cfg(any(target_arch = "wasm32", target_os = "android"))]
use winit::event_loop::EventLoopProxy;
use winit::window::WindowId;

use crate::gpu::GpuContext;
use crate::platform::lifecycle::LifecycleManager;
use crate::signal::{ReadSignal, WriteSignal, create_signal};

pub enum AppEvent {
    /// The device a browser shell asked for asynchronously, for the
    /// window it was asked for (`Windows::attach_gpu`).
    GpuReady { window: WindowId, gpu: GpuContext },
}

/// The engine's own application handler: one window through
/// [`Windows`], with everything a window needs in its [`WindowHost`], and
/// what the app keeps beside them (the theme, the frame signal, the
/// lifecycle, the loop's proxy). It draws nothing but its perf HUD; an
/// app with a view keeps the same shape and builds its scene into each
/// host's compositor before presenting.
pub struct App {
    pub(crate) windows: Windows,
    theme: crate::theme::Theme,
    frame_read: ReadSignal<u64>,
    frame_write: WriteSignal<u64>,
    lifecycle: LifecycleManager,
    /// Continuous frames on every window while true (`set_animating`);
    /// applied to a window as it opens.
    is_animating: bool,
    #[cfg(any(target_arch = "wasm32", target_os = "android"))]
    pub(crate) event_loop_proxy: Option<EventLoopProxy<AppEvent>>,
    #[cfg(hot_reload)]
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
        let (frame_read, frame_write) = create_signal(0u64);
        Self {
            windows: Windows::new(),
            theme: crate::theme::Theme::default(),
            frame_read,
            frame_write,
            lifecycle: LifecycleManager::new(),
            is_animating: false,
            #[cfg(hot_reload)]
            narrate_watcher: {
                let dirs = crate::hot_reload::source_dirs();
                let dirs: Vec<&std::path::Path> = dirs.iter().map(|d| d.as_path()).collect();
                match crate::hot_reload::NarrateWatcher::new(&dirs) {
                    Ok(w) => {
                        log::info!("narrate hot reload on: crates/*/src");
                        Some(w)
                    }
                    Err(e) => {
                        log::error!("Failed to start narrate watcher: {e}");
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
        self.windows.set_animating(animating);
    }

    pub fn is_animating(&self) -> bool {
        self.is_animating
    }

    /// The app's windows, one after `resumed`.
    pub fn windows(&self) -> &Windows {
        &self.windows
    }

    pub fn windows_mut(&mut self) -> &mut Windows {
        &mut self.windows
    }

    #[cfg(any(target_arch = "wasm32", target_os = "android"))]
    pub fn new_with_proxy(proxy: EventLoopProxy<AppEvent>) -> Self {
        let (frame_read, frame_write) = create_signal(0u64);
        Self {
            windows: Windows::new(),
            theme: crate::theme::Theme::default(),
            frame_read,
            frame_write,
            lifecycle: LifecycleManager::new(),
            is_animating: false,
            event_loop_proxy: Some(proxy),
        }
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
