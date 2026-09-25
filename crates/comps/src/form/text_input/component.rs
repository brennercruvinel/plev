//! TextInput: the editing core of a single-line field. Owns the buffer,
//! focus, the caret blink, IME commits, click-to-caret mapping and the
//! caret and selection geometry. It draws nothing:
//! [`TextField`](crate::form::TextField) wraps it with the HOFF glass
//! field, and any other look can wrap it the same way.
//!
//! The arrow keys move on screen: in a right-to-left run the left arrow
//! goes forward in the text. Home and End go to the logical ends, and the
//! `select` variants extend the selection, which stays one logical range
//! and draws as the boxes it covers (see `super::visual`).

use engine::text::TextStyle;

use super::buffer::TextBuffer;
use super::visual::VisualLine;
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
        self.blink.reset();
    }

    pub fn unfocus(&mut self) {
        self.focused = false;
    }

    /// Advance the caret blink by `dt` seconds. While focused the answer
    /// is the blink's: `now` when the caret flipped, `next` the time to
    /// the next flip, so a shell draws two or three frames a second for
    /// a caret at rest, not sixty. Unfocused there is no caret and
    /// nothing pending.
    pub fn wake(&mut self, dt: f32) -> Wake {
        if !self.focused {
            return Wake::idle();
        }
        self.blink.advance(dt, CURSOR_BLINK_INTERVAL)
    }

    pub fn reset_blink(&mut self) {
        self.blink.reset();
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

    /// One caret stop left on screen (in a right-to-left run, forward in
    /// the text); drops the selection.
    pub fn handle_left(&mut self) {
        self.step(false, false);
    }

    /// One caret stop right on screen; drops the selection.
    pub fn handle_right(&mut self) {
        self.step(true, false);
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

    /// Shift+Left: extend the selection one caret stop left on screen.
    pub fn handle_select_left(&mut self) {
        self.step(false, true);
    }

    /// Shift+Right: extend the selection one caret stop right on screen.
    pub fn handle_select_right(&mut self) {
        self.step(true, true);
    }

    /// Shift+Home: extend the selection to the start of the text.
    pub fn handle_select_home(&mut self) {
        if !self.focused {
            return;
        }
        self.buffer.select_to(0);
        self.reset_blink();
    }

    /// Shift+End: extend the selection to the end of the text.
    pub fn handle_select_end(&mut self) {
        if !self.focused {
            return;
        }
        self.buffer.select_to(self.buffer.len());
        self.reset_blink();
    }

    /// Move (or, `extend`, select) to the neighbouring caret stop on
    /// screen. The order comes from the bidi levels, which the text alone
    /// fixes, so any style gives it; the default one is used. At the end
    /// of the line the caret stays.
    fn step(&mut self, rightward: bool, extend: bool) {
        if !self.focused {
            return;
        }
        let pos = self.buffer.cursor();
        let line = VisualLine::new(self.buffer.text(), &TextStyle::default(), None);
        let next = if rightward {
            line.right_of(pos)
        } else {
            line.left_of(pos)
        };
        let next = next.unwrap_or(pos);
        if extend {
            self.buffer.select_to(next);
        } else {
            self.buffer.set_cursor(next);
        }
        self.reset_blink();
    }

    /// Where the caret draws, px from the text origin, for the text laid
    /// out with `style` in a box `width` wide (`None`: unbounded; a
    /// right-to-left paragraph hangs from the box's right edge).
    pub fn caret_x(&self, style: &TextStyle, width: Option<f32>) -> f32 {
        VisualLine::new(self.buffer.text(), style, width).caret_x(self.buffer.cursor())
    }

    /// The selection as the boxes it covers on screen, `(x, w)` from the
    /// text origin, left to right: one box in Latin text, more where the
    /// range crosses a change of direction. Empty without a selection.
    pub fn selection_boxes(&self, style: &TextStyle, width: Option<f32>) -> Vec<(f32, f32)> {
        let Some((a, b)) = self.buffer.selection() else {
            return Vec::new();
        };
        VisualLine::new(self.buffer.text(), style, width).boxes(a.min(b), a.max(b))
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
        self.handle_click_in(local_x, style, None);
    }

    /// [`handle_click`](Self::handle_click) for text drawn in a box
    /// `width` wide: the caret goes to the stop nearest the click, the
    /// place it is drawn.
    pub fn handle_click_in(&mut self, local_x: f32, style: &TextStyle, width: Option<f32>) {
        self.focus();
        let pos = VisualLine::new(self.buffer.text(), style, width).hit(local_x);
        self.buffer.set_cursor(pos);
        self.reset_blink();
    }
}
