//! TextInput: the editing core of a single-line field. Owns the buffer,
//! focus, the caret blink, the input method's composition, click-to-caret
//! mapping and the caret and selection geometry. It draws nothing:
//! [`TextField`](crate::form::TextField) wraps it with the HOFF glass
//! field, and any other look can wrap it the same way.
//!
//! The arrow keys move on screen: in a right-to-left run the left arrow
//! goes forward in the text. Home and End go to the logical ends, and the
//! `select` variants extend the selection, which stays one logical range
//! and draws as the boxes it covers (see `super::visual`).
//!
//! An input method's preedit (the text still being composed, as in
//! Japanese or Chinese) is kept apart from the buffer: it draws in place
//! of the selection, or at the caret, until the input method commits it
//! as typed text or drops it.

use std::ops::Range;

use crate::runtime::platform::clipboard::ClipboardProvider;
use crate::drawing::text::TextStyle;

use super::buffer::TextBuffer;
use super::visual::VisualLine;
use crate::widgets::wake::{Blink, Wake};

/// Caret blink half-period in seconds (the platform convention: 530ms
/// on macOS and windows).
pub const CURSOR_BLINK_INTERVAL: f32 = 0.53;

#[derive(Clone, Debug)]
pub struct TextInput {
    pub buffer: TextBuffer,
    pub focused: bool,
    pub placeholder: String,
    blink: Blink,
    preedit: Option<Preedit>,
}

/// The text an input method is composing and its caret in it (a byte
/// offset of `text`).
#[derive(Clone, Debug)]
struct Preedit {
    text: String,
    caret: usize,
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
            preedit: None,
        }
    }

    pub fn with_placeholder(mut self, placeholder: &str) -> Self {
        self.placeholder = placeholder.to_string();
        self
    }

    /// Caret currently drawn: focused and on the visible half of the
    /// blink, or composing (the caret holds still in a preedit, as in the
    /// editor).
    pub fn cursor_visible(&self) -> bool {
        self.focused && (self.preedit.is_some() || self.blink.visible())
    }

    pub fn focus(&mut self) {
        self.focused = true;
        self.blink.reset();
    }

    /// Blur; a composition in progress is dropped with the focus.
    pub fn unfocus(&mut self) {
        self.focused = false;
        self.preedit = None;
    }

    /// Advance the caret blink by `dt` seconds. While focused the answer
    /// is the blink's: `now` when the caret flipped, `next` the time to
    /// the next flip, so a shell draws two or three frames a second for
    /// a caret at rest, not sixty. Unfocused there is no caret and
    /// nothing pending.
    pub fn wake(&mut self, dt: f32) -> Wake {
        if !self.focused || self.preedit.is_some() {
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
    /// right-to-left paragraph hangs from the box's right edge). While
    /// composing, the caret is the input method's, in the preedit, laid
    /// out in the text as drawn ([`composed`](Self::composed)).
    pub fn caret_x(&self, style: &TextStyle, width: Option<f32>) -> f32 {
        match (self.composed(), &self.preedit) {
            (Some((text, span)), Some(p)) => {
                VisualLine::new(&text, style, width).caret_x(span.start + p.caret)
            }
            _ => VisualLine::new(self.buffer.text(), style, width).caret_x(self.buffer.cursor()),
        }
    }

    /// The selection as the boxes it covers on screen, `(x, w)` from the
    /// text origin, left to right: one box in Latin text, more where the
    /// range crosses a change of direction. Empty without a selection,
    /// and while composing (the preedit draws in its place).
    pub fn selection_boxes(&self, style: &TextStyle, width: Option<f32>) -> Vec<(f32, f32)> {
        let Some((a, b)) = self.buffer.selection() else {
            return Vec::new();
        };
        if self.preedit.is_some() {
            return Vec::new();
        }
        VisualLine::new(self.buffer.text(), style, width).boxes(a.min(b), a.max(b))
    }

    /// The text the input method is composing; empty when none.
    pub fn preedit(&self) -> &str {
        self.preedit.as_ref().map_or("", |p| p.text.as_str())
    }

    /// The text as drawn while composing: the selection, or the caret,
    /// replaced by the preedit (what a commit of it would leave), and the
    /// byte range the preedit takes in it. `None` when not composing.
    pub fn composed(&self) -> Option<(String, Range<usize>)> {
        let p = self.preedit.as_ref()?;
        let text = self.buffer.text();
        let cursor = self.buffer.cursor();
        let (lo, hi) = self
            .buffer
            .selection()
            .map_or((cursor, cursor), |(a, b)| (a.min(b), a.max(b)));
        let composed = format!("{}{}{}", &text[..lo], p.text, &text[hi..]);
        Some((composed, lo..lo + p.text.len()))
    }

    /// The preedit as the boxes it covers on screen, `(x, w)` from the
    /// text origin, for its underline. Empty when not composing.
    pub fn preedit_boxes(&self, style: &TextStyle, width: Option<f32>) -> Vec<(f32, f32)> {
        match self.composed() {
            Some((text, span)) => VisualLine::new(&text, style, width).boxes(span.start, span.end),
            None => Vec::new(),
        }
    }

    pub fn handle_select_all(&mut self) {
        if !self.focused {
            return;
        }
        self.buffer.select_all();
        self.reset_blink();
    }

    /// An input method's step, the preedit's caret at its end (see
    /// [`handle_ime_at`](Self::handle_ime_at)).
    pub fn handle_ime(&mut self, committed: &str, preedit: &str) {
        self.handle_ime_at(committed, preedit, None);
    }

    /// An input method's step: `committed` goes in as typed text (over
    /// the selection), then `preedit`, the text still being composed,
    /// shows at the caret until the next step replaces it; an empty one
    /// ends the composition. `cursor` is the input method's caret in
    /// `preedit`, a byte range as winit reports it (its start draws);
    /// `None`, or a range that does not fall on a character, puts it at
    /// the end. The preedit never enters the buffer. Unfocused, nothing
    /// happens.
    pub fn handle_ime_at(
        &mut self,
        committed: &str,
        preedit: &str,
        cursor: Option<(usize, usize)>,
    ) {
        if !self.focused {
            return;
        }
        if !committed.is_empty() {
            self.buffer.insert_str(committed);
        }
        self.preedit = (!preedit.is_empty()).then(|| Preedit {
            text: preedit.to_owned(),
            caret: match cursor {
                Some((start, _)) if preedit.is_char_boundary(start) => start,
                _ => preedit.len(),
            },
        });
        self.reset_blink();
    }

    /// Drop the composition (the input method went away); the text
    /// stays. `true` when there was one.
    pub fn clear_preedit(&mut self) -> bool {
        self.preedit.take().is_some()
    }

    /// Copy the selection to `clipboard`. `false`, the clipboard left as
    /// it was, with nothing selected or unfocused.
    pub fn handle_copy(&self, clipboard: &mut dyn ClipboardProvider) -> bool {
        if !self.focused {
            return false;
        }
        match self.buffer.selected_text() {
            Some(text) if !text.is_empty() => {
                clipboard.set_text(text);
                true
            }
            _ => false,
        }
    }

    /// Copy the selection, then delete it. `false` with nothing selected.
    pub fn handle_cut(&mut self, clipboard: &mut dyn ClipboardProvider) -> bool {
        if !self.handle_copy(clipboard) {
            return false;
        }
        self.buffer.delete_selection();
        self.reset_blink();
        true
    }

    /// Type what `clipboard` holds, over the selection, on one line (see
    /// [`single_line`]). `false` with an empty clipboard or unfocused.
    pub fn handle_paste(&mut self, clipboard: &mut dyn ClipboardProvider) -> bool {
        if !self.focused {
            return false;
        }
        let text = single_line(&clipboard.get_text().unwrap_or_default());
        if text.is_empty() {
            return false;
        }
        self.buffer.insert_str(&text);
        self.reset_blink();
        true
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

/// Text pasted into a single-line field: the line breaks at its ends go,
/// each one inside (`\r\n` counts as one) and each tab becomes a space,
/// and any other control character is dropped.
pub fn single_line(text: &str) -> String {
    let text = text.trim_matches(['\r', '\n']).replace("\r\n", "\n");
    text.chars()
        .filter_map(|c| match c {
            '\n' | '\r' | '\t' => Some(' '),
            c if c.is_control() => None,
            c => Some(c),
        })
        .collect()
}
