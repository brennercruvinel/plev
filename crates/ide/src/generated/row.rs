//! Where the generated column sits in the window, what a person's
//! pointer does on its line, and how it is drawn.
//!
//! The column is at the right of the window, from the top to the bottom:
//! a card's width (`size.card_w`) while no surface is open, so the line
//! is always there to write an intent; half the window with a surface, as
//! the Silences gives its surface half the page. The workspace takes the
//! rest, through its own `resize`. Inside the column: the line on top (the
//! field, the proposer, propose and close on one line while the field
//! keeps a card's minimum width, else under it; then the status), and the
//! surface under the line, drawn at its natural height and clipped to the
//! column, so what shows without scrolling is what the frame binds.

use comps::prelude::{EventResult, Rect, TextField, WidgetEvent};
use engine::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::text::{TextMeasurer, TextStyle};
use engine::theme::{IconSize, Theme};

use super::{Generated, Said};

/// The column and what goes in it.
pub(crate) struct Column {
    pub rect: Rect,
    pub row: Row,
    /// Where the surface is drawn and clipped: under the line, to the
    /// column's bottom.
    pub surface: Rect,
}

/// The line: its band, the field, the proposer, the two buttons and the
/// status message.
pub(crate) struct Row {
    pub band: Rect,
    pub intent: Rect,
    pub proposer: Rect,
    pub propose: Rect,
    pub close: Rect,
    pub message: Rect,
}

/// What a person's pointer on the line asked.
#[derive(Default)]
pub(crate) struct Asked {
    pub propose: bool,
    pub close: bool,
    /// The open options panel took the event: nothing under it sees it.
    pub taken: bool,
}

fn message_style(theme: &Theme) -> TextStyle {
    theme.typography.caption_sm()
}

fn bounds(r: Rect) -> [f32; 4] {
    [r.x, r.y, r.w, r.h]
}

impl Generated {
    /// How wide the column is in a window `vw` wide.
    pub fn column_w(&self, vw: f32, theme: &Theme) -> f32 {
        let card = theme.size.card_w;
        let w = if self.open.is_some() {
            ((vw - theme.spacing.xl) / 2.0).max(card)
        } else {
            card
        };
        w.min(vw).max(0.0)
    }

    /// The column in a window `vw` by `vh`.
    pub fn column(&self, vw: f32, vh: f32, theme: &Theme) -> Column {
        let w = self.column_w(vw, theme);
        let rect = Rect::new(vw - w, 0.0, w, vh);
        let pad = theme.spacing.lg;
        let inner = (w - pad * 2.0).max(0.0);
        let row = self.row(rect.x + pad, pad, inner, theme);
        let top = row.band.bottom();
        let surface = Rect::new(rect.x + pad, top, inner, (vh - top - pad).max(0.0));
        Column { rect, row, surface }
    }

    /// The width the proposer takes: its widest option, the paddings and
    /// the chevron, as the select draws them.
    fn proposer_w(&self, theme: &Theme) -> f32 {
        let style = theme.typography.base_2m();
        let text = self
            .proposer
            .options
            .iter()
            .map(|o| TextMeasurer::measure_styled(o, &style, None).0)
            .fold(0.0, f32::max);
        let chevron = theme.control.icon(IconSize::Sm);
        (text + theme.spacing.lg * 2.0 + theme.spacing.xs + chevron).ceil()
    }

    /// The line at `(x, y)`, `w` wide: the field, the proposer and the two
    /// buttons on one line while the field keeps a card's minimum width
    /// (an intent is a sentence), the three under the field otherwise;
    /// then the message, one line at least, so a message arriving moves
    /// nothing below it.
    fn row(&self, x: f32, y: f32, w: f32, theme: &Theme) -> Row {
        let gap = theme.spacing.sm;
        let h = TextField::height(theme);
        let (bw, bh) = self.propose.preferred_size(theme);
        let (cw, ch) = self.close.preferred_size(theme);
        let sw = self.proposer_w(theme).min(w);
        let controls = sw + gap + bw + gap + cw + gap;
        let (intent, left, line) = if w - controls >= theme.size.card_min_w {
            let field_w = w - controls;
            (Rect::new(x, y, field_w, h), x + field_w + gap, y)
        } else {
            (Rect::new(x, y, w, h), x, y + h + gap)
        };
        let proposer = Rect::new(left, line, sw, h);
        let propose = Rect::new(proposer.right() + gap, line + (h - bh) / 2.0, bw, bh);
        let close = Rect::new(propose.right() + gap, line + (h - ch) / 2.0, cw, ch);
        let below = line + h + gap;
        let style = message_style(theme);
        let one = TextMeasurer::measure_styled("Ag", &style, None).1;
        let wrapped = TextMeasurer::measure_styled(&self.message(), &style, Some(w)).1;
        let message = Rect::new(x, below, w, one.max(wrapped));
        let bottom = message.bottom() + gap;
        Row {
            band: Rect::new(x, y, w, bottom - y),
            intent,
            proposer,
            propose,
            close,
            message,
        }
    }

    /// A person's pointer on the line: the field takes focus (a press
    /// outside it blurs it), the proposer opens and chooses, the buttons
    /// are clicked. While the proposer's panel is open it sees the event
    /// alone.
    pub fn handle_row(
        &mut self,
        event: &WidgetEvent,
        row: &Row,
        theme: &Theme,
    ) -> (EventResult, Asked) {
        self.refresh();
        if self.proposer.is_open() {
            let panel = self.proposer.handle_event(event, row.proposer, theme);
            let asked = Asked {
                taken: true,
                ..Asked::default()
            };
            return (panel, asked);
        }
        let field = self.intent.handle_event(event, row.intent, theme);
        let select = self.proposer.handle_event(event, row.proposer, theme);
        let propose = self.propose.handle_event(event, row.propose);
        let close = self.close.handle_event(event, row.close);
        let asked = Asked {
            propose: propose.clicked,
            close: close.clicked,
            taken: false,
        };
        (field.merge(select).merge(propose).merge(close), asked)
    }

    /// The column: its ground and the hairline at its left edge, the
    /// line, the surface clipped to its place with what is left of a
    /// dismissed one fading there, and the proposer's panel over all.
    pub fn render_column(&mut self, c: &mut Compositor, col: &Column, theme: &Theme) {
        self.refresh();
        let r = col.rect;
        c.push(SceneNode::Rect {
            x: r.x,
            y: r.y,
            w: r.w,
            h: r.h,
            color: theme.colors.bg.0,
        });
        c.push(SceneNode::Rect {
            x: r.x,
            y: r.y,
            w: theme.control.edge_width,
            h: r.h,
            color: theme.colors.divider.0,
        });
        let row = &col.row;
        if let Some(ids) = &self.ids {
            c.bind(&ids.region, bounds(row.band));
            c.bind(&ids.status, bounds(row.message));
        }
        self.intent.render(c, row.intent, theme);
        self.proposer.render(c, row.proposer, theme);
        self.propose.render(c, row.propose, theme);
        self.close.render(c, row.close, theme);
        let text = self.message();
        if !text.is_empty() {
            let color = match &self.said {
                Some(Said::Refused(_)) => theme.colors.danger.0,
                _ => theme.glass.text_faint.0,
            };
            let style = message_style(theme);
            c.push(SceneNode::Text {
                key: TextNodeKey::from_style(&text, &style, Some(row.message.w)),
                x: row.message.x,
                y: row.message.y,
                color,
            });
        }
        let s = col.surface;
        c.push(SceneNode::PushClip {
            x: s.x,
            y: s.y,
            w: s.w,
            h: s.h,
        });
        for surface in self.host.iter_mut() {
            surface.render(c, s, theme);
        }
        self.host.draw_closing(c, theme);
        c.push(SceneNode::PopClip);
        if self.proposer.is_open() {
            self.proposer
                .render_dropdown(c, LayerId::DEFAULT, row.proposer, theme);
        }
    }
}
