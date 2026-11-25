//! Showcase application shell: the winit `ApplicationHandler` that owns the
//! windows (`engine::runtime::window::Windows`: each with its GPU context, its
//! compositor and its clock) and the `ShowcaseView`, plus the per-platform
//! entry points (desktop `run`, web `run_web`, android `android_main`, iOS
//! `uigallery_ios_main`). The view stays winit-free; this is the only
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
//! asks for its next flip: the view's tick answers a [`engine::widgets::Wake`], and
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

mod devmode;
pub mod system;
pub mod window;

use crate::views::ShowcaseView;
use engine::compose::animation::FrameClock;
use engine::drawing::gpu::GpuContext;
use engine::runtime::perf::stages::{self, Stage};
#[cfg(test)]
use engine::runtime::window::WindowHost;
use engine::runtime::window::Windows;
use engine::widgets::prelude::{SafeArea, WidgetEvent};
use winit::application::ApplicationHandler;
use winit::event::{StartCause, WindowEvent};
#[cfg(target_arch = "wasm32")]
use winit::event_loop::EventLoopProxy;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
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
    /// A click outside the window (an item of the bar or of the tray, the
    /// tray icon, a notification): the engine woke the loop from the
    /// platform's thread; poll them all and dispatch them, desktop only.
    Clicked,
    /// A link was delivered while the app runs (`links.rs`): poll the
    /// source and follow it, as a link and never as the person.
    Link,
    /// The main window's device, requested off the loop (wasm).
    GpuReady { gpu: GpuContext },
    /// A face the web build fetched arrived: draw the text again with it
    /// (`engine::drawing::text::on_faces_arrived`, wasm).
    Faces,
}

pub struct App {
    /// Every window with what it needs: the main one (the chrome and the
    /// page) and, while open, the page's own window (`detached.rs`).
    windows: Windows,
    main: Option<WindowId>,
    /// The page's own window, desktop only (task-76 item 2).
    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    page_window: Option<window::detached::PageWindow>,
    view: ShowcaseView,
    clock: FrameClock,
    /// A catch-up tick since the last frame changed something visible
    /// (a deadline came due as the loop woke): the next frame shows it.
    caught_up: bool,
    /// The cursor in the main window, logical pixels (the page's own
    /// window keeps its own).
    cursor: (f32, f32),
    /// The first finger on a touch screen (`touch.rs`).
    finger: system::touch::Finger,
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
    recording: Option<window::record::Recording>,
    /// The native menu bar, built from what the gallery projects (`menu`;
    /// desktop; `None` where the platform has none, or before the main
    /// window is up).
    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    menu: Option<system::menu::Menu>,
    /// The tray icon with the shell's menu (`tray`; desktop; `None` where
    /// the platform has none, or before the main window is up).
    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    tray: Option<system::tray::TrayIcon>,
    /// The platform's notifications and what the background jobs came
    /// to (`notify`; desktop).
    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    notices: Option<system::notify::Notices>,
    /// The window of ours that has the focus, if any: with none, what
    /// lands is told to the person by notification (`notify`).
    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    focus: Option<WindowId>,
    /// Where the app's links come from (`links`; `None` before the main
    /// window is up).
    links: Option<Box<dyn engine::runtime::platform::links::LinkSource>>,
    /// The dev mode over any page (`engine::runtime::dev`): there when
    /// the launch has `PLEV_DEV=1`.
    dev: engine::runtime::dev::DevMode,
    /// View source (`source`): a cmd+click taking the button until it is
    /// released, a literal edited in place.
    #[cfg(all(
        debug_assertions,
        any(target_os = "macos", target_os = "linux", target_os = "windows")
    ))]
    source: window::source::ViewSource,
}

impl App {
    pub fn new() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let args = std::env::args().skip(1);
        // the browser has no process arguments: the page's query stands
        // in for them (`?page=Data%20Table&theme=hoff`)
        #[cfg(target_arch = "wasm32")]
        let args = engine::runtime::window::launch_args(&["page", "theme"]).into_iter();
        Self::with_view(Self::launch_view(args))
    }

    /// The gallery as the launch arguments ask. Optional launch state for
    /// demos/snapshots: the page title (as the sidebar shows it, "Data
    /// Table"), theme, then the page scroll offset in px. (On wasm the
    /// page's query: `page` and `theme`.) An argument that starts
    /// with `plev-showcase://` is a link, never one of these: the link
    /// source reads it once the window is up (`links.rs`).
    fn launch_view(args: impl Iterator<Item = String>) -> ShowcaseView {
        let mut view = ShowcaseView::new(1200.0, 800.0);
        let mut args =
            args.filter(|a| !engine::runtime::platform::links::is_link(a, system::links::SCHEME));
        if let Some(page) = args.next() {
            view.jump_to_page(&page);
        }
        if let Some(theme) = args.next() {
            view.apply_theme(&theme);
        }
        if let Some(offset) = args.next().and_then(|a| a.parse::<f32>().ok()) {
            view.scroll_page_to(offset);
        }
        view.set_prefs(Self::launch_prefs());
        view
    }

    /// The person's preferences (ADR 0086), as the system says them
    /// (`runtime::platform::prefs`), the text size overridden by
    /// `PLEV_TEXT_SCALE` where the app is given one (macOS has no system
    /// value: there it is the app's).
    fn launch_prefs() -> engine::styling::theme::Prefs {
        #[cfg_attr(target_arch = "wasm32", allow(unused_mut))] // the override is native
        let mut prefs = engine::runtime::platform::prefs::read();
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(f) = std::env::var("PLEV_TEXT_SCALE")
            .ok()
            .and_then(|v| v.trim().parse::<f32>().ok())
        {
            prefs.text_scale = engine::styling::theme::TextScale::new(f);
        }
        prefs
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
            finger: system::touch::Finger::default(),
            modifiers: ModifiersState::empty(),
            #[cfg(target_arch = "wasm32")]
            proxy: None,
            #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
            access: Access::default(),
            #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
            recording: None,
            #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
            menu: None,
            #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
            tray: None,
            #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
            notices: None,
            #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
            focus: None,
            links: None,
            dev: engine::runtime::dev::DevMode::from_launch(),
            #[cfg(all(
                debug_assertions,
                any(target_os = "macos", target_os = "linux", target_os = "windows")
            ))]
            source: window::source::ViewSource::default(),
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
            // the host's insets are physical, the view's logical
            let sa = host.safe_area();
            self.view.set_safe_area(SafeArea {
                top: sa.top / sf,
                right: sa.right / sf,
                bottom: sa.bottom / sf,
                left: sa.left / sf,
            });
            // the open dev mode takes its share of the window
            let (lw, lh) = self.dev.page_size(lw, lh);
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
        if let Some(changed) = self.dev_event(id, event) {
            return changed;
        }
        // the page's own handling is the frame's input stage: one event
        let _input = stages::span(Stage::Input).items(1);
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
            let recorder = self
                .recording
                .as_ref()
                .map(window::record::Recording::recorder);
            if self
                .view
                .serve_agent_recorded(port, host.compositor(), recorder)
            {
                self.windows.invalidate_all();
            }
            // a watch that just came gets the frames on screen (task-79)
            self.offer_scenes();
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

    /// The first time, the main window; afterwards (the platform brought
    /// the app back) the wake: new frames, the answers that arrived asleep.
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.main.is_some() {
            #[cfg(target_os = "android")]
            if self.windows.has_suspended() {
                self.resume(event_loop);
            }
            self.wake();
            // a link that came asleep waited in its source (`links.rs`)
            self.links_arrived();
            return;
        }

        #[allow(unused_mut)] // mut needed on the desktop (inner size, visibility)
        let mut attrs = WindowAttributes::default().with_title(TITLE);
        #[cfg(target_os = "macos")]
        {
            attrs = system::background::window(attrs);
        }

        // AccessKit attaches before the window is first shown.
        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
        {
            attrs = attrs.with_visible(false);
        }

        // Desktop gets a fixed initial size. On the web the canvas size is
        // owned by CSS (100vw/100vh in index.html): setting an inner size
        // here would pin inline styles on the canvas and stop it from
        // tracking the browser window. On iOS winit makes the size the
        // UIWindow's frame (a 1200x800 window on a 402x874 phone, cropped
        // on the right and short at the bottom); mobile takes the screen.
        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
        {
            attrs = attrs.with_inner_size(winit::dpi::LogicalSize::new(1200u32, 800u32));
        }

        // Native: the window and its device (blocking, as before). Wasm:
        // the window alone, the device below.
        // the dev mode reads the GPU's own times: its device asks for the
        // timestamps, which no other launch does
        #[cfg(not(target_arch = "wasm32"))]
        let id = {
            let config = engine::drawing::gpu::RenderConfig {
                gpu_timestamps: self.dev.is_available(),
                ..Default::default()
            };
            self.windows
                .open_with_config(event_loop, attrs, config)
                .unwrap()
        };
        #[cfg(target_arch = "wasm32")]
        let id = self.windows.open(event_loop, attrs).unwrap();
        self.main = Some(id);
        // read on the desktop and the web only; mobile just checks it opened
        #[cfg_attr(any(target_os = "android", target_os = "ios"), allow(unused_variables))]
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
        if let Err(e) = engine::runtime::window::setup_wasm_canvas(&window) {
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

        // The bar, the tray and the notifications, from what the gallery
        // projects (desktop; the platform replaces winit's default bar).
        // Once the window is up: the main thread, the app running. No
        // window of ours has the focus until the platform says so (with
        // `PLEV_BACKGROUND=1` none ever does): what lands meanwhile is
        // told by notification.
        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
        {
            self.install_menu();
            self.install_tray();
            self.install_notices();
            self.install_print();
            self.install_capture();
            self.view.set_window_focused(false);
        }

        // The links, the launch's read now that a refusal has a window to
        // show in; a link is followed as a link, never as the person.
        self.install_links();
        self.links_arrived();
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
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
            // A screen reader's person may ask the shell for the page's
            // own window or the recording's save, as the keys do.
            UserEvent::Access => {
                #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
                {
                    self.access.carry_out(&mut self.view);
                    self.save_requested();
                }
                self.changed();
                self.detach_requested(event_loop);
            }
            UserEvent::Agent => self.serve_agent(),
            UserEvent::Faces => self.changed(),
            // The clicks on the bar, the tray and the notifications,
            // dispatched as the person's requests; what the shell owns
            // (the windows, the recording) is taken right after.
            UserEvent::Clicked => {
                #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
                {
                    let menu = self.menu_clicks();
                    let tray = self.tray_clicks();
                    let notice = self.notice_clicks();
                    if menu || tray || notice {
                        self.changed();
                    }
                    self.show_requested();
                    self.save_requested();
                }
                self.detach_requested(event_loop);
            }
            UserEvent::Link => self.links_arrived(),
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

    /// The platform put the app in the background (mobile, web; never the
    /// desktop): asleep until `resumed`. The surfaces stay: dropping them
    /// is the mobile bring-up's.
    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.sleep();
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {}

    /// The loop ends (the window closed, or the platform's quit key): the recording's
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

#[cfg(target_os = "android")]
pub use window::entry::android_main;
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
pub use window::entry::run;
#[cfg(target_arch = "wasm32")]
pub use window::entry::run_web;
#[cfg(target_os = "ios")]
pub use window::entry::uigallery_ios_main;

/// `uigallery --render`: one frame of a page without a window, as png, as
/// svg with its report, and as the page's semantic projection.
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
pub use window::headless::{
    RenderError, StagedPage, render_page, semantics_json, stage_page, stage_page_with,
};

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
use window::access::Access;
