//! the view's handlers. each one moves only the view's own state (the
//! instant, the pointer, the pick, a scroll) and says whether a frame is
//! due. none of them has anything to send: the view holds the session and
//! nothing that could act on it.

use engine::widgets::{Rect, WidgetEvent};
use accord::Millis;

use crate::view::{InspectorView, Key, Press};

impl InspectorView {
    /// the pointer moved to `(x, y)`, in logical window pixels.
    pub fn pointer_move(&mut self, x: f32, y: f32) -> bool {
        self.pointer = Some((x, y));
        let event = WidgetEvent::MouseMove { x, y };
        let (shell, _) = self.shell.handle_event(&event, &self.theme);
        let mut changed = shell.changed;
        let g = self.strip.geometry(self.layout().strip, &self.theme);
        let scrub = self.strip.scrub.handle_event(&event, g.scrub, &self.theme);
        changed |= scrub.changed;
        match self.press {
            Some(Press::Scrub) => changed |= self.seek_at(self.strip.at()),
            Some(Press::Track) => changed |= self.seek_x(x, g.track),
            _ => {}
        }
        changed | self.refresh_hover()
    }

    /// a press at `(x, y)`.
    pub fn pointer_down(&mut self, x: f32, y: f32) -> bool {
        self.pointer = Some((x, y));
        let event = WidgetEvent::MouseDown { x, y };
        let (shell, nav) = self.shell.handle_event(&event, &self.theme);
        if let Some(index) = nav {
            return self.go_to_request(index) | shell.changed;
        }
        if shell.handled {
            return shell.changed;
        }
        let layout = self.layout();
        let g = self.strip.geometry(layout.strip, &self.theme);
        if self
            .strip
            .scrub
            .handle_event(&event, g.scrub, &self.theme)
            .handled
        {
            self.press = Some(Press::Scrub);
            self.seek_at(self.strip.at());
            return true;
        }
        if g.track.contains(x, y) {
            self.press = Some(Press::Track);
            self.seek_x(x, g.track);
            return true;
        }
        if let Some(fit) = self.fit()
            && let Some((fx, fy)) = fit.to_frame(x, y)
        {
            self.press = Some(Press::Frame(self.session.hit_at(self.t, fx, fy)));
        }
        false
    }

    /// a press let go at `(x, y)`: a press and a release over the same
    /// control of the frame pick it; over none, they drop the pick.
    pub fn pointer_up(&mut self, x: f32, y: f32) -> bool {
        self.pointer = Some((x, y));
        let event = WidgetEvent::MouseUp { x, y };
        let (shell, nav) = self.shell.handle_event(&event, &self.theme);
        if let Some(index) = nav {
            return self.go_to_request(index) | shell.changed;
        }
        let g = self.strip.geometry(self.layout().strip, &self.theme);
        let scrub = self.strip.scrub.handle_event(&event, g.scrub, &self.theme);
        let press = self.press.take();
        let mut changed = shell.changed | scrub.changed;
        if let Some(Press::Frame(down)) = press {
            let up = self.fit().and_then(|fit| {
                let (fx, fy) = fit.to_frame(x, y)?;
                Some(self.session.hit_at(self.t, fx, fy))
            });
            if up == Some(down.clone()) {
                changed |= self.pick(down);
            }
        }
        changed
    }

    /// a wheel or trackpad scroll at `(x, y)`: the panel's rows, or the
    /// list of requests.
    pub fn scroll(&mut self, x: f32, y: f32, delta: f32) -> bool {
        let layout = self.layout();
        if layout.panel.contains(x, y) && !self.block.rows.is_empty() {
            let max = self.block.max_scroll(layout.panel, &self.theme);
            let to = (self.block.scroll + delta).clamp(0.0, max);
            let moved = to != self.block.scroll;
            self.block.scroll = to;
            return moved;
        }
        let event = WidgetEvent::Scroll { x, y, delta };
        self.shell.handle_event(&event, &self.theme).0.changed
    }

    pub fn key(&mut self, key: Key) -> bool {
        let at = self.at();
        match key {
            Key::Previous => self.seek_at(self.strip.step(at, false)),
            Key::Next => self.seek_at(self.strip.step(at, true)),
            Key::First => self.seek_at(0),
            Key::Last => self.seek_at(self.strip.span),
            Key::Close => {
                let drawer = std::mem::take(&mut self.shell.sidebar.drawer_open);
                self.pick(None) | drawer
            }
        }
    }

    /// to `at` from the start of the session.
    pub fn seek_at(&mut self, at: Millis) -> bool {
        self.seek(self.start.saturating_add(at))
    }

    /// to the time under `x` on the track.
    fn seek_x(&mut self, x: f32, track: Rect) -> bool {
        if track.w <= 0.0 {
            return false;
        }
        let ratio = ((x - track.x) / track.w).clamp(0.0, 1.0);
        self.seek_at((ratio * self.strip.span as f32).round() as Millis)
    }
}
