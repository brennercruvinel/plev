//! The menubar: a row of triggers, each opening its [`DropdownMenu`], as
//! the kit's toolbar and asset header line their popovers up. One menu is
//! open at a time; while one is, the pointer moving onto another trigger
//! opens that one instead, and the keys step to the next or the previous
//! menu. Each menu keeps its own popover (placement, stack entry, fade,
//! projection); the bar only says which one is open.

use accord::SemId;

use crate::drawing::compositor::{Compositor, LayerId};
use crate::styling::theme::Theme;
use crate::widgets::buttons::paint_surface;
use crate::widgets::feedback::MenuEntry;
use crate::widgets::overlay::dropdown_menu::DropdownMenu;
use crate::widgets::overlay::popover::Align;
use crate::widgets::widget::Rect;

/// The bar at `rect` (`theme.overlay.menubar`): its shadows, its fill, its
/// border; the titles are drawn after, inside it less its `pad`.
pub fn render_bar(c: &mut Compositor, layer: LayerId, rect: Rect, theme: &Theme) {
    let t = &theme.overlay.menubar;
    let radius = t.radius.min(rect.h / 2.0);
    paint_surface(c, layer, rect, radius, &t.surface, 1.0, false, theme);
}

/// A row of menus, at most one open.
#[derive(Clone, Debug, Default)]
pub struct Menubar {
    /// Each menu's title and its dropdown, in order.
    pub menus: Vec<(String, DropdownMenu)>,
}

impl Menubar {
    /// Each menu below its title, its start on the title's start.
    pub fn new(menus: Vec<(String, Vec<MenuEntry>)>) -> Self {
        let menus = menus
            .into_iter()
            .map(|(title, entries)| {
                let mut menu = DropdownMenu::new(entries);
                menu.popover.align = Align::Start;
                (title, menu)
            })
            .collect();
        Self { menus }
    }

    /// The bar's identity: menu `i`'s trigger is `<bar>[<i>]`, its menu
    /// `<bar>[<i>].panel`.
    pub fn sem(mut self, bar: &SemId) -> Self {
        for (i, (_, menu)) in self.menus.iter_mut().enumerate() {
            *menu =
                std::mem::replace(menu, DropdownMenu::new(Vec::new())).sem(&bar.key(i.to_string()));
        }
        self
    }

    /// The menu open now.
    pub fn open_index(&self) -> Option<usize> {
        self.menus.iter().position(|(_, m)| m.is_open())
    }

    /// Opens menu `i`, closing any other.
    pub fn open(&mut self, i: usize) {
        for (at, (_, menu)) in self.menus.iter_mut().enumerate() {
            if at == i {
                menu.open();
            } else {
                menu.close();
            }
        }
    }

    pub fn close(&mut self) -> bool {
        let open = self.open_index();
        if let Some(i) = open {
            self.menus[i].1.close();
        }
        open.is_some()
    }

    /// A press of trigger `i`: its menu opens, or closes when it was open.
    pub fn press(&mut self, i: usize) {
        if self.open_index() == Some(i) {
            self.close();
        } else {
            self.open(i);
        }
    }

    /// The pointer on trigger `i`: while another menu is open, `i`'s opens
    /// instead. `true` when that changed which menu is open.
    pub fn hover(&mut self, i: usize) -> bool {
        match self.open_index() {
            Some(open) if open != i && i < self.menus.len() => {
                self.open(i);
                true
            }
            _ => false,
        }
    }

    /// The keys step `by` menus from the open one, wrapping. `false` with
    /// none open.
    pub fn step(&mut self, by: isize) -> bool {
        let (Some(open), n) = (self.open_index(), self.menus.len() as isize) else {
            return false;
        };
        let next = (open as isize + by).rem_euclid(n.max(1)) as usize;
        self.open(next);
        true
    }
}
