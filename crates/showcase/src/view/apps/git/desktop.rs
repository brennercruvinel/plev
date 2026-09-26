//! The desktop git section: the ide workbench over a real `GitClient`.
//! Every click and key that acts on the repository or the commit form goes
//! through the workbench's dispatcher, the same path as in the ide.

use comps::prelude::{EventResult, Rect, Wake, WidgetEvent};
use engine::compositor::Compositor;
use engine::theme::Theme;

use super::super::text;
use super::{GitWake, NavKey};
use crate::view::clipboard::Keys;
use std::sync::mpsc::{Receiver, Sender, channel};

use git::{GitClient, GitEvent};
use ide::views::workspace::{Side, ThemeMode};
use ide::workbench::Workbench;

pub struct GitPage {
    bench: Workbench,
    /// Worker events parked until the host applies them (the worker runs
    /// on its own thread; the view is single-threaded; every event is
    /// followed by a call to the host's wake).
    events: Receiver<GitEvent>,
    sender: Sender<GitEvent>,
    /// Why there is no repository (none at the cwd).
    error: Option<String>,
}

impl GitPage {
    pub fn new() -> Self {
        let (sender, events) = channel();
        Self {
            bench: Workbench::new(1.0, 1.0),
            events,
            sender,
            error: None,
        }
    }

    /// Open the repository containing the cwd and start loading it. The
    /// host passes the callback that wakes its event loop.
    pub fn connect(&mut self, wake: GitWake) {
        let cwd = match std::env::current_dir() {
            Ok(p) => p,
            Err(e) => {
                self.error = Some(format!("cwd is not accessible: {e}"));
                return;
            }
        };
        let sender = self.sender.clone();
        match GitClient::spawn(&cwd, move |event| {
            // Park the event, then wake the loop; send_event only fails
            // after loop shutdown.
            let _ = sender.send(event);
            wake();
        }) {
            Ok(client) => self.bench.connect(client),
            Err(e) => {
                self.error = Some(format!("no git repository at {}: {e}", cwd.display()));
            }
        }
    }

    pub fn set_theme(&mut self, name: &str) {
        self.bench.workspace_mut().theme_mode = if name == "light" {
            ThemeMode::Light
        } else {
            ThemeMode::Dark
        };
    }

    /// The three panels fill the viewport and scroll internally.
    pub fn content_height(&self, content: Rect) -> f32 {
        content.h
    }

    fn frame(&mut self, content: Rect) {
        self.bench
            .workspace_mut()
            .set_frame(content.x, content.y, content.w, content.h);
    }

    pub fn handle_event(&mut self, event: &WidgetEvent, content: Rect) -> EventResult {
        self.frame(content);
        let changed = match *event {
            WidgetEvent::MouseDown { x, y } if content.contains(x, y) => {
                let ws = self.bench.workspace_mut();
                match ws.hit_test_handle(x) {
                    Some(Side::Left) => {
                        ws.begin_drag_left(x);
                        true
                    }
                    Some(Side::Right) => {
                        ws.begin_drag_right(x);
                        true
                    }
                    None => self.bench.click(x, y),
                }
            }
            WidgetEvent::MouseUp { .. }
                if self.bench.workspace().dragging_left
                    || self.bench.workspace().dragging_right =>
            {
                self.bench.workspace_mut().end_drag();
                true
            }
            WidgetEvent::MouseMove { x, y } => {
                let ws = self.bench.workspace_mut();
                let mut changed = false;
                if ws.dragging_left || ws.dragging_right {
                    ws.update_drag(x);
                    changed = true;
                }
                let inside = content.contains(x, y);
                let (lh, rh) = (ws.hover_left_handle, ws.hover_right_handle);
                let handle = if inside { ws.hit_test_handle(x) } else { None };
                ws.hover_left_handle = handle == Some(Side::Left);
                ws.hover_right_handle = handle == Some(Side::Right);
                changed |= (lh, rh) != (ws.hover_left_handle, ws.hover_right_handle);
                changed |= if !ws.overlay_mgr.is_empty() {
                    ws.handle_overlay_hover(x, y)
                } else {
                    ws.handle_hover(x, y)
                };
                changed
            }
            WidgetEvent::Scroll { x, y, delta } if content.contains(x, y) => {
                self.bench.workspace_mut().scroll(x, delta);
                // Consumed either way: the page must not scroll under it.
                return EventResult::changed();
            }
            _ => false,
        };
        if changed {
            EventResult::changed()
        } else {
            EventResult::IGNORED
        }
    }

    pub fn handle_right_click(&mut self, x: f32, y: f32, content: Rect) -> bool {
        if !content.contains(x, y) {
            return false;
        }
        self.frame(content);
        self.bench.right_click(x, y)
    }

    /// The visible commit form's field has the keys (a chord, the
    /// field's menu).
    pub(in crate::view) fn keys(&mut self) -> Keys<'_> {
        match self.bench.workspace_mut().commit_form.typing_field() {
            Some(field) => Keys::Field(field),
            None => Keys::Nobody,
        }
    }

    /// Plain characters: the visible commit form takes them as text;
    /// otherwise `s` stages, `d` discards, `c` opens the commit form.
    pub fn handle_text(&mut self, key: &str) -> bool {
        let form = &mut self.bench.workspace_mut().commit_form;
        if form.visible {
            for ch in key.chars() {
                form.append_char(ch);
            }
            return true;
        }
        match key {
            "s" => self.bench.stage_selected(),
            "d" => self.bench.discard_selected(),
            "c" => self.bench.toggle_commit_form(),
            _ => false,
        }
    }

    pub fn handle_nav(&mut self, key: NavKey) -> bool {
        match key {
            NavKey::Up => self.bench.nav_up(),
            NavKey::Down => self.bench.nav_down(),
        }
    }

    pub fn handle_backspace(&mut self) -> bool {
        let form = &mut self.bench.workspace_mut().commit_form;
        if !form.visible {
            return false;
        }
        form.backspace();
        true
    }

    /// Enter submits the commit form when it is open, otherwise refreshes
    /// the diff of the selected file.
    pub fn handle_enter(&mut self) -> bool {
        if self.bench.workspace().commit_form.visible {
            self.bench.submit_commit()
        } else {
            self.bench.show_selected_diff()
        }
    }

    /// Escape: the overlay first, then the commit form. `false` when
    /// there was nothing to close.
    pub fn handle_escape(&mut self) -> bool {
        if self.bench.close_top_overlay() {
            return true;
        }
        if self.bench.workspace().commit_form.visible {
            return self.bench.cancel_commit_form();
        }
        false
    }

    /// Apply the parked worker events. `true` when there were any: the
    /// workspace changed, whether or not the page is on screen.
    pub fn apply_events(&mut self) -> bool {
        let mut changed = false;
        while let Ok(event) = self.events.try_recv() {
            self.bench.apply(event);
            changed = true;
        }
        changed
    }

    /// Advance the widgets that animate: the workbench's ask (motion
    /// every turn, a caret flip or a toast leaving as a deadline). Worker
    /// events are not applied here but by [`GitPage::apply_events`], when
    /// the worker wakes the host, after the host caught up to that
    /// instant: a toast an event pushes then counts its time from the
    /// event.
    pub fn wake(&mut self, dt: f32) -> Wake {
        self.bench.wake(dt)
    }

    pub fn render(&mut self, c: &mut Compositor, content: Rect, theme: &Theme) {
        if let Some(error) = &self.error {
            text(
                c,
                error,
                14.0,
                400,
                content.x,
                content.y + theme.spacing.md,
                theme.glass.text_placeholder.0,
            );
            return;
        }
        self.frame(content);
        self.bench.render(c);
    }
}

#[cfg(test)]
mod tests;
