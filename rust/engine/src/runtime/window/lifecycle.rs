use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{WindowAttributes, WindowId};

use super::AppEvent;
use super::windows::Routed;
use crate::platform::lifecycle::AppState;

/// The window's title, which also names the root of its accessibility
/// tree.
const TITLE: &str = "plev, showcase";

impl super::App {
    /// The one window this `App` opens: titled, sized on the desktop,
    /// hidden until the accessibility bridge attached, both orientations
    /// on iOS.
    fn window_attributes() -> WindowAttributes {
        #[allow(unused_mut)]
        // mut needed for cfg(feature=accessibility), cfg(desktop), cfg(ios)
        let mut attrs = WindowAttributes::default().with_title(TITLE);

        // AccessKit attaches before the window is first shown.
        #[cfg(all(feature = "accessibility", not(target_arch = "wasm32")))]
        {
            attrs = attrs.with_visible(false);
        }

        #[cfg(not(any(target_os = "android", target_os = "ios", target_arch = "wasm32")))]
        {
            attrs = attrs.with_inner_size(winit::dpi::LogicalSize::new(1024, 820));
        }

        #[cfg(target_os = "ios")]
        {
            use winit::platform::ios::WindowAttributesExtIOS;
            attrs = attrs.with_valid_orientations(
                winit::platform::ios::ValidOrientations::LandscapeAndPortrait,
            );
        }

        attrs
    }

    pub(crate) fn handle_resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Back from the background (android): the windows come back with
        // their surfaces on the kept devices.
        if self.windows.has_suspended() {
            #[cfg(not(target_arch = "wasm32"))]
            match self.windows.resume(event_loop, Self::window_attributes) {
                Ok(ids) => log::info!("Resumed from suspended state: {} window(s)", ids.len()),
                Err(e) => log::error!("plev: failed to recreate a window: {e}"),
            }
            self.lifecycle.transition_to(AppState::Active);
            return;
        }
        if !self.windows.is_empty() {
            return;
        }

        let id = self
            .windows
            .open(event_loop, Self::window_attributes())
            .expect("plev: failed to create window");
        let animating = self.is_animating;
        let host = self
            .windows
            .get_mut(id)
            .expect("the window just opened has a host");

        #[cfg(target_arch = "wasm32")]
        if let Some(window) = host.window()
            && let Err(e) = super::setup_wasm_canvas(window)
        {
            log::error!("WASM canvas setup failed: {e}");
        }

        // No meaning to project and nothing to act on, so the bridge
        // needs no wake: the next frame sends the tree.
        #[cfg(all(feature = "accessibility", not(target_arch = "wasm32")))]
        if let Some(window) = host.window().cloned() {
            match crate::accessibility::Bridge::new(event_loop, &window, TITLE, || {}) {
                Ok(bridge) => host.set_accessibility(bridge),
                Err(e) => log::warn!("accessibility: {e}"),
            }
            window.set_visible(true);
        }

        // Browsers forbid blocking on the device inside the event loop:
        // the request is spawned and its result comes back through the
        // proxy as `AppEvent::GpuReady` for this window.
        #[cfg(target_arch = "wasm32")]
        if let (Some(proxy), Some(window)) = (self.event_loop_proxy.take(), host.window().cloned())
        {
            wasm_bindgen_futures::spawn_local(async move {
                let gpu = crate::gpu::GpuContext::new(window).await;
                let _ = proxy.send_event(AppEvent::GpuReady { window: id, gpu });
            });
        }

        host.set_animating(animating);
        host.request_redraw();
        self.lifecycle.transition_to(AppState::Active);
    }

    pub(crate) fn handle_suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.lifecycle.transition_to(AppState::Suspended);
        self.windows.suspend();
    }

    pub(crate) fn handle_user_event(&mut self, _event_loop: &ActiveEventLoop, event: AppEvent) {
        match event {
            AppEvent::GpuReady { window, gpu } => {
                log::info!("GPU context ready (async)");
                if !self.windows.attach_gpu(window, gpu) {
                    log::warn!("GPU context ready for a window that closed meanwhile");
                }
            }
        }
    }

    pub(crate) fn handle_memory_warning(&mut self, _event_loop: &ActiveEventLoop) {
        self.lifecycle.fire_memory_warning();
        self.windows.purge_caches();
    }

    /// One event of one window: its host takes it; a redraw request draws
    /// that window alone, and only while it shows (a covered window, or a
    /// sleeping app, draws on the frame asked as it shows again); a close
    /// closes that window alone, and the app when it was the last.
    pub(crate) fn handle_window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match self.windows.route(window_id, &event) {
            Routed::Redraw if self.windows.awake(window_id) => self.render_window(window_id),
            Routed::Redraw => {}
            Routed::CloseRequested => {
                self.windows.close(window_id);
                if self.windows.should_exit() {
                    event_loop.exit();
                }
            }
            Routed::Handled | Routed::Unknown => {}
        }
    }

    /// The frame of window `id`: the frame signal advances, the host draws
    /// (its own tail, `render.rs`), and the next frame is asked for only
    /// while that window animates or new work arrived during the frame.
    fn render_window(&mut self, id: WindowId) {
        let Some(host) = self.windows.get_mut(id) else {
            return;
        };
        let counter_value = self.frame_read.get();
        self.frame_write.set(counter_value + 1);
        host.render(&self.theme);
        if host.needs_redraw() {
            host.request_redraw();
        }
    }

    pub(crate) fn handle_about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        #[cfg(hot_reload)]
        if self.check_narrate_reload() {
            self.windows.invalidate_all();
        }

        // Render on demand, per window: a window with a change, an
        // animation or a pending request asks for its frame; the others
        // stay idle.
        self.windows.request_pending_redraws();
    }
}

/// Set up the HTML canvas for WASM rendering.
///
/// Attaches the winit canvas to `<body>` and sizes it to the viewport at the
/// device pixel ratio. Returns `Err` if any web-sys API is unavailable (e.g.
/// restricted iframe, headless context).
///
/// Re-exported as `engine::window::setup_wasm_canvas` so downstream apps with
/// their own winit event loop can reuse it. The initial width/height set
/// here are a first-paint fallback: with the canvas styled via CSS (e.g.
/// `width: 100vw; height: 100vh`), winit's `ResizeObserver` emits
/// `WindowEvent::Resized` and the app's `surface.configure` keeps the
/// buffer in sync afterwards.
#[cfg(target_arch = "wasm32")]
pub fn setup_wasm_canvas(window: &winit::window::Window) -> crate::error::PlevResult<()> {
    use crate::error::PlevError;
    use winit::platform::web::WindowExtWebSys;

    let canvas = window
        .canvas()
        .ok_or(PlevError::Wasm("canvas not available on window"))?;

    let doc = web_sys::window()
        .and_then(|w| w.document())
        .ok_or(PlevError::Wasm("no document available"))?;

    let body = doc
        .body()
        .ok_or(PlevError::Wasm("no <body> element in document"))?;

    body.append_child(&canvas)
        .map_err(|_| PlevError::Wasm("failed to append canvas to <body>"))?;

    let win = web_sys::window().ok_or(PlevError::Wasm("window object unavailable for sizing"))?;
    let dpr = win.device_pixel_ratio();

    let w = win
        .inner_width()
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(800.0);
    let h = win
        .inner_height()
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(600.0);

    canvas.set_width((w * dpr).round().max(1.0) as u32);
    canvas.set_height((h * dpr).round().max(1.0) as u32);

    Ok(())
}
