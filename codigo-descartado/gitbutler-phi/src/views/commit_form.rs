use phi::compositor::{Compositor, SceneNode, TextNodeKey};
use crate::theme::Theme;
use crate::components::button::{ButtonKind, ButtonSize, draw as draw_button};

const PAD: f32 = 12.0;
const INPUT_H: f32 = 32.0;
const BTN_H: f32 = 32.0;
const FONT_SIZE: f32 = 13.0;

/// State for the inline commit form.
pub struct CommitForm {
    pub message: String,
    pub visible: bool,
    commit_btn_rect: (f32, f32, f32, f32),
    cancel_btn_rect: (f32, f32, f32, f32),
    input_rect: (f32, f32, f32, f32),
}

pub enum CommitFormAction {
    None,
    Commit,
    Cancel,
}

impl CommitForm {
    pub fn new() -> Self {
        Self {
            message: String::new(),
            visible: false,
            commit_btn_rect: (0.0, 0.0, 0.0, 0.0),
            cancel_btn_rect: (0.0, 0.0, 0.0, 0.0),
            input_rect: (0.0, 0.0, 0.0, 0.0),
        }
    }

    pub fn show(&mut self) {
        self.visible = true;
        self.message.clear();
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Hit-test a click. Returns the action.
    pub fn hit_test_click(&self, cx: f32, cy: f32) -> CommitFormAction {
        if !self.visible { return CommitFormAction::None; }

        let (bx, by, bw, bh) = self.commit_btn_rect;
        if cx >= bx && cx <= bx + bw && cy >= by && cy <= by + bh {
            return CommitFormAction::Commit;
        }

        let (bx, by, bw, bh) = self.cancel_btn_rect;
        if cx >= bx && cx <= bx + bw && cy >= by && cy <= by + bh {
            return CommitFormAction::Cancel;
        }

        CommitFormAction::None
    }

    pub fn append_char(&mut self, c: char) {
        if self.visible {
            self.message.push(c);
        }
    }

    pub fn backspace(&mut self) {
        if self.visible {
            self.message.pop();
        }
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
        if !self.visible { return 0.0; }

        let total_h = PAD + INPUT_H + PAD + BTN_H + PAD;

        // Form background
        compositor.push(SceneNode::Rect {
            x, y, w, h: total_h,
            color: theme.bg_3.to_array(),
        });

        // Top divider
        compositor.push(SceneNode::Rect {
            x, y, w, h: 1.0,
            color: theme.border.to_array(),
        });

        // Input field (simple text box)
        let input_x = x + PAD;
        let input_y = y + PAD;
        let input_w = w - PAD * 2.0;
        self.input_rect = (input_x, input_y, input_w, INPUT_H);

        compositor.push(SceneNode::RoundedRect {
            x: input_x, y: input_y, w: input_w, h: INPUT_H,
            color: theme.bg_1.to_array(),
            corner_radius: 4.0,
            border_width: 1.0,
            border_color: theme.border_focus.to_array(),
        });

        // Message text or placeholder
        let display_text = if self.message.is_empty() {
            "Commit message..."
        } else {
            &self.message
        };
        let text_color = if self.message.is_empty() {
            theme.text_3.to_array()
        } else {
            theme.text_1.to_array()
        };
        compositor.push(SceneNode::Text {
            key: TextNodeKey::new(display_text, FONT_SIZE, 18.0, Some(input_w - 16.0)).with_weight(400),
            x: input_x + 8.0,
            y: input_y + (INPUT_H - 18.0) / 2.0,
            color: text_color,
        });

        // Buttons row
        let btn_y = input_y + INPUT_H + PAD;
        self.commit_btn_rect = draw_button(
            compositor, theme,
            input_x, btn_y,
            "Commit",
            ButtonKind::Solid,
            ButtonSize::Sm,
            false,
            self.message.is_empty(),
        );

        let cancel_x = input_x + self.commit_btn_rect.2 + 8.0;
        self.cancel_btn_rect = draw_button(
            compositor, theme,
            cancel_x, btn_y,
            "Cancel",
            ButtonKind::Ghost,
            ButtonSize::Sm,
            false,
            false,
        );

        total_h
    }
}
