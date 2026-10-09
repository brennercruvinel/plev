//! Where the line and the generated surface sit in the window, what a
//! person's pointer does on the line, and how both are drawn.
//!
//! The line is a strip along the bottom of the window, the whole width
//! (task-96, phase b): the field (a card wide at most, an intent is a
//! sentence), the proposer, propose and close on one line, and the
//! message beside them while the width leaves it a card's minimum, under
//! them otherwise; when the field would be narrower than a card's minimum,
//! the three go under it. The fixed views keep the whole width of the
//! window and give the strip only its height, through the workspace's own
//! `resize`. A surface has a column only while it is open: at the right,
//! over the strip, half the window, as the Silences gives its surface half
//! the page; the surface is drawn at its natural height, clipped to the
//! column, so what shows without scrolling is what the frame binds.
//!
//! The proposer is the comps `Select`, which lays its panel out under the
//! rect it is given; under the strip there is no window, so the line gives
//! it, for the panel, the control lifted by the panel and its two gaps:
//! the panel opens upward, over the workspace, and the control is drawn
//! and hit where it is.

use comps::prelude::{EventResult, Rect, TextField, WidgetEvent};
use engine::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::text::{TextMeasurer, TextStyle};
use engine::theme::{IconSize, Theme};

use super::{Generated, Said};

/// Where the line and the surface sit in the window.
pub(crate) struct Place {
    /// The strip of the line, along the bottom of the window.
    pub strip: Rect,
    pub row: Row,
    /// The column of the surface, at the right over the strip, while one
    /// is open.
    pub column: Option<Rect>,
    /// Where the surface is drawn and clipped: the column less its
    /// padding, while one is open.
    pub surface: Option<Rect>,
}

/// The line: its band, the field, the proposer, the two buttons, the
/// status message and, while the Claude is asked, the cancel at its end.
pub(crate) struct Row {
    pub band: Rect,
    pub intent: Rect,
    pub proposer: Rect,
    /// The rect the proposer's panel is laid out from, so it opens above
    /// the control.
    pub panel: Rect,
    pub propose: Rect,
    pub close: Rect,
    pub message: Rect,
    pub cancel: Option<Rect>,
}

/// What a person's pointer on the line asked.
#[derive(Default)]
pub(crate) struct Asked {
    pub propose: bool,
    pub close: bool,
    pub cancel: bool,
    /// The open options panel took the event: nothing under it sees it.
    pub taken: bool,
}

fn message_style(theme: &Theme) -> TextStyle {
    theme.typography.caption_sm()
}

fn bounds(r: Rect) -> [f32; 4] {
    [r.x, r.y, r.w, r.h]
}

/// A hairline of the divider's tone.
fn hairline(c: &mut Compositor, r: Rect, theme: &Theme) {
    c.push(SceneNode::Rect {
        x: r.x,
        y: r.y,
        w: r.w,
        h: r.h,
        color: theme.colors.divider.0,
    });
}

/// A ground of the page's tone.
fn ground(c: &mut Compositor, r: Rect, theme: &Theme) {
    c.push(SceneNode::Rect {
        x: r.x,
        y: r.y,
        w: r.w,
        h: r.h,
        color: theme.colors.bg.0,
    });
}

impl Generated {
    /// How wide the column of the surface is in a window `vw` wide: half
    /// the window (a card at least) while a surface is open, none
    /// otherwise.
    pub fn column_w(&self, vw: f32, theme: &Theme) -> f32 {
        if self.open.is_none() {
            return 0.0;
        }
        let w = ((vw - theme.spacing.xl) / 2.0).max(theme.size.card_w);
        w.min(vw).max(0.0)
    }

    /// The line and the surface in a window `vw` by `vh`.
    pub fn place(&self, vw: f32, vh: f32, theme: &Theme) -> Place {
        let (px, py) = (theme.spacing.lg, theme.spacing.sm);
        let inner = (vw - px * 2.0).max(0.0);
        // the band's height depends on the width only (and a message
        // longer than the lines it has), so it is measured first
        let band_h = self.row(px, 0.0, inner, theme).band.h;
        let top = (vh - band_h - py * 2.0).max(0.0);
        let strip = Rect::new(0.0, top, vw, vh - top);
        let row = self.row(px, top + py, inner, theme);
        let w = self.column_w(vw, theme);
        let column = (w > 0.0).then(|| Rect::new(vw - w, 0.0, w, top));
        let pad = theme.spacing.lg;
        let surface = column.map(|c| {
            let (sw, sh) = ((c.w - pad * 2.0).max(0.0), (c.h - pad * 2.0).max(0.0));
            Rect::new(c.x + pad, c.y + pad, sw, sh)
        });
        Place {
            strip,
            row,
            column,
            surface,
        }
    }

    /// What the workspace is given of a window `vw` by `vh`: the width
    /// less the column of an open surface, the height less the strip.
    pub fn workspace_size(&self, vw: f32, vh: f32, theme: &Theme) -> (f32, f32) {
        let place = self.place(vw, vh, theme);
        let w = place.column.map_or(vw, |c| c.x);
        (w.max(0.0), place.strip.y)
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

    /// The rect the proposer's panel is laid out from: the `Select` lays
    /// its panel out under the rect it is given (`Select::dropdown_rect`),
    /// and there is no window under the strip, so this is `control` lifted
    /// by the panel and the gap twice: the panel opens upward, a gap above
    /// the control.
    fn lifted(&self, control: Rect, theme: &Theme) -> Rect {
        let panel = self.proposer.dropdown_rect(control, theme);
        let gap = panel.y - control.bottom();
        let y = control.y - gap - panel.h - gap - control.h;
        Rect::new(control.x, y, control.w, control.h)
    }

    /// The line at `(x, y)`, `w` wide. The field, the proposer and the two
    /// buttons on one line, and the message beside them, while the field
    /// keeps a card's minimum width and the message has one too; the
    /// message under them when it would not; the three under the field
    /// when the field would not keep its minimum either. The message has
    /// one line at least, and as much as the cancel takes, so a message
    /// arriving or the Claude being asked moves nothing; the cancel at its
    /// end while the Claude is asked.
    fn row(&self, x: f32, y: f32, w: f32, theme: &Theme) -> Row {
        let gap = theme.spacing.sm;
        let h = TextField::height(theme);
        let (bw, bh) = self.propose.preferred_size(theme);
        let (cw, ch) = self.close.preferred_size(theme);
        let (kw, kh) = self.cancel.preferred_size(theme);
        let sw = self.proposer_w(theme).min(w);
        let controls = sw + gap + bw + gap + cw;
        let min = theme.size.card_min_w;
        let style = message_style(theme);
        let one = TextMeasurer::measure_styled("Ag", &style, None).1;
        let waiting = self.waiting();
        // the message in `mw`: the width of its text and its height
        let said = |mw: f32| {
            let text_w = if waiting {
                (mw - kw - gap).max(0.0)
            } else {
                mw
            };
            let wrapped = TextMeasurer::measure_styled(&self.message(), &style, Some(text_w)).1;
            (text_w, one.max(wrapped))
        };
        let beside = w - controls - gap * 2.0 >= min * 2.0;
        let under = w - controls - gap >= min;
        // the field; where the controls start, on which line and how tall
        // it is; where the message starts, on which line, and its width
        let (field_w, left, line, line_h, mx, my, mw) = if beside {
            let field_w = (w - controls - gap * 2.0 - min).min(theme.size.card_w);
            let mx = x + field_w + gap + controls + gap;
            let mw = x + w - mx;
            let line_h = h.max(said(mw).1).max(kh);
            (field_w, x + field_w + gap, y, line_h, mx, y, mw)
        } else if under {
            let field_w = w - controls - gap;
            (field_w, x + field_w + gap, y, h, x, y + h + gap, w)
        } else {
            (w, x, y + h + gap, h, x, y + (h + gap) * 2.0, w)
        };
        let (text_w, text_h) = said(mw);
        let msg_h = if beside { line_h } else { text_h.max(kh) };
        let intent = Rect::new(x, y + (line_h - h) / 2.0, field_w, h);
        let proposer = Rect::new(left, line + (line_h - h) / 2.0, sw, h);
        let propose = Rect::new(proposer.right() + gap, line + (line_h - bh) / 2.0, bw, bh);
        let close = Rect::new(propose.right() + gap, line + (line_h - ch) / 2.0, cw, ch);
        let message = Rect::new(mx, my + (msg_h - text_h) / 2.0, text_w, text_h);
        let cancel = waiting.then(|| Rect::new(mx + mw - kw, my + (msg_h - kh) / 2.0, kw, kh));
        let bottom = if beside { y + line_h } else { my + msg_h };
        Row {
            band: Rect::new(x, y, w, bottom - y),
            intent,
            proposer,
            panel: self.lifted(proposer, theme),
            propose,
            close,
            message,
            cancel,
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
            let panel = self.proposer.handle_event(event, row.panel, theme);
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
        let cancel = match row.cancel {
            Some(r) => self.cancel.handle_event(event, r),
            None => EventResult::IGNORED,
        };
        let asked = Asked {
            propose: propose.clicked,
            close: close.clicked,
            cancel: cancel.clicked,
            taken: false,
        };
        let all = field.merge(select).merge(propose).merge(close);
        (all.merge(cancel), asked)
    }

    /// The strip and, with a surface open, its column: their grounds and
    /// the hairlines where they meet the workspace, the line, the surface
    /// clipped to its place, what is left of a dismissed one fading where
    /// it was (above the strip), and the proposer's panel over all.
    pub fn render_place(&mut self, c: &mut Compositor, place: &Place, theme: &Theme) {
        self.refresh();
        let edge = theme.control.edge_width;
        let s = place.strip;
        ground(c, s, theme);
        hairline(c, Rect::new(s.x, s.y, s.w, edge), theme);
        if let Some(col) = place.column {
            ground(c, col, theme);
            hairline(c, Rect::new(col.x, col.y, edge, col.h), theme);
        }
        let row = &place.row;
        if let Some(ids) = &self.ids {
            c.bind(&ids.region, bounds(row.band));
            c.bind(&ids.status, bounds(row.message));
        }
        self.intent.render(c, row.intent, theme);
        self.proposer.render(c, row.proposer, theme);
        self.propose.render(c, row.propose, theme);
        self.close.render(c, row.close, theme);
        if let Some(r) = row.cancel {
            self.cancel.render(c, r, theme);
        }
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
        if let Some(area) = place.surface {
            clip(c, area);
            for surface in self.host.iter_mut() {
                surface.render(c, area, theme);
            }
            c.push(SceneNode::PopClip);
        }
        clip(c, Rect::new(0.0, 0.0, s.w, s.y));
        self.host.draw_closing(c, theme);
        c.push(SceneNode::PopClip);
        if self.proposer.is_open() {
            self.proposer
                .render_dropdown(c, LayerId::DEFAULT, row.panel, theme);
        }
    }
}

fn clip(c: &mut Compositor, r: Rect) {
    c.push(SceneNode::PushClip {
        x: r.x,
        y: r.y,
        w: r.w,
        h: r.h,
    });
}
