//! The dev mode in the gallery's shell (`engine::runtime::dev`): the key
//! that opens and closes its panel, and the pointer, which goes to it
//! before the page. The frame's side of the mount is in `window/pace`:
//! the page's share of the window, the draw after the page, and the
//! frames the panel asks for.

use engine::widgets::prelude::WidgetEvent;
use winit::window::WindowId;

use super::App;

impl App {
    /// F12 in the main window: the dev mode's panel opens or closes, as a
    /// press on its gear or its close does, and the page takes its new
    /// share. `false` in a launch without a dev mode: the key is the
    /// page's.
    pub(super) fn dev_key(&mut self, id: WindowId) -> bool {
        if !self.dev.is_available() || Some(id) != self.main {
            return false;
        }
        self.dev.set_open(!self.dev.is_open());
        self.viewport_changed(id);
        true
    }

    /// A pointer event in the main window for the dev mode first; `Some`
    /// with whether a frame is needed when it kept the event from the
    /// page. A panel that opened, closed or moved resizes the page.
    pub(super) fn dev_event(&mut self, id: WindowId, event: &WidgetEvent) -> Option<bool> {
        if !self.dev.is_available() || Some(id) != self.main {
            return None;
        }
        let host = self.windows.get_mut(id)?;
        let (w, h) = host.logical_size();
        let took = self.dev.event(event, w, h, host.compositor());
        // a frame the panel asks for on its own account (the pointer over
        // one of its places, a press on one) is the tool's, not the app's:
        // it stays out of the history the panel shows
        if took.taken && took.changed {
            host.dev_mut().mark_tool();
        }
        if took.resized {
            self.viewport_changed(id);
        }
        if took.changed {
            self.invalidate_in(id);
        }
        took.taken.then_some(took.changed)
    }
}
