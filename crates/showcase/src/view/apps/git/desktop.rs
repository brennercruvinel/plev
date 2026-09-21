//! The desktop git section: the ide workspace over a real `GitClient`.

use comps::prelude::{EventResult, Rect, WidgetEvent};
use engine::compositor::Compositor;
use engine::theme::Theme;

use super::super::text;
use super::{NavKey, Wake};
use std::sync::mpsc::{Receiver, Sender, channel};

use git::{GitClient, GitEvent};
use ide::session::GitSession;
use ide::views::workspace::{Side, ThemeMode, WorkspaceView};

pub struct GitPage {
    workspace: WorkspaceView,
    session: Option<GitSession>,
    /// Worker events parked until the next tick (the worker runs on its
    /// own thread; the view is single-threaded).
    events: Receiver<GitEvent>,
    sender: Sender<GitEvent>,
    /// Why there is no session (no repository at the cwd).
    error: Option<String>,
}

impl GitPage {
    pub fn new() -> Self {
        let (sender, events) = channel();
        Self {
            workspace: WorkspaceView::new(1.0, 1.0),
            session: None,
            events,
            sender,
            error: None,
        }
    }

    /// Open the repository containing the cwd and start loading it. The
    /// host passes the callback that wakes its event loop.
    pub fn connect(&mut self, wake: Wake) {
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
            Ok(client) => {
                let session = GitSession::new(client);
                self.workspace.repo_label = session.repo_label();
                session.refresh();
                self.session = Some(session);
            }
            Err(e) => {
                self.error = Some(format!("no git repository at {}: {e}", cwd.display()));
            }
        }
    }

    pub fn set_theme(&mut self, name: &str) {
        self.workspace.theme_mode = if name == "light" {
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
        self.workspace
            .set_frame(content.x, content.y, content.w, content.h);
    }

    /// Hand the views' queued git operations to the worker.
    fn flush(&mut self) {
        if let Some(session) = &mut self.session {
            session.flush_requests(&mut self.workspace);
        }
    }

    pub fn handle_event(&mut self, event: &WidgetEvent, content: Rect) -> EventResult {
        self.frame(content);
        let ws = &mut self.workspace;
        let changed = match *event {
            WidgetEvent::MouseDown { x, y } if content.contains(x, y) => {
                match ws.hit_test_handle(x) {
                    Some(Side::Left) => {
                        ws.begin_drag_left(x);
                        true
                    }
                    Some(Side::Right) => {
                        ws.begin_drag_right(x);
                        true
                    }
                    None => ws.handle_click(x, y),
                }
            }
            WidgetEvent::MouseUp { .. } if ws.dragging_left || ws.dragging_right => {
                ws.end_drag();
                true
            }
            WidgetEvent::MouseMove { x, y } => {
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
                ws.scroll(x, delta);
                // Consumed either way: the page must not scroll under it.
                self.flush();
                return EventResult::changed();
            }
            _ => false,
        };
        self.flush();
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
        let opened = self.workspace.handle_right_click(x, y);
        self.flush();
        opened
    }

    /// Plain characters: the visible commit form takes them as text;
    /// otherwise `s` stages, `d` discards, `c` opens the commit form.
    pub fn handle_text(&mut self, key: &str) -> bool {
        let ws = &mut self.workspace;
        let changed = if ws.commit_form.visible {
            for ch in key.chars() {
                ws.commit_form.append_char(ch);
            }
            true
        } else {
            match key {
                "s" => ws.stage_selected(),
                "d" => ws.discard_selected(),
                "c" => ws.toggle_commit_form(),
                _ => false,
            }
        };
        self.flush();
        changed
    }

    pub fn handle_nav(&mut self, key: NavKey) -> bool {
        let changed = match key {
            NavKey::Up => self.workspace.nav_up(),
            NavKey::Down => self.workspace.nav_down(),
        };
        self.flush();
        changed
    }

    pub fn handle_backspace(&mut self) -> bool {
        if !self.workspace.commit_form.visible {
            return false;
        }
        self.workspace.commit_form.backspace();
        true
    }

    /// Enter submits the commit form when it is open, otherwise refreshes
    /// the diff of the selected file.
    pub fn handle_enter(&mut self) -> bool {
        let ws = &mut self.workspace;
        let changed = if ws.commit_form.visible {
            ws.submit_commit()
        } else {
            ws.show_selected_diff()
        };
        self.flush();
        changed
    }

    /// Escape: the overlay first, then the commit form. `false` when
    /// there was nothing to close.
    pub fn handle_escape(&mut self) -> bool {
        if self.workspace.close_top_overlay() {
            return true;
        }
        if self.workspace.commit_form.visible {
            self.workspace.commit_form.hide();
            return true;
        }
        false
    }

    /// Apply parked worker events and advance the widgets that animate.
    /// `true` while a frame is needed.
    pub fn tick(&mut self, dt: f32) -> bool {
        let mut changed = false;
        while let Ok(event) = self.events.try_recv() {
            if let Some(session) = &mut self.session {
                session.apply(&mut self.workspace, event);
                changed = true;
            }
        }
        self.flush();
        changed | self.workspace.tick(dt)
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
        self.workspace.render(c);
    }
}
