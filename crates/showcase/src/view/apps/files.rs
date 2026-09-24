//! Apps Demo, Files: the file dialog and the file drop as capabilities
//! of the contract (task-76, item 3). A person opens a file through the
//! platform's dialog (`engine::platform::files`) or drops files on the
//! window (`engine::input::FileDrop`, the engine's one event for the
//! platform's three); both reach the page as requests through the
//! gallery's one dispatcher (the todo page lends it, as to the Silences
//! page), and the files received are the page's projected state. An
//! agent through the port reads the list and may drop files; `open_file`
//! is refused to it: a native dialog is opened only by a person
//! (`semantics.rs`). The dialog behind the button is the platform's in
//! the window and the fake in the crate's tests; nothing here opens one
//! on its own.
//!
//! Wiring contract for the chrome (view/dispatch.rs, view/input.rs):
//! every entry takes the todo page (the dispatcher's owner) and the
//! inert flag; the shell hands the engine's drop event to the view's
//! `handle_file_drop`, which reaches this page; Tab reaches the button
//! through `handle_edit_key`, Enter presses it through `handle_enter`,
//! Escape lets it go through `handle_escape`.

mod semantics;
#[cfg(test)]
mod tests;

use std::path::PathBuf;

use comps::prelude::{Button, EventResult, Rect, WidgetEvent, rounded_rect, rounded_rect_stroke};
use engine::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::input::FileDrop;
use engine::platform::files::FileDialogs;
use engine::text::{TextMeasurer, TextStyle};
use engine::theme::{ControlSize, Intent, Theme};
use sem::Value;

use super::app::TodoPage;
use super::{EditKey, panel};
use semantics::{Ids, Target, bounds};

/// What the status line says: what the last request came to.
enum Said {
    Nothing,
    Received(usize),
    Cancelled,
    NoDialog,
}

pub struct FilesPage {
    /// Every file received, in the order it came: the page's state.
    received: Vec<PathBuf>,
    open: Button,
    /// Files are held over the window: the zone lights up.
    hovering: bool,
    /// Keyboard focus, on the button.
    focused: bool,
    /// The ids the page projects and binds; `None` only if the constant
    /// ids failed to parse (a test proves they do not), which leaves the
    /// page without semantics: every request is then refused.
    ids: Option<Ids>,
    /// Set by the view while an overlay holds the pointer: the page
    /// projects itself inert and takes no input.
    inert: bool,
    /// The platform's dialogs behind `open_file`: native in the window,
    /// the fake in the crate's tests.
    dialogs: Box<dyn FileDialogs>,
    said: Said,
}

/// One frame of geometry: the card, the drop zone, the button with the
/// status beside it, one row per file received.
struct Layout {
    panel: Rect,
    zone: Rect,
    open: Rect,
    status: Rect,
    rows: Vec<Rect>,
}

impl FilesPage {
    /// The page over the platform's dialogs; in the crate's tests over
    /// the fake, so no test ever opens a native dialog by mistake.
    pub fn new() -> Self {
        #[cfg(not(test))]
        let dialogs = engine::platform::files::default_dialogs();
        #[cfg(test)]
        let dialogs: Box<dyn FileDialogs> = Box::new(engine::platform::files::FakeDialogs::new());
        Self::with(dialogs)
    }

    /// The page over other dialogs (the fake of a test).
    pub(crate) fn with(dialogs: Box<dyn FileDialogs>) -> Self {
        let ids = Ids::new()
            .inspect_err(|e| log::error!("files: the page ids do not parse: {e}"))
            .ok();
        let mut open = Button::new("open file").icon("folder-open");
        if let Some(ids) = &ids {
            open = open.sem(ids.open.clone());
        }
        Self {
            received: Vec::new(),
            open,
            hovering: false,
            focused: false,
            ids,
            inert: false,
            dialogs,
            said: Said::Nothing,
        }
    }

    /// Other dialogs for a page already built (a test on the gallery).
    #[cfg(test)]
    pub(crate) fn set_dialogs(&mut self, dialogs: Box<dyn FileDialogs>) {
        self.dialogs = dialogs;
    }

    /// The view says whether an overlay holds the pointer. While it does,
    /// the page is projected inert, so any request comes back `Inert`,
    /// and keys fall through to the chrome.
    pub fn set_inert(&mut self, inert: bool) {
        self.inert = inert;
    }

    fn status_text(&self) -> String {
        match self.said {
            Said::Nothing => "no files yet: drop some on the window, or open one".to_owned(),
            Said::Received(now) => format!("{now} received now, {} in all", self.received.len()),
            Said::Cancelled => "the dialog was cancelled".to_owned(),
            Said::NoDialog => "no file dialog on this target".to_owned(),
        }
    }

    fn hint_style(theme: &Theme) -> TextStyle {
        theme.typography.base_m()
    }

    fn status_style(theme: &Theme) -> TextStyle {
        theme.typography.caption_r()
    }

    fn path_style(theme: &Theme) -> TextStyle {
        theme.typography.mono()
    }

    /// The card fills the readable width, centered; the zone is as tall
    /// as two of the tallest controls with a gap between; the button and
    /// the status share a row; the files received stack under them.
    fn layout(&self, content: Rect, theme: &Theme) -> Layout {
        let pad = theme.spacing.lg;
        let w = content.w.min(theme.size.readable_max_w);
        let x = (content.x + (content.w - w) / 2.0).floor();
        let inner_w = (w - pad * 2.0).max(0.0);
        let mut y = content.y + pad;
        let zone_h = theme.control.height(ControlSize::Xl) * 2.0 + theme.spacing.xl;
        let zone = Rect::new(x + pad, y, inner_w, zone_h);
        y += zone_h + theme.spacing.md;
        let (bw, bh) = self.open.preferred_size(theme);
        let open = Rect::new(x + pad, y, bw, bh);
        let status_h = Self::status_style(theme).line_height;
        let status = Rect::new(
            open.right() + theme.spacing.md,
            y + ((bh - status_h) / 2.0).floor(),
            (inner_w - bw - theme.spacing.md).max(0.0),
            status_h,
        );
        y += bh;
        let row_h = Self::path_style(theme).line_height + theme.spacing.xs;
        let rows: Vec<Rect> = if self.received.is_empty() {
            Vec::new()
        } else {
            y += theme.spacing.lg;
            (0..self.received.len())
                .map(|i| Rect::new(x + pad, y + i as f32 * row_h, inner_w, row_h))
                .collect()
        };
        y += rows.len() as f32 * row_h;
        let panel = Rect::new(x, content.y, w, y - content.y + pad);
        Layout {
            panel,
            zone,
            open,
            status,
            rows,
        }
    }

    /// Natural height: the card as it stands, growing with the files.
    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        self.layout(content, theme).panel.h
    }

    /// The drop zone's rect for `content` (the tests aim drops at it).
    #[cfg(test)]
    pub(crate) fn zone_rect(&self, content: Rect, theme: &Theme) -> Rect {
        self.layout(content, theme).zone
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
        todo: &mut TodoPage,
        inert: bool,
    ) -> EventResult {
        self.set_inert(inert);
        let l = self.layout(content, theme);
        let mut r = EventResult::IGNORED;
        // a click anywhere lets the keyboard focus go, as in a field
        if let WidgetEvent::MouseDown { .. } = event
            && self.focused
        {
            self.set_focus(false);
            r.changed = true;
        }
        let button = self.open.handle_event(event, l.open);
        r = r.merge(button);
        if button.clicked {
            r = r.merge(self.ask(todo, Target::Open, "open_file", None));
        }
        r
    }

    /// Files dragged over the window, as the engine says it. While they
    /// hover the zone lights up; a drop released over the zone, or with
    /// no position at all (winit 0.30 gives none), becomes `drop_files`
    /// by the person, with the paths as its param; released elsewhere on
    /// the page it is not received.
    pub fn handle_file_drop(
        &mut self,
        drop: &FileDrop,
        content: Rect,
        theme: &Theme,
        todo: &mut TodoPage,
        inert: bool,
    ) -> EventResult {
        self.set_inert(inert);
        let was = self.hovering;
        match drop {
            FileDrop::Hovered { .. } => {
                self.hovering = true;
                EventResult {
                    handled: true,
                    changed: !was,
                    ..EventResult::IGNORED
                }
            }
            FileDrop::Cancelled => {
                self.hovering = false;
                EventResult {
                    handled: true,
                    changed: was,
                    ..EventResult::IGNORED
                }
            }
            FileDrop::Dropped { paths, position } => {
                self.hovering = false;
                let zone = self.layout(content, theme).zone;
                if let Some((x, y)) = *position
                    && !zone.contains(x, y)
                {
                    return EventResult {
                        handled: true,
                        changed: was,
                        ..EventResult::IGNORED
                    };
                }
                let paths: Vec<String> = paths.iter().map(|p| p.display().to_string()).collect();
                let asked = self.ask(todo, Target::Drop, "drop_files", Some(Value::List(paths)));
                EventResult {
                    handled: true,
                    changed: was || asked.changed,
                    ..asked
                }
            }
        }
    }

    /// Tab moves the keyboard focus onto the button and off it again.
    pub fn handle_edit_key(&mut self, key: EditKey) -> bool {
        if self.inert || key != EditKey::Tab {
            return false;
        }
        let on = !self.focused;
        self.set_focus(on);
        true
    }

    /// Enter presses the focused button: `open_file` by the person.
    pub fn handle_enter(&mut self, todo: &mut TodoPage, inert: bool) -> bool {
        self.set_inert(inert);
        if inert || !self.focused {
            return false;
        }
        self.ask(todo, Target::Open, "open_file", None);
        true
    }

    /// Escape lets the keyboard focus go; `false` when there was none.
    pub fn handle_escape(&mut self) -> bool {
        if !self.focused {
            return false;
        }
        self.set_focus(false);
        true
    }

    fn set_focus(&mut self, on: bool) {
        self.focused = on;
        self.open.set_focused(on);
    }

    /// The card, the zone (its edge the accent while files hover), the
    /// button, the status and the files received. What the page draws
    /// for a capability it binds, beside the scene: the card as `files`,
    /// the zone as `files.drop`, the status; the button binds itself.
    pub fn render(&mut self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let l = self.layout(content, theme);
        panel(c, l.panel, theme);
        if let Some(ids) = &self.ids {
            c.bind(&ids.region, bounds(l.panel));
            c.bind(&ids.drop, bounds(l.zone));
            c.bind(&ids.status, bounds(l.status));
        }
        c.push(rounded_rect(
            l.zone.x,
            l.zone.y,
            l.zone.w,
            l.zone.h,
            theme.radius.md,
            theme.glass.field.0,
        ));
        let (edge, width, ink) = if self.hovering {
            let accent = theme.colors.accent.0;
            (accent, theme.control.edge_width_strong, accent)
        } else {
            (
                theme.glass.edge.0,
                theme.control.edge_width,
                theme.glass.text_faint.0,
            )
        };
        c.push(rounded_rect_stroke(
            l.zone.x,
            l.zone.y,
            l.zone.w,
            l.zone.h,
            theme.radius.md,
            edge,
            width,
        ));
        let hint = if self.hovering {
            "release to receive"
        } else {
            "drop files here"
        };
        let style = Self::hint_style(theme);
        let (tw, th) = TextMeasurer::measure_styled(hint, &style, None);
        c.push(SceneNode::Text {
            key: TextNodeKey::from_style(hint, &style, None),
            x: l.zone.x + ((l.zone.w - tw) / 2.0).floor(),
            y: l.zone.y + ((l.zone.h - th) / 2.0).floor(),
            color: ink,
        });
        self.open.render(c, l.open, theme);
        let status = self.status_text();
        let style = Self::status_style(theme);
        let color = match self.said {
            Said::NoDialog => theme.intent_color(Intent::Destructive).0,
            _ => theme.colors.text_mid.0,
        };
        c.push(SceneNode::Text {
            key: TextNodeKey::from_style(&status, &style, None),
            x: l.status.x,
            y: l.status.y,
            color,
        });
        let style = Self::path_style(theme);
        for (path, row) in self.received.iter().zip(&l.rows) {
            let text = path.display().to_string();
            c.push(SceneNode::Text {
                key: TextNodeKey::from_style(&text, &style, None),
                x: row.x,
                y: row.y,
                color: theme.colors.text.0,
            });
        }
    }
}

impl Default for FilesPage {
    fn default() -> Self {
        Self::new()
    }
}
