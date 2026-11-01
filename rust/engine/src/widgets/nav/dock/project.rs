//! The dock told to a reader and an agent: the panel is a navigation
//! landmark, `expanded` while it shows (or is on its way in), its `mode`
//! (`peek`, `pinned`, `drawer`) and `pinned`, with a `toggle_expanded`
//! (in peek it shows or hides it, in the drawer it opens or closes it);
//! the pin is a toggle with its state, `on`, and a `toggle`; in the
//! drawer the menu button is a button, `expanded` while the drawer is
//! open, with an `activate`. The panel's own links are the sidebar's,
//! projected under the panel's id by the app.

use accord::{Role, SemId, SnapshotBuilder};

use crate::styling::theme::Theme;

use super::{DockMode, SidebarDock};

/// The pin's name: what it does, the same pinned or not (its state says
/// which).
pub const PIN_NAME: &str = "Keep the sidebar open";

/// The drawer's menu button's name.
pub const MENU_NAME: &str = "Open the navigation";

impl SidebarDock {
    /// Declares the panel as its id, named `label`, under `parent`, then
    /// its pin, and in the drawer its menu button: the order a reader
    /// meets them (the pin heads the panel, the app's links follow it
    /// under the same id). Nothing without an id; `true` when declared.
    #[track_caller]
    pub fn project(
        &self,
        b: &mut SnapshotBuilder,
        label: &str,
        parent: Option<&SemId>,
        theme: &Theme,
        w: f32,
    ) -> bool {
        let Some(id) = self.sem.as_ref() else {
            return false;
        };
        let mode = self.mode(theme, w);
        let mut node = b.node(id, Role::Navigation, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.state("expanded", self.expanded(theme, w))
            .state("mode", mode.name())
            .state("pinned", self.pinned);
        node.action("toggle_expanded")
            .reads(format!("{id}:expanded"));
        if let Some(pin) = self.pin_id() {
            let mut node = b.node(&pin, Role::Toggle, PIN_NAME);
            node.parent(id)
                .state("on", self.pinned)
                .enabled(mode != DockMode::Drawer);
            node.action("toggle").reads(format!("{pin}:on"));
        }
        if mode == DockMode::Drawer
            && let Some(menu) = self.menu_id()
        {
            let mut node = b.node(&menu, Role::Button, MENU_NAME);
            node.parent(id).state("expanded", self.drawer_open);
            node.action("activate").reads(format!("{menu}:expanded"));
        }
        true
    }
}
