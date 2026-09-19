//! TextInput: the editing core of a single-line field. Owns the buffer,
//! focus, the caret blink, IME commits and click-to-caret mapping. It
//! draws nothing: [`TextField`](crate::form::TextField) wraps it with the
//! HOFF glass field, and any other look can wrap it the same way.

use engine::text::{TextMeasurer, TextStyle};

use super::buffer::TextBuffer;

/// Caret blink half-period in seconds (the platform convention: 530ms
/// on macOS and windows).
const CURSOR_BLINK_INTERVAL: f32 = 0.53;

#[derive(Clone, Debug)]
pub struct TextInput {
    pub buffer: TextBuffer,
    pub focused: bool,
    pub placeholder: String,
    pub(crate) cursor_visible: bool,
    cursor_blink_timer: f32,
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}

impl TextInput {
    pub fn new() -> Self {
        Self {
            buffer: TextBuffer::new(),
            focused: false,
            placeholder: String::new(),
            cursor_visible: true,
            cursor_blink_timer: 0.0,
        }
    }

    pub fn with_placeholder(mut self, placeholder: &str) -> Self {
        self.placeholder = placeholder.to_string();
        self
    }

    /// Caret currently drawn (focused and on the visible half of the
    /// blink).
    pub fn cursor_visible(&self) -> bool {
        self.focused && self.cursor_visible
    }

    pub fn focus(&mut self) {
        self.focused = true;
        self.cursor_visible = true;
        self.cursor_blink_timer = 0.0;
    }

    pub fn unfocus(&mut self) {
        self.focused = false;
    }

    pub fn tick(&mut self, dt: f32) {
        if !self.focused {
            return;
        }
        self.cursor_blink_timer += dt;
        if self.cursor_blink_timer >= CURSOR_BLINK_INTERVAL {
            self.cursor_blink_timer -= CURSOR_BLINK_INTERVAL;
            self.cursor_visible = !self.cursor_visible;
        }
    }

    pub fn reset_blink(&mut self) {
        self.cursor_visible = true;
        self.cursor_blink_timer = 0.0;
    }

    pub fn handle_char(&mut self, c: char) {
        if !self.focused {
            return;
        }
        self.buffer.insert_char(c);
        self.reset_blink();
    }

    pub fn handle_backspace(&mut self) {
        if !self.focused {
            return;
        }
        self.buffer.delete_back();
        self.reset_blink();
    }

    pub fn handle_delete(&mut self) {
        if !self.focused {
            return;
        }
        self.buffer.delete_forward();
        self.reset_blink();
    }

    pub fn handle_left(&mut self) {
        if !self.focused {
            return;
        }
        self.buffer.move_left();
        self.reset_blink();
    }

    pub fn handle_right(&mut self) {
        if !self.focused {
            return;
        }
        self.buffer.move_right();
        self.reset_blink();
    }

    pub fn handle_home(&mut self) {
        if !self.focused {
            return;
        }
        self.buffer.move_home();
        self.reset_blink();
    }

    pub fn handle_end(&mut self) {
        if !self.focused {
            return;
        }
        self.buffer.move_end();
        self.reset_blink();
    }

    pub fn handle_select_all(&mut self) {
        if !self.focused {
            return;
        }
        self.buffer.select_all();
        self.reset_blink();
    }

    pub fn handle_ime(&mut self, committed: &str, preedit: &str) {
        if !self.focused {
            return;
        }
        if !committed.is_empty() {
            self.buffer.insert_str(committed);
            self.reset_blink();
        }
        // preedit stored for future inline rendering
        let _ = preedit;
    }

    /// Focus and place the caret at `local_x` (px from the text origin),
    /// hit-tested with the same `style` the owner draws the text with.
    pub fn handle_click(&mut self, local_x: f32, style: &TextStyle) {
        self.focus();
        // Single-line input: hit-test on the first (only) line, no wrapping.
        let line_middle = style.line_height / 2.0;
        let cursor_pos =
            TextMeasurer::hit_test_styled(self.buffer.text(), style, None, local_x, line_middle);
        self.buffer.cursor = cursor_pos;
        self.buffer.selection = None;
        self.reset_blink();
    }
}
