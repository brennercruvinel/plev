//! Several windows in one app: a map `WindowId -> WindowHost`, the routing
//! of each `WindowEvent` to its owner, a frame only for the window that
//! asked, one window closed without the app, and the app closed when the
//! last window is (unless the app says otherwise).
//!
//! The app keeps what the windows share (the state, the dispatcher, the
//! theme); a window is a view of it. `Windows` never draws: `route`
//! answers [`Routed::Redraw`] and the caller builds that window's scene
//! and presents it through its host, so the app decides what each window
//! shows and nothing is drawn for a window that did not ask.

use std::sync::Arc;

use web_time::Instant;
use winit::error::OsError;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{WindowAttributes, WindowId};

#[cfg(not(target_arch = "wasm32"))]
use crate::gpu::RenderConfig;

use super::host::WindowHost;

/// What `Windows::route` did with an event, and what is left to its
/// caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Routed {
    /// The owner took it (input buffered, a resize applied).
    Handled,
    /// The window asked for a frame and its buffered events are in its
    /// input state: build the scene and present it through the host.
    Redraw,
    /// The person asked to close the window: `close` it, or keep it.
    CloseRequested,
    /// No host has this id (closed already, or never ours).
    Unknown,
}

pub struct Windows {
    /// In the order they were opened; the first is the main window.
    hosts: Vec<(WindowId, WindowHost)>,
    /// Hosts whose window the platform took (android): they come back
    /// with a new window and a new id on `resume`.
    suspended: Vec<WindowHost>,
    /// A window was opened at some point: an empty map then means the last
    /// one closed, not that none opened yet.
    opened: bool,
    exit_when_empty: bool,
}

impl Default for Windows {
    fn default() -> Self {
        Self::new()
    }
}

impl Windows {
    pub fn new() -> Self {
        Self {
            hosts: Vec::new(),
            suspended: Vec::new(),
            opened: false,
            exit_when_empty: true,
        }
    }

    pub fn len(&self) -> usize {
        self.hosts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.hosts.is_empty()
    }

    /// The ids in opening order.
    pub fn ids(&self) -> impl Iterator<Item = WindowId> + '_ {
        self.hosts.iter().map(|(id, _)| *id)
    }

    /// The main window: the first one still open.
    pub fn first(&self) -> Option<WindowId> {
        self.hosts.first().map(|(id, _)| *id)
    }

    pub fn contains(&self, id: WindowId) -> bool {
        self.hosts.iter().any(|(k, _)| *k == id)
    }

    pub fn get(&self, id: WindowId) -> Option<&WindowHost> {
        self.hosts.iter().find(|(k, _)| *k == id).map(|(_, h)| h)
    }

    pub fn get_mut(&mut self, id: WindowId) -> Option<&mut WindowHost> {
        self.hosts
            .iter_mut()
            .find(|(k, _)| *k == id)
            .map(|(_, h)| h)
    }

    pub fn iter(&self) -> impl Iterator<Item = (WindowId, &WindowHost)> + '_ {
        self.hosts.iter().map(|(id, h)| (*id, h))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (WindowId, &mut WindowHost)> + '_ {
        self.hosts.iter_mut().map(|(id, h)| (*id, h))
    }

    /// Keep `host` under `id`, replacing one there. For a host built by
    /// the caller: a headless one in a test under `WindowId::dummy()`, or
    /// one whose window needed something before its device.
    pub fn insert(&mut self, id: WindowId, host: WindowHost) -> WindowId {
        self.opened = true;
        match self.hosts.iter_mut().find(|(k, _)| *k == id) {
            Some(slot) => slot.1 = host,
            None => self.hosts.push((id, host)),
        }
        id
    }

    /// Open a window with `attrs` and the host for it. On native targets
    /// the host comes with its device (blocking on the adapter, as every
    /// constructor did); on wasm the device request cannot block, so the
    /// host waits for [`attach_gpu`](Self::attach_gpu) with the result of
    /// `GpuContext::new` the shell spawned.
    pub fn open(
        &mut self,
        event_loop: &ActiveEventLoop,
        attrs: WindowAttributes,
    ) -> Result<WindowId, OsError> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.open_with_config(event_loop, attrs, RenderConfig::default())
        }
        #[cfg(target_arch = "wasm32")]
        {
            let window = Arc::new(event_loop.create_window(attrs)?);
            let id = window.id();
            Ok(self.insert(id, WindowHost::new(window)))
        }
    }

    /// [`open`](Self::open) with the render config of the new context.
    /// Each window has a context of its own, with its own `wgpu::Instance`:
    /// the constructor makes one per context, and a recovery asks that
    /// instance for the next adapter.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn open_with_config(
        &mut self,
        event_loop: &ActiveEventLoop,
        attrs: WindowAttributes,
        config: RenderConfig,
    ) -> Result<WindowId, OsError> {
        let window = Arc::new(event_loop.create_window(attrs)?);
        let id = window.id();
        Ok(self.insert(id, WindowHost::open_with_config(window, config)))
    }

    /// The device a shell requested asynchronously for window `id`
    /// (wasm). `false` when the window closed meanwhile.
    pub fn attach_gpu(&mut self, id: WindowId, gpu: crate::gpu::GpuContext) -> bool {
        match self.get_mut(id) {
            Some(host) => {
                host.attach_gpu(gpu);
                true
            }
            None => false,
        }
    }

    /// One event of window `id` to its host. The host takes input, IME,
    /// drops, resizes and scale changes; the close and the redraw request
    /// come back as the answer, since only the caller knows whether to
    /// close and what to draw. Under the accessibility feature the
    /// window's bridge sees the event first.
    pub fn route(&mut self, id: WindowId, event: &WindowEvent) -> Routed {
        let Some(host) = self.get_mut(id) else {
            return Routed::Unknown;
        };
        #[cfg(all(feature = "accessibility", not(target_arch = "wasm32")))]
        host.accessibility_event(event);
        match event {
            WindowEvent::CloseRequested => Routed::CloseRequested,
            WindowEvent::RedrawRequested => {
                host.process_events();
                Routed::Redraw
            }
            other => {
                host.handle_event(other);
                Routed::Handled
            }
        }
    }

    /// Close window `id`: its host leaves the map and is returned, so the
    /// caller may keep something of it; dropping it drops the window and
    /// its context (the platform sends `Destroyed` afterwards, which routes
    /// as `Unknown`). The other windows are untouched.
    pub fn close(&mut self, id: WindowId) -> Option<WindowHost> {
        let at = self.hosts.iter().position(|(k, _)| *k == id)?;
        Some(self.hosts.remove(at).1)
    }

    /// The last window closed and the app is to close with it: `true`
    /// once a window was opened, none is left, and `exit_when_last_closes`
    /// is on (the default). A shell calls `event_loop.exit()` on it.
    pub fn should_exit(&self) -> bool {
        self.exit_when_empty && self.opened && self.hosts.is_empty() && self.suspended.is_empty()
    }

    /// Whether closing the last window closes the app (default `true`).
    /// Off, the loop keeps running without a window (a tray app, item 5).
    pub fn set_exit_when_last_closes(&mut self, on: bool) {
        self.exit_when_empty = on;
    }

    pub fn exit_when_last_closes(&self) -> bool {
        self.exit_when_empty
    }

    /// Ask for a frame of window `id` alone. `false` for an unknown id.
    pub fn request_redraw(&mut self, id: WindowId) -> bool {
        match self.get_mut(id) {
            Some(host) => {
                host.request_redraw();
                true
            }
            None => false,
        }
    }

    /// Render on demand, per window: every host with a reason to draw
    /// asks the platform for its frame, the others stay idle. For a
    /// shell's `about_to_wait`. Returns how many asked.
    pub fn request_pending_redraws(&mut self) -> usize {
        let mut asked = 0;
        for (_, host) in &mut self.hosts {
            if host.needs_redraw() {
                host.request_redraw();
                asked += 1;
            }
        }
        asked
    }

    /// Every window's scene is stale (a theme change, a hot reload): each
    /// is invalidated and asks for a frame.
    pub fn invalidate_all(&mut self) {
        for (_, host) in &mut self.hosts {
            host.compositor_mut().invalidate();
            host.request_redraw();
        }
    }

    /// Continuous frames on every window, or none (`WindowHost::set_animating`).
    pub fn set_animating(&mut self, on: bool) {
        for (_, host) in &mut self.hosts {
            host.set_animating(on);
        }
    }

    /// The earliest instant any window needs a frame on its own
    /// (`WindowHost::set_deadline`); the loop's `WaitUntil`.
    pub fn earliest_deadline(&self) -> Option<Instant> {
        self.hosts
            .iter()
            .filter_map(|(_, host)| host.deadline())
            .min()
    }

    /// The system is short of memory: every window's glyph caches go.
    pub fn purge_caches(&mut self) {
        for (_, host) in &mut self.hosts {
            host.purge_caches();
        }
    }

    /// The app went to the background and the platform takes its windows
    /// (android): every host drops its surface and window and waits for
    /// `resume`. The map is empty meanwhile; `should_exit` stays false.
    pub fn suspend(&mut self) {
        for (_, mut host) in self.hosts.drain(..) {
            host.suspend();
            self.suspended.push(host);
        }
    }

    pub fn has_suspended(&self) -> bool {
        !self.suspended.is_empty()
    }

    /// Back from the background: each suspended host gets a new window
    /// from `attrs` (called once per host) and its surface again. The new
    /// ids, in the old order.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn resume(
        &mut self,
        event_loop: &ActiveEventLoop,
        mut attrs: impl FnMut() -> WindowAttributes,
    ) -> Result<Vec<WindowId>, OsError> {
        let mut ids = Vec::new();
        for mut host in std::mem::take(&mut self.suspended) {
            let window = Arc::new(event_loop.create_window(attrs())?);
            let id = window.id();
            host.resume(window);
            self.insert(id, host);
            ids.push(id);
        }
        Ok(ids)
    }
}
