//! Input routing of the gallery: keyboard shortcuts and the text routes
//! of the pages that take typing, the pointer chain (toasts, the active
//! overlay, the open select, the chrome, the page, the page scroll), and
//! the demo overlays the Dialog and Context Menu pages open.

use comps::overlay::OverlayKind;
use comps::prelude::{EventResult, Modal, ModalAction, Rect, WidgetEvent};
use engine::theme::Intent;

use super::components::context_menu;
use super::{ActiveOverlay, HOTKEY_PAGES, PAGES, Pane, ShowcaseView};

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
}

/// The field-level subset. Tab is focus traversal, never an edit, so it
/// is the one key that panics here: callers match it out first.
impl From<EditKey> for comps::form::EditKey {
    fn from(key: EditKey) -> Self {
        match key {
            EditKey::Backspace => Self::Backspace,
            EditKey::Delete => Self::Delete,
            EditKey::Left => Self::Left,
            EditKey::Right => Self::Right,
            EditKey::Home => Self::Home,
            EditKey::End => Self::End,
            EditKey::Tab => unreachable!("Tab is focus traversal, handled by the page"),
        }
    }
}

impl ShowcaseView {
    // -- Input ---------------------------------------------------------------

    /// Returns `true` when the key was consumed (and a redraw is needed).
    pub fn handle_key(&mut self, key: &str) -> bool {
        // A focused text field owns the characters: "t" types, it does
        // not switch the theme.
        let typed = match self.pane() {
            Some(Pane::Input) => self.panes.input.handle_text(key),
            Some(Pane::Todo) => self.panes.todo.handle_text(key),
            Some(Pane::Git) => self.panes.git.handle_text(key),
            _ => false,
        };
        if typed {
            return true;
        }
        match key {
            "t" | "T" => {
                let next = match self.theme_name.as_str() {
                    "core" => "hoff",
                    "hoff" => "dark",
                    "dark" => "light",
                    _ => "core",
                };
                self.set_theme(next);
                true
            }
            d @ ("1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9") => {
                let idx = (d.as_bytes()[0] - b'1') as usize;
                self.set_page(idx);
                true
            }
            // "0" reads as 10 (matching the sidebar hints); the pages
            // past the tenth are reachable via the sidebar.
            "0" if PAGES.len() >= HOTKEY_PAGES => {
                self.set_page(HOTKEY_PAGES - 1);
                true
            }
            _ => false,
        }
    }

    /// Close the topmost overlay (Escape): the page's focus blurs first
    /// (an open select closes), then the overlay stack. Returns `false`
    /// when there was nothing to close (caller may quit).
    pub fn close_top_overlay(&mut self) -> bool {
        let blurred = match self.pane() {
            Some(Pane::Input) => self.panes.input.handle_escape(),
            Some(Pane::Checkbox) => self.panes.checkbox.handle_escape(),
            Some(Pane::Switch) => self.panes.switch.handle_escape(),
            Some(Pane::Slider) => self.panes.slider.handle_escape(),
            Some(Pane::Select) => self.panes.select.handle_escape(),
            Some(Pane::Tabs) => self.panes.tabs.handle_escape(),
            Some(Pane::Todo) => self.panes.todo.handle_escape(),
            Some(Pane::Git) => self.panes.git.handle_escape(),
            _ => false,
        };
        if blurred {
            return true;
        }
        self.overlay_mgr.pop_animated().is_some()
    }

    /// Non-character editing keys (Tab, Backspace, arrows) forwarded by
    /// the platform shell: Tab cycles the page's focusable widgets, the
    /// rest edit its focused text field.
    pub fn handle_edit_key(&mut self, key: EditKey) -> bool {
        match self.pane() {
            Some(Pane::Input) => self.panes.input.handle_edit_key(key),
            Some(Pane::Checkbox) => self.panes.checkbox.handle_edit_key(key),
            Some(Pane::Switch) => self.panes.switch.handle_edit_key(key),
            Some(Pane::Slider) => self.panes.slider.handle_edit_key(key),
            Some(Pane::Select) => self.panes.select.handle_edit_key(key),
            Some(Pane::Tabs) => self.panes.tabs.handle_edit_key(key),
            Some(Pane::Todo) => self.panes.todo.handle_edit_key(key),
            Some(Pane::Git) if key == EditKey::Backspace => self.panes.git.handle_backspace(),
            _ => false,
        }
    }

    /// Enter: the todo app adds the field text as a todo; the Git tab
    /// submits its commit form.
    pub fn handle_enter(&mut self) -> bool {
        match self.pane() {
            Some(Pane::Todo) => self.panes.todo.handle_enter(),
            Some(Pane::Git) => self.panes.git.handle_enter(),
            _ => false,
        }
    }

    pub fn handle_right_click(&mut self, x: f32, y: f32) -> bool {
        match self.pane() {
            Some(Pane::Git) => {
                let content = self.page_rect();
                self.panes.git.handle_right_click(x, y, content)
            }
            Some(Pane::ContextMenu) if self.active_overlay.is_none() => {
                let content = self.page_rect();
                if !context_menu::ContextMenuPage::area(content, &self.theme).contains(x, y) {
                    return false;
                }
                self.open_menu(x, y);
                true
            }
            _ => false,
        }
    }

    fn open_menu(&mut self, x: f32, y: f32) {
        let widget = context_menu::demo_menu();
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

    pub(super) fn open_modal(&mut self, destructive: bool) {
        let (widget, intent) = if destructive {
            (
                Modal::new(
                    "Delete repository?",
                    "This permanently removes the repository and all of its \
                     history. This action cannot be undone.",
                    "Delete",
                    "Cancel",
                )
                .intent(Intent::Destructive),
                Intent::Destructive,
            )
        } else {
            (
                Modal::new(
                    "Apply settings?",
                    "Your workspace will reload to apply the new configuration.",
                    "Apply",
                    "Cancel",
                )
                .intent(Intent::Neutral),
                Intent::Neutral,
            )
        };
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

    /// Route a pointer event. Returns `true` if a redraw is needed.
    pub fn handle_event(&mut self, event: &WidgetEvent) -> bool {
        let (vw, vh) = (self.width, self.height);

        // Toasts float above everything.
        let toast_result = self.toasts.handle_event(event, &self.theme, vw, vh);
        if toast_result.clicked {
            return true;
        }
        let mut result = toast_result;

        // Active overlay is exclusive while it is not fading out.
        if let Some(active) = &mut self.active_overlay {
            let id = active.id();
            let closing = !self
                .overlay_mgr
                .stack
                .iter()
                .any(|o| o.id == id && !o.is_closing());
            if !closing {
                match active {
                    ActiveOverlay::Modal { widget, .. } => {
                        let (action, r) = widget.handle_event(event, &self.theme, vw, vh);
                        match action {
                            ModalAction::Confirm => {
                                let destructive = widget.intent == Intent::Destructive;
                                self.overlay_mgr.pop_id_animated(id);
                                if destructive {
                                    self.toasts.push(
                                        "Repository deleted (not really).",
                                        Intent::Destructive,
                                        &self.theme,
                                    );
                                } else {
                                    self.toasts.push(
                                        "Settings applied.",
                                        Intent::Constructive,
                                        &self.theme,
                                    );
                                }
                            }
                            ModalAction::Cancel => self.overlay_mgr.pop_id_animated(id),
                            ModalAction::None => {}
                        }
                        return r.changed || action != ModalAction::None;
                    }
                    ActiveOverlay::Menu { widget, x, y, .. } => {
                        let (r, clicked) = widget.handle_event(event, *x, *y, &self.theme);
                        if let Some(item) = clicked {
                            let label = context_menu::menu_label(item);
                            self.overlay_mgr.pop_id_animated(id);
                            self.toasts.push(
                                format!("Context menu: {label}"),
                                Intent::Informational,
                                &self.theme,
                            );
                            return true;
                        }
                        if !r.handled
                            && let WidgetEvent::MouseDown { x: px, y: py } = *event
                        {
                            let (w, h) = widget.size(&self.theme);
                            if !Rect::new(*x, *y, w, h).contains(px, py) {
                                self.overlay_mgr.pop_id_animated(id);
                                return true;
                            }
                        }
                        return r.changed;
                    }
                }
            }
        }

        // An open select dropdown gets priority over everything beneath.
        if self.pane() == Some(Pane::Select) && self.panes.select.is_open() {
            let content = self.page_rect();
            let r = self.panes.select.handle_event(event, content, &self.theme);
            if r.handled || r.changed {
                return r.changed;
            }
        }

        // The chrome first: sidebar navigation (a drawer is exclusive),
        // the menu button on a phone.
        let (shell_result, nav) = self.shell.handle_event(event, &self.theme);
        if let Some(i) = nav {
            self.set_page(i);
            return true;
        }
        result = result.merge(shell_result);
        if shell_result.handled {
            return result.changed;
        }

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
