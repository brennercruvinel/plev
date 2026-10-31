//! Apps Demo, Files: the file dialog and the file drop as capabilities
//! of the contract (task-76, item 3). A person opens a file through the
//! platform's dialog (`engine::runtime::platform::files`) or drops files on the
//! window (`engine::control::input::FileDrop`, the engine's one event for the
//! platform's three); both reach the page as requests through the
//! gallery's one dispatcher (the todo page lends it, as to the Silences
//! page), and the files received are the page's projected state. An
//! agent through the port reads the list and may drop files; `open_file`
//! is refused to it: a native dialog is opened only by a person
//! (`semantics.rs`). The dialog behind the button is the platform's in
//! the window and the fake in the crate's tests; nothing here opens one
//! on its own.
//!
//! The native alert is a capability the same way (task-76, item 8): the
//! "warn" button asks `engine::runtime::platform::alerts` for the platform's
//! warning panel through the dispatcher, a person's alone, and the text
//! the person saw is the page's projected state (`files:alert`), so an
//! agent reads what was shown and never raises a panel itself.
//!
//! The system notification is the app's own doing (task-76, item 5):
//! files that land while no window of the app has the focus (the shell
//! says so, `set_window_focused`) are told to the person by a notification
//! the page asks for (`take_notifications`; the shell shows it through
//! `engine::runtime::platform::notify`), with the open button as its one button,
//! and the text is the page's projected state (`files:notice`) whether the
//! platform showed it or not, so an agent reads what the person was told.
//!
//! Wiring contract for the chrome (view/dispatch.rs, view/input.rs):
//! every entry takes the todo page (the dispatcher's owner) and the
//! inert flag; the shell hands the engine's drop event to the view's
//! `handle_file_drop`, which reaches this page; Tab walks the two
//! buttons through `handle_edit_key`, Enter presses the focused one
//! through `handle_enter`, Escape lets it go through `handle_escape`.

mod semantics;
#[cfg(test)]
mod tests;

use std::path::PathBuf;

use accord::Value;
use engine::control::input::FileDrop;
use engine::drawing::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::drawing::text::{TextMeasurer, TextStyle};
use engine::runtime::platform::alerts::Alerts;
use engine::runtime::platform::files::FileDialogs;
use engine::runtime::platform::notify::Notification;
use engine::styling::theme::{ControlSize, Intent, Theme};
use engine::widgets::content::Panel;
use engine::widgets::message::attachment::{AttachmentKind, AttachmentTray};
use engine::widgets::prelude::{
    Button, EventResult, Rect, WidgetEvent, rounded_rect, rounded_rect_stroke,
};

use super::EditKey;
use super::todo::TodoPage;
use semantics::{Ids, Target, bounds};

/// What the status line says: what the last request came to.
enum Said {
    Nothing,
    Received(usize),
    Cancelled,
    NoDialog,
    Warned,
    NoAlert,
}

pub struct FilesPage {
    /// Every file received, in the order it came: the page's state.
    received: Vec<PathBuf>,
    open: Button,
    warn: Button,
    /// Files are held over the window: the zone lights up.
    hovering: bool,
    /// Keyboard focus, on one of the two buttons.
    focused: Option<Target>,
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
    /// The platform's panels behind `warn`: native in the window, the
    /// fake in the crate's tests.
    alerts: Box<dyn Alerts>,
    /// The body of the last alert the person saw, projected as
    /// `files:alert`; empty until one was shown.
    alerted: String,
    /// Whether a window of the app has the focus, as the shell says. A
    /// drop that lands while none has is told by notification: nobody
    /// was looking. A page on its own is looked at.
    window_focused: bool,
    /// The notifications asked for and not yet taken by the shell.
    notices: Vec<Notification>,
    /// The body of the last notification asked for, projected as
    /// `files:notice`; empty until one was.
    notice: String,
    said: Said,
}

/// One frame of geometry: the card, the drop zone, the two buttons with
/// the status beside them, one row per file received.
struct Layout {
    panel: Rect,
    zone: Rect,
    open: Rect,
    warn: Rect,
    /// The status: beside the buttons where it fits one line there,
    /// else under them across the card, wrapped.
    status: Rect,
    /// The files received, the kit's attachment tray; empty for none.
    tray: Rect,
}

impl FilesPage {
    /// The page over the platform's dialogs and panels; in the crate's
    /// tests over the fakes, so no test ever opens a native one by
    /// mistake.
    pub fn new() -> Self {
        #[cfg(not(test))]
        let (dialogs, alerts) = (
            engine::runtime::platform::files::default_dialogs(),
            engine::runtime::platform::alerts::default_alerts(),
        );
        #[cfg(test)]
        let (dialogs, alerts): (Box<dyn FileDialogs>, Box<dyn Alerts>) = (
            Box::new(engine::runtime::platform::files::FakeDialogs::new()),
            Box::new(engine::runtime::platform::alerts::FakeAlerts::new()),
        );
        Self::with(dialogs, alerts)
    }

    /// The page over other dialogs and panels (the fakes of a test).
    pub(crate) fn with(dialogs: Box<dyn FileDialogs>, alerts: Box<dyn Alerts>) -> Self {
        let ids = Ids::new()
            .inspect_err(|e| log::error!("files: the page ids do not parse: {e}"))
            .ok();
        let mut open = Button::new("open file").icon("folder-open");
        let mut warn = Button::new("warn").icon("alert-triangle");
        if let Some(ids) = &ids {
            open = open.sem(ids.open.clone());
            warn = warn.sem(ids.warn.clone());
        }
        Self {
            received: Vec::new(),
            open,
            warn,
            hovering: false,
            focused: None,
            ids,
            inert: false,
            dialogs,
            alerts,
            alerted: String::new(),
            window_focused: true,
            notices: Vec::new(),
            notice: String::new(),
            said: Said::Nothing,
        }
    }

    /// The shell says whether one of its windows has the focus. While
    /// none has, files that land are told to the person by notification.
    pub fn set_window_focused(&mut self, on: bool) {
        self.window_focused = on;
    }

    /// The notifications asked for since the shell last took them, in
    /// order; the shell shows them through the platform.
    pub fn take_notifications(&mut self) -> Vec<Notification> {
        std::mem::take(&mut self.notices)
    }

    /// Other dialogs for a page already built (a test on the gallery).
    #[cfg(test)]
    pub(crate) fn set_dialogs(&mut self, dialogs: Box<dyn FileDialogs>) {
        self.dialogs = dialogs;
    }

    /// Other panels for a page already built (a test on the gallery).
    #[cfg(test)]
    pub(crate) fn set_alerts(&mut self, alerts: Box<dyn Alerts>) {
        self.alerts = alerts;
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
            Said::Warned => "the warning was shown".to_owned(),
            Said::NoAlert => "no native alert on this target".to_owned(),
        }
    }

    fn hint_style(theme: &Theme) -> TextStyle {
        theme.typography.base_m()
    }

    fn status_style(theme: &Theme) -> TextStyle {
        theme.typography.caption_r()
    }

    /// The files received as the kit's attachment tiles: each its name,
    /// its folder under it.
    fn tray(&self) -> AttachmentTray {
        let mut tray = AttachmentTray::new();
        for path in &self.received {
            let name = path.file_name().map_or_else(
                || path.display().to_string(),
                |n| n.to_string_lossy().into_owned(),
            );
            let folder = path
                .parent()
                .map(|p| p.display().to_string())
                .unwrap_or_default();
            tray.push(name, folder, AttachmentKind::File);
        }
        tray
    }

    /// The card fills the readable width, centered; the zone is as tall
    /// as two of the tallest controls with a gap between; the two buttons
    /// and the status share a row; the files received stack under them.
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
        let (ww, wh) = self.warn.preferred_size(theme);
        let warn = Rect::new(open.right() + theme.spacing.sm, y, ww, wh.max(bh));
        let style = Self::status_style(theme);
        let text = self.status_text();
        let beside_x = warn.right() + theme.spacing.md;
        let beside_w = (x + pad + inner_w - beside_x).max(0.0);
        let (tw, _) = TextMeasurer::measure_styled(&text, &style, None);
        let status = if tw <= beside_w {
            let sy = y + ((bh - style.line_height) / 2.0).floor();
            y += bh;
            Rect::new(beside_x, sy, beside_w, style.line_height)
        } else {
            y += bh + theme.spacing.sm;
            let (_, th) = TextMeasurer::measure_styled(&text, &style, Some(inner_w));
            let at = Rect::new(x + pad, y, inner_w, th.max(style.line_height));
            y += at.h;
            at
        };
        let tray = if self.received.is_empty() {
            Rect::new(x + pad, y, inner_w, 0.0)
        } else {
            y += theme.spacing.lg;
            let h = self.tray().height(inner_w, theme);
            let at = Rect::new(x + pad, y, inner_w, h);
            y += h;
            at
        };
        let panel = Rect::new(x, content.y, w, y - content.y + pad);
        Layout {
            panel,
            zone,
            open,
            warn,
            status,
            tray,
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
            && self.focused.is_some()
        {
            self.set_focus(None);
            r.changed = true;
        }
        let open = self.open.handle_event(event, l.open);
        r = r.merge(open);
        if open.clicked {
            r = r.merge(self.ask(todo, Target::Open, "open_file", None));
        }
        let warn = self.warn.handle_event(event, l.warn);
        r = r.merge(warn);
        if warn.clicked {
            r = r.merge(self.ask(todo, Target::Warn, "warn", None));
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

    /// Tab walks the keyboard focus over the two buttons and off again:
    /// open file, warn, none.
    pub fn handle_edit_key(&mut self, key: EditKey) -> bool {
        if self.inert || key != EditKey::Tab {
            return false;
        }
        let next = match self.focused {
            None => Some(Target::Open),
            Some(Target::Open) => Some(Target::Warn),
            Some(Target::Warn | Target::Drop) => None,
        };
        self.set_focus(next);
        true
    }

    /// Enter presses the focused button: `open_file` or `warn` by the
    /// person.
    pub fn handle_enter(&mut self, todo: &mut TodoPage, inert: bool) -> bool {
        self.set_inert(inert);
        if inert {
            return false;
        }
        match self.focused {
            Some(Target::Open) => self.ask(todo, Target::Open, "open_file", None),
            Some(Target::Warn) => self.ask(todo, Target::Warn, "warn", None),
            Some(Target::Drop) | None => return false,
        };
        true
    }

    /// Escape lets the keyboard focus go; `false` when there was none.
    pub fn handle_escape(&mut self) -> bool {
        if self.focused.is_none() {
            return false;
        }
        self.set_focus(None);
        true
    }

    fn set_focus(&mut self, on: Option<Target>) {
        self.focused = on;
        self.open.set_focused(on == Some(Target::Open));
        self.warn.set_focused(on == Some(Target::Warn));
    }

    /// The card, the zone (its edge the accent while files hover), the
    /// two buttons, the status and the files received. What the page
    /// draws for a capability it binds, beside the scene: the card as
    /// `files`, the zone as `files.drop`, the status; each button binds
    /// itself.
    pub fn render(&mut self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let l = self.layout(content, theme);
        Panel::new().render(c, l.panel, theme);
        if let Some(ids) = &self.ids {
            c.bind(&ids.region, bounds(l.panel));
            c.bind(&ids.drop, bounds(l.zone));
            c.bind(&ids.status, bounds(l.status));
        }
        // The zone in the kit's file tile (`theme.message.attachment`):
        // its fill, its border at its radius; the accent rim while files
        // hover over it.
        let t = &theme.message.attachment;
        let radius = t.radius.min(l.zone.h / 2.0);
        c.push(rounded_rect(
            l.zone.x,
            l.zone.y,
            l.zone.w,
            l.zone.h,
            radius,
            t.file_fill.0,
        ));
        let (edge, width, ink) = if self.hovering {
            let accent = theme.colors.accent.0;
            (accent, theme.control.edge_width_strong, accent)
        } else {
            (
                t.file_border.0,
                t.file_border_width,
                theme.colors.text_mid.0,
            )
        };
        c.push(rounded_rect_stroke(
            l.zone.x, l.zone.y, l.zone.w, l.zone.h, radius, edge, width,
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
        self.warn.render(c, l.warn, theme);
        let status = self.status_text();
        let style = Self::status_style(theme);
        let color = match self.said {
            Said::NoDialog | Said::NoAlert => theme.intent_color(Intent::Destructive).0,
            _ => theme.colors.text_mid.0,
        };
        c.push(SceneNode::Text {
            key: TextNodeKey::from_style(&status, &style, Some(l.status.w)),
            x: l.status.x,
            y: l.status.y,
            color,
        });
        if !self.received.is_empty() {
            self.tray().render(c, l.tray, theme);
        }
    }
}

impl Default for FilesPage {
    fn default() -> Self {
        Self::new()
    }
}
