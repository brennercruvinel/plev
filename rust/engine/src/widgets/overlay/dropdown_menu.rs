//! The dropdown menu of the hoff core kit (`Layout/Header/UserDropdown`,
//! the MenuPage's menus): a [`ContextMenu`] opened by a trigger, as the
//! kit's headlessui `Menu` opens its `MenuItems`, placed by the trigger
//! inside the container that receives it ([`Popover`]'s placement, its
//! stack entry and fade, its dismissal).
//!
//! Projected ([`DropdownMenu::project`]): the trigger, declared by its own
//! widget, gains its `expanded`; while open, the menu `<trigger>.panel`
//! (`Role::Menu`, with a `close`) and its items `<trigger>.panel[<item
//! id>]`, as the context menu projects them. The arrow keys move among
//! the items through the projection (the gallery's keyboard), and the
//! highlight follows ([`DropdownMenu::highlight`]).

use accord::{NodeBuilder, SemId, SnapshotBuilder};

use crate::drawing::compositor::{Compositor, LayerId};
use crate::styling::theme::Theme;
use crate::widgets::feedback::{ContextMenu, MenuEntry};
use crate::widgets::overlay::popover::{Placement, Popover};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

/// A menu that a trigger opens.
#[derive(Clone, Debug)]
pub struct DropdownMenu {
    pub popover: Popover,
    pub menu: ContextMenu,
}

impl DropdownMenu {
    /// Below its trigger, its end on the trigger's end (the kit's `bottom
    /// end`).
    pub fn new(entries: Vec<MenuEntry>) -> Self {
        Self {
            popover: Popover::new(),
            menu: ContextMenu::new(entries),
        }
    }

    /// The identity of the trigger the menu hangs from: the menu binds as
    /// `<trigger>.panel`, each item as `<trigger>.panel[<item id>]`.
    pub fn sem(mut self, trigger: &SemId) -> Self {
        if let Some(panel) = Popover::panel_id(trigger) {
            self.menu = self.menu.sem(panel);
        }
        self
    }

    pub fn is_open(&self) -> bool {
        self.popover.is_open()
    }

    pub fn open(&mut self) -> bool {
        self.popover.open()
    }

    /// Closes it; the highlight goes with it.
    pub fn close(&mut self) -> bool {
        self.menu.set_highlighted(None);
        self.popover.close()
    }

    pub fn toggle(&mut self) {
        if self.is_open() {
            self.close();
        } else {
            self.open();
        }
    }

    /// The item `id` highlighted (the keys, a reader's focus), or none.
    pub fn highlight(&mut self, id: Option<u64>) {
        self.menu.set_highlighted(id);
    }

    /// Where the menu goes by `anchor` inside `bounds`.
    pub fn placement(&self, anchor: Rect, bounds: Rect, theme: &Theme) -> Placement {
        self.popover
            .placement(anchor, self.menu.size(theme), bounds, theme)
    }

    /// The pointer on the open menu at `panel` (its placed rect) and its
    /// `trigger`: the highlight follows the pointer, a press on an item
    /// picks it (its id; the menu closes), a press outside both closes it.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        trigger: Rect,
        panel: Rect,
        theme: &Theme,
    ) -> (EventResult, Option<u64>) {
        if !self.is_open() {
            return (EventResult::IGNORED, None);
        }
        let (r, picked) = self.menu.handle_event(event, panel.x, panel.y, theme);
        if picked.is_some() {
            self.close();
            return (r, picked);
        }
        if self.popover.dismiss(event, trigger, panel) {
            self.menu.set_highlighted(None);
            return (EventResult::changed(), None);
        }
        (r, None)
    }

    /// The menu at `panel` on `layer`.
    pub fn render(&self, c: &mut Compositor, layer: LayerId, panel: Rect, theme: &Theme) {
        self.menu.render(c, layer, theme, panel.x, panel.y);
    }

    /// Adds the menu to a projection where its trigger is declared: the
    /// trigger's `expanded`; while open, the menu named `label` under
    /// `parent`. Returns the menu's node, `None` while closed.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        trigger: &SemId,
        label: &str,
        parent: Option<&SemId>,
    ) -> Option<NodeBuilder<'b>> {
        b.reopen(trigger)?.state("expanded", self.is_open());
        if !self.is_open() {
            return None;
        }
        let id = self.menu.sem_id()?.clone();
        Some(self.menu.project(b, &id, label, parent))
    }
}
