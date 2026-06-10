//! Input handling: keyboard, mouse and IME events -> document operations.

use std::ops::Range;

use editor_core::{Selection, SelectionSet, Transaction, movement};
use unicode_segmentation::UnicodeSegmentation;
use winit::event::Ime;
use winit::keyboard::{Key, ModifiersState, NamedKey};

use super::view::{ClickRecord, DragState, EditorView, Preedit};

/// Max delay between clicks of a multi-click, in seconds.
const MULTI_CLICK_SECS: f32 = 0.4;
/// Max cursor travel between clicks of a multi-click, in pixels.
const MULTI_CLICK_SLOP: f32 = 4.0;

/// Mouse input in window coordinates, already filtered to the left button
/// by the caller (winit reports button and motion separately).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MouseEvent {
    /// Left button pressed. `alt` adds a cursor, `shift` extends the
    /// primary selection. Double/triple clicks are detected internally.
    Down {
        x: f32,
        y: f32,
        alt: bool,
        shift: bool,
    },
    /// Cursor moved while the left button is held.
    Drag { x: f32, y: f32 },
    /// Left button released.
    Up,
    /// Scroll wheel in pixels; positive `dy` scrolls down.
    Wheel { dy: f32 },
}

impl EditorView {
    // -- keyboard ----------------------------------------------------------

    /// Handle a pressed key. Returns true when the event was consumed (the
    /// caller should redraw); unhandled combinations (e.g. `cmd-s`) return
    /// false so the app can act on them.
    pub fn handle_key(&mut self, key: &Key, mods: ModifiersState) -> bool {
        if self.preedit.is_some() {
            // The IME owns the keyboard while composing.
            return false;
        }
        let shift = mods.shift_key();
        let alt = mods.alt_key();
        let primary = mods.super_key() || mods.control_key();

        let handled = match key {
            Key::Named(NamedKey::Backspace) => self.edit(|v| v.document.delete_backward()),
            Key::Named(NamedKey::Delete) => self.edit(|v| v.document.delete_forward()),
            Key::Named(NamedKey::Enter) => self.edit(|v| v.document.insert("\n")),
            Key::Named(NamedKey::Tab) => {
                let tab = self.config.tab_text();
                self.edit(|v| v.document.insert(&tab))
            }
            Key::Named(NamedKey::Space) if !primary => self.edit(|v| v.document.insert(" ")),
            Key::Named(NamedKey::ArrowLeft) => {
                self.move_each(move |rope, sel| match (primary, alt) {
                    (true, _) => movement::line_start(rope, sel, shift),
                    (_, true) => movement::move_word_left(rope, sel, shift),
                    _ => movement::move_left(rope, sel, shift),
                });
                true
            }
            Key::Named(NamedKey::ArrowRight) => {
                self.move_each(move |rope, sel| match (primary, alt) {
                    (true, _) => movement::line_end(rope, sel, shift),
                    (_, true) => movement::move_word_right(rope, sel, shift),
                    _ => movement::move_right(rope, sel, shift),
                });
                true
            }
            Key::Named(NamedKey::ArrowUp) if primary => {
                self.move_each(move |_, sel| head_to(sel, 0, shift));
                true
            }
            Key::Named(NamedKey::ArrowDown) if primary => {
                let end = self.document.len_bytes();
                self.move_each(move |_, sel| head_to(sel, end, shift));
                true
            }
            Key::Named(NamedKey::ArrowUp) => {
                self.move_vertical(-1, shift);
                true
            }
            Key::Named(NamedKey::ArrowDown) => {
                self.move_vertical(1, shift);
                true
            }
            Key::Named(NamedKey::Home) => {
                self.move_each(move |rope, sel| movement::smart_home(rope, sel, shift));
                true
            }
            Key::Named(NamedKey::End) => {
                self.move_each(move |rope, sel| movement::line_end(rope, sel, shift));
                true
            }
            Key::Named(NamedKey::PageUp) => {
                self.move_vertical(-(self.page_lines() as isize), shift);
                true
            }
            Key::Named(NamedKey::PageDown) => {
                self.move_vertical(self.page_lines() as isize, shift);
                true
            }
            Key::Character(text) if primary => match text.to_lowercase().as_str() {
                "z" if shift => self.edit(|v| {
                    v.document.redo();
                }),
                "z" => self.edit(|v| {
                    v.document.undo();
                }),
                "a" => {
                    self.select_all();
                    true
                }
                "c" => self.copy(),
                "x" => self.cut(),
                "v" => self.paste(),
                _ => false,
            },
            Key::Character(text) => {
                if text.chars().any(char::is_control) {
                    false
                } else {
                    self.edit(|v| v.document.insert(text))
                }
            }
            _ => false,
        };

        if handled {
            self.reset_blink();
        }
        handled
    }

    /// Run an edit, then reset vertical-movement state and follow the cursor.
    fn edit(&mut self, f: impl FnOnce(&mut Self)) -> bool {
        f(self);
        self.goal.clear();
        self.scroll_to_cursor();
        true
    }

    /// Apply a horizontal/jump movement to every selection.
    fn move_each(&mut self, f: impl FnMut(&editor_core::Rope, Selection) -> Selection) {
        self.document.transform_selections(f);
        self.goal.clear();
        self.scroll_to_cursor();
    }

    /// Move every selection `delta` lines (negative = up), keeping one goal
    /// column per selection across consecutive vertical moves.
    fn move_vertical(&mut self, delta: isize, extend: bool) {
        let count = self.document.selections().len();
        if self.goal.len() != count {
            self.goal = vec![editor_core::GoalColumn::new(); count];
        }
        let goals = &mut self.goal;
        let mut i = 0;
        self.document.transform_selections(|rope, sel| {
            let goal = &mut goals[i];
            i += 1;
            if delta < 0 {
                movement::page_up(rope, sel, extend, delta.unsigned_abs(), goal)
            } else {
                movement::page_down(rope, sel, extend, delta as usize, goal)
            }
        });
        self.scroll_to_cursor();
    }

    /// One viewport worth of lines, for PageUp/PageDown.
    fn page_lines(&self) -> usize {
        (self.bounds.height / self.config.line_height)
            .floor()
            .max(1.0) as usize
    }

    /// Select the whole document (single selection, head at the end).
    pub fn select_all(&mut self) {
        let end = self.document.len_bytes();
        self.document
            .set_selections(SelectionSet::single(Selection::new(0, end)));
        self.goal.clear();
        self.reset_blink();
    }

    // -- clipboard ---------------------------------------------------------

    /// Copy the selected text. Multiple selections are joined with `\n`.
    /// Returns false (no clipboard change) when every selection is a caret.
    pub fn copy(&mut self) -> bool {
        let rope = self.document.rope();
        let pieces: Vec<String> = self
            .document
            .selections()
            .iter()
            .filter(|s| !s.is_caret())
            .map(|s| rope.byte_slice(s.min()..s.max()).to_string())
            .collect();
        if pieces.is_empty() {
            return false;
        }
        self.clipboard.set_text(&pieces.join("\n"));
        true
    }

    /// Copy, then delete the selected text.
    pub fn cut(&mut self) -> bool {
        if !self.copy() {
            return false;
        }
        let tx = Transaction::change(
            self.document
                .selections()
                .iter()
                .filter(|s| !s.is_caret())
                .map(|s| (s.min()..s.max(), "")),
        );
        self.edit(|v| v.document.apply(tx))
    }

    /// Paste the clipboard at every cursor. When the clipboard holds exactly
    /// one `\n`-separated piece per cursor (a multi-cursor copy), each cursor
    /// receives its own piece.
    pub fn paste(&mut self) -> bool {
        let Some(text) = self.clipboard.get_text() else {
            return false;
        };
        if text.is_empty() {
            return false;
        }
        let selections = self.document.selections().clone();
        let pieces: Vec<&str> = text.split('\n').collect();
        if selections.len() > 1 && pieces.len() == selections.len() {
            let tx = Transaction::change(
                selections
                    .iter()
                    .zip(pieces)
                    .map(|(s, piece)| (s.min()..s.max(), piece)),
            );
            self.edit(|v| v.document.apply(tx))
        } else {
            self.edit(|v| v.document.insert(&text))
        }
    }

    // -- mouse -------------------------------------------------------------

    /// Handle a mouse event. Coordinates are window-relative; the view uses
    /// the bounds from the last `render`/`set_bounds`. Returns true when the
    /// editor state changed.
    pub fn handle_mouse(&mut self, event: MouseEvent) -> bool {
        match event {
            MouseEvent::Down { x, y, alt, shift } => {
                self.on_mouse_down(x, y, alt, shift);
                self.reset_blink();
                true
            }
            MouseEvent::Drag { x, y } => {
                let Some(drag) = self.drag else { return false };
                let head = self.hit_test_point(x, y);
                self.set_primary_selection(Selection::new(drag.anchor, head));
                true
            }
            MouseEvent::Up => {
                let was_dragging = self.drag.take().is_some();
                was_dragging
            }
            MouseEvent::Wheel { dy } => {
                let before = self.scroll.offset();
                self.scroll.scroll_by(dy);
                self.scroll.offset() != before
            }
        }
    }

    fn on_mouse_down(&mut self, x: f32, y: f32, alt: bool, shift: bool) {
        let pos = self.hit_test_point(x, y);
        self.goal.clear();

        if alt {
            // Multi-cursor: add a caret, keeping the existing selections.
            let mut set = self.document.selections().clone();
            set.add_selection(Selection::caret(pos));
            self.document.set_selections(set);
            self.drag = Some(DragState { anchor: pos });
            self.last_click = None;
            return;
        }

        if shift {
            let anchor = self.document.selections().primary().anchor;
            self.document
                .set_selections(SelectionSet::single(Selection::new(anchor, pos)));
            self.drag = Some(DragState { anchor });
            self.last_click = None;
            return;
        }

        let selection = match self.click_count(x, y) {
            2 => range_selection(self.word_range_at(pos)),
            3 => range_selection(self.line_range_at(pos)),
            _ => Selection::caret(pos),
        };
        self.drag = Some(DragState {
            anchor: selection.anchor,
        });
        self.document
            .set_selections(SelectionSet::single(selection));
    }

    /// Replace the primary selection, keeping the others (used by drags).
    fn set_primary_selection(&mut self, sel: Selection) {
        let set = self.document.selections();
        let mut selections = set.selections().to_vec();
        let idx = set.primary_index();
        selections[idx] = sel;
        self.document
            .set_selections(SelectionSet::new(selections, idx));
    }

    /// 1 for a single click, 2/3 for double/triple clicks (cycles back to 1).
    fn click_count(&mut self, x: f32, y: f32) -> u8 {
        let now = web_time::Instant::now();
        let count = match self.last_click {
            Some(prev)
                if now.duration_since(prev.at).as_secs_f32() <= MULTI_CLICK_SECS
                    && (prev.x - x).abs() <= MULTI_CLICK_SLOP
                    && (prev.y - y).abs() <= MULTI_CLICK_SLOP =>
            {
                prev.count % 3 + 1
            }
            _ => 1,
        };
        self.last_click = Some(ClickRecord {
            at: now,
            x,
            y,
            count,
        });
        count
    }

    /// Byte range of the word (Unicode word boundaries) containing `pos`.
    pub(super) fn word_range_at(&self, pos: usize) -> Range<usize> {
        let rope = self.document.rope();
        let pos = pos.min(rope.len_bytes());
        let line = rope.byte_to_line(pos);
        let lstart = rope.line_to_byte(line);
        let text = self.line_text(line);
        let rel = pos - lstart;
        let mut last: Option<(usize, &str)> = None;
        for (i, word) in text.split_word_bound_indices() {
            if rel >= i && rel < i + word.len() {
                return lstart + i..lstart + i + word.len();
            }
            last = Some((i, word));
        }
        // Click at/past the end of the line content: take the last segment.
        match last {
            Some((i, word)) => lstart + i..lstart + i + word.len(),
            None => pos..pos,
        }
    }

    /// Byte range of the whole line containing `pos`, newline included.
    pub(super) fn line_range_at(&self, pos: usize) -> Range<usize> {
        let rope = self.document.rope();
        let pos = pos.min(rope.len_bytes());
        let line = rope.byte_to_line(pos);
        let start = rope.line_to_byte(line);
        let end = if line + 1 < rope.len_lines() {
            rope.line_to_byte(line + 1)
        } else {
            rope.len_bytes()
        };
        start..end
    }

    // -- IME ---------------------------------------------------------------

    /// Handle a winit IME event. Preedit text is stored for inline rendering
    /// (`render` splices it into the primary cursor's line) and commits are
    /// inserted at every cursor. Returns true when a redraw is needed.
    pub fn handle_ime(&mut self, ime: &Ime) -> bool {
        match ime {
            Ime::Enabled => false,
            Ime::Preedit(text, cursor) => {
                self.preedit = (!text.is_empty()).then(|| Preedit {
                    text: text.clone(),
                    cursor: *cursor,
                });
                self.reset_blink();
                true
            }
            Ime::Commit(text) => {
                self.preedit = None;
                if !text.is_empty() {
                    self.edit(|v| v.document.insert(text));
                }
                self.reset_blink();
                true
            }
            Ime::Disabled => self.preedit.take().is_some(),
        }
    }
}

/// Move the head to `pos`, collapsing unless extending.
fn head_to(sel: Selection, pos: usize, extend: bool) -> Selection {
    if extend {
        Selection::new(sel.anchor, pos)
    } else {
        Selection::caret(pos)
    }
}

fn range_selection(range: Range<usize>) -> Selection {
    Selection::new(range.start, range.end)
}
