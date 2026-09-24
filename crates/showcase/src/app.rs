//! Showcase application shell: the winit `ApplicationHandler` that owns the
//! windows (`engine::window::Windows`: each with its GPU context, its
//! compositor and its clock) and the `ShowcaseView`, plus the per-platform
//! entry points (desktop `run`, web `run_web`, android `android_main`, iOS
//! `showcase_ios_main`). The view stays winit-free; this is the only
//! module that touches the event loop.
//!
//! One window shows the gallery: the chrome and the active page. On the
//! desktop the active page can show in a window of its own as well (`D`,
//! and `D` again to close it; `detached.rs`): the same page state and the
//! same dispatcher, drawn into that window's compositor and laid out in
//! its size, so a change made in either window shows in the other on its
//! next frame. Each window draws only when something in it changed
//! (render on demand, per window); closing the page's window leaves the
//! page in the main one; closing the main window quits.
//!
//! Frames render on demand; animations (springs, fades) keep requesting
//! frames only while something is actually moving, and a caret at rest
//! asks for its next flip: the view's tick answers a [`comps::Wake`], and
//! the loop sleeps until the earliest deadline of any window
//! (`ControlFlow::WaitUntil`), or for good when nothing is pending. Input,
//! the git worker, the screen reader and the agent port wake it as before:
//! a wait ends on any event (winit: "suspend the thread until either
//! another event arrives or the given time is reached"). Each loop
//! iteration first advances what animates to the instant it woke
//! (`catch_up`), so what the events then change counts its time from the
//! change, not from the last frame. On wasm GPU init is async: `resumed`
//! spawns `GpuContext::new` and the result comes back through a
//! `UserEvent::GpuReady` on the event loop proxy. On desktop and mobile
//! GPU init blocks inside `resumed` instead.

use crate::view::ShowcaseView;
use comps::prelude::WidgetEvent;
use engine::animation::FrameClock;
use engine::gpu::GpuContext;
#[cfg(test)]
use engine::window::WindowHost;
use engine::window::Windows;
use winit::application::ApplicationHandler;
use winit::event::{StartCause, WindowEvent};
#[cfg(target_arch = "wasm32")]
use winit::event_loop::EventLoopProxy;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::ModifiersState;
use winit::window::{WindowAttributes, WindowId};

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
    /// The main window's device, requested off the loop (wasm).
    GpuReady { gpu: GpuContext },
}

pub struct App {
    /// Every window with what it needs: the main one (the chrome and the
    /// page) and, while open, the page's own window (`detached.rs`).
    windows: Windows,
    main: Option<WindowId>,
    /// The page's own window, desktop only (task-76 item 2).
    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    page_window: Option<detached::PageWindow>,
    view: ShowcaseView,
    clock: FrameClock,
    /// A catch-up tick since the last frame changed something visible
    /// (a deadline came due as the loop woke): the next frame shows it.
    caught_up: bool,
    /// The cursor in the main window, logical pixels (the page's own
    /// window keeps its own).
    cursor: (f32, f32),
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
            windows: Windows::new(),
            main: None,
            #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
            page_window: None,
            view,
            clock: FrameClock::new(),
            caught_up: false,
            cursor: (0.0, 0.0),
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

    /// The shell around `view` with a main window that has no platform
    /// window and no device (`WindowHost::bare`): frames build into its
    /// compositor and nothing is drawn. For the tests.
    #[cfg(test)]
    fn with_bare_window(view: ShowcaseView) -> Self {
        let mut app = Self::with_view(view);
        app.main = Some(WindowId::dummy());
        app.windows.insert(WindowId::dummy(), WindowHost::bare());
        app
    }

    /// Whether `id` is the page's own window.
    fn is_page_window(&self, id: WindowId) -> bool {
        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
        {
            self.page_window.as_ref().is_some_and(|w| w.id == id)
        }
        #[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))]
        {
            let _ = id;
            false
        }
    }

    /// The window the agent port reads the page's bounds from: the page's
    /// own while it is open, the main one otherwise.
    fn page_window_id(&self) -> Option<WindowId> {
        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
        if let Some(page) = &self.page_window {
            return Some(page.id);
        }
        self.main
    }

    /// The cursor in window `id`, logical pixels.
    fn cursor_in(&self, id: WindowId) -> (f32, f32) {
        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
        if let Some(page) = self.page_window.as_ref().filter(|w| w.id == id) {
            return page.cursor;
        }
        let _ = id;
        self.cursor
    }

    fn set_cursor(&mut self, id: WindowId, at: (f32, f32)) {
        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
        if let Some(page) = self.page_window.as_mut().filter(|w| w.id == id) {
            page.cursor = at;
            return;
        }
        let _ = id;
        self.cursor = at;
    }

    /// Every window's scene is stale (a state change may show in the
    /// chrome and in the page, wherever the page is): each asks for a
    /// frame.
    fn invalidate(&mut self) {
        self.windows.invalidate_all();
    }

    /// Window `id` alone is stale (a hover, a scroll in it).
    fn invalidate_in(&mut self, id: WindowId) {
        if let Some(host) = self.windows.get_mut(id) {
            host.compositor_mut().invalidate();
            host.request_redraw();
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

    /// Sync the view's layout to window `id`'s size and scale (its host
    /// keeps the surface and the projection in step itself): the main
    /// window lays out the chrome and the page, the page's own window the
    /// page alone. Harmless to call again.
    fn viewport_changed(&mut self, id: WindowId) {
        let Some(host) = self.windows.get(id) else {
            return;
        };
        let (lw, lh) = host.logical_size();
        let sf = host.scale_factor() as f32;
        if Some(id) == self.main {
            self.view.resize(lw, lh, sf);
        } else if self.is_page_window(id) {
            self.view.resize_detached(lw, lh, sf);
        }
        self.invalidate_in(id);
    }

    /// A pointer event in window `id`, in its logical pixels: to the page
    /// alone in its own window, to the whole view in the main one. `true`
    /// when a frame is needed.
    fn pointer(&mut self, id: WindowId, event: &WidgetEvent) -> bool {
        if self.is_page_window(id) {
            self.view.detached_event(event)
        } else {
            self.view.handle_event(event)
        }
    }

    /// What agents asked through the port, served on this thread; a frame
    /// only when an act ran, since an observation changes nothing. The
    /// recording keeps who observed what, and the state an act left. The
    /// bounds come from the window that shows the page.
    fn serve_agent(&mut self) {
        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
        if let Some(shown_in) = self.page_window_id()
            && let Some(port) = self.access.agent.as_mut()
            && let Some(host) = self.windows.get(shown_in)
        {
            let recorder = self.recording.as_ref().map(record::Recording::recorder);
            if self
                .view
                .serve_agent_recorded(port, host.compositor(), recorder)
            {
                self.windows.invalidate_all();
            }
        }
    }

    /// `D` outside a text field: the page to a window of its own, or
    /// back. Desktop only; elsewhere the request is logged and dropped.
    fn detach_requested(&mut self, event_loop: &ActiveEventLoop) {
        if !self.view.take_detach_request() {
            return;
        }
        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
        self.toggle_page_window(event_loop);
        #[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))]
        {
            let _ = event_loop;
            log::info!("a page window needs the desktop: one window here");
        }
    }
}

impl ApplicationHandler<UserEvent> for App {
    /// The loop woke: time catches up first, so whatever woke it lands at
    /// this instant. When a deadline a tick asked for came due (the caret
    /// flips), the wait goes back to `Wait` and the frame is asked of the
    /// window whose deadline it was, since a `WaitUntil` in the past would
    /// fire on every iteration if the redraw does not come (an occluded
    /// window); the redraw's own wake sets the next.
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        self.catch_up();
        if let StartCause::ResumeTimeReached { .. } = cause {
            event_loop.set_control_flow(ControlFlow::Wait);
            self.deadlines_due();
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.main.is_some() {
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

        // Native: the window and its device (blocking, as before). Wasm:
        // the window alone, the device below.
        let id = self.windows.open(event_loop, attrs).unwrap();
        self.main = Some(id);
        let window = self
            .windows
            .get(id)
            .and_then(|host| host.window().cloned())
            .expect("the window just opened");

        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
        {
            self.access.attach(event_loop, &window);
            window.set_visible(true);
        }

        #[cfg(target_arch = "wasm32")]
        if let Err(e) = engine::window::setup_wasm_canvas(&window) {
            log::error!("WASM canvas setup failed: {e}");
        }

        // Browsers forbid blocking on async GPU setup inside the event
        // loop: spawn it and hand the result back via the proxy.
        #[cfg(target_arch = "wasm32")]
        if let Some(proxy) = self.proxy.take() {
            wasm_bindgen_futures::spawn_local(async move {
                let gpu = GpuContext::new(window).await;
                let _ = proxy.send_event(UserEvent::GpuReady { gpu });
            });
        }

        self.viewport_changed(id);
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
            UserEvent::GpuReady { gpu } => {
                log::info!("GPU context ready (async)");
                if let Some(id) = self.main {
                    self.windows.attach_gpu(id, gpu);
                    // The canvas may have been resized while the adapter
                    // and device were being requested; re-sync everything.
                    self.viewport_changed(id);
                }
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        self.handle_window_event(event_loop, id, event);
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

mod events;
mod pace;

/// The page in a window of its own (task-76 item 2), desktop only.
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod detached;

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
