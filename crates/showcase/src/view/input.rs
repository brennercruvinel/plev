//! Input routing of the gallery: keyboard shortcuts and the text routes
//! of the sections that take typing, the pointer chain (toasts, the
//! active overlay, the open select, the chrome, the section, the page
//! scroll), and the demo overlays the Overlays section opens.

use comps::overlay::OverlayKind;
use comps::prelude::{EventResult, Modal, ModalAction, Rect, WidgetEvent};
use engine::theme::Intent;

use super::{ActiveOverlay, EditKey, Section, ShowcaseView, overlays};

impl ShowcaseView {
    // -- Input ---------------------------------------------------------------

    /// Returns `true` when the key was consumed (and a redraw is needed).
    pub fn handle_key(&mut self, key: &str) -> bool {
        // A focused text field owns the characters: "t" types, it does
        // not switch the theme.
        if self.section == Section::Forms && self.forms.handle_text(key) {
            return true;
        }
        if self.section == Section::App && self.app.handle_text(key) {
            return true;
        }
        if self.section == Section::Git && self.git.handle_text(key) {
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
                let idx = d.as_bytes()[0] - b'1';
                self.set_section(Section::ALL[idx as usize]);
                true
            }
            // "0" reads as 10 (matching the sidebar numbering); sections
            // past the tenth are reachable via the sidebar.
            "0" if Section::ALL.len() >= 10 => {
                self.set_section(Section::ALL[9]);
                true
            }
            _ => false,
        }
    }

    /// Close the topmost overlay (Escape): forms focus blurs first, then
    /// the open select, then the overlay stack. Returns `false` when
    /// there was nothing to close (caller may quit).
    pub fn close_top_overlay(&mut self) -> bool {
        if self.section == Section::Forms && self.forms.handle_escape() {
            return true;
        }
        if self.section == Section::App && self.app.handle_escape() {
            return true;
        }
        if self.section == Section::Git && self.git.handle_escape() {
            return true;
        }
        if self.forms.select_is_open() {
            self.forms.close_select();
            return true;
        }
        self.overlay_mgr.pop_animated().is_some()
    }

    /// Non-character editing keys (Tab, Backspace, arrows) forwarded by
    /// the platform shell. Forms: Tab cycles its widgets, the rest edit
    /// the focused text field. App: the add field.
    pub fn handle_edit_key(&mut self, key: EditKey) -> bool {
        match self.section {
            Section::Forms => self.forms.handle_edit_key(key),
            Section::App => self.app.handle_edit_key(key),
            Section::Git if key == EditKey::Backspace => self.git.handle_backspace(),
            _ => false,
        }
    }

    /// Enter: the App section adds the field text as a todo; the Git tab
    /// submits its commit form.
    pub fn handle_enter(&mut self) -> bool {
        match self.section {
            Section::App => self.app.handle_enter(),
            Section::Git => self.git.handle_enter(),
            _ => false,
        }
    }

    pub fn handle_right_click(&mut self, x: f32, y: f32) -> bool {
        if self.section == Section::Git {
            let content = self.page_rect();
            return self.git.handle_right_click(x, y, content);
        }
        if self.section != Section::Overlays || self.active_overlay.is_some() {
            return false;
        }
        let content = self.page_rect();
        if !self.overlays.menu_area(content, &self.theme).contains(x, y) {
            return false;
        }
        self.open_menu(x, y);
        true
    }

    fn open_menu(&mut self, x: f32, y: f32) {
        let widget = overlays::demo_menu();
        let (w, h) = widget.size(&self.theme);
        // Keep the menu inside the viewport.
        let mx = x.min(self.width - w - 8.0);
        let my = y.min(self.height - h - 8.0);
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

    fn open_modal(&mut self, destructive: bool) {
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
                            let label = overlays::menu_label(item);
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

        // Open select dropdown gets priority over everything beneath it.
        if self.section == Section::Forms && self.forms.select_is_open() {
            let r = self
                .forms
                .route_select(event, self.page_rect(), &self.theme);
            if r.handled || r.changed {
                return r.changed;
            }
        }

        // The chrome first: sidebar navigation (a drawer is exclusive),
        // the menu button on a phone.
        let (shell_result, nav) = self.shell.handle_event(event, &self.theme);
        if let Some(i) = nav {
            self.set_section(Section::ALL[i]);
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

        let content = self.page_rect();
        let section_result = match self.section {
            Section::Cards => self.cards.handle_event(event, content, &self.theme),
            Section::Buttons => self.buttons.handle_event(event, content, &self.theme),
            Section::Forms => self.forms.handle_event(event, content, &self.theme),
            Section::Overlays => {
                let (r, action) = self.overlays.handle_event(event, content, &self.theme);
                match action {
                    overlays::OverlayAction::OpenModal { destructive } => {
                        self.open_modal(destructive)
                    }
                    overlays::OverlayAction::PushToast(intent) => {
                        let msg = match intent {
                            Intent::Neutral => "Neutral toast — plain information.",
                            Intent::Constructive => "Saved! Everything went fine.",
                            Intent::Destructive => "Failed to push: remote rejected.",
                            Intent::Informational => "3 new commits fetched from origin.",
                        };
                        self.toasts.push(msg, intent, &self.theme);
                    }
                    overlays::OverlayAction::None => {}
                }
                r
            }
            Section::Lists => self.lists.handle_event(event, content, &self.theme),
            Section::Icons => self.icons_gallery.handle_event(event, content),
            Section::Charts => self.charts.handle_event(event, content),
            Section::Dock => self.dock.handle_event(event, content),
            Section::App => self.app.handle_event(event, content, &self.theme),
            Section::Builder => self.builder_tour.handle_event(event, content),
            Section::Extras => self.extras.handle_event(event, content, &self.theme),
            Section::Typography => self.typography.handle_event(event, content),
            Section::Effects => self.effects.handle_event(event, content),
            Section::Chrome => self.chrome.handle_event(event, content, &self.theme),
            Section::Editor => self.editor.handle_event(event, content),
            Section::Git => self.git.handle_event(event, content),
            Section::Theme => {
                let (r, picked) = self.themes.handle_event(event, content);
                if let Some(name) = picked {
                    self.set_theme(name);
                }
                r
            }
        };
        result = result.merge(section_result);

        // Page scroll: when no widget consumed the wheel, scroll the
        // section itself (HOFF pages overflow the window). Clamped by
        // ScrollState; only an actual offset change requests a frame.
        if let WidgetEvent::Scroll { x, delta, .. } = *event
            && !result.handled
            && x >= page_x
        {
            self.sync_page_scroll();
            let idx = self.section_idx();
            let scroll = &mut self.page_scroll[idx];
            let old = scroll.offset();
            scroll.scroll_by(delta);
            if scroll.offset() != old {
                result = result.merge(EventResult::changed());
            }
        }

        result.changed
    }
}
