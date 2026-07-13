//! TextBuffer: text editing core with char-aware cursor and selection.

/// Editable text buffer with cursor position and optional selection range.
#[derive(Clone, Debug)]
pub struct TextBuffer {
    text: String,
    pub(crate) cursor: usize,
    pub(crate) selection: Option<(usize, usize)>,
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl TextBuffer {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
            selection: None,
        }
    }

    pub fn with_text(text: &str) -> Self {
        let len = text.len();
        Self {
            text: text.to_string(),
            cursor: len,
            selection: None,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn selection(&self) -> Option<(usize, usize)> {
        self.selection
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    pub fn len(&self) -> usize {
        self.text.len()
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
        self.cursor = self.text.len();
        self.selection = None;
    }

    pub fn insert_char(&mut self, c: char) {
        self.delete_selection();
        self.text.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    pub fn insert_str(&mut self, s: &str) {
        self.delete_selection();
        self.text.insert_str(self.cursor, s);
        self.cursor += s.len();
    }

    pub fn delete_back(&mut self) {
        if self.selection.is_some() {
            self.delete_selection();
            return;
        }
        if self.cursor == 0 {
            return;
        }
        let prev = self.prev_char_boundary();
        self.text.drain(prev..self.cursor);
        self.cursor = prev;
    }

    pub fn delete_forward(&mut self) {
        if self.selection.is_some() {
            self.delete_selection();
            return;
        }
        if self.cursor >= self.text.len() {
            return;
        }
        let next = self.next_char_boundary();
        self.text.drain(self.cursor..next);
    }

    pub fn move_left(&mut self) {
        self.selection = None;
        if self.cursor > 0 {
            self.cursor = self.prev_char_boundary();
        }
    }

    pub fn move_right(&mut self) {
        self.selection = None;
        if self.cursor < self.text.len() {
            self.cursor = self.next_char_boundary();
        }
    }

    pub fn move_home(&mut self) {
        self.selection = None;
        self.cursor = 0;
    }

    pub fn move_end(&mut self) {
        self.selection = None;
        self.cursor = self.text.len();
    }

    pub fn select_all(&mut self) {
        if self.text.is_empty() {
            return;
        }
        self.selection = Some((0, self.text.len()));
        self.cursor = self.text.len();
    }

    pub fn delete_selection(&mut self) {
        if let Some((start, end)) = self.selection.take() {
            let (lo, hi) = if start <= end {
                (start, end)
            } else {
                (end, start)
            };
            self.text.drain(lo..hi);
            self.cursor = lo;
        }
    }

    pub fn selected_text(&self) -> Option<&str> {
        self.selection.map(|(start, end)| {
            let (lo, hi) = if start <= end {
                (start, end)
            } else {
                (end, start)
            };
            &self.text[lo..hi]
        })
    }

    fn prev_char_boundary(&self) -> usize {
        let mut pos = self.cursor.saturating_sub(1);
        while pos > 0 && !self.text.is_char_boundary(pos) {
            pos -= 1;
        }
        pos
    }

    fn next_char_boundary(&self) -> usize {
        let mut pos = self.cursor + 1;
        while pos < self.text.len() && !self.text.is_char_boundary(pos) {
            pos += 1;
        }
        pos.min(self.text.len())
    }
}
