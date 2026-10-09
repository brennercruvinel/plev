//! Inline commit form: a design-system [`TextField`] for the message and
//! two pill buttons (Commit, Cancel) on the column surface.
//!
//! With its ids set ([`CommitForm::set_sem`]) the render binds the form
//! surface as `commit.form`, the field as `commit.message` and the Commit
//! button as `commit.submit`. Cancel is the form's own `cancel`, so it
//! sits inside the form's binding instead of carrying a second id.

use accord::SemId;
use engine::drawing::compositor::{Compositor, SceneNode};
use engine::runtime::platform::clipboard::ClipboardProvider;
use engine::styling::theme::Theme;
use engine::widgets::buttons::{Button, ButtonVariant};
use engine::widgets::form::{ClipKey, EditKey, TextField};
use engine::widgets::prelude::{EventResult, Rect, Wake, WidgetEvent};

/// State for the inline commit form.
pub struct CommitForm {
    pub visible: bool,
    field: TextField,
    commit_btn: Button,
    cancel_btn: Button,
    commit_btn_rect: Rect,
    cancel_btn_rect: Rect,
    input_rect: Rect,
    /// `commit.form`, when the form binds.
    sem: Option<SemId>,
}

pub enum CommitFormAction {
    None,
    Commit,
    Cancel,
}

impl CommitForm {
    pub fn new() -> Self {
        Self {
            visible: false,
            field: TextField::new("Commit message..."),
            commit_btn: Button::new("Commit"),
            cancel_btn: Button::new("Cancel").variant(ButtonVariant::Ghost),
            commit_btn_rect: Rect::default(),
            cancel_btn_rect: Rect::default(),
            input_rect: Rect::default(),
            sem: None,
        }
    }

    /// The ids the render binds: the form surface, the message field and
    /// the Commit button.
    pub(crate) fn set_sem(&mut self, form: SemId, message: SemId, submit: SemId) {
        self.field = self.field.clone().sem(message);
        self.commit_btn = self.commit_btn.clone().sem(submit);
        self.sem = Some(form);
    }

    /// The message field (its projection helper reads it).
    pub fn field(&self) -> &TextField {
        &self.field
    }

    /// The Commit button's label.
    pub fn submit_label(&self) -> &str {
        &self.commit_btn.label
    }

    pub fn message(&self) -> &str {
        self.field.text()
    }

    /// Replaces the message (`commit.message`'s `set_value`).
    pub(crate) fn set_message(&mut self, text: &str) {
        self.field.set_text(text);
    }

    /// The message, leaving the field empty.
    pub fn take_message(&mut self) -> String {
        let message = self.field.text().to_string();
        self.field.set_text("");
        message
    }

    pub fn show(&mut self) {
        self.visible = true;
        self.field.set_text("");
        self.field.focus();
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.field.unfocus();
    }

    /// Hit-test a click. Returns the action. A click on a Commit button
    /// drawn disabled is still a commit asked for: the dispatcher refuses
    /// it and says why.
    pub fn hit_test_click(&self, cx: f32, cy: f32) -> CommitFormAction {
        if !self.visible {
            return CommitFormAction::None;
        }
        if self.commit_btn_rect.contains(cx, cy) {
            return CommitFormAction::Commit;
        }
        if self.cancel_btn_rect.contains(cx, cy) {
            return CommitFormAction::Cancel;
        }
        CommitFormAction::None
    }

    /// Route a pointer event to the field and the buttons (hover, caret).
    pub fn handle_event(&mut self, event: &WidgetEvent, theme: &Theme) -> EventResult {
        if !self.visible {
            return EventResult::IGNORED;
        }
        let mut r = self.field.handle_event(event, self.input_rect, theme);
        r = r.merge(self.commit_btn.handle_event(event, self.commit_btn_rect));
        r.merge(self.cancel_btn.handle_event(event, self.cancel_btn_rect))
    }

    pub fn append_char(&mut self, c: char) {
        if self.visible {
            self.field.focus();
            self.field.insert(&c.to_string());
        }
    }

    pub fn backspace(&mut self) {
        if self.visible {
            self.field.focus();
            self.field.edit(EditKey::Backspace);
        }
    }

    /// The message field while the form shows, focused as typing
    /// focuses it (`append_char`): what the clipboard keys and the
    /// field's menu reach. `None` while hidden.
    pub fn typing_field(&mut self) -> Option<&mut TextField> {
        if !self.visible {
            return None;
        }
        self.field.focus();
        Some(&mut self.field)
    }

    /// A clipboard key on the message field while the form shows (the
    /// person's keys only). `false` hidden, or when it did nothing.
    pub fn clip(&mut self, key: ClipKey, clipboard: &mut dyn ClipboardProvider) -> bool {
        self.typing_field().is_some_and(|f| f.clip(key, clipboard))
    }

    /// Advance the caret blink: while the form shows, the field's ask (a
    /// frame when the caret flipped, the time to the next flip while it
    /// is focused); hidden, nothing.
    pub fn wake(&mut self, dt: f32) -> Wake {
        if self.visible {
            self.field.wake(dt)
        } else {
            Wake::idle()
        }
    }

    /// Form height for a theme (0 when hidden).
    pub fn height(&self, theme: &Theme) -> f32 {
        if !self.visible {
            return 0.0;
        }
        let pad = theme.spacing.md;
        pad + TextField::height(theme) + pad + self.commit_btn.size.height(theme) + pad
    }

    /// Returns the height consumed by the form (0 if hidden). The Commit
    /// button draws enabled when `can_submit`, the same answer the
    /// projection gives for `commit.submit`.
    pub fn render(
        &mut self,
        compositor: &mut Compositor,
        theme: &Theme,
        x: f32,
        y: f32,
        w: f32,
        can_submit: bool,
    ) -> f32 {
        if !self.visible {
            return 0.0;
        }
        let pad = theme.spacing.md;
        let total_h = self.height(theme);

        // Column surface behind the form (same as the Changes column).
        if let Some(id) = &self.sem {
            compositor.bind(id, [x, y, w, total_h]);
        }
        compositor.push(SceneNode::Rect {
            x,
            y,
            w,
            h: total_h,
            color: theme.colors.surface.0,
        });

        self.input_rect = Rect::new(x + pad, y + pad, w - pad * 2.0, TextField::height(theme));
        self.field.render(compositor, self.input_rect, theme);

        self.commit_btn.disabled = !can_submit;
        let btn_y = self.input_rect.y + self.input_rect.h + pad;
        let (cw, ch) = self.commit_btn.preferred_size(theme);
        self.commit_btn_rect = Rect::new(x + pad, btn_y, cw, ch);
        self.commit_btn
            .render(compositor, self.commit_btn_rect, theme);

        let (xw, xh) = self.cancel_btn.preferred_size(theme);
        self.cancel_btn_rect = Rect::new(
            self.commit_btn_rect.right() + theme.spacing.sm,
            btn_y,
            xw,
            xh,
        );
        self.cancel_btn
            .render(compositor, self.cancel_btn_rect, theme);

        total_h
    }
}

impl Default for CommitForm {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use engine::runtime::platform::clipboard::{ClipboardProvider, LocalClipboard};
    use engine::widgets::form::ClipKey;

    use super::CommitForm;

    /// The message field's clipboard keys (task-74), on a
    /// `LocalClipboard`: only while the form shows.
    #[test]
    fn the_message_takes_the_clipboard_keys_while_the_form_shows() {
        let mut board = LocalClipboard::new();
        board.set_text("fix the\r\nbuild");
        let mut form = CommitForm::new();
        assert!(!form.clip(ClipKey::Paste, &mut board), "hidden");
        form.show();
        assert!(form.clip(ClipKey::Paste, &mut board));
        assert_eq!(form.message(), "fix the build");
        assert!(form.clip(ClipKey::SelectAll, &mut board));
        assert!(form.clip(ClipKey::Cut, &mut board));
        assert_eq!(form.message(), "");
        assert_eq!(board.get_text().as_deref(), Some("fix the build"));
        form.hide();
        assert!(!form.clip(ClipKey::Paste, &mut board));
        assert_eq!(form.message(), "");
    }
}
