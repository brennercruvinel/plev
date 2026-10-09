//! A person's pointer and keys on the line of the generated surface and
//! on the column of an open one (task-96), and their drawing: a click on
//! the strip is the line's widgets, on the column the surface's, each act
//! a request through the one dispatcher (the line's buttons observed on
//! the whole projection as the person, the surface's widgets building
//! their own, `surface::Surface::handle_event`); the wheel and the hover
//! the same way; the keys while the intent field or a field of a surface
//! has the keyboard. Where they sit is `crate::generated::row`.

use engine::widgets::form::text_input::single_line;
use engine::widgets::form::{ClipKey, EditKey};
use engine::widgets::prelude::{EventResult, WidgetEvent};
use engine::runtime::platform::clipboard::ClipboardProvider;
use engine::drawing::compositor::Compositor;
use accord::{Actor, Receipt, Request, SemId, Snapshot};
use screen::SurfaceHost;

use super::super::capability::Workbench;
use crate::generative::row::Asked;

/// The workbench as a surface reaches it: the one dispatcher, and the
/// workspace's own projection, which is what the surface shows.
struct App<'a>(&'a mut Workbench);

impl screen::Dispatch for App<'_> {
    fn dispatch(&mut self, req: Request) -> Option<Receipt> {
        let id = self.0.dispatch(req);
        self.0.receipt(id).cloned()
    }

    fn project(&self) -> Option<Snapshot> {
        self.0.app_snapshot().ok()
    }
}

impl Workbench {
    /// Lends the host to `f` with the workbench, so a surface reaches the
    /// dispatcher while the workbench holds the host.
    fn with_host<R>(&mut self, f: impl FnOnce(&mut SurfaceHost, &mut Workbench) -> R) -> Option<R> {
        let mut host = std::mem::take(&mut self.generated.as_mut()?.host);
        let out = f(&mut host, self);
        if let Some(g) = self.generated.as_mut() {
            g.host = host;
        }
        Some(out)
    }

    /// A person asks `action` of the line's node `target`, observed on
    /// the whole projection now, through the dispatcher.
    fn ask_line(&mut self, target: Option<SemId>, action: &str) -> bool {
        let Some(target) = target else {
            return false;
        };
        let seen = match self.snapshot() {
            Ok(seen) => seen,
            Err(errors) => {
                log::warn!("ide: the projection does not validate: {errors}");
                return false;
            }
        };
        match Request::observe(&seen, Actor::Human, &target, action, None) {
            Ok(req) => {
                let id = self.dispatch(req);
                log::debug!("ide: {target} {action}: {:?}", self.receipt(id));
                true
            }
            Err(e) => {
                log::warn!("ide: {e}");
                false
            }
        }
    }

    /// A pointer event on the line or on the column of an open surface,
    /// when it is theirs: the line's widgets, then the surface's. `None`
    /// when the block is off, an overlay of the workspace holds the
    /// screen, or the point is on neither and no panel of the line is
    /// open; a move reaches their widgets wherever it lands, so what it
    /// left stops being hovered.
    fn point_column(&mut self, events: &[WidgetEvent]) -> Option<EventResult> {
        let g = self.generated.as_ref()?;
        let first = events.first()?;
        let (x, y) = first.pos();
        if !self.workspace.overlay_mgr.is_empty() {
            return None;
        }
        let theme = self.workspace.theme().clone();
        let place = g.place(self.window.0, self.window.1, &theme);
        let moving = matches!(first, WidgetEvent::MouseMove { .. });
        let on = place.strip.contains(x, y) || place.column.is_some_and(|c| c.contains(x, y));
        if !on && !g.proposer.is_open() && !moving {
            return None;
        }
        let mut result = EventResult::IGNORED;
        let mut asked = Asked::default();
        for event in events {
            let g = self.generated.as_mut()?;
            let (r, a) = g.handle_row(event, &place.row, &theme);
            result = result.merge(r);
            asked.propose |= a.propose;
            asked.close |= a.close;
            asked.cancel |= a.cancel;
            asked.taken |= a.taken;
        }
        if asked.taken {
            return Some(result);
        }
        let ids = self.generated.as_ref().and_then(|g| g.ids.as_ref());
        let (propose, close) = (ids.map(|i| i.propose.clone()), ids.map(|i| i.close.clone()));
        let cancel = ids.map(|i| i.cancel.clone());
        if asked.propose && self.ask_line(propose, "propose") {
            result = result.merge(EventResult::changed());
        }
        if asked.cancel && self.ask_line(cancel, "activate") {
            result = result.merge(EventResult::changed());
        }
        if asked.close && self.ask_line(close, "close") {
            result = result.merge(EventResult::changed());
        }
        if place.surface.is_some_and(|s| s.contains(x, y)) || moving {
            let surface = self.with_host(|host, bench| {
                let mut app = App(bench);
                let mut r = EventResult::IGNORED;
                for event in events {
                    for s in host.iter_mut() {
                        r = r.merge(s.handle_event(event, &theme, &mut app));
                    }
                }
                r
            });
            result = result.merge(surface.unwrap_or(EventResult::IGNORED));
        }
        Some(result)
    }

    /// A left click on the line or the column, as a press and a release
    /// there (the workspace takes a click on the press); `None` when it is
    /// neither's. A click on the workspace blurs the intent field.
    pub(super) fn click_column(&mut self, x: f32, y: f32) -> Option<bool> {
        let events = [
            WidgetEvent::MouseDown { x, y },
            WidgetEvent::MouseUp { x, y },
        ];
        match self.point_column(&events) {
            Some(r) => Some(r.changed || r.clicked || r.handled),
            None => {
                if let Some(g) = self.generated.as_mut() {
                    g.intent.unfocus();
                }
                None
            }
        }
    }

    /// The pointer moved to `(x, y)`: the hover of the column's widgets.
    /// `true` when something shown changed.
    pub fn hover(&mut self, x: f32, y: f32) -> bool {
        self.point_column(&[WidgetEvent::MouseMove { x, y }])
            .is_some_and(|r| r.changed)
    }

    /// The wheel at `(x, y)`: the line's or the column's (the proposer's
    /// panel, a surface's table or code) or the workspace's panel under
    /// it. `true` when an offset moved.
    pub fn scroll(&mut self, x: f32, y: f32, delta: f32) -> bool {
        match self.point_column(&[WidgetEvent::Scroll { x, y, delta }]) {
            Some(r) => r.changed,
            None => self.workspace.scroll(x, delta),
        }
    }

    /// True while the intent field or a field of a surface has the
    /// keyboard: what is typed is theirs, never the keymap's.
    pub fn has_keyboard(&self) -> bool {
        self.generated.as_ref().is_some_and(|g| {
            g.intent.is_focused()
                || g.proposer.is_open()
                || g.host.iter().any(|s| s.focused().is_some())
        })
    }

    /// Text typed while the line or a field of a surface has the
    /// keyboard: into the intent field, or into the surface's field (its
    /// `set_value`, through the dispatcher). `false` when neither has it.
    pub fn type_text(&mut self, text: &str) -> bool {
        let Some(g) = self.generated.as_mut() else {
            return false;
        };
        if g.intent.insert(text) {
            return true;
        }
        self.with_host(|host, bench| {
            let mut app = App(bench);
            let mut typed = false;
            for c in text.chars() {
                let one = c.to_string();
                typed |= host.iter_mut().any(|s| s.handle_text(&one, &mut app));
            }
            typed
        })
        .unwrap_or(false)
    }

    /// An editing key (backspace, arrows, home, end) for the field that
    /// has the keyboard.
    pub fn edit_key(&mut self, key: EditKey) -> bool {
        let Some(g) = self.generated.as_mut() else {
            return false;
        };
        if g.intent.edit(key) {
            return true;
        }
        self.with_host(|host, bench| {
            let mut app = App(bench);
            host.iter_mut().any(|s| s.handle_edit_key(key, &mut app))
        })
        .unwrap_or(false)
    }

    /// A clipboard key (the person's cmd or ctrl with c, x, v or a) for
    /// the field that has the keyboard: the intent field takes all four; a
    /// field of a surface takes a paste as typing and select all as its
    /// editing key (its copy and cut are the `surface` crate's, not there
    /// yet). `false` when nothing happened.
    pub fn clip(&mut self, key: ClipKey, clipboard: &mut dyn ClipboardProvider) -> bool {
        let Some(g) = self.generated.as_mut() else {
            return false;
        };
        if g.intent.is_focused() {
            return g.intent.clip(key, clipboard);
        }
        match key {
            ClipKey::Paste => {
                let text = single_line(&clipboard.get_text().unwrap_or_default());
                !text.is_empty() && self.type_text(&text)
            }
            ClipKey::SelectAll => self.edit_key(EditKey::SelectAll),
            ClipKey::Copy | ClipKey::Cut => false,
        }
    }

    /// Enter: in the intent field the person proposes; in a field of a
    /// surface, what that field submits.
    pub fn enter(&mut self) -> bool {
        let Some(g) = self.generated.as_ref() else {
            return false;
        };
        if g.intent.is_focused() {
            let propose = g.ids.as_ref().map(|ids| ids.propose.clone());
            self.ask_line(propose, "propose");
            return true;
        }
        self.with_host(|host, bench| {
            let mut app = App(bench);
            host.iter_mut().any(|s| s.handle_enter(&mut app))
        })
        .unwrap_or(false)
    }

    /// Escape: the proposer's panel closes, or the field that has the
    /// keyboard lets it go. `false` when nothing of the column had it.
    pub fn escape(&mut self) -> bool {
        let Some(g) = self.generated.as_mut() else {
            return false;
        };
        if g.proposer.is_open() {
            g.proposer.close();
            return true;
        }
        if g.intent.is_focused() {
            g.intent.unfocus();
            return true;
        }
        g.host.iter_mut().any(|s| s.handle_escape())
    }

    /// Draws the line and the column of an open surface beside the frame
    /// the workspace drew.
    pub(super) fn render_column(&mut self, compositor: &mut Compositor) {
        let theme = self.workspace.theme().clone();
        let (w, h) = self.window;
        if let Some(g) = self.generated.as_mut() {
            let place = g.place(w, h, &theme);
            g.render_place(compositor, &place, &theme);
        }
    }
}
