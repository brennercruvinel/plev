//! Input routing of the gallery: keyboard shortcuts and the text routes
//! of the pages that take typing, the keyboard of the component pages and
//! of the overlays (`dispatch::keyboard`), the pointer chain (toasts, the
//! active overlay, the open select, the chrome, the page, the page
//! scroll), and the demo overlays the Dialog and Context Menu pages open.

use engine::control::input::FileDrop;
use engine::styling::theme::Intent;
use engine::widgets::overlay::OverlayKind;
use engine::widgets::overlay::drawer::DrawerAction;
use engine::widgets::prelude::{ContextMenu, EventResult, ModalAction, Rect, WidgetEvent};

use accord::Value;

use super::super::components::{context_menu, dialog};
use super::super::dispatcher::keyboard::Walk;
use super::super::dispatcher::{ShellTarget, overlay};
use super::super::{ActiveOverlay, HOTKEY_PAGES, PAGES, Pane, ShowcaseView};

/// Non-character editing keys the platform shell forwards (winit-free so
/// headless tests can drive them).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditKey {
    Tab,
    Backspace,
    Delete,
    Left,
    Right,
    Home,
    End,
    /// The four keys above with Shift: a focused text field extends its
    /// selection; anywhere else they move as the key without Shift.
    SelectLeft,
    SelectRight,
    SelectHome,
    SelectEnd,
}

impl EditKey {
    /// The key without Shift (itself when it is no `Select` key).
    pub fn unshifted(self) -> Self {
        match self {
            Self::SelectLeft => Self::Left,
            Self::SelectRight => Self::Right,
            Self::SelectHome => Self::Home,
            Self::SelectEnd => Self::End,
            other => other,
        }
    }
}

/// The field-level subset. Tab is focus traversal, never an edit, so it
/// is the one key that panics here: callers match it out first.
impl From<EditKey> for engine::widgets::form::EditKey {
    fn from(key: EditKey) -> Self {
        match key {
            EditKey::Backspace => Self::Backspace,
            EditKey::Delete => Self::Delete,
            EditKey::Left => Self::Left,
            EditKey::Right => Self::Right,
            EditKey::Home => Self::Home,
            EditKey::End => Self::End,
            EditKey::SelectLeft => Self::SelectLeft,
            EditKey::SelectRight => Self::SelectRight,
            EditKey::SelectHome => Self::SelectHome,
            EditKey::SelectEnd => Self::SelectEnd,
            EditKey::Tab => unreachable!("Tab is focus traversal, handled by the page"),
        }
    }
}

impl ShowcaseView {
    // -- Input ---------------------------------------------------------------

    /// Returns `true` when the key was consumed (and a redraw is needed).
    pub fn handle_key(&mut self, key: &str) -> bool {
        // A focused text field owns the characters: "t" types, it does
        // not switch the theme. Not beneath an overlay that holds the
        // pointer: the keyboard is the overlay's then.
        let held = self.overlay_holds_pointer();
        let pane = self.pane();
        let typed = !held
            && if let Some(host) = self.panes.text_host_mut(pane) {
                host.handle_text(key)
            } else {
                match pane {
                    Some(Pane::Todo) => self.panes.todo.handle_text(key),
                    Some(Pane::Surface) => {
                        self.panes.surface.handle_text(key, &mut self.panes.todo)
                    }
                    Some(Pane::Silences) => {
                        self.panes.silences.handle_text(key, &mut self.panes.todo)
                    }
                    Some(Pane::Git) => self.panes.git.handle_text(key),
                    _ => false,
                }
            };
        if typed {
            return true;
        }
        // Space presses what has the focus on a component page or in the
        // overlay, as Enter does.
        if key == " " && self.walks() && self.walk(Walk::Press) {
            return true;
        }
        // The shell's keys are its capabilities (`dispatch::shell`): each
        // is a person's request through the gallery's dispatcher, the
        // path the menu, a screen reader and an agent take. A refused
        // request still counts as handled: the person aimed at the shell.
        match key {
            "t" | "T" => {
                let next = match self.theme_name.as_str() {
                    "core" => "hoff",
                    "hoff" => "dark",
                    "dark" => "light",
                    _ => "core",
                };
                self.ask_shell(ShellTarget::Theme, "select", Some(Value::Text(next.into())));
                true
            }
            d @ ("1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9") => {
                let idx = (d.as_bytes()[0] - b'1') as usize;
                self.ask_page(idx);
                true
            }
            // "0" reads as 10 (matching the sidebar hints); the pages
            // past the tenth are reachable via the sidebar.
            "0" if PAGES.len() >= HOTKEY_PAGES => {
                self.ask_page(HOTKEY_PAGES - 1);
                true
            }
            // The page to a window of its own, or back (task-76 item 2):
            // the shell owns the windows and takes the request.
            "d" | "D" => {
                self.ask_shell(ShellTarget::Detach, "toggle", None);
                true
            }
            _ => false,
        }
    }

    /// A person picks the page at `index` of the table (a digit, a
    /// sidebar link): `shell.page`, through the dispatcher.
    pub(in crate::views) fn ask_page(&mut self, index: usize) -> bool {
        let Some(page) = PAGES.get(index) else {
            return false;
        };
        let title = Value::Text(page.title.to_owned());
        self.ask_shell(ShellTarget::Page, "select", Some(title))
    }

    /// Close the topmost overlay (Escape): the modal or the menu that
    /// holds the pointer first (the page beneath keeps its focus, the
    /// control that opened it), then the page's focus blurs (an open
    /// select closes), then the rest of the overlay stack. Returns `false`
    /// when there was nothing to close (caller may quit).
    pub fn close_top_overlay(&mut self) -> bool {
        if self.overlay_holds_pointer() && self.overlay_mgr.pop_animated().is_some() {
            return true;
        }
        if self.escape_sidebar() {
            return true;
        }
        if self.blur_page() {
            return true;
        }
        self.overlay_mgr.pop_animated().is_some()
    }

    /// The page lets its focus go (Escape, or the sidebar took the
    /// focus): a focused text blurs, an open select closes. `true` when
    /// it had one.
    pub(in crate::views) fn blur_page(&mut self) -> bool {
        let pane = self.pane();
        if let Some(host) = self.panes.text_host_mut(pane) {
            return host.handle_escape();
        }
        match pane {
            Some(Pane::Checkbox) => self.panes.checkbox.handle_escape(),
            Some(Pane::Switch) => self.panes.switch.handle_escape(),
            Some(Pane::Slider) => self.panes.slider.handle_escape(),
            Some(Pane::Select) => self.panes.select.handle_escape(),
            Some(Pane::Tabs) => self.panes.tabs.handle_escape(),
            Some(Pane::Todo) => self.panes.todo.handle_escape(),
            Some(Pane::Surface) => self.panes.surface.handle_escape(&mut self.panes.todo),
            Some(Pane::Silences) => self.panes.silences.handle_escape(),
            Some(Pane::Files) => self.panes.files.handle_escape(),
            Some(Pane::Capture) => self.panes.capture.handle_escape(),
            Some(Pane::Git) => self.panes.git.handle_escape(),
            _ => self.blur_component(),
        }
    }

    /// Files dragged over the window, as the engine says it (the shell
    /// forwards the event whole, translating nothing): the active page
    /// takes it. `true` when a frame is needed.
    pub fn handle_file_drop(&mut self, drop: &FileDrop) -> bool {
        self.page_file_drop(drop).changed
    }

    /// Non-character editing keys (Tab, Backspace, arrows) forwarded by
    /// the platform shell. On a component page, or over the modal or the
    /// menu, Tab walks the projection and the arrows, Home and End move in
    /// the widget the focus is in (`dispatch::keyboard`), unless a focused
    /// text field edits with them; on an app page, the page's own (Tab
    /// the page leaves walks to the sidebar). A
    /// `Select` key the page does not take moves as the key without Shift.
    pub fn handle_edit_key(&mut self, key: EditKey) -> bool {
        let plain = key.unshifted();
        if !self.walks() {
            let took = self.page_edit_key(key) || (plain != key && self.page_edit_key(plain));
            // a page with no Tab of its own: Tab reaches the sidebar
            return took || (plain == EditKey::Tab && self.walk(Walk::Tab));
        }
        let walk = match plain {
            EditKey::Tab => return self.walk(Walk::Tab),
            EditKey::Left => Walk::Left,
            EditKey::Right => Walk::Right,
            EditKey::Home => Walk::Home,
            EditKey::End => Walk::End,
            // Backspace and Delete (`plain` is never a `Select` key).
            _ => return self.page_edit_key(key),
        };
        self.page_edit_key(key) || self.walk(walk)
    }

    /// The active page's own editing keys: a focused text field edits, an
    /// app page walks its own focus. Nothing beneath an overlay that
    /// holds the pointer.
    fn page_edit_key(&mut self, key: EditKey) -> bool {
        if self.overlay_holds_pointer() {
            return false;
        }
        let pane = self.pane();
        if let Some(host) = self.panes.text_host_mut(pane) {
            return host.handle_edit_key(key);
        }
        match pane {
            Some(Pane::Todo) => self.panes.todo.handle_edit_key(key),
            Some(Pane::Surface) => self
                .panes
                .surface
                .handle_edit_key(key, &mut self.panes.todo),
            Some(Pane::Silences) => self
                .panes
                .silences
                .handle_edit_key(key, &mut self.panes.todo),
            Some(Pane::Files) => self.panes.files.handle_edit_key(key),
            Some(Pane::Capture) => self.panes.capture.handle_edit_key(key),
            Some(Pane::Git) if key == EditKey::Backspace => self.panes.git.handle_backspace(),
            _ => false,
        }
    }

    /// Enter: on a component page, or over the modal or the menu, what
    /// has the focus is pressed (`dispatch::keyboard`); the todo app adds
    /// the field text as a todo; the Surface and Silences pages propose
    /// from their intent field; the Files page presses its focused button;
    /// the Git tab submits its commit form.
    pub fn handle_enter(&mut self) -> bool {
        // A focused text that takes Enter (a line break, a pick from a
        // filtered list) has it before the focus walk.
        let pane = self.pane();
        if !self.overlay_holds_pointer()
            && let Some(host) = self.panes.text_host_mut(pane)
            && host.handle_enter()
        {
            return true;
        }
        if self.walks() {
            return self.walk(Walk::Press);
        }
        match self.pane() {
            Some(Pane::Todo) => {
                let inert = self.overlay_holds_pointer();
                self.panes.todo.set_inert(inert);
                self.panes.todo.handle_enter()
            }
            Some(Pane::Surface) => {
                let inert = self.overlay_holds_pointer();
                self.panes.surface.handle_enter(&mut self.panes.todo, inert)
            }
            Some(Pane::Silences) => {
                let inert = self.overlay_holds_pointer();
                self.panes
                    .silences
                    .handle_enter(&mut self.panes.todo, inert)
            }
            Some(Pane::Files) => {
                let inert = self.overlay_holds_pointer();
                self.panes.files.handle_enter(&mut self.panes.todo, inert)
            }
            Some(Pane::Capture) => {
                let inert = self.overlay_holds_pointer();
                self.panes.capture.handle_enter(&mut self.panes.todo, inert)
            }
            Some(Pane::Git) => self.panes.git.handle_enter(),
            _ => false,
        }
    }

    pub fn handle_right_click(&mut self, x: f32, y: f32) -> bool {
        // A text field's own menu (`field_menu.rs`).
        if self.panes.text_host(self.pane()).is_some() {
            return self.open_field_menu(x, y);
        }
        match self.pane() {
            Some(Pane::Git) => {
                let content = self.page_rect();
                self.panes.git.handle_right_click(x, y, content)
            }
            Some(Pane::ContextMenu) if self.active_overlay.is_none() => {
                let content = self.page_rect();
                if !self
                    .panes
                    .context_menu
                    .area(content, &self.theme)
                    .contains(x, y)
                {
                    return false;
                }
                self.open_menu(x, y);
                true
            }
            _ => false,
        }
    }

    fn open_menu(&mut self, x: f32, y: f32) {
        self.show_menu(context_menu::demo_menu(), x, y);
    }

    /// The Context Menu page's menu at the middle of its area: a reader's
    /// press or an agent's `open` there, as a right click.
    pub(crate) fn open_page_menu(&mut self) {
        if self.active_overlay.is_some() {
            return;
        }
        let area = self.panes.context_menu.area(self.page_rect(), &self.theme);
        self.open_menu(area.x + area.w / 2.0, area.y + area.h / 2.0);
    }

    /// `widget` opens as the context menu at `(x, y)`, kept inside the
    /// viewport: the Context Menu page's demo, or a text field's menu
    /// (`field_menu.rs`).
    pub(super) fn show_menu(&mut self, widget: ContextMenu, x: f32, y: f32) {
        let mut widget = widget;
        if let Some(id) = overlay::id(overlay::MENU) {
            widget = widget.sem(id);
        }
        self.overlay_focus = None;
        let (w, h) = widget.size(&self.theme);
        // Keep the menu inside the viewport.
        let margin = self.theme.spacing.sm;
        let mx = x.min(self.width - w - margin);
        let my = y.min(self.height - h - margin);
        let id = self.overlay_mgr.push_animated(
            OverlayKind::ContextMenu { items: vec![] },
            mx,
            my,
            w,
            h,
            &self.theme.intent_motion(Intent::Neutral),
        );
        self.active_overlay = Some(ActiveOverlay::Menu {
            id,
            widget,
            x: mx,
            y: my,
        });
    }

    pub(crate) fn open_modal(&mut self, destructive: bool) {
        // The Dialog page's questions (`components/dialog.rs`).
        let (widget, intent) = if destructive {
            (dialog::destructive(), Intent::Destructive)
        } else {
            (dialog::plain(), Intent::Neutral)
        };
        let widget = match overlay::id(overlay::DIALOG) {
            Some(id) => widget.sem(id),
            None => widget,
        };
        self.overlay_focus = None;
        let dialog = widget.dialog_rect(&self.theme, self.width, self.height);
        let id = self.overlay_mgr.push_animated(
            OverlayKind::Modal {
                title: widget.title.clone(),
                body: widget.body.clone(),
                confirm: String::new(),
                cancel: String::new(),
            },
            dialog.x,
            dialog.y,
            dialog.w,
            dialog.h,
            &self.theme.intent_motion(intent),
        );
        self.active_overlay = Some(ActiveOverlay::Modal {
            id,
            widget: Box::new(widget),
        });
    }

    /// True while the active overlay takes every pointer event: a modal
    /// or a menu that is not fading out. The page beneath is then inert
    /// (the todo page projects itself so, see `TodoPage::set_inert`).
    pub(crate) fn overlay_holds_pointer(&self) -> bool {
        self.active_overlay
            .as_ref()
            .is_some_and(|active| overlay::holds(&self.overlay_mgr, active))
    }

    /// Route a pointer event. Returns `true` if a redraw is needed.
    pub fn handle_event(&mut self, event: &WidgetEvent) -> bool {
        let (vw, vh) = (self.width, self.height);
        // a focus the pointer moves is where the person looks: the page
        // does not scroll to it (`compositor/reveal.rs`)
        if matches!(event, WidgetEvent::MouseDown { .. }) {
            self.reveal.pointer();
        }

        // Toasts float above everything.
        let toast_result = self.toasts.handle_event(event, &self.theme, vw, vh);
        if toast_result.clicked {
            return true;
        }
        let mut result = toast_result;

        // Active overlay is exclusive while it is not fading out. What a
        // click asks of it (a button of the modal, an item of the menu, the
        // menu closed by a click outside) is the person's request, through
        // the dispatcher (`dispatch::overlay`), as a reader's press is.
        let held = self.overlay_holds_pointer();
        if held && let Some(active) = &mut self.active_overlay {
            let (asked, changed): (Option<(accord::SemId, &str)>, bool) = match active {
                ActiveOverlay::Modal { widget, .. } => {
                    let (action, r) = widget.handle_event(event, &self.theme, vw, vh);
                    let button = match action {
                        ModalAction::Confirm => Some("confirm"),
                        ModalAction::Cancel => Some("cancel"),
                        ModalAction::None => None,
                    };
                    let asked =
                        button.and_then(|b| widget.sem_id().map(|id| (id.key(b), "activate")));
                    (asked, r.changed || button.is_some())
                }
                ActiveOverlay::Menu { widget, x, y, .. } => {
                    let (r, clicked) = widget.handle_event(event, *x, *y, &self.theme);
                    let (w, h) = widget.size(&self.theme);
                    let outside = !r.handled
                        && matches!(*event, WidgetEvent::MouseDown { x: px, y: py }
                            if !Rect::new(*x, *y, w, h).contains(px, py));
                    let asked = match (clicked, outside, widget.sem_id()) {
                        (Some(item), _, Some(id)) => Some((id.key(item.to_string()), "activate")),
                        (None, true, Some(id)) => Some((id.clone(), "close")),
                        _ => None,
                    };
                    (asked, r.changed)
                }
                // task-104 front B: the edge panels (`dispatcher/overlay/edge.rs`).
                ActiveOverlay::Sheet { widget, .. } => {
                    let window = Rect::new(0.0, 0.0, vw, vh);
                    let (action, r) = widget.handle_event(event, &self.theme, window);
                    let button = match action {
                        ModalAction::Confirm => Some("confirm"),
                        ModalAction::Cancel => Some("cancel"),
                        ModalAction::None => None,
                    };
                    let asked =
                        button.and_then(|b| widget.sem_id().map(|id| (id.key(b), "activate")));
                    (asked, r.changed || button.is_some())
                }
                ActiveOverlay::Drawer { widget, .. } => {
                    let window = Rect::new(0.0, 0.0, vw, vh);
                    let (action, r) = widget.handle_event(event, &self.theme, window);
                    let key = match action {
                        DrawerAction::Pick(i) => Some(i.to_string()),
                        DrawerAction::Close => Some("close".to_owned()),
                        DrawerAction::None => None,
                    };
                    let acted = key.is_some();
                    let asked = key.and_then(|k| widget.sem_id().map(|id| (id.key(k), "activate")));
                    (asked, r.changed || acted)
                }
            };
            return match asked {
                Some((target, action)) => {
                    self.ask_overlay(&target, action);
                    true
                }
                None => changed,
            };
        }

        // An open select dropdown gets priority over everything beneath.
        if self.pane() == Some(Pane::Select) && self.panes.select.is_open() {
            let content = self.page_rect();
            let r = self.panes.select.handle_event(event, content, &self.theme);
            if r.handled || r.changed {
                return r.changed;
            }
        }

        // The chrome first: the sidebar (an open drawer is exclusive),
        // the drawer's menu button (`navigation.rs`).
        let (taken, changed) = self.chrome_event(event);
        if taken {
            return result.changed || changed;
        }
        result.changed |= changed;

        // Clicks on the header band belong to the chrome: widgets scrolled
        // underneath it must not receive them.
        let viewport = self.content_rect();
        let page_x = self.page_x();
        if let WidgetEvent::MouseDown { x, y } = *event
            && x >= page_x
            && y < viewport.y
        {
            return result.changed;
        }

        let page_result = self.page_event(event);
        result = result.merge(page_result);

        // Page scroll: when no widget consumed the wheel, scroll the page
        // itself (HOFF pages overflow the window). Clamped by
        // ScrollState; only an actual offset change requests a frame.
        if let WidgetEvent::Scroll { x, delta, .. } = *event
            && !result.handled
            && x >= page_x
        {
            self.sync_page_scroll();
            self.glide.stop();
            let scroll = &mut self.page_scroll[self.page];
            let old = scroll.offset();
            scroll.scroll_by(delta);
            if scroll.offset() != old {
                result = result.merge(EventResult::changed());
            }
        }

        result.changed
    }
}
