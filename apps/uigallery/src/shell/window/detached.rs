//! The page in a window of its own (task-76 item 2), desktop only: `D`
//! outside a text field opens a second window for the active page (the
//! same page state, the same dispatcher; `ShowcaseView::detach`) and `D`
//! again, Escape in it or its close button closes it. The main window
//! keeps the chrome and the page: a change made in either window shows in
//! the other on its next frame. Each window has its own host (context,
//! compositor, clock): a hover or a scroll draws the window it happened
//! in alone, a change draws both, and the page's motion asks frames of
//! both (`ShowcaseView::detached_wake`); idle, neither draws.
//!
//! What the outside actors see: the agent port's bounds come from the
//! compositor of the page's own window while it is open
//! (`App::page_window_id`), and the projection carries a `window` node
//! then (`ShowcaseView::semantics`). What stays with the main window: the
//! accessibility bridge (the page's own window has none), the recording's
//! frames (the page window's pointer inputs are recorded in that window's
//! coordinates, its frames are not), the right click and the overlays a
//! page opens (a modal, a toast: drawn in the main window, where the
//! overlay layers live).

use winit::event_loop::ActiveEventLoop;
use winit::window::{WindowAttributes, WindowId};

use super::super::App;

/// The page's own window and the cursor in it (logical pixels).
pub(crate) struct PageWindow {
    pub(crate) id: WindowId,
    pub(crate) cursor: (f32, f32),
}

impl App {
    /// `D`: the page to a window of its own, or back.
    pub(crate) fn toggle_page_window(&mut self, event_loop: &ActiveEventLoop) {
        if self.page_window.is_some() {
            self.close_page_window();
        } else {
            self.open_page_window(event_loop);
        }
    }

    /// Opens the page's own window. A placeholder page has nothing to
    /// show; a window that does not open is logged.
    fn open_page_window(&mut self, event_loop: &ActiveEventLoop) {
        let title = self.view.page_title();
        if self.view.current().pane().is_none() {
            log::info!("the {title} page has nothing to show in a window of its own");
            return;
        }
        #[allow(unused_mut)] // mut needed for cfg(macos)
        let mut attrs = WindowAttributes::default()
            .with_title(format!("plev, {title}"))
            .with_inner_size(winit::dpi::LogicalSize::new(900u32, 640u32));
        #[cfg(target_os = "macos")]
        {
            attrs = super::super::system::background::window(attrs);
        }
        let id = match self.windows.open(event_loop, attrs) {
            Ok(id) => id,
            Err(e) => {
                log::error!("the page window did not open: {e}");
                return;
            }
        };
        let Some(host) = self.windows.get(id) else {
            return;
        };
        let (lw, lh) = host.logical_size();
        let sf = host.scale_factor() as f32;
        if !self.view.detach(lw, lh, sf) {
            self.windows.close(id);
            return;
        }
        self.page_window = Some(PageWindow {
            id,
            cursor: (0.0, 0.0),
        });
        log::info!("the {title} page shows in a window of its own too");
        self.invalidate();
    }

    /// The page's own window closes; the main window is again the only
    /// one showing the page.
    pub(super) fn close_page_window(&mut self) {
        let Some(page) = self.page_window.take() else {
            return;
        };
        self.view.attach();
        self.windows.close(page.id);
        self.capture_follows();
        log::info!(
            "the {} page shows in the main window alone",
            self.view.page_title()
        );
        self.invalidate();
    }
}
