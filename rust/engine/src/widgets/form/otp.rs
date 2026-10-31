//! Input OTP: a one-time code typed one character per cell, the cells in
//! groups with a minus between them (`3 - 3`). The kit has no code input;
//! every value is derived from its Field (`theme.form`): each cell is the
//! Field's input box made square, its surface at rest, hovered and
//! focused, its text and marks, and the spaces between cells are
//! `theme.form.otp`. A label above and a message line under it are the
//! Field's ([`Label`], [`Message`]).
//!
//! The value lives in one [`TextField`] (`otp.input`), as a code input of
//! the web keeps one hidden input under its slots: typing, the editing
//! keys, a paste and an input method reach it as they reach any field,
//! and its [`Accept`] takes only the code's characters, as many as it
//! has cells. The cells draw that field's text; the cell of the caret
//! is the active one, with the focus ring, and an empty active cell
//! blinks the caret. A press on a filled cell selects its character, so
//! what is typed next replaces it.
//!
//! The width is the `Rect`'s: the cells keep the box's height while the
//! row fits and shrink with their spaces, all by one factor, where it
//! does not, never below a line of text and the border; narrower than
//! that, each group takes a row of its own ([`InputOtp::min_width`] is a
//! group at the least). The text never shrinks.
//!
//! Projected ([`InputOtp::project`]): the text field, named by the label,
//! described by the message, `invalid` with an error, its `max_len` the
//! cells, and secret: a one-time code is a credential, so the snapshot
//! keeps its length only, as it does a password's.

use accord::{Constraints, SemId, SnapshotBuilder};

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::theme::Theme;
use crate::widgets::form::field::Message;
use crate::widgets::form::label::Label;
use crate::widgets::form::text_field::TextField;
use crate::widgets::form::text_field::render::draw_box;
use crate::widgets::form::text_input::{Accept, Chars};
use crate::widgets::icons;
use crate::widgets::recipe::{focus_ring, rounded_rect};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent, with_alpha};

/// Where the parts of a code go.
#[derive(Clone, Debug, PartialEq)]
pub struct OtpLayout {
    pub label: Option<Rect>,
    /// One per character, start to end.
    pub cells: Vec<Rect>,
    /// The minus between two groups.
    pub separators: Vec<Rect>,
    pub message: Option<Rect>,
}

impl OtpLayout {
    /// The box the cells take together, the gaps between them included:
    /// what a press focuses and what the code binds.
    pub fn row(&self) -> Option<Rect> {
        let first = *self.cells.first()?;
        Some(self.cells.iter().fold(first, |u, r| {
            let (x, y) = (u.x.min(r.x), u.y.min(r.y));
            Rect::new(
                x,
                y,
                u.right().max(r.right()) - x,
                u.bottom().max(r.bottom()) - y,
            )
        }))
    }

    pub fn bottom(&self) -> f32 {
        let cells = self.cells.iter().fold(0.0_f32, |b, r| b.max(r.bottom()));
        self.message.map_or(cells, |m| m.bottom())
    }
}

/// A one-time code input.
#[derive(Clone, Debug)]
pub struct InputOtp {
    /// The value, its focus, caret, selection and composition.
    pub input: TextField,
    /// The cells of each group, start to end.
    groups: Vec<usize>,
    label: Option<Label>,
    message: Option<Message>,
    /// The cell under the pointer.
    hovered: Option<usize>,
    sem: Option<SemId>,
}

impl InputOtp {
    /// A code of `groups` cells of digits, `[3, 3]` for six.
    pub fn new(groups: &[usize]) -> Self {
        let groups: Vec<usize> = groups.iter().copied().filter(|&n| n > 0).collect();
        let n = groups.iter().sum();
        Self {
            input: TextField::new("").accept(Accept::digits(n)),
            groups,
            label: None,
            message: None,
            hovered: None,
            sem: None,
        }
    }

    /// Letters as well as digits (a recovery code).
    pub fn alphanumeric(mut self) -> Self {
        self.input.input.accept.chars = Chars::Alphanumeric;
        self
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

    /// A bullet in each filled cell in place of its character.
    pub fn masked(mut self, masked: bool) -> Self {
        self.input.set_masked(masked);
        self
    }

    /// The code typed so far; what does not fit the cells, or is not the
    /// code's, is dropped ([`TextField::set_value`] refuses it whole).
    pub fn with_text(mut self, text: &str) -> Self {
        let a = self.input.input.accept;
        self.input.set_text(&a.admit(text, 0, 0));
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.input.disabled = disabled;
        if let Some(label) = &mut self.label {
            label.set_disabled(disabled);
        }
        self
    }

    /// The id the code binds to and projects as.
    pub fn sem(mut self, id: SemId) -> Self {
        self.input = self.input.sem(id.clone());
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    /// How many cells the code has.
    pub fn cells(&self) -> usize {
        self.groups.iter().sum()
    }

    /// Every cell is filled.
    pub fn is_complete(&self) -> bool {
        self.input.text().chars().count() == self.cells()
    }

    pub fn message(&self) -> Option<&Message> {
        self.message.as_ref()
    }

    /// The width of `groups` side by side with no shrinking: the cells as
    /// tall as the box and as wide, their gaps, and a minus with a gap
    /// either side of it between two groups.
    fn span(groups: &[usize], theme: &Theme) -> f32 {
        let (cell, o) = (TextField::height(theme), &theme.form.otp);
        let n = groups.iter().sum::<usize>() as f32;
        let joins = groups.len().saturating_sub(1) as f32;
        n * cell + (n - 1.0 - joins).max(0.0) * o.gap + joins * (o.separator + o.gap * 2.0)
    }

    /// The whole code on one row, unshrunk.
    pub fn natural_width(&self, theme: &Theme) -> f32 {
        Self::span(&self.groups, theme)
    }

    /// The least a cell shrinks to, as a factor of the box: one line of
    /// text and its two borders.
    fn least(theme: &Theme) -> f32 {
        let r = &theme.form.input;
        ((r.text.line_height + r.border_width * 2.0) / TextField::height(theme)).min(1.0)
    }

    /// The narrowest the code lays out in without leaving its width: its
    /// widest group at the least scale.
    pub fn min_width(&self, theme: &Theme) -> f32 {
        let widest = self
            .groups
            .iter()
            .map(|g| Self::span(std::slice::from_ref(g), theme))
            .fold(0.0, f32::max);
        widest * Self::least(theme)
    }

    /// How the code lays out in `w`: on one row while it fits at the least
    /// scale, otherwise a group per row; and the factor the cells and
    /// their spaces shrink by (1 while the row fits).
    fn fit(&self, w: f32, theme: &Theme) -> (bool, f32) {
        let least = Self::least(theme);
        let one_row = self.natural_width(theme);
        let stacked = self.groups.len() > 1 && one_row * least > w;
        let widest = if stacked {
            self.min_width(theme) / least
        } else {
            one_row
        };
        if widest <= 0.0 {
            return (stacked, 1.0);
        }
        (stacked, (w / widest).clamp(least, 1.0))
    }

    /// Where every part goes in `bounds` (its height is not read: the
    /// code takes [`height_for`](Self::height_for) of the width).
    pub fn layout(&self, bounds: Rect, theme: &Theme) -> OtpLayout {
        let f = &theme.form.field;
        let mut y = bounds.y;
        let label = self.label.as_ref().map(|l| {
            let (w, h) = l.size(Some(bounds.w), theme);
            let r = Rect::new(bounds.x, y, w, h);
            y = r.bottom() + f.label_gap;
            r
        });
        let (stacked, k) = self.fit(bounds.w, theme);
        let (cell, o) = (TextField::height(theme) * k, &theme.form.otp);
        let (gap, sep) = (o.gap * k, o.separator * k);
        let mut cells = Vec::with_capacity(self.cells());
        let mut separators = Vec::new();
        let mut x = bounds.x;
        for (g, &n) in self.groups.iter().enumerate() {
            if g > 0 && stacked {
                x = bounds.x;
                y += cell + gap;
            } else if g > 0 {
                x += gap;
                separators.push(Rect::new(x, y + (cell - sep) / 2.0, sep, sep));
                x += sep + gap;
            }
            for i in 0..n {
                if i > 0 {
                    x += gap;
                }
                cells.push(Rect::new(x, y, cell, cell));
                x += cell;
            }
        }
        let message = self.message.as_ref().map(|m| {
            let (w, h) = TextMeasurer::measure_styled(m.text(), &f.message, Some(bounds.w));
            Rect::new(
                bounds.x,
                y + cell + f.message_gap,
                w.ceil().min(bounds.w),
                h,
            )
        });
        OtpLayout {
            label,
            cells,
            separators,
            message,
        }
    }

    /// The code's height at width `w`.
    pub fn height_for(&self, w: f32, theme: &Theme) -> f32 {
        self.layout(Rect::new(0.0, 0.0, w, 0.0), theme).bottom()
    }

    /// The width the cells take in `w`: the natural width where it fits,
    /// less where they shrink or a group goes under the other.
    pub fn width_for(&self, w: f32, theme: &Theme) -> f32 {
        let l = self.layout(Rect::new(0.0, 0.0, w, 0.0), theme);
        l.cells.iter().fold(0.0, |r, c| r.max(c.right()))
    }

    /// The cell at `(x, y)`.
    pub fn cell_at(&self, x: f32, y: f32, bounds: Rect, theme: &Theme) -> Option<usize> {
        self.layout(bounds, theme)
            .cells
            .iter()
            .position(|r| r.contains(x, y))
    }

    /// The characters of the value as drawn (the composition in it, a
    /// bullet each when masked), one per cell.
    fn shown(&self) -> Vec<char> {
        self.input.input.shown().chars().collect()
    }

    /// The char index of the byte offset `at` of the value.
    fn index(&self, at: usize) -> usize {
        let text = self.input.text();
        text[..at.min(text.len())].chars().count()
    }

    /// The byte offset of the `i`th character, or the end.
    fn byte(&self, i: usize) -> usize {
        let text = self.input.text();
        text.char_indices().nth(i).map_or(text.len(), |(b, _)| b)
    }

    /// The cells the selection covers, as a range of indices.
    fn selected(&self) -> Option<std::ops::Range<usize>> {
        let (a, b) = self.input.input.buffer.selection()?;
        let (lo, hi) = (self.index(a.min(b)), self.index(a.max(b)));
        (lo < hi).then_some(lo..hi)
    }

    /// The cell the caret is in: the selection's first, or the caret's,
    /// the last cell when the code is full.
    pub fn active(&self) -> usize {
        let at = match self.selected() {
            Some(r) => r.start,
            None => self.index(self.input.input.buffer.cursor()),
        };
        at.min(self.cells().saturating_sub(1))
    }

    /// Whether `(x, y)` is on the code's row, a cell or a gap.
    pub fn on_row(&self, x: f32, y: f32, bounds: Rect, theme: &Theme) -> bool {
        self.layout(bounds, theme)
            .row()
            .is_some_and(|r| r.contains(x, y))
    }

    /// The pointer: hover by cell; a press on a filled cell focuses the
    /// code and selects its character, one on an empty cell or a gap of
    /// the row puts the caret after the last; a press anywhere else blurs
    /// it.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        if self.input.disabled {
            let was = self.hovered.take().is_some() || self.input.is_focused();
            self.input.unfocus();
            return if was {
                EventResult::changed()
            } else {
                EventResult::IGNORED
            };
        }
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let over = self.cell_at(x, y, bounds, theme);
                if over == self.hovered {
                    return EventResult::IGNORED;
                }
                self.hovered = over;
                EventResult::changed()
            }
            WidgetEvent::MouseDown { x, y } => match self.cell_at(x, y, bounds, theme) {
                None if self.on_row(x, y, bounds, theme) => {
                    self.input.focus();
                    self.input.input.buffer.move_end();
                    self.input.input.reset_blink();
                    EventResult::clicked()
                }
                Some(i) => {
                    self.input.focus();
                    let len = self.input.text().chars().count();
                    let (from, to) = (self.byte(i), self.byte(i + 1));
                    let buffer = &mut self.input.input.buffer;
                    if i < len {
                        buffer.set_cursor(from);
                        buffer.select_to(to);
                    } else {
                        buffer.move_end();
                    }
                    self.input.input.reset_blink();
                    EventResult::clicked()
                }
                None if self.input.is_focused() => {
                    self.input.unfocus();
                    EventResult::changed()
                }
                None => EventResult::IGNORED,
            },
            _ => EventResult::IGNORED,
        }
    }

    /// Where the input method's window goes: the active cell.
    pub fn ime_area(&self, bounds: Rect, theme: &Theme) -> Rect {
        let l = self.layout(bounds, theme);
        l.cells.get(self.active()).copied().unwrap_or(bounds)
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        self.render_to_layer(c, LayerId::DEFAULT, bounds, theme);
    }

    pub fn render_to_layer(&self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        let l = self.layout(bounds, theme);
        let form = &theme.form;
        let r = &form.input;
        let alpha = if self.input.disabled {
            r.disabled_alpha
        } else {
            1.0
        };
        let fade = |color: crate::styling::color::Color| with_alpha(color, color.0[3] * alpha);
        if let (Some(label), Some(at)) = (&self.label, l.label) {
            label.draw(c, layer, at.x, at.y, Some(at.w), theme);
        }
        if let (Some(id), Some(row)) = (&self.sem, l.row()) {
            c.bind_to_layer(layer, id, [row.x, row.y, row.w, row.h]);
        }
        let focused = self.input.is_focused();
        let active = self.active();
        let shown = self.shown();
        let selected = self.selected().filter(|_| focused);
        let composing = self.input.input.composed().map(|(text, span)| {
            text[..span.start].chars().count()..text[..span.end].chars().count()
        });
        let style = &r.text;
        for (i, cell) in l.cells.iter().enumerate() {
            let surface = if focused && i == active {
                &r.focus
            } else if self.hovered == Some(i) {
                &r.hover
            } else {
                &r.rest
            };
            if focused && i == active {
                let radius = r.radius.min(cell.h / 2.0);
                c.push_to_layer(layer, focus_ring(*cell, radius, theme));
            }
            draw_box(c, layer, *cell, r, surface, alpha);
            if selected.as_ref().is_some_and(|s| s.contains(&i)) {
                let wash = cell.inset(r.border_width);
                let radius = (r.radius - r.border_width).clamp(0.0, wash.h / 2.0);
                c.push_to_layer(
                    layer,
                    rounded_rect(wash.x, wash.y, wash.w, wash.h, radius, r.selection.0),
                );
            }
            let ch = shown.get(i).map(|ch| ch.to_string());
            let y = cell.y + TextMeasurer::vertical_center(style, cell.h);
            let mut caret_x = cell.x + cell.w / 2.0;
            if let Some(ch) = &ch {
                let (w, _) = TextMeasurer::measure_styled(ch, style, None);
                let x = cell.x + (cell.w - w) / 2.0;
                caret_x = x;
                c.push_to_layer(
                    layer,
                    SceneNode::Text {
                        key: TextNodeKey::from_style(ch, style, None),
                        x,
                        y,
                        color: fade(r.text_color),
                    },
                );
                if composing.as_ref().is_some_and(|s| s.contains(&i)) {
                    let under = theme.control.edge_width;
                    c.push_to_layer(
                        layer,
                        SceneNode::Rect {
                            x,
                            y: y + style.line_height - under,
                            w,
                            h: under,
                            color: r.composing.0,
                        },
                    );
                }
            }
            // The caret in the active cell, before its character, when
            // nothing is selected.
            if i == active && selected.is_none() && self.input.input.cursor_visible() {
                let past = self.index(self.input.input.buffer.cursor()) > i;
                if !past || composing.is_some() {
                    self.caret(c, layer, caret_x, y, style, theme);
                }
            }
        }
        let tint = fade(form.adorn.icon_color);
        for s in &l.separators {
            if let Some(node) = icons::icon_at("minus", s.w, tint, s.x, s.y) {
                c.push_to_layer(layer, node);
            }
        }
        if let (Some(m), Some(at)) = (&self.message, l.message) {
            let color = match m {
                Message::Error(_) => form.field.error_color,
                Message::Helper(_) => form.field.helper_color,
            };
            c.push_to_layer(
                layer,
                SceneNode::Text {
                    key: TextNodeKey::from_style(m.text(), &form.field.message, Some(at.w)),
                    x: at.x,
                    y: at.y,
                    color: fade(color),
                },
            );
        }
    }

    fn caret(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        x: f32,
        y: f32,
        style: &TextStyle,
        theme: &Theme,
    ) {
        let w = theme.control.focus_ring_width;
        c.push_to_layer(
            layer,
            SceneNode::Rect {
                x: x - w / 2.0,
                y,
                w,
                h: style.line_height,
                color: theme.form.input.caret.0,
            },
        );
    }

    /// Declares the code as `id` (the id given to [`InputOtp::sem`]): the
    /// text field named by the label (`label` without one), described by
    /// the message, `invalid` with an error, at most its cells long, and
    /// secret. It only reads the widget: the owner runs `set_value`
    /// ([`TextField::set_value`], which refuses what is not the code's).
    #[track_caller]
    pub fn project(
        &self,
        b: &mut SnapshotBuilder,
        id: &SemId,
        label: &str,
        parent: Option<&SemId>,
    ) {
        let name = self.label.as_ref().map_or(label, Label::text);
        let mut node = self.input.project(b, id, name, parent);
        node.state("invalid", matches!(self.message, Some(Message::Error(_))))
            .constraints(Constraints {
                editable: true,
                max_len: Some(self.cells()),
                secret: true,
                ..Constraints::default()
            });
        if let Some(message) = &self.message {
            node.description(message.text());
        }
    }
}
