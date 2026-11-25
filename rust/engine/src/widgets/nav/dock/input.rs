//! What moves the dock: the pointer at the edge and over the panel, the
//! pauses running out, the keyboard focus, the pin, the drawer's menu
//! button, Escape, a finger. Every ask runs the same path, whoever made
//! it: a person's click on the pin and an agent's `toggle` on `<id>.pin`
//! both call [`SidebarDock::toggle_pin`].

use std::time::Duration;

use crate::styling::theme::Theme;
use crate::widgets::wake::Wake;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

use super::{DockMode, SidebarDock};

impl SidebarDock {
    /// The window is `w` wide now: a mode that changed under the panel
    /// (the window narrowed past the content's rule) settles at once.
    pub fn sync(&mut self, theme: &Theme, w: f32) {
        self.settle(theme, w);
    }

    /// Whether `(x, y)` is on the panel as it stands (shown or sliding):
    /// the events there are the panel's, not the page's.
    pub fn covers(&self, x: f32, y: f32, theme: &Theme, window: Rect) -> bool {
        self.layout(theme, window)
            .panel
            .is_some_and(|p| p.contains(x, y))
    }

    /// A pointer event, before the page sees it. In peek, a pointer in
    /// the zone, or over the panel while it shows, asks it in, after the
    /// pause; one that leaves asks it out, after its own. In the open
    /// drawer every event is the dock's (the page is inert) and a press
    /// off the panel closes it. `handled` when the page must not see the
    /// event: it is on the panel, or the drawer holds the window.
    pub fn pointer(&mut self, event: &WidgetEvent, theme: &Theme, window: Rect) -> EventResult {
        self.sync(theme, window.w);
        let l = self.layout(theme, window);
        let (x, y) = event.pos();
        let on_panel = l.panel.is_some_and(|p| p.contains(x, y));
        if l.mode != DockMode::Drawer && matches!(event, WidgetEvent::MouseMove { .. }) {
            // pinned too: a pointer still on the panel as the pin lets it
            // go keeps it. Shown (a pause to hide running or not) or on its
            // way in, the whole panel holds it; out, the zone calls it.
            let holds = self.target > 0.5 || self.pending.is_some_and(|p| p.show);
            let near = if l.mode == DockMode::Pinned || holds {
                l.rest.contains(x, y)
            } else {
                l.zone.is_some_and(|z| z.contains(x, y))
            };
            if near != self.pointer {
                self.pointer = near;
                if l.mode == DockMode::Peek {
                    self.follow(theme, window.w, true);
                }
            }
        }
        match l.mode {
            DockMode::Drawer if self.drawer_open => {
                if matches!(event, WidgetEvent::MouseDown { .. }) && !on_panel {
                    self.close_drawer(theme, window.w);
                    return EventResult::clicked();
                }
                EventResult {
                    handled: true,
                    ..EventResult::IGNORED
                }
            }
            _ => EventResult {
                handled: on_panel,
                ..EventResult::IGNORED
            },
        }
    }

    /// The pauses and the slide: a frame now when the panel moved, the
    /// next one at once while it slides, a deadline while a pause runs.
    pub fn wake(&mut self, dt: f32, theme: &Theme, w: f32) -> Wake {
        self.sync(theme, w);
        let mut wake = Wake::idle();
        if let Some(p) = &mut self.pending {
            p.left -= dt;
            if p.left <= 0.0 {
                let show = p.show;
                self.slide(show, theme, false);
                wake = Wake::now();
            } else {
                wake = Wake::after(Duration::from_secs_f32(p.left));
            }
        }
        let was = self.reveal.is_animating();
        self.reveal.tick(dt);
        let moving = if self.reveal.is_animating() {
            Wake::continuous()
        } else if was {
            Wake::now()
        } else {
            Wake::idle()
        };
        wake.merge(moving)
    }

    /// The keyboard focus walked into the panel (`true`) or out of it: in
    /// peek it shows the panel at once (Tab reaches it while it is out)
    /// and hides it as the focus leaves.
    pub fn set_focus_inside(&mut self, on: bool, theme: &Theme, w: f32) {
        if self.focus != on {
            self.focus = on;
            self.follow(theme, w, false);
        }
    }

    pub fn has_focus_inside(&self) -> bool {
        self.focus
    }

    /// A reader's or an agent's `toggle_expanded` on the panel: in peek,
    /// shown or hidden at once; in the drawer, open or closed; pinned,
    /// nothing (it is always shown). `true` when something changed.
    pub fn toggle_expanded(&mut self, theme: &Theme, w: f32) -> bool {
        match self.mode(theme, w) {
            DockMode::Pinned => false,
            DockMode::Drawer => {
                if self.drawer_open {
                    self.close_drawer(theme, w)
                } else {
                    self.open_drawer(theme, w)
                }
            }
            DockMode::Peek => {
                let shown = self.is_expanded();
                self.held = !shown;
                if shown {
                    // whatever held it, it goes
                    self.pointer = false;
                    self.focus = false;
                }
                self.follow(theme, w, false);
                true
            }
        }
    }

    /// The pin: pinned, the panel stays beside the page; let go, it
    /// peeks again, staying while the pointer is still on it.
    pub fn toggle_pin(&mut self, theme: &Theme, w: f32) -> bool {
        self.pinned = !self.pinned;
        let mode = self.mode(theme, w);
        if mode == DockMode::Drawer {
            // a drawer stays one; the preference waits for a wider window
            return true;
        }
        self.mode = mode;
        if self.pinned {
            self.slide(true, theme, true);
        } else {
            self.follow(theme, w, false);
        }
        true
    }

    /// The drawer's menu button: the panel opens over a scrim.
    pub fn open_drawer(&mut self, theme: &Theme, w: f32) -> bool {
        if self.mode(theme, w) != DockMode::Drawer || self.drawer_open {
            return false;
        }
        self.drawer_open = true;
        // the button the drawer now covers keeps no hover nor press
        self.menu_press.hovered = false;
        self.menu_press.pressed = false;
        self.follow(theme, w, false);
        true
    }

    /// The drawer closes (Escape, a press off it, a link picked).
    pub fn close_drawer(&mut self, theme: &Theme, w: f32) -> bool {
        if !self.drawer_open {
            return false;
        }
        self.drawer_open = false;
        self.focus = false;
        self.close_press = Default::default();
        self.follow(theme, w, false);
        true
    }

    /// Escape: the open drawer closes; a panel shown by the focus or by an
    /// ask hides. `true` when it did something.
    pub fn escape(&mut self, theme: &Theme, w: f32) -> bool {
        match self.mode(theme, w) {
            DockMode::Drawer => self.close_drawer(theme, w),
            DockMode::Peek if self.focus || self.held => {
                self.focus = false;
                self.held = false;
                self.follow(theme, w, false);
                true
            }
            _ => false,
        }
    }

    /// A link was picked: the drawer closes; a peeking panel stays while
    /// the pointer is on it, as it would.
    pub fn picked(&mut self, theme: &Theme, w: f32) {
        self.held = false;
        self.close_drawer(theme, w);
        self.follow(theme, w, false);
    }

    /// The last input was a finger (`true`) or a pointer: a finger has
    /// no hover to peek with, so the panel is a drawer.
    pub fn set_touch(&mut self, on: bool, theme: &Theme, w: f32) {
        if self.touch != on {
            self.touch = on;
            self.sync(theme, w);
        }
    }
}
