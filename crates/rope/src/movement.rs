//! Pure cursor movement functions: `(rope, selection, extend) -> Selection`.
//!
//! Horizontal movement is grapheme-aware (ZWJ emoji clusters move as one
//! unit), word movement uses Unicode word boundaries, and vertical movement
//! keeps a persistent goal column so crossing a short line and coming back
//! restores the original column.

use std::borrow::Cow;

use ropey::Rope;
use unicode_segmentation::UnicodeSegmentation;

use crate::selection::Selection;

/// Remembered target column (in graphemes) for vertical movement. The
/// caller keeps it across consecutive up/down moves and resets it on any
/// horizontal movement or edit.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GoalColumn {
    col: Option<usize>,
}

impl GoalColumn {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.col = None;
    }

    pub fn get(&self) -> Option<usize> {
        self.col
    }
}

fn line_str(rope: &Rope, line_idx: usize) -> Cow<'_, str> {
    rope.line(line_idx).into()
}

/// Byte position just past the last grapheme starting at or before `byte`.
/// Graphemes never span line breaks, so scanning one line suffices.
pub fn next_grapheme_boundary(rope: &Rope, byte: usize) -> usize {
    let total = rope.len_bytes();
    if byte >= total {
        return total;
    }
    let line_idx = rope.byte_to_line(byte);
    let line_start = rope.line_to_byte(line_idx);
    let s = line_str(rope, line_idx);
    let rel = byte - line_start;
    for (i, g) in s.grapheme_indices(true) {
        if i <= rel && rel < i + g.len() {
            return line_start + i + g.len();
        }
    }
    total
}

/// Largest grapheme boundary strictly before `byte`.
pub fn prev_grapheme_boundary(rope: &Rope, byte: usize) -> usize {
    if byte == 0 {
        return 0;
    }
    let byte = byte.min(rope.len_bytes());
    let line_idx = rope.byte_to_line(byte - 1);
    let line_start = rope.line_to_byte(line_idx);
    let s = line_str(rope, line_idx);
    let rel = byte - line_start;
    let mut prev = line_start;
    for (i, _) in s.grapheme_indices(true) {
        if i >= rel {
            break;
        }
        prev = line_start + i;
    }
    prev
}

/// Byte position of the end of a line's content (before its `\n`, if any).
pub(crate) fn line_content_end(rope: &Rope, line_idx: usize) -> usize {
    let start = rope.line_to_byte(line_idx);
    let line = rope.line(line_idx);
    let mut len = line.len_bytes();
    if line.len_chars() > 0 && line.char(line.len_chars() - 1) == '\n' {
        len -= 1;
    }
    start + len
}

fn grapheme_col(rope: &Rope, pos: usize) -> usize {
    let line_idx = rope.byte_to_line(pos);
    let line_start = rope.line_to_byte(line_idx);
    let s = line_str(rope, line_idx);
    let rel = pos - line_start;
    s.grapheme_indices(true)
        .take_while(|(i, _)| *i < rel)
        .count()
}

fn pos_at_grapheme_col(rope: &Rope, line_idx: usize, col: usize) -> usize {
    let line_start = rope.line_to_byte(line_idx);
    let s = line_str(rope, line_idx);
    let content = s.strip_suffix('\n').unwrap_or(s.as_ref());
    for (count, (i, _)) in content.grapheme_indices(true).enumerate() {
        if count == col {
            return line_start + i;
        }
    }
    line_start + content.len()
}

fn next_word_end(rope: &Rope, pos: usize) -> usize {
    let total = rope.len_bytes();
    if pos >= total {
        return total;
    }
    let mut line_idx = rope.byte_to_line(pos);
    while line_idx < rope.len_lines() {
        let line_start = rope.line_to_byte(line_idx);
        let s = line_str(rope, line_idx);
        for (i, word) in s.split_word_bound_indices() {
            let end = line_start + i + word.len();
            if end <= pos {
                continue;
            }
            if word.chars().any(char::is_alphanumeric) {
                return end;
            }
        }
        line_idx += 1;
    }
    total
}

fn prev_word_start(rope: &Rope, pos: usize) -> usize {
    if pos == 0 || rope.len_bytes() == 0 {
        return 0;
    }
    let pos = pos.min(rope.len_bytes());
    let mut line_idx = rope.byte_to_line(pos - 1);
    loop {
        let line_start = rope.line_to_byte(line_idx);
        let s = line_str(rope, line_idx);
        let mut found = None;
        for (i, word) in s.split_word_bound_indices() {
            let start = line_start + i;
            if start >= pos {
                break;
            }
            if word.chars().any(char::is_alphanumeric) {
                found = Some(start);
            }
        }
        if let Some(start) = found {
            return start;
        }
        if line_idx == 0 {
            return 0;
        }
        line_idx -= 1;
    }
}

fn with_head(sel: Selection, head: usize, extend: bool) -> Selection {
    if extend {
        Selection::new(sel.anchor, head)
    } else {
        Selection::caret(head)
    }
}

/// Move one grapheme left. A non-caret selection collapses to its start
/// when not extending.
pub fn move_left(rope: &Rope, sel: Selection, extend: bool) -> Selection {
    if !extend && !sel.is_caret() {
        return Selection::caret(sel.min());
    }
    with_head(sel, prev_grapheme_boundary(rope, sel.head), extend)
}

/// Move one grapheme right. A non-caret selection collapses to its end
/// when not extending.
pub fn move_right(rope: &Rope, sel: Selection, extend: bool) -> Selection {
    if !extend && !sel.is_caret() {
        return Selection::caret(sel.max());
    }
    with_head(sel, next_grapheme_boundary(rope, sel.head), extend)
}

/// Move to the start of the previous word (Unicode word boundaries).
pub fn move_word_left(rope: &Rope, sel: Selection, extend: bool) -> Selection {
    with_head(sel, prev_word_start(rope, sel.head), extend)
}

/// Move to the end of the next word (Unicode word boundaries).
pub fn move_word_right(rope: &Rope, sel: Selection, extend: bool) -> Selection {
    with_head(sel, next_word_end(rope, sel.head), extend)
}

/// Move to column 0 of the head's line.
pub fn line_start(rope: &Rope, sel: Selection, extend: bool) -> Selection {
    let head = rope.line_to_byte(rope.byte_to_line(sel.head));
    with_head(sel, head, extend)
}

/// Move to the end of the head's line content (before the newline).
pub fn line_end(rope: &Rope, sel: Selection, extend: bool) -> Selection {
    let head = line_content_end(rope, rope.byte_to_line(sel.head));
    with_head(sel, head, extend)
}

/// Toggle between the first non-whitespace column and column 0.
pub fn smart_home(rope: &Rope, sel: Selection, extend: bool) -> Selection {
    let line_idx = rope.byte_to_line(sel.head);
    let start = rope.line_to_byte(line_idx);
    let s = line_str(rope, line_idx);
    let content = s.strip_suffix('\n').unwrap_or(s.as_ref());
    let first_non_ws = content
        .char_indices()
        .find(|(_, c)| !c.is_whitespace())
        .map(|(i, _)| start + i)
        .unwrap_or(start);
    let head = if sel.head == first_non_ws {
        start
    } else {
        first_non_ws
    };
    with_head(sel, head, extend)
}

fn vertical(
    rope: &Rope,
    sel: Selection,
    extend: bool,
    delta_lines: isize,
    goal: &mut GoalColumn,
) -> Selection {
    let line = rope.byte_to_line(sel.head) as isize;
    let col = match goal.col {
        Some(col) => col,
        None => {
            let col = grapheme_col(rope, sel.head);
            goal.col = Some(col);
            col
        }
    };
    let target = line + delta_lines;
    let head = if target < 0 {
        0
    } else if target as usize >= rope.len_lines() {
        rope.len_bytes()
    } else {
        pos_at_grapheme_col(rope, target as usize, col)
    };
    with_head(sel, head, extend)
}

/// Move one line up, keeping the goal column.
pub fn move_up(rope: &Rope, sel: Selection, extend: bool, goal: &mut GoalColumn) -> Selection {
    vertical(rope, sel, extend, -1, goal)
}

/// Move one line down, keeping the goal column.
pub fn move_down(rope: &Rope, sel: Selection, extend: bool, goal: &mut GoalColumn) -> Selection {
    vertical(rope, sel, extend, 1, goal)
}

/// Move `lines` up, keeping the goal column.
pub fn page_up(
    rope: &Rope,
    sel: Selection,
    extend: bool,
    lines: usize,
    goal: &mut GoalColumn,
) -> Selection {
    vertical(rope, sel, extend, -(lines as isize), goal)
}

/// Move `lines` down, keeping the goal column.
pub fn page_down(
    rope: &Rope,
    sel: Selection,
    extend: bool,
    lines: usize,
    goal: &mut GoalColumn,
) -> Selection {
    vertical(rope, sel, extend, lines as isize, goal)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FAMILY: &str = "👨‍👩‍👧‍👦"; // 4 emoji + 3 ZWJ = 25 bytes, one grapheme

    fn rope(s: &str) -> Rope {
        Rope::from_str(s)
    }

    fn caret_at(rope: &Rope, sel: Selection) -> usize {
        let _ = rope;
        assert!(sel.is_caret());
        sel.head
    }

    #[test]
    fn left_right_ascii() {
        let r = rope("ab");
        assert_eq!(
            move_right(&r, Selection::caret(0), false),
            Selection::caret(1)
        );
        assert_eq!(
            move_right(&r, Selection::caret(2), false),
            Selection::caret(2)
        );
        assert_eq!(
            move_left(&r, Selection::caret(1), false),
            Selection::caret(0)
        );
        assert_eq!(
            move_left(&r, Selection::caret(0), false),
            Selection::caret(0)
        );
    }

    #[test]
    fn right_steps_multibyte_graphemes() {
        let r = rope("ção");
        let mut pos = 0;
        let mut seen = vec![0];
        loop {
            let next = caret_at(&r, move_right(&r, Selection::caret(pos), false));
            if next == pos {
                break;
            }
            seen.push(next);
            pos = next;
        }
        assert_eq!(seen, vec![0, 2, 4, 5]);
    }

    #[test]
    fn right_steps_cjk() {
        let r = rope("日本語");
        assert_eq!(
            move_right(&r, Selection::caret(0), false),
            Selection::caret(3)
        );
        assert_eq!(
            move_right(&r, Selection::caret(3), false),
            Selection::caret(6)
        );
        assert_eq!(
            move_left(&r, Selection::caret(6), false),
            Selection::caret(3)
        );
    }

    #[test]
    fn zwj_emoji_is_one_step() {
        let text = format!("a{FAMILY}b");
        let r = rope(&text);
        assert_eq!(FAMILY.len(), 25);
        assert_eq!(
            move_right(&r, Selection::caret(1), false),
            Selection::caret(26)
        );
        assert_eq!(
            move_left(&r, Selection::caret(26), false),
            Selection::caret(1)
        );
    }

    #[test]
    fn left_right_cross_newline() {
        let r = rope("a\nb");
        assert_eq!(
            move_right(&r, Selection::caret(1), false),
            Selection::caret(2)
        );
        assert_eq!(
            move_left(&r, Selection::caret(2), false),
            Selection::caret(1)
        );
    }

    #[test]
    fn collapse_without_extend() {
        let r = rope("hello");
        let sel = Selection::new(1, 4);
        assert_eq!(move_left(&r, sel, false), Selection::caret(1));
        assert_eq!(move_right(&r, sel, false), Selection::caret(4));
    }

    #[test]
    fn extend_moves_head_keeps_anchor() {
        let r = rope("hello");
        let sel = Selection::caret(2);
        let extended = move_right(&r, sel, true);
        assert_eq!(extended, Selection::new(2, 3));
        let extended = move_left(&r, extended, true);
        assert_eq!(extended, Selection::new(2, 2));
    }

    #[test]
    fn word_right_and_left_ascii() {
        let r = rope("foo bar_baz qux");
        assert_eq!(
            move_word_right(&r, Selection::caret(0), false),
            Selection::caret(3)
        );
        assert_eq!(
            move_word_right(&r, Selection::caret(3), false),
            Selection::caret(11)
        );
        assert_eq!(
            move_word_left(&r, Selection::caret(11), false),
            Selection::caret(4)
        );
        assert_eq!(
            move_word_left(&r, Selection::caret(4), false),
            Selection::caret(0)
        );
    }

    #[test]
    fn word_right_multibyte() {
        let r = rope("ção mundo");
        assert_eq!(
            move_word_right(&r, Selection::caret(0), false),
            Selection::caret(5)
        );
        assert_eq!(
            move_word_right(&r, Selection::caret(5), false),
            Selection::caret(11)
        );
        assert_eq!(
            move_word_left(&r, Selection::caret(11), false),
            Selection::caret(6)
        );
        assert_eq!(
            move_word_left(&r, Selection::caret(6), false),
            Selection::caret(0)
        );
    }

    #[test]
    fn word_movement_crosses_lines() {
        let r = rope("foo\n  bar");
        assert_eq!(
            move_word_right(&r, Selection::caret(3), false),
            Selection::caret(9)
        );
        assert_eq!(
            move_word_left(&r, Selection::caret(6), false),
            Selection::caret(0)
        );
    }

    #[test]
    fn line_start_end_basic() {
        let r = rope("ab\ncdef");
        assert_eq!(
            line_start(&r, Selection::caret(5), false),
            Selection::caret(3)
        );
        assert_eq!(
            line_end(&r, Selection::caret(5), false),
            Selection::caret(7)
        );
        assert_eq!(
            line_end(&r, Selection::caret(0), false),
            Selection::caret(2)
        );
    }

    #[test]
    fn smart_home_toggles() {
        let r = rope("  héllo");
        assert_eq!(
            smart_home(&r, Selection::caret(7), false),
            Selection::caret(2)
        );
        assert_eq!(
            smart_home(&r, Selection::caret(2), false),
            Selection::caret(0)
        );
        assert_eq!(
            smart_home(&r, Selection::caret(0), false),
            Selection::caret(2)
        );
    }

    #[test]
    fn goal_column_through_short_line() {
        let r = rope("abcdef\nxy\nabcdef");
        let mut goal = GoalColumn::new();
        let sel = Selection::caret(4);
        let down1 = move_down(&r, sel, false, &mut goal);
        assert_eq!(down1, Selection::caret(9)); // clamped to end of "xy"
        let down2 = move_down(&r, down1, false, &mut goal);
        assert_eq!(down2, Selection::caret(14)); // back to column 4
        let up1 = move_up(&r, down2, false, &mut goal);
        assert_eq!(up1, Selection::caret(9));
        let up2 = move_up(&r, up1, false, &mut goal);
        assert_eq!(up2, Selection::caret(4));
    }

    #[test]
    fn goal_column_counts_graphemes() {
        let text = format!("a{FAMILY}b\nx\nc{FAMILY}d");
        let r = rope(&text);
        // Caret after the emoji on line 0 (column 2, byte 26).
        let mut goal = GoalColumn::new();
        let down1 = move_down(&r, Selection::caret(26), false, &mut goal);
        assert_eq!(down1, Selection::caret(29)); // end of "x"
        let down2 = move_down(&r, down1, false, &mut goal);
        assert_eq!(down2, Selection::caret(56)); // column 2 again, before 'd'
    }

    #[test]
    fn vertical_clamps_at_document_edges() {
        let r = rope("ab\ncd");
        let mut goal = GoalColumn::new();
        assert_eq!(
            move_up(&r, Selection::caret(1), false, &mut goal),
            Selection::caret(0)
        );
        goal.reset();
        assert_eq!(
            move_down(&r, Selection::caret(4), false, &mut goal),
            Selection::caret(5)
        );
    }

    #[test]
    fn page_up_down_keep_goal_column() {
        let r = rope("aaaa\nb\ncccc\nd\neeee");
        let mut goal = GoalColumn::new();
        let sel = Selection::caret(3); // line 0, col 3
        let down = page_down(&r, sel, false, 2, &mut goal);
        assert_eq!(down, Selection::caret(10)); // line 2, col 3
        let down = page_down(&r, down, false, 2, &mut goal);
        assert_eq!(down, Selection::caret(17)); // line 4, col 3
        let up = page_up(&r, down, false, 4, &mut goal);
        assert_eq!(up, Selection::caret(3));
    }

    #[test]
    fn grapheme_boundaries_at_line_edges() {
        let r = rope("ab\ncd");
        assert_eq!(prev_grapheme_boundary(&r, 3), 2); // start of line 1 -> before '\n'
        assert_eq!(next_grapheme_boundary(&r, 2), 3); // over the '\n'
        assert_eq!(next_grapheme_boundary(&r, 5), 5);
        assert_eq!(prev_grapheme_boundary(&r, 0), 0);
    }
}
