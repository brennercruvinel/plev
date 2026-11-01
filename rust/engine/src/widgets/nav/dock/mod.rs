//! The dock of a sidebar: where an app's navigation panel is and how it
//! comes and goes. It draws nothing of the panel (the app's sidebar,
//! [`HomeSidebar`](super::HomeSidebar) in the kit's look, draws into the
//! rect the dock gives it); it is the behavior: three modes, the state
//! that moves between them, the geometry, the timers and the projection.
//!
//! - **Peek**, the default: the panel is out of the window and the page
//!   has all of it. A pointer that comes near the window's left edge
//!   (the recipe's `zone`) and stays a moment (`enter_delay`) slides the
//!   panel in, over the page; a pointer that leaves it, after a moment
//!   (`leave_delay`), slides it out. A keyboard focus that walks into it
//!   shows it as well, and a reader's or an agent's `toggle_expanded`.
//! - **Pinned**: the panel stays, beside the page, which starts past it.
//!   The app's pin button asks it ([`SidebarDock::toggle_pin`]); a
//!   person, a reader and an agent ask the same toggle.
//! - **Drawer**: the window cannot hold the panel and the page's
//!   narrowest content beside it ([`DockRecipe::fits`], a rule on the
//!   content, not on the screen's class), or the input is a finger: a
//!   menu button opens the panel as a modal drawer, over a scrim, which
//!   Escape, a press off the panel and a link picked close.
//!
//! Every value is `theme.navigation.dock`, a [`DockRecipe`] (the theme's
//! size and motion tokens). With reduced motion the slide is a cut (the
//! `Tween` lands at once).
//!
//! `mod.rs` is the state and the geometry, `input.rs` the pointer, the
//! timers and the keys, `pin.rs` the pin the dock draws in the panel's
//! head, `project.rs` what is told to a reader and an agent.
//!
//! [`DockRecipe`]: crate::widgets::nav::DockRecipe
//! [`DockRecipe::fits`]: crate::widgets::nav::DockRecipe::fits

mod input;
mod pin;
mod project;
#[cfg(test)]
mod tests;

use accord::SemId;

use crate::compose::animation::{Tween, reduced_motion};
use crate::styling::theme::Theme;
use crate::widgets::buttons::Press;
use crate::widgets::widget::Rect;

/// How the panel stands beside the page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DockMode {
    Peek,
    Pinned,
    Drawer,
}

impl DockMode {
    /// The name a reader and an agent read (`mode`).
    pub fn name(self) -> &'static str {
        match self {
            DockMode::Peek => "peek",
            DockMode::Pinned => "pinned",
            DockMode::Drawer => "drawer",
        }
    }
}

/// One frame of the dock's geometry in a window.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DockLayout {
    pub mode: DockMode,
    /// The panel settled: held to the window's left edge, its full
    /// height.
    pub rest: Rect,
    /// The panel now, slid by the part not shown; `None` while it is out
    /// of the window altogether.
    pub panel: Option<Rect>,
    /// How much of it shows, 0 to 1.
    pub shown: f32,
    /// Where the page's column starts: past the panel while pinned, at
    /// the window's edge otherwise (the panel floats over the page).
    pub page_x: f32,
    /// The strip a pointer reveals a peeking panel from; `None` but in
    /// peek.
    pub zone: Option<Rect>,
    /// The panel stands over the page (peek, drawer): drawn above it, on
    /// the floating surface.
    pub floats: bool,
    /// The drawer is open: a scrim over the window, the rest inert.
    pub modal: bool,
    /// The drawer's menu button is wanted (drawer, closed or open).
    pub menu: bool,
}

/// A change asked to come after a pause: the panel shown (`true`) or
/// hidden, in `left` seconds.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Pending {
    show: bool,
    left: f32,
}

#[derive(Clone, Debug)]
pub struct SidebarDock {
    pinned: bool,
    /// The last input was a finger: the panel is a drawer.
    touch: bool,
    drawer_open: bool,
    /// A pointer is in the zone or over the shown panel (peek).
    pointer: bool,
    /// The keyboard focus is in the panel.
    focus: bool,
    /// A reader or an agent asked it shown (`toggle_expanded`).
    held: bool,
    pending: Option<Pending>,
    /// The part shown, 0 to 1, and where it goes.
    reveal: Tween<f32>,
    target: f32,
    /// The mode of the last layout: a mode that changes under the panel
    /// (the window narrowed) settles it at once.
    mode: DockMode,
    /// The pin's hover, press and focus (`pin.rs`).
    pin_press: Press,
    sem: Option<SemId>,
}

impl Default for SidebarDock {
    fn default() -> Self {
        Self::new()
    }
}

impl SidebarDock {
    /// Peeking, out of the window.
    pub fn new() -> Self {
        Self {
            pinned: false,
            touch: false,
            drawer_open: false,
            pointer: false,
            focus: false,
            held: false,
            pending: None,
            reveal: Tween::new(0.0, 0.0, Default::default()),
            target: 0.0,
            mode: DockMode::Peek,
            pin_press: Press::default(),
            sem: None,
        }
    }

    /// Pinned open from the start (the app's saved preference).
    pub fn pinned(mut self, on: bool) -> Self {
        self.pinned = on;
        self.mode = if on { DockMode::Pinned } else { DockMode::Peek };
        let to = if on { 1.0 } else { 0.0 };
        self.reveal.reset(to);
        self.target = to;
        self
    }

    /// `id` is the panel's (a navigation landmark); its pin is
    /// `<id>.pin`, the drawer's menu button `<id>.menu`.
    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    pub fn pin_id(&self) -> Option<SemId> {
        self.sem.as_ref()?.child("pin").ok()
    }

    pub fn menu_id(&self) -> Option<SemId> {
        self.sem.as_ref()?.child("menu").ok()
    }

    pub fn is_pinned(&self) -> bool {
        self.pinned
    }

    pub fn is_drawer_open(&self) -> bool {
        self.drawer_open
    }

    /// The mode in a window `w` wide.
    pub fn mode(&self, theme: &Theme, w: f32) -> DockMode {
        if self.touch || !theme.navigation.dock.fits(w, theme) {
            DockMode::Drawer
        } else if self.pinned {
            DockMode::Pinned
        } else {
            DockMode::Peek
        }
    }

    /// Whether the panel is asked to show in `mode`, the pauses aside.
    fn wanted(&self, mode: DockMode) -> bool {
        match mode {
            DockMode::Pinned => true,
            DockMode::Drawer => self.drawer_open,
            DockMode::Peek => self.pointer || self.focus || self.held,
        }
    }

    /// Shown or on its way in (a pause still running counts as where it
    /// goes), as the panel last settled.
    pub fn is_expanded(&self) -> bool {
        match self.pending {
            Some(p) => p.show,
            None => self.target > 0.5,
        }
    }

    /// What a reader and an agent read as `expanded` in a window `w`
    /// wide: always while pinned, while open as a drawer, shown or on its
    /// way in while it peeks.
    pub fn expanded(&self, theme: &Theme, w: f32) -> bool {
        let mode = self.mode(theme, w);
        match mode {
            DockMode::Pinned => true,
            DockMode::Drawer => self.drawer_open,
            DockMode::Peek if mode != self.mode => self.wanted(mode),
            DockMode::Peek => self.is_expanded(),
        }
    }

    /// The part shown now, 0 to 1.
    pub fn shown(&self) -> f32 {
        self.reveal.get().clamp(0.0, 1.0)
    }

    pub fn is_animating(&self) -> bool {
        self.reveal.is_animating()
    }

    pub fn layout(&self, theme: &Theme, window: Rect) -> DockLayout {
        let r = theme.navigation.dock;
        let mode = self.mode(theme, window.w);
        let w = r.width.min(window.w);
        let rest = Rect::new(window.x, window.y, w, window.h);
        // a mode the dock has not settled into yet (the window changed
        // since its last sync) stands as it will: settled at once
        let shown = if mode == DockMode::Pinned {
            1.0
        } else if mode != self.mode {
            if self.wanted(mode) { 1.0 } else { 0.0 }
        } else {
            self.shown()
        };
        let panel = (shown > 0.0).then(|| {
            let mut p = rest;
            p.x -= p.w * (1.0 - shown);
            p
        });
        let peek = mode == DockMode::Peek;
        DockLayout {
            mode,
            rest,
            panel,
            shown,
            page_x: if mode == DockMode::Pinned {
                rest.right()
            } else {
                window.x
            },
            zone: peek.then(|| Rect::new(window.x, window.y, r.zone, window.h)),
            floats: mode != DockMode::Pinned,
            modal: mode == DockMode::Drawer && self.drawer_open,
            menu: mode == DockMode::Drawer,
        }
    }

    /// Slides toward `show` now (a cut with reduced motion, or when
    /// `cut`), dropping any pause.
    fn slide(&mut self, show: bool, theme: &Theme, cut: bool) {
        self.pending = None;
        let to = if show { 1.0 } else { 0.0 };
        let under_way = self.target == to && self.reveal.is_animating();
        self.target = to;
        if cut || reduced_motion() {
            self.reveal.reset(to);
            return;
        }
        let from = self.reveal.get();
        if !under_way && from != to {
            let r = theme.navigation.dock;
            self.reveal = Tween::new(from, r.slide, r.slide_easing);
            self.reveal.set_target(to);
        }
    }

    /// Settles toward what `mode` wants: at once, or after the pause the
    /// recipe gives a pointer.
    fn follow(&mut self, theme: &Theme, w: f32, paused: bool) {
        if self.settle(theme, w) {
            return;
        }
        let mode = self.mode;
        let want = self.wanted(mode);
        let at = self.target > 0.5;
        if want == at {
            self.pending = None;
            return;
        }
        if paused && mode == DockMode::Peek {
            let r = theme.navigation.dock;
            let left = if want { r.enter_delay } else { r.leave_delay };
            if self.pending.map(|p| p.show) != Some(want) {
                self.pending = Some(Pending { show: want, left });
            }
        } else {
            self.slide(want, theme, false);
        }
    }

    /// A mode that changed under the panel (the window crossed the
    /// content's rule, a finger came or went) settles at once, without a
    /// slide and without a pause. `true` when it did.
    fn settle(&mut self, theme: &Theme, w: f32) -> bool {
        let mode = self.mode(theme, w);
        if mode == self.mode {
            return false;
        }
        self.mode = mode;
        let want = self.wanted(mode);
        self.slide(want, theme, true);
        true
    }
}
