//! Textarea: the kit's Field with `textarea` (`components/Field/
//! index.tsx:55`), text over several lines in the Field's box, three
//! lines tall at rest (`h-20 py-4`), with the Field's label above and
//! message line under it. It can also grow with its text up to a number of
//! lines, as the kit's prompt and comment boxes do (`react-textarea-
//! autosize`, `maxRows={5}`), and scrolls past that. Every value is
//! `theme.form`'s: the box is the input recipe, `theme.form.area` its
//! vertical padding and rows.
//!
//! The editing core is the text field's (`area.input`, never drawn as its
//! box), multiline: typing, the clipboard (a paste keeps its line
//! breaks), the input method and Backspace reach it as they reach any
//! field. What is a text area's own is the geometry: the text wraps to
//! the box's width ([`lines`]), and Left and Right cross the lines, Home
//! and End go to a line's ends, Up and Down keep the column
//! ([`TextArea::edit`], [`TextArea::vertical`]), Enter breaks the line.
//!
//! Projected ([`TextArea::project`]): the text field, named by the label,
//! described by the message, `invalid` with an error, `multiline`.

mod lines;
mod render;

use std::cell::{Cell, RefCell};

use accord::{SemId, SnapshotBuilder};

use crate::drawing::text::TextStyle;
use crate::styling::theme::Theme;
use crate::widgets::form::field::Message;
use crate::widgets::form::label::Label;
use crate::widgets::form::text_field::{EditKey, TextField};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

use lines::Lines;

/// Where the parts of a text area go.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AreaLayout {
    pub label: Option<Rect>,
    /// The box.
    pub input: Rect,
    /// The text's box inside it: the lines wrap to its width, and the
    /// ones scrolled out of it are clipped.
    pub text: Rect,
    pub message: Option<Rect>,
}

impl AreaLayout {
    pub fn bottom(&self) -> f32 {
        self.message.map_or(self.input.bottom(), |m| m.bottom())
    }
}

/// A multi-line text field.
#[derive(Clone, Debug)]
pub struct TextArea {
    pub input: TextField,
    label: Option<Label>,
    message: Option<Message>,
    /// The most lines it grows to; `None`, the kit's box of
    /// `theme.form.area.rows` lines.
    grows: Option<usize>,
    hovered: bool,
    /// How far the lines are scrolled up, px: kept by the draw so the
    /// caret stays in the box, and by the wheel.
    scroll: Cell<f32>,
    /// The column Up and Down keep, with the offset it was taken at.
    goal: Option<(usize, f32)>,
    /// The style and the width the last draw or press laid the lines out
    /// in: the keys move over the lines on screen.
    laid: RefCell<Option<(TextStyle, f32)>>,
    sem: Option<SemId>,
}

impl TextArea {
    pub fn new(placeholder: impl AsRef<str>) -> Self {
        let mut input = TextField::new(placeholder);
        input.input.multiline = true;
        Self {
            input,
            label: None,
            message: None,
            grows: None,
            hovered: false,
            scroll: Cell::new(0.0),
            goal: None,
            laid: RefCell::new(None),
            sem: None,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(Label::new(label).disabled(self.input.disabled));
        self
    }

    pub fn error(mut self, error: impl Into<String>) -> Self {
        self.message = Some(Message::Error(error.into()));
        self
    }

    pub fn helper(mut self, helper: impl Into<String>) -> Self {
        self.message = Some(Message::Helper(helper.into()));
        self
    }

    /// Grow with the text from the box's rows up to `max` lines.
    pub fn grows(mut self, max: usize) -> Self {
        self.grows = Some(max.max(1));
        self
    }

    pub fn with_text(mut self, text: &str) -> Self {
        self.input.set_text(text);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.input.disabled = disabled;
        if let Some(label) = &mut self.label {
            label.set_disabled(disabled);
        }
        self
    }

    /// The id the box binds to and projects as.
    pub fn sem(mut self, id: SemId) -> Self {
        self.input = self.input.sem(id.clone());
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    pub fn message(&self) -> Option<&Message> {
        self.message.as_ref()
    }

    pub fn is_hovered(&self) -> bool {
        self.hovered
    }

    pub(crate) fn style(&self, theme: &Theme) -> TextStyle {
        self.input.style(theme)
    }

    /// The text as laid out: the field's, the composition in it while
    /// composing.
    pub(crate) fn shown(&self) -> String {
        self.input.input.shown()
    }

    /// The caret's offset in [`shown`](Self::shown): the field's, or the
    /// end of the composition.
    pub(crate) fn caret_pos(&self) -> usize {
        match self.input.input.composed() {
            Some((_, span)) => span.end,
            None => self.input.input.buffer.cursor(),
        }
    }

    /// The box's height for `lines` lines of text.
    fn box_height(&self, lines: usize, theme: &Theme) -> f32 {
        let a = &theme.form.area;
        let rows = match self.grows {
            Some(max) => lines.clamp(a.rows.min(max), max),
            None => a.rows,
        };
        a.pad_y * 2.0 + rows as f32 * self.style(theme).line_height
    }

    /// Where every part goes in `bounds` (its height is not read: the
    /// area takes [`height_for`](Self::height_for) of the width).
    pub fn layout(&self, bounds: Rect, theme: &Theme) -> AreaLayout {
        let f = &theme.form.field;
        let mut y = bounds.y;
        let label = self.label.as_ref().map(|l| {
            let (w, h) = l.size(Some(bounds.w), theme);
            let r = Rect::new(bounds.x, y, w, h);
            y = r.bottom() + f.label_gap;
            r
        });
        let pad_x = theme.form.input.pad_x;
        let text_w = (bounds.w - pad_x * 2.0).max(0.0);
        let style = self.style(theme);
        let shown = self.shown();
        let count = Lines::new(&shown, &style, text_w).count();
        let input = Rect::new(bounds.x, y, bounds.w, self.box_height(count, theme));
        let pad_y = theme.form.area.pad_y;
        let text = Rect::new(
            input.x + pad_x,
            input.y + pad_y,
            text_w,
            (input.h - pad_y * 2.0).max(0.0),
        );
        let message = self.message.as_ref().map(|m| {
            let (w, h) = crate::drawing::text::TextMeasurer::measure_styled(
                m.text(),
                &f.message,
                Some(bounds.w),
            );
            Rect::new(
                bounds.x,
                input.bottom() + f.message_gap,
                w.ceil().min(bounds.w),
                h,
            )
        });
        AreaLayout {
            label,
            input,
            text,
            message,
        }
    }

    pub fn height_for(&self, w: f32, theme: &Theme) -> f32 {
        self.layout(Rect::new(0.0, 0.0, w, 0.0), theme).bottom()
    }

    /// How far the lines can scroll in a text box `view` tall.
    fn max_scroll(&self, lines: usize, view: f32, theme: &Theme) -> f32 {
        (lines as f32 * self.style(theme).line_height - view).max(0.0)
    }

    /// The scroll that shows the caret: the last one, moved as little as
    /// it takes. The draw keeps it.
    pub(crate) fn follow(&self, caret_line: usize, lines: usize, view: f32, theme: &Theme) -> f32 {
        let lh = self.style(theme).line_height;
        let top = caret_line as f32 * lh;
        let mut s = self.scroll.get();
        if self.input.is_focused() {
            s = s.clamp((top + lh - view).min(top), top);
        }
        let s = s.clamp(0.0, self.max_scroll(lines, view, theme));
        self.scroll.set(s);
        s
    }

    /// Keeps the style and the text box's width the lines are laid out
    /// in, for the keys.
    pub(crate) fn remember(&self, text: Rect, theme: &Theme) {
        *self.laid.borrow_mut() = Some((self.style(theme), text.w));
    }

    /// The text's lines as last laid out (one unwrapped line per
    /// paragraph before the first draw).
    fn with_lines<R>(&self, f: impl FnOnce(&Lines<'_>) -> R) -> R {
        let laid = self.laid.borrow();
        let (style, w) = laid
            .clone()
            .unwrap_or_else(|| (TextStyle::default(), f32::MAX / 4.0));
        let text = self.input.text();
        f(&Lines::new(text, &style, w))
    }

    /// The offset at `(x, y)` inside the text box `text`.
    fn hit(&self, x: f32, y: f32, text: Rect, theme: &Theme) -> usize {
        let style = self.style(theme);
        let shown = self.input.text().to_owned();
        let lines = Lines::new(&shown, &style, text.w);
        let row = ((y - text.y + self.scroll.get()) / style.line_height).floor();
        lines.hit(x - text.x, row.max(0.0) as usize)
    }

    /// The pointer: hover on the box; a press in it focuses it and puts
    /// the caret at the press, a press anywhere else blurs it; the wheel
    /// over it scrolls its lines while they overflow.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        let l = self.layout(bounds, theme);
        self.remember(l.text, theme);
        if self.input.disabled {
            let was = std::mem::take(&mut self.hovered) || self.input.is_focused();
            self.input.unfocus();
            return if was {
                EventResult::changed()
            } else {
                EventResult::IGNORED
            };
        }
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let over = l.input.contains(x, y);
                if over == self.hovered {
                    return EventResult::IGNORED;
                }
                self.hovered = over;
                EventResult::changed()
            }
            WidgetEvent::MouseDown { x, y } if l.input.contains(x, y) => {
                self.input.focus();
                let pos = self.hit(x, y, l.text, theme);
                self.input.input.buffer.set_cursor(pos);
                self.input.input.reset_blink();
                self.goal = None;
                EventResult::clicked()
            }
            WidgetEvent::MouseDown { .. } if self.input.is_focused() => {
                self.input.unfocus();
                EventResult::changed()
            }
            WidgetEvent::Scroll { x, y, delta } if l.input.contains(x, y) => {
                let style = self.style(theme);
                let shown = self.shown();
                let count = Lines::new(&shown, &style, l.text.w).count();
                let max = self.max_scroll(count, l.text.h, theme);
                let s = (self.scroll.get() + delta).clamp(0.0, max);
                if s == self.scroll.get() {
                    return EventResult::IGNORED;
                }
                self.scroll.set(s);
                EventResult::changed()
            }
            _ => EventResult::IGNORED,
        }
    }

    /// The caret to `pos`, the selection extended to it with `extend`.
    fn go(&mut self, pos: usize, extend: bool) {
        let buffer = &mut self.input.input.buffer;
        if extend {
            buffer.select_to(pos);
        } else {
            buffer.set_cursor(pos);
        }
        self.input.input.reset_blink();
    }

    /// An editing key over the lines as last laid out: Left and Right
    /// cross the lines, Home and End go to the ends of the caret's line;
    /// the others are the field's. `false` when unfocused or disabled.
    pub fn edit(&mut self, key: EditKey) -> bool {
        if !self.input.is_focused() || self.input.disabled {
            return false;
        }
        let pos = self.input.input.buffer.cursor();
        let selection = self.input.input.buffer.selection();
        let (to, extend) = self
            .with_lines(|lines| {
                let line = lines.lines[lines.line_of(pos)];
                Some(match key {
                    // A selection collapses to its edge first, as a field's.
                    EditKey::Left | EditKey::Right if selection.is_some() => {
                        let (a, b) = selection.unwrap_or((pos, pos));
                        let edge = if key == EditKey::Left {
                            a.min(b)
                        } else {
                            a.max(b)
                        };
                        (edge, false)
                    }
                    EditKey::Left => (lines.step(pos, false), false),
                    EditKey::Right => (lines.step(pos, true), false),
                    EditKey::SelectLeft => (lines.step(pos, false), true),
                    EditKey::SelectRight => (lines.step(pos, true), true),
                    EditKey::Home => (line.start, false),
                    EditKey::End => (line.end, false),
                    EditKey::SelectHome => (line.start, true),
                    EditKey::SelectEnd => (line.end, true),
                    _ => return None,
                })
            })
            .map_or((None, false), |(to, extend)| (Some(to), extend));
        self.goal = None;
        match to {
            Some(to) => {
                self.go(to, extend);
                true
            }
            None => self.input.edit(key),
        }
    }

    /// Up (`down` false) or Down a line, keeping the column the first of
    /// these moves started at; past the first or last line, to the
    /// text's start or end. `false` when unfocused or disabled.
    pub fn vertical(&mut self, down: bool, extend: bool) -> bool {
        if !self.input.is_focused() || self.input.disabled {
            return false;
        }
        let pos = self.input.input.buffer.cursor();
        let end = self.input.text().len();
        let goal = self.goal;
        let (to, x) = self.with_lines(|lines| {
            let (x, i) = lines.caret(pos);
            let x = match goal {
                Some((at, x)) if at == pos => x,
                _ => x,
            };
            let to = match (down, i) {
                (false, 0) => 0,
                (true, i) if i + 1 >= lines.count() => end,
                (false, i) => lines.hit(x, i - 1),
                (true, i) => lines.hit(x, i + 1),
            };
            (to, x)
        });
        self.go(to, extend);
        self.goal = Some((to, x));
        true
    }

    /// Enter: a line break at the caret. `false` when unfocused or
    /// disabled.
    pub fn newline(&mut self) -> bool {
        self.goal = None;
        self.input.insert("\n")
    }

    /// Declares the area as `id` (the id given to [`TextArea::sem`]): the
    /// text field named by the label (the placeholder without one),
    /// described by the message, `invalid` with an error, `multiline`.
    #[track_caller]
    pub fn project(&self, b: &mut SnapshotBuilder, id: &SemId, parent: Option<&SemId>) {
        let name = self.label.as_ref().map_or_else(
            || self.input.input.placeholder.clone(),
            |l| l.text().to_owned(),
        );
        let mut node = self.input.project(b, id, name, parent);
        node.state("invalid", matches!(self.message, Some(Message::Error(_))))
            .state("multiline", true);
        if let Some(message) = &self.message {
            node.description(message.text());
        }
    }
}
