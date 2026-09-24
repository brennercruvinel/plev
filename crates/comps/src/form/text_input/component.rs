//! TextInput: the editing core of a single-line field. Owns the buffer,
//! focus, the caret blink, IME commits and click-to-caret mapping. It
//! draws nothing: [`TextField`](crate::form::TextField) wraps it with the
//! HOFF glass field, and any other look can wrap it the same way.

use engine::text::{TextMeasurer, TextStyle};

use super::buffer::TextBuffer;
use crate::wake::{Blink, Wake};

/// Caret blink half-period in seconds (the platform convention: 530ms
/// on macOS and windows).
pub const CURSOR_BLINK_INTERVAL: f32 = 0.53;

#[derive(Clone, Debug)]
pub struct TextInput {
    pub buffer: TextBuffer,
    pub focused: bool,
    pub placeholder: String,
    blink: Blink,
    /// Focused or edited since the last tick: the half-period restarted
    /// at that event, so the next tick counts none of the time before it
    /// (a shell passes the real time since the last frame, which can be
    /// seconds of idle before the key that focused).
    fresh: bool,
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
            blink: Blink::new(),
            fresh: false,
        }
    }

    pub fn with_placeholder(mut self, placeholder: &str) -> Self {
        self.placeholder = placeholder.to_string();
        self
    }

    /// Caret currently drawn (focused and on the visible half of the
    /// blink).
    pub fn cursor_visible(&self) -> bool {
        self.focused && self.blink.visible()
    }

    pub fn focus(&mut self) {
        self.focused = true;
        self.reset_blink();
    }

    pub fn unfocus(&mut self) {
        self.focused = false;
    }

    /// Advance the caret blink by `dt` seconds. While focused the answer
    /// is the blink's: `now` when the caret flipped, `next` the time to
    /// the next flip, so a shell draws two or three frames a second for
    /// a caret at rest, not sixty. Unfocused there is no caret and
    /// nothing pending. The first tick after a focus or an edit counts
    /// no time: the half-period restarted at that event, not at the last
    /// frame.
    pub fn wake(&mut self, dt: f32) -> Wake {
        if !self.focused {
            return Wake::idle();
        }
        let dt = if std::mem::take(&mut self.fresh) {
            0.0
        } else {
            dt
        };
        self.blink.advance(dt, CURSOR_BLINK_INTERVAL)
    }

    /// Solid caret, the half-period restarted from this event (any edit
    /// or caret move calls this so the caret never blinks away
    /// mid-typing).
    pub fn reset_blink(&mut self) {
        self.blink.reset();
        self.fresh = true;
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
