//! urnaui application shell: the winit `ApplicationHandler` that owns the
//! window, GPU state, compositor and `UrnauiView`, plus the per-platform
//! entry points (desktop `run`, web `run_web`, android `android_main`, iOS
//! `urnaui_ios_main`). The view stays winit-free; this is the only module
//! that touches the event loop.
//!
//! Frames render on demand, paced as the showcase paces them: the view's
//! tick answers a [`Wake`], motion (a spinner, a scrollbar fade, a toast
//! springing) draws every turn, and a caret or a toast at rest sleeps the
//! loop until its deadline (`ControlFlow::WaitUntil`), or for good when
//! nothing is pending. Each loop iteration first advances what animates
//! to the instant it woke, so what the events and the worker's results
//! then change counts its time from the change, not from the last frame.
//! On wasm GPU init is async: `resumed` spawns
//! `GpuContext::new` and the result comes back through a
//! `UserEvent::GpuReady` on the event loop proxy. On desktop and mobile
//! GPU init blocks inside `resumed` instead. Backend results arrive over
//! the worker channel, drained from `about_to_wait`.

use std::sync::Arc;

use crate::view::UrnauiView;
use crate::{keyboard, renderer};
use engine::widgets::prelude::{Wake, WidgetEvent};
use engine::compose::animation::FrameClock;
use engine::drawing::compositor::Compositor;
use engine::drawing::gpu::GpuContext;
use engine::drawing::text::TextSystem;
use web_time::Instant;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, StartCause, WindowEvent};
#[cfg(target_arch = "wasm32")]
use winit::event_loop::EventLoopProxy;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId};

/// Events from outside the winit loop: the worker thread's wake (native),
/// and the async GPU / file-picker tasks on wasm. `GpuReady` carries the
/// whole GPU state and is delivered once; boxing it would only add an
/// indirection for the one wasm handoff.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
#[allow(clippy::large_enum_variant)]
pub(crate) enum UserEvent {
    /// The backend worker queued a result: drain it and redraw. Sent
    /// from the worker thread through the event-loop proxy (never on the
    /// web, where the inline worker answers before `send` returns).
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    WorkerWoke,
    GpuReady {
        gpu: GpuContext,
        text_system: TextSystem,
        effects: engine::drawing::effects::EffectProcessor,
        texture_pool: engine::drawing::gpu::texture_pool::TexturePool,
    },
    /// The browser file picker delivered the picked .urna's bytes.
    #[cfg(target_arch = "wasm32")]
    FileLoaded { name: String, bytes: Vec<u8> },
}

// Ready is ~2464 B vs 0 for Uninitialized; one instance lives for the
// whole process, so boxing would only add indirection on the render path.
#[allow(clippy::large_enum_variant)]
enum GpuState {
    Uninitialized,
    Ready {
        gpu: GpuContext,
        text_system: TextSystem,
        effects: engine::drawing::effects::EffectProcessor,
        texture_pool: engine::drawing::gpu::texture_pool::TexturePool,
    },
}

pub struct App {
    window: Option<Arc<Window>>,
    state: GpuState,
    compositor: Compositor,
    view: UrnauiView,
    clock: FrameClock,
    /// A catch-up tick since the last frame changed something visible
    /// (a deadline came due as the loop woke): the next frame shows it.
    caught_up: bool,
    cursor: (f32, f32),
    scale_factor: f64,
    /// Modifier state, tracked for Cmd/Ctrl-V paste into text fields.
    modifiers: winit::keyboard::ModifiersState,
    #[cfg(target_arch = "wasm32")]
    proxy: Option<EventLoopProxy<UserEvent>>,
}

impl App {
    pub fn new() -> Self {
        // `mut` is only needed on native, where the launch-argument open
        // below mutates the view; on wasm there are no process args.
        #[cfg(not(target_arch = "wasm32"))]
        let mut view = UrnauiView::new(1200.0, 800.0);
        #[cfg(target_arch = "wasm32")]
        let view = UrnauiView::new(1200.0, 800.0);
        // Optional launch argument on native: a .urna path to open at
        // startup. (Empty iterator on wasm — no process args in the
        // browser.)
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(path) = std::env::args().nth(1) {
            view.open_database(std::path::PathBuf::from(path));
        }
        Self {
            window: None,
            state: GpuState::Uninitialized,
            compositor: Compositor::new(),
            view,
            clock: FrameClock::new(),
            caught_up: false,
            cursor: (0.0, 0.0),
            scale_factor: 1.0,
            modifiers: winit::keyboard::ModifiersState::default(),
            #[cfg(target_arch = "wasm32")]
            proxy: None,
        }
    }

    fn invalidate(&mut self) {
        self.compositor.invalidate();
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }

    /// Advances what animates to now, before anything from outside the
    /// frame lands (input, a worker result, a picked file). The tick gets
    /// the real time since the last one, so a change applied between two
    /// frames would otherwise take the whole gap on the next: a toast
    /// pushed after ten idle seconds would start ten seconds old, a caret
    /// focused then would start in any phase. This tick draws nothing; if
    /// it changed something visible, it asks for the frame.
    fn catch_up(&mut self) {
        let since_last = self.clock.tick().since_last;
        if self.view.wake(since_last).now {
            self.caught_up = true;
            self.invalidate();
        }
    }

    /// What the frame's [`Wake`] asks of the loop, after the frame that
    /// carries its `now` was drawn: continuous motion draws the next
    /// frame; a deadline sets the wait to that instant; nothing pending
    /// sleeps until an event. Any event (input, the worker's wake) ends a
    /// wait early.
    fn pace(&mut self, event_loop: &ActiveEventLoop, wake: Wake) {
        match wake.next {
            Some(next) if next.is_zero() => {
                // Something still moves: keep the frames coming.
                self.invalidate();
                event_loop.set_control_flow(ControlFlow::Wait);
            }
            Some(next) => {
                event_loop.set_control_flow(ControlFlow::WaitUntil(Instant::now() + next))
            }
            None => event_loop.set_control_flow(ControlFlow::Wait),
        }
    }

    /// Sync surface, projection, and view layout to the window's current
    /// inner size. Called once GPU state becomes `Ready` (and harmless to
    /// call again — `gpu.resize` clamps to at least 1x1).
    fn configure_viewport(&mut self) {
        let Some(window) = self.window.clone() else {
            return;
        };
        let size = window.inner_size();
        let sf = self.scale_factor as f32;
        let (lw, lh) = (size.width as f32 / sf, size.height as f32 / sf);
        if let GpuState::Ready { gpu, .. } = &mut self.state {
            gpu.resize(size.width, size.height);
            gpu.set_projection(lw, lh);
        }
        self.view.resize(lw, lh, sf);
        self.invalidate();
    }
}

impl ApplicationHandler<UserEvent> for App {
    /// The loop woke: time catches up first, so whatever woke it (input,
    /// the worker's results drained in `about_to_wait`) lands at this
    /// instant. When a deadline a tick asked for came due (a caret flips,
    /// a toast leaves), the wait goes back to `Wait` and the frame is
    /// asked for, since a `WaitUntil` in the past would fire on every
    /// iteration if the redraw does not come (an occluded window); the
    /// redraw's own wake sets the next.
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        self.catch_up();
        if let StartCause::ResumeTimeReached { .. } = cause {
            event_loop.set_control_flow(ControlFlow::Wait);
            self.invalidate();
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        #[allow(unused_mut)] // mut needed for cfg(not(wasm)) inner_size
        let mut attrs = WindowAttributes::default().with_title("urnaui — urna database explorer");

        // Desktop gets a fixed initial size. On the web the canvas size is
        // owned by CSS: setting an inner size here would pin inline styles
        // on the canvas and stop it from tracking the browser window.
        #[cfg(not(target_arch = "wasm32"))]
        {
            attrs = attrs.with_inner_size(winit::dpi::LogicalSize::new(1200u32, 800u32));
        }

        let window = Arc::new(event_loop.create_window(attrs).unwrap());

        #[cfg(target_arch = "wasm32")]
        if let Err(e) = engine::runtime::window::setup_wasm_canvas(&window) {
            log::error!("WASM canvas setup failed: {e}");
        }

        self.window = Some(window.clone());
        self.scale_factor = window.scale_factor();

        #[cfg(not(target_arch = "wasm32"))]
        {
            let gpu = pollster::block_on(GpuContext::new(window));
            let text_system = TextSystem::new(&gpu.device, &gpu.text_bind_group_layout);
            let effects = engine::drawing::effects::EffectProcessor::new(&gpu.device, gpu.surface_format());
            self.state = GpuState::Ready {
                gpu,
                text_system,
                effects,
                texture_pool: engine::drawing::gpu::texture_pool::TexturePool::new(),
            };
            self.configure_viewport();
        }

        // Browsers forbid blocking on async GPU setup inside the event
        // loop: spawn it and hand the result back via the proxy.
        #[cfg(target_arch = "wasm32")]
        if let Some(proxy) = self.proxy.as_ref().cloned() {
            wasm_bindgen_futures::spawn_local(async move {
                let gpu = GpuContext::new(window).await;
                let text_system = TextSystem::new(&gpu.device, &gpu.text_bind_group_layout);
                let effects =
                    engine::drawing::effects::EffectProcessor::new(&gpu.device, gpu.surface_format());
                let _ = proxy.send_event(UserEvent::GpuReady {
                    gpu,
                    text_system,
                    effects,
                    texture_pool: engine::drawing::gpu::texture_pool::TexturePool::new(),
                });
            });
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::WorkerWoke => {
                if self.view.poll_backend() {
                    self.invalidate();
                }
            }
            #[cfg(target_arch = "wasm32")]
            UserEvent::FileLoaded { name, bytes } => {
                self.view.file_picked(name, bytes);
                self.invalidate();
            }
            UserEvent::GpuReady {
                gpu,
                text_system,
                effects,
                texture_pool,
            } => {
                log::info!("GPU context ready (async)");
                self.state = GpuState::Ready {
                    gpu,
                    text_system,
                    effects,
                    texture_pool,
                };
                // The canvas may have been resized while the adapter and
                // device were being requested; re-sync everything.
                self.configure_viewport();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers.state();
            }

            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
                if key_event.state != ElementState::Pressed {
                    return;
                }
                match &key_event.logical_key {
                    Key::Named(NamedKey::Escape) => {
                        // Escape closes the open dropdown first, quits last.
                        if !self.view.close_top_overlay() {
                            event_loop.exit();
                        } else {
                            self.invalidate();
                        }
                    }
                    // Named editing keys bridge through keys.rs (the view
                    // stays winit-free).
                    Key::Named(named) => {
                        if keys::handle_named(&mut self.view, named, self.modifiers.shift_key()) {
                            self.invalidate();
                        }
                    }
                    Key::Character(c) => {
                        // Cmd/Ctrl-V pastes; other modified characters are
                        // shortcuts, not text.
                        let paste = (self.modifiers.super_key() || self.modifiers.control_key())
                            && c.eq_ignore_ascii_case("v");
                        let plain = !self.modifiers.super_key()
                            && !self.modifiers.control_key()
                            && !self.modifiers.alt_key();
                        #[cfg(not(any(
                            target_arch = "wasm32",
                            target_os = "android",
                            target_os = "ios"
                        )))]
                        if paste {
                            use engine::runtime::platform::clipboard::ClipboardProvider;
                            let mut clipboard = engine::runtime::platform::clipboard::SystemClipboard::new();
                            if let Some(text) = clipboard.get_text()
                                && self.view.handle_paste(&text)
                            {
                                self.invalidate();
                            }
                            return;
                        }
                        if plain && self.view.handle_key(c.as_str()) {
                            self.invalidate();
                        }
                        // Cmd/Ctrl shortcuts: tab jumps and Open (bridged
                        // to engine Keystrokes here so the view stays
                        // winit-free).
                        if !plain
                            && !paste
                            && let Some(ks) = engine::control::keymap::keystroke_from_logical_key(
                                &key_event.logical_key,
                                self.modifiers,
                            )
                            && self.view.handle_keystroke(&ks)
                        {
                            self.invalidate();
                        }
                    }
                    _ => {}
                }
            }

            WindowEvent::Resized(size) => {
                let sf = self.scale_factor as f32;
                let (lw, lh) = (size.width as f32 / sf, size.height as f32 / sf);
                if let GpuState::Ready { gpu, .. } = &mut self.state {
                    gpu.resize(size.width, size.height);
                    gpu.set_projection(lw, lh);
                }
                self.view.resize(lw, lh, sf);
                self.invalidate();
            }

            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                // A Resized follows on most platforms, but invalidating here
                // is cheap and guarantees a frame on DPI changes.
                self.scale_factor = scale_factor;
                self.invalidate();
            }

            WindowEvent::CursorMoved { position, .. } => {
                let sf = self.scale_factor as f32;
                let (x, y) = (position.x as f32 / sf, position.y as f32 / sf);
                self.cursor = (x, y);
                if self.view.handle_event(&WidgetEvent::MouseMove { x, y }) {
                    self.invalidate();
                }
            }

            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                let (x, y) = self.cursor;
                let ev = match state {
                    ElementState::Pressed => WidgetEvent::MouseDown { x, y },
                    ElementState::Released => WidgetEvent::MouseUp { x, y },
                };
                if self.view.handle_event(&ev) {
                    self.invalidate();
                }
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let (x, y) = self.cursor;
                let delta = match delta {
                    MouseScrollDelta::LineDelta(_, dy) => -dy * 24.0,
                    MouseScrollDelta::PixelDelta(pos) => -pos.y as f32,
                };
                if self.view.handle_event(&WidgetEvent::Scroll { x, y, delta }) {
                    self.invalidate();
                }
            }

            WindowEvent::RedrawRequested => {
                if !matches!(self.state, GpuState::Ready { .. }) {
                    return;
                }
                // The real time since the last tick: a deadline the loop
                // slept through arrives whole (`dt` is clamped for motion).
                // `now` also carries what a catch-up changed since the
                // last frame.
                let since_last = self.clock.tick().since_last;
                let mut wake = self.view.wake(since_last);
                wake.now |= std::mem::take(&mut self.caught_up);
                log::debug!(
                    "frame: {:.3} s since the last tick, wake now {} next {:?}",
                    since_last,
                    wake.now,
                    wake.next
                );
                let GpuState::Ready {
                    gpu,
                    text_system,
                    effects,
                    texture_pool,
                } = &mut self.state
                else {
                    return;
                };
                renderer::render_frame(
                    gpu,
                    text_system,
                    effects,
                    texture_pool,
                    &mut self.compositor,
                    &mut self.view,
                );
                // `wake.now` is carried by this frame: the scene was built
                // after the tick, so a flipped caret is already in it.
                self.pace(event_loop, wake);
            }

            // A file dragged over or dropped on the window is the engine's
            // event (it knows which platform events say so); the view
            // opens what was dropped.
            other => {
                if let Some(drop) = engine::control::input::FileDrop::from_window_event(&other)
                    && self.view.handle_file_drop(&drop)
                {
                    self.invalidate();
                }
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Backend results arrive over the worker channel (native) or the
        // inline worker queue (web); folding them into view state here
        // means the UI reacts even with no input pending.
        if self.view.poll_backend() {
            self.invalidate();
        }
        // The Open screen asked for the browser's file picker.
        #[cfg(target_arch = "wasm32")]
        if self.view.take_pick_request()
            && let Some(proxy) = self.proxy.as_ref().cloned()
        {
            crate::web_picker::trigger(proxy);
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

/// Desktop entry: build the winit event loop and run urnaui.
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    let event_loop = EventLoop::<UserEvent>::with_user_event().build().unwrap();
    let mut app = App::new();
    let proxy = event_loop.create_proxy();
    app.view.set_wake(Box::new(move || {
        let _ = proxy.send_event(UserEvent::WorkerWoke);
    }));
    event_loop.run_app(&mut app).unwrap();
}

/// Browser entry: GPU init is async, so hand the proxy to the app and let
/// `spawn_app` keep the loop alive inside the browser's own event loop
/// (`run_app` would throw to escape `main`).
#[cfg(target_arch = "wasm32")]
pub fn run_web() {
    use winit::platform::web::EventLoopExtWebSys;

    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Info).expect("failed to init console_log");

    let event_loop = EventLoop::<UserEvent>::with_user_event().build().unwrap();
    let mut app = App::new();
    app.proxy = Some(event_loop.create_proxy());
    event_loop.spawn_app(app);
}

/// Android entry: GameActivity calls `android_main` on the native side. The
/// `android-game-activity` winit backend (enabled via the
/// `plev/android-game-activity` feature) provides the JNI glue; we own the
/// event loop here.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub fn android_main(android_app: winit::platform::android::activity::AndroidApp) {
    use winit::platform::android::EventLoopBuilderExtAndroid;

    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag("urnaui"),
    );

    let event_loop = EventLoop::<UserEvent>::with_user_event()
        .with_android_app(android_app)
        .build()
        .expect("android event loop");
    let mut app = App::new();
    let proxy = event_loop.create_proxy();
    app.view.set_wake(Box::new(move || {
        let _ = proxy.send_event(UserEvent::WorkerWoke);
    }));
    if let Err(e) = event_loop.run_app(&mut app) {
        log::error!("event loop error: {e:?}");
    }
}

/// iOS entry: the thin Objective-C `main` in the Xcode app calls this; winit
/// drives `UIApplicationMain` from `run_app` and never returns.
#[cfg(target_os = "ios")]
#[unsafe(no_mangle)]
pub extern "C" fn urnaui_ios_main() {
    let event_loop = EventLoop::<UserEvent>::with_user_event()
        .build()
        .expect("ios event loop");
    let mut app = App::new();
    let proxy = event_loop.create_proxy();
    app.view.set_wake(Box::new(move || {
        let _ = proxy.send_event(UserEvent::WorkerWoke);
    }));
    let _ = event_loop.run_app(&mut app);
}
