//! Inline commit form: a design-system [`TextField`] for the message and
//! two pill buttons (Commit, Cancel) on the column surface.

use comps::action::{Button, ButtonVariant};
use comps::form::{EditKey, TextField};
use comps::prelude::{EventResult, Rect, WidgetEvent};
use engine::compositor::{Compositor, SceneNode};
use engine::theme::Theme;

/// State for the inline commit form.
pub struct CommitForm {
    pub visible: bool,
    field: TextField,
    commit_btn: Button,
    cancel_btn: Button,
    commit_btn_rect: Rect,
    cancel_btn_rect: Rect,
    input_rect: Rect,
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
        }
    }

    pub fn message(&self) -> &str {
        self.field.text()
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

    /// Hit-test a click. Returns the action.
    pub fn hit_test_click(&self, cx: f32, cy: f32) -> CommitFormAction {
        if !self.visible {
            return CommitFormAction::None;
        }
        if self.commit_btn_rect.contains(cx, cy) && !self.commit_btn.disabled {
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

    /// Advance the caret blink; `true` while the form wants frames.
    pub fn tick(&mut self, dt: f32) -> bool {
        self.visible && self.field.tick(dt)
    }

    /// Form height for a theme (0 when hidden).
    pub fn height(&self, theme: &Theme) -> f32 {
        if !self.visible {
            return 0.0;
        }
        let pad = theme.spacing.md;
        pad + TextField::height(theme) + pad + self.commit_btn.size.height(theme) + pad
    }

    /// Returns the height consumed by the form (0 if hidden).
    pub fn render(
        &mut self,
        compositor: &mut Compositor,
        theme: &Theme,
        x: f32,
        y: f32,
        w: f32,
    ) -> f32 {
        if !self.visible {
            return 0.0;
        }
        let pad = theme.spacing.md;
        let total_h = self.height(theme);

        // Column surface behind the form (same as the Changes column).
        compositor.push(SceneNode::Rect {
            x,
            y,
            w,
            h: total_h,
            color: theme.colors.surface.0,
        });

        self.input_rect = Rect::new(x + pad, y + pad, w - pad * 2.0, TextField::height(theme));
        self.field.render(compositor, self.input_rect, theme);

        self.commit_btn.disabled = self.field.is_empty();
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
