//! Showcase application shell: the winit `ApplicationHandler` that owns the
//! window, GPU state, compositor and `ShowcaseView`, plus the per-platform
//! entry points (desktop `run`, web `run_web`, android `android_main`, iOS
//! `showcase_ios_main`). The view stays winit-free; this is the only module
//! that touches the event loop.
//!
//! Frames render on demand; animations (springs, fades) keep requesting
//! frames only while something is actually moving, and a caret at rest
//! asks for its next flip: the view's tick answers a [`comps::Wake`], and
//! the loop sleeps until the earliest deadline (`ControlFlow::WaitUntil`),
//! or for good when nothing is pending. Input, the git worker, the screen
//! reader and the agent port wake it as before: a wait ends on any event
//! (winit: "suspend the thread until either another event arrives or the
//! given time is reached"). Each loop iteration first advances what
//! animates to the instant it woke (`catch_up`), so what the events then
//! change counts its time from the change, not from the last frame. On
//! wasm GPU init is
//! async: `resumed` spawns `GpuContext::new` and the result comes back
//! through a `UserEvent::GpuReady` on the event loop proxy. On desktop and
//! mobile GPU init blocks inside `resumed` instead.

use std::sync::Arc;

use crate::keys;
use crate::view::ShowcaseView;
use comps::prelude::WidgetEvent;
use engine::animation::FrameClock;
use engine::compositor::Compositor;
use engine::gpu::GpuContext;
use engine::text::TextSystem;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, StartCause, WindowEvent};
#[cfg(target_arch = "wasm32")]
use winit::event_loop::EventLoopProxy;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, ModifiersState, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId};

/// The window's title, which also names the root of its accessibility
/// tree.
const TITLE: &str = "plev, design system showcase";

/// Messages from other threads and tasks into the event loop.
///
/// `GpuReady` is sent by the async GPU-init task on wasm and never
/// constructed natively (desktop blocks on `GpuContext::new` inside
/// `resumed` instead); `Wake`, `Access` and `Agent` come from the git
/// worker, the accessibility adapter and the agent port, desktop only.
/// One instance is in flight at a time, so the size gap is irrelevant.
#[allow(dead_code, clippy::large_enum_variant)]
enum UserEvent {
    /// The git worker finished a command: apply it now, draw only if the
    /// Git page shows it.
    Wake,
    /// A screen reader started listening or asked for something: carry
    /// out what it asked and draw, which sends it the tree.
    Access,
    /// An agent asked through the port: serve it, draw if an act ran.
    Agent,
    GpuReady {
        gpu: GpuContext,
        text_system: TextSystem,
        effects: engine::effects::EffectProcessor,
        texture_pool: engine::gpu::texture_pool::TexturePool,
    },
}

// Ready is ~2464 B vs 0 for Uninitialized; one instance lives for the
// whole process, so boxing would only add indirection on the render path.
#[allow(clippy::large_enum_variant)]
enum GpuState {
    Uninitialized,
    Ready {
        gpu: GpuContext,
        text_system: TextSystem,
        effects: engine::effects::EffectProcessor,
        texture_pool: engine::gpu::texture_pool::TexturePool,
    },
}

pub struct App {
    window: Option<Arc<Window>>,
    state: GpuState,
    compositor: Compositor,
    view: ShowcaseView,
    clock: FrameClock,
    /// A catch-up tick since the last frame changed something visible
    /// (a deadline came due as the loop woke): the next frame shows it.
    caught_up: bool,
    cursor: (f32, f32),
    scale_factor: f64,
    modifiers: ModifiersState,
    #[cfg(target_arch = "wasm32")]
    proxy: Option<EventLoopProxy<UserEvent>>,
    /// The window's outside actors: a screen reader (the tree projected
    /// from the active page's meaning) and agents (the port), whose
    /// requests go into its dispatcher (desktop; mobile is next).
    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    access: Access,
    /// The session's recording, when `PLEV_RECORD=1` in a debug build
    /// (`record`; desktop, where the port is).
    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    recording: Option<record::Recording>,
    /// A cmd+click is taking the button until it is released (`source`).
    #[cfg(all(
        debug_assertions,
        any(target_os = "macos", target_os = "linux", target_os = "windows")
    ))]
    source_click: bool,
}

impl App {
    pub fn new() -> Self {
        let mut view = ShowcaseView::new(1200.0, 800.0);
        // Optional launch state for demos/snapshots: the page title (as
        // the sidebar shows it, "Data Table"), theme, then the page scroll
        // offset in px. (Empty iterator on wasm: no process args in the
        // browser.)
        let mut args = std::env::args().skip(1);
        if let Some(page) = args.next() {
            view.jump_to_page(&page);
        }
        if let Some(theme) = args.next() {
            view.apply_theme(&theme);
        }
        if let Some(offset) = args.next().and_then(|a| a.parse::<f32>().ok()) {
            view.scroll_page_to(offset);
        }
        Self::with_view(view)
    }

    /// The shell around `view`, with no window yet.
    fn with_view(view: ShowcaseView) -> Self {
        Self {
            window: None,
            state: GpuState::Uninitialized,
            compositor: Compositor::new(),
            view,
            clock: FrameClock::new(),
            caught_up: false,
            cursor: (0.0, 0.0),
            scale_factor: 1.0,
            modifiers: ModifiersState::empty(),
            #[cfg(target_arch = "wasm32")]
            proxy: None,
            #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
            access: Access::default(),
            #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
            recording: None,
            #[cfg(all(
                debug_assertions,
                any(target_os = "macos", target_os = "linux", target_os = "windows")
            ))]
            source_click: false,
        }
    }

    fn invalidate(&mut self) {
        self.compositor.invalidate();
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }

    /// A person's input, or an outside actor's request, changed
    /// something: the state as it left it, when the session is recorded,
    /// and a frame.
    fn changed(&mut self) {
        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
        self.record_state();
        self.invalidate();
    }

    /// Sync surface, projection, and view layout to the window's current
    /// inner size. Called once GPU state becomes `Ready` (and harmless to
    /// call again: `gpu.resize` clamps to at least 1x1).
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

    /// What agents asked through the port, served on this thread; a frame
    /// only when an act ran, since an observation changes nothing. The
    /// recording keeps who observed what, and the state an act left.
    fn serve_agent(&mut self) {
        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
        if let Some(port) = self.access.agent.as_mut() {
            let recorder = self.recording.as_ref().map(record::Recording::recorder);
            if self
                .view
                .serve_agent_recorded(port, &self.compositor, recorder)
            {
                self.invalidate();
            }
        }
    }
}

impl ApplicationHandler<UserEvent> for App {
    /// The loop woke: time catches up first, so whatever woke it lands at
    /// this instant. When a deadline a tick asked for came due (the caret
    /// flips), the wait goes back to `Wait` and the frame is asked for,
    /// since a `WaitUntil` in the past would fire on every iteration if
    /// the redraw does not come (an occluded window); the redraw's own
    /// wake sets the next.
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
        let mut attrs = WindowAttributes::default().with_title(TITLE);
        #[cfg(target_os = "macos")]
        {
            attrs = background::window(attrs);
        }

        // AccessKit attaches before the window is first shown.
        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
        {
            attrs = attrs.with_visible(false);
        }

        // Desktop gets a fixed initial size. On the web the canvas size is
        // owned by CSS (100vw/100vh in index.html): setting an inner size
        // here would pin inline styles on the canvas and stop it from
        // tracking the browser window.
        #[cfg(not(target_arch = "wasm32"))]
        {
            attrs = attrs.with_inner_size(winit::dpi::LogicalSize::new(1200u32, 800u32));
        }

        let window = Arc::new(event_loop.create_window(attrs).unwrap());

        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
        {
            self.access.attach(event_loop, &window);
            window.set_visible(true);
        }

        #[cfg(target_arch = "wasm32")]
        if let Err(e) = engine::window::setup_wasm_canvas(&window) {
            log::error!("WASM canvas setup failed: {e}");
        }

        self.window = Some(window.clone());
        self.scale_factor = window.scale_factor();

        #[cfg(not(target_arch = "wasm32"))]
        {
            let gpu = pollster::block_on(GpuContext::new(window));
            let text_system = TextSystem::new(&gpu.device, &gpu.text_bind_group_layout);
            let effects = engine::effects::EffectProcessor::new(&gpu.device, gpu.surface_format());
            self.state = GpuState::Ready {
                gpu,
                text_system,
                effects,
                texture_pool: engine::gpu::texture_pool::TexturePool::new(),
            };
            self.configure_viewport();
        }

        // Browsers forbid blocking on async GPU setup inside the event
        // loop: spawn it and hand the result back via the proxy.
        #[cfg(target_arch = "wasm32")]
        if let Some(proxy) = self.proxy.take() {
            wasm_bindgen_futures::spawn_local(async move {
                let gpu = GpuContext::new(window).await;
                let text_system = TextSystem::new(&gpu.device, &gpu.text_bind_group_layout);
                let effects =
                    engine::effects::EffectProcessor::new(&gpu.device, gpu.surface_format());
                let _ = proxy.send_event(UserEvent::GpuReady {
                    gpu,
                    text_system,
                    effects,
                    texture_pool: engine::gpu::texture_pool::TexturePool::new(),
                });
            });
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::Wake => {
                // The embedded workspace takes the events whichever page
                // shows; a frame only when the change is on screen.
                let shown = self.view.apply_git();
                log::debug!("git worker woke: frame {shown}");
                if shown {
                    self.changed();
                }
            }
            UserEvent::Access => {
                #[cfg(not(any(
                    target_arch = "wasm32",
                    target_os = "android",
                    target_os = "ios"
                )))]
                self.access.carry_out(&mut self.view);
                self.changed();
            }
            UserEvent::Agent => self.serve_agent(),
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
        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
        if let (Some(bridge), Some(window)) = (self.access.bridge.as_mut(), self.window.as_ref()) {
            bridge.process_event(window, &event);
        }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::ModifiersChanged(mods) => {
                self.modifiers = mods.state();
                self.view.set_modifiers(self.modifiers);
            }

            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
                if key_event.state != ElementState::Pressed {
                    return;
                }
                // The save keys of the recording, when it records.
                #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
                if self.record_key(&key_event.logical_key) {
                    return;
                }
                match &key_event.logical_key {
                    Key::Named(NamedKey::Escape) => {
                        if !self.view.close_top_overlay() {
                            event_loop.exit();
                        } else {
                            self.changed();
                        }
                    }
                    // The editor owns the keyboard while it is showing.
                    key if self.view.handle_editor_key(key, self.modifiers) => {
                        self.changed();
                    }
                    // Space and the editing keys bridge through keys.rs
                    // (the view stays winit-free).
                    Key::Named(named) => {
                        if keys::handle_named(&mut self.view, named) {
                            self.changed();
                        }
                    }
                    Key::Character(c) if self.view.handle_key(c.as_str()) => {
                        self.changed();
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
                #[cfg(all(
                    debug_assertions,
                    any(target_os = "macos", target_os = "linux", target_os = "windows")
                ))]
                if self.view_source(state) {
                    return;
                }
                let (x, y) = self.cursor;
                #[cfg(not(any(
                    target_arch = "wasm32",
                    target_os = "android",
                    target_os = "ios"
                )))]
                self.record_pointer(state, x, y);
                let ev = match state {
                    ElementState::Pressed => WidgetEvent::MouseDown { x, y },
                    ElementState::Released => WidgetEvent::MouseUp { x, y },
                };
                if self.view.handle_event(&ev) {
                    self.changed();
                }
            }

            WindowEvent::MouseInput {
                button: MouseButton::Right,
                state: ElementState::Pressed,
                ..
            } => {
                let (x, y) = self.cursor;
                #[cfg(not(any(
                    target_arch = "wasm32",
                    target_os = "android",
                    target_os = "ios"
                )))]
                self.record_pointer(ElementState::Pressed, x, y);
                if self.view.handle_right_click(x, y) {
                    self.changed();
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

            WindowEvent::RedrawRequested => self.redraw(event_loop),

            // A file dragged over or dropped on the window is the engine's
            // event (it knows which platform events say so); the active
            // page takes it (task-76: the Files page).
            other => {
                if let Some(drop) = engine::input::FileDrop::from_window_event(&other)
                    && self.view.handle_file_drop(&drop)
                {
                    self.changed();
                }
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {}

    /// The loop ends (the window closed, or Escape quit): the recording's
    /// window is saved.
    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.save_session("close");
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

/// Desktop entry: build the winit event loop and run the showcase.
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
pub fn run() {
    // A recording says where its files go: the gallery's own info lines
    // show by default when it records. Off unless `PLEV_RECORD=1` in a
    // debug build; nothing is created then.
    let recording = record::Recording::from_env();
    let filter = if recording.is_some() {
        "warn,showcase=info"
    } else {
        "warn"
    };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(filter)).init();
    let mut builder = EventLoop::<UserEvent>::with_user_event();
    #[cfg(target_os = "macos")]
    background::event_loop(&mut builder);
    let event_loop = builder.build().unwrap();
    let mut app = App::new();
    app.record(recording);
    let proxy = event_loop.create_proxy();
    app.access.proxy = Some(proxy.clone());
    let agent = proxy.clone();
    app.access.agent = ShowcaseView::agent_port(move || {
        let _ = agent.send_event(UserEvent::Agent);
    });
    app.view.connect_git(Box::new(move || {
        let _ = proxy.send_event(UserEvent::Wake);
    }));
    event_loop.run_app(&mut app).unwrap();
}

/// `showcase --render`: one frame of a page without a window, as png, as
/// svg with its report, and as the page's semantic projection.
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
pub use headless::{RenderError, StagedPage, render_page, semantics_json, stage_page};

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod headless;

mod pace;

/// `PLEV_BACKGROUND=1`: the window comes up without taking the focus.
#[cfg(target_os = "macos")]
mod background;

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod access;
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
use access::Access;

/// `PLEV_RECORD=1` in a debug build: the session recorded for the
/// inspector, as the ide records its own.
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
pub mod record;

/// cmd+click to the code: desktop debug builds, the terms on which the
/// engine compiles `hot_reload` (its build.rs), so nothing of it exists in
/// release, wasm or mobile.
#[cfg(all(
    debug_assertions,
    any(target_os = "macos", target_os = "linux", target_os = "windows")
))]
mod source;

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
            .with_tag("plev-showcase"),
    );

    let event_loop = EventLoop::<UserEvent>::with_user_event()
        .with_android_app(android_app)
        .build()
        .expect("android event loop");
    let mut app = App::new();
    if let Err(e) = event_loop.run_app(&mut app) {
        log::error!("event loop error: {e:?}");
    }
}

/// iOS entry: the thin Objective-C `main` in the Xcode app calls this; winit
/// drives `UIApplicationMain` from `run_app` and never returns.
#[cfg(target_os = "ios")]
#[unsafe(no_mangle)]
pub extern "C" fn showcase_ios_main() {
    let event_loop = EventLoop::<UserEvent>::with_user_event()
        .build()
        .expect("ios event loop");
    let mut app = App::new();
    let _ = event_loop.run_app(&mut app);
}
