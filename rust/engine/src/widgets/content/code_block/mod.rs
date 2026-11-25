//! Code block: code on a well, wrapped at the block width, with an
//! optional caption line. Every visual value is the theme's
//! `navigation.code` recipe. The `code_style` helper three gallery
//! sections grew is this one widget. [`CodeBlock::project`] declares it
//! as a label named by its caption, its code as the value.
//!
//! Rows (task-97): given `rows`, the block draws them instead of `code`,
//! one line box each, cut at the block's width with an ellipsis, clipped
//! to the area under the caption and moved up by `scroll`. Each row's
//! kind (a diff's context, a line added or removed, a hunk's header) is a
//! wash behind it, from the theme's tokens; its text is the app's, the
//! `+` and `-` of a diff included, so the kind never rests on the hue
//! alone. Without rows the block draws exactly as it did.

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::theme::Theme;
use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use crate::widgets::recipe::{rounded_rect, rounded_rect_stroke};
use crate::widgets::widget::Rect;

/// The kind of one row of a code block: what a diff says of the line.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LineKind {
    #[default]
    Context,
    Added,
    Removed,
    Header,
}

impl LineKind {
    pub const ALL: [LineKind; 4] = [
        LineKind::Context,
        LineKind::Added,
        LineKind::Removed,
        LineKind::Header,
    ];

    /// The wash behind a row of this kind, from the recipe: the
    /// success wash for a line added, the danger one for a line removed,
    /// the header's; none for context.
    pub fn wash(self, theme: &Theme) -> Option<[f32; 4]> {
        let r = &theme.navigation.code;
        match self {
            LineKind::Context => None,
            LineKind::Added => Some(r.added.0),
            LineKind::Removed => Some(r.removed.0),
            LineKind::Header => Some(r.header.0),
        }
    }

    /// The sign a diff's row opens with and the code after it: `+` on a
    /// line added, `-` on one removed, a space on context; none on a
    /// header, or where the row opens with anything else.
    pub fn marker(self, text: &str) -> Option<(&str, &str)> {
        let sign = match self {
            LineKind::Added => '+',
            LineKind::Removed => '-',
            LineKind::Context => ' ',
            LineKind::Header => return None,
        };
        text.strip_prefix(sign)
            .map(|rest| (&text[..sign.len_utf8()], rest))
    }

    /// The marker column's width: the widest of the three signs.
    pub fn marker_w(style: &TextStyle) -> f32 {
        ["+", "-", " "]
            .iter()
            .map(|s| TextMeasurer::measure_styled(s, style, None).0)
            .fold(0.0, f32::max)
    }

    /// The ink of a row's text: one tone for every kind, the one that
    /// reads at WCAG AA over every wash on the block (the code's own tone
    /// may not over a wash).
    pub fn ink(self, theme: &Theme) -> [f32; 4] {
        theme.navigation.code.row_ink.0
    }
}

/// One row of a code block: its text and its kind.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CodeLine {
    pub text: String,
    pub kind: LineKind,
}

impl CodeLine {
    pub fn new(text: impl Into<String>, kind: LineKind) -> Self {
        Self {
            text: text.into(),
            kind,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CodeBlock {
    pub code: String,
    /// Small caption above the code (a file name, a language).
    pub caption: Option<String>,
    /// The code as rows, each with its kind; when there are any, the block
    /// draws them instead of `code`.
    pub rows: Vec<CodeLine>,
    /// How far down the rows are scrolled, in px; the block keeps it
    /// within `0..=max_scroll`.
    pub scroll: f32,
    sem: Option<SemId>,
}

impl CodeBlock {
    pub fn new(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            caption: None,
            rows: Vec::new(),
            scroll: 0.0,
            sem: None,
        }
    }

    /// A block of rows.
    pub fn lines(rows: Vec<CodeLine>) -> Self {
        Self {
            rows,
            ..Self::new("")
        }
    }

    pub fn caption(mut self, caption: impl Into<String>) -> Self {
        self.caption = Some(caption.into());
        self
    }

    /// The logical identity the render binds the block to.
    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    /// The code as text: `code`, or the rows one per line.
    pub fn text(&self) -> String {
        if self.rows.is_empty() {
            return self.code.clone();
        }
        let rows: Vec<&str> = self.rows.iter().map(|r| r.text.as_str()).collect();
        rows.join("\n")
    }

    /// Declares the block as `id`: a `Role::Label` named by its caption
    /// ("code" without one), its code as the `value` a reader hears. It
    /// only reads the widget. Returns the node for the app to add to.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        let name = self.caption.as_deref().unwrap_or("code");
        let mut node = b.node(id, Role::Label, name);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.state("value", self.text());
        node
    }

    pub fn text_style(theme: &Theme) -> TextStyle {
        theme.navigation.code.text.style(&theme.typography)
    }

    fn caption_style(theme: &Theme) -> TextStyle {
        theme.navigation.code.caption.style(&theme.typography)
    }

    /// Inner padding.
    fn pad(theme: &Theme) -> f32 {
        theme.navigation.code.pad
    }

    /// The band the caption takes above the code, its gap included; none
    /// without a caption.
    fn caption_band(&self, theme: &Theme) -> f32 {
        self.caption
            .as_ref()
            .map(|_| Self::caption_style(theme).line_height + theme.navigation.code.caption_gap)
            .unwrap_or(0.0)
    }

    /// Height for a given width: the code wrapped at the inner width plus
    /// the padding and the caption band; with rows, a line box per row
    /// (at least one), whatever the width.
    pub fn height_for(&self, width: f32, theme: &Theme) -> f32 {
        let pad = Self::pad(theme);
        let style = Self::text_style(theme);
        let caption = self.caption_band(theme);
        if !self.rows.is_empty() {
            return pad + caption + self.content_h(theme) + pad;
        }
        let inner = (width - pad * 2.0).max(0.0);
        let (_, th) = TextMeasurer::measure_styled(&self.code, &style, Some(inner));
        pad + caption + th.max(style.line_height) + pad
    }

    /// The height of one row: a line box of the mono step.
    pub fn row_h(theme: &Theme) -> f32 {
        Self::text_style(theme).line_height
    }

    /// The height of every row.
    pub fn content_h(&self, theme: &Theme) -> f32 {
        self.rows.len() as f32 * Self::row_h(theme)
    }

    /// Where the rows are drawn and clipped in a block at `bounds`: the
    /// block's width, between the caption band and the bottom padding.
    pub fn rows_rect(&self, bounds: Rect, theme: &Theme) -> Rect {
        let pad = Self::pad(theme);
        let top = pad + self.caption_band(theme);
        Rect::new(
            bounds.x,
            bounds.y + top,
            bounds.w,
            (bounds.h - top - pad).max(0.0),
        )
    }

    /// The farthest the rows scroll in a block at `bounds`: what they pass
    /// its rows area by.
    pub fn max_scroll(&self, bounds: Rect, theme: &Theme) -> f32 {
        (self.content_h(theme) - self.rows_rect(bounds, theme).h).max(0.0)
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        self.render_to_layer(c, LayerId::DEFAULT, bounds, theme);
    }

    pub fn render_to_layer(&self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        let r = &theme.navigation.code;
        if let Some(id) = &self.sem {
            c.bind_to_layer(layer, id, [bounds.x, bounds.y, bounds.w, bounds.h]);
        }
        let pad = Self::pad(theme);
        let Rect { x, y, w, h } = bounds;
        c.push_to_layer(layer, rounded_rect(x, y, w, h, r.radius, r.fill.0));
        let rim = rounded_rect_stroke(x, y, w, h, r.radius, r.border.0, r.border_width);
        c.push_to_layer(layer, rim);
        let inner = (bounds.w - pad * 2.0).max(0.0);
        let mut y = bounds.y + pad;
        if let Some(caption) = &self.caption {
            let style = Self::caption_style(theme);
            c.push_to_layer(
                layer,
                SceneNode::Text {
                    key: TextNodeKey::from_style(caption, &style, Some(inner)),
                    x: bounds.x + pad,
                    y,
                    color: r.caption_color.0,
                },
            );
            y += style.line_height + r.caption_gap;
        }
        if !self.rows.is_empty() {
            self.render_rows(c, layer, bounds, theme);
            return;
        }
        c.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::from_style(&self.code, &Self::text_style(theme), Some(inner)),
                x: bounds.x + pad,
                y,
                color: r.code.0,
            },
        );
    }

    /// The rows in their area, scrolled: the wash of each row's kind
    /// inside the block's edge, then its text cut at the inner width. Only
    /// the rows that show are pushed.
    fn render_rows(&self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        let area = self.rows_rect(bounds, theme);
        let style = Self::text_style(theme);
        let row_h = style.line_height;
        let pad = Self::pad(theme);
        let edge = theme.navigation.code.border_width;
        let inner = (bounds.w - pad * 2.0).max(0.0);
        let scroll = self.scroll.clamp(0.0, self.max_scroll(bounds, theme));
        let marker_w = LineKind::marker_w(&style);
        // Only a diff's rows have markers: code with an indent is no diff.
        let diff = self
            .rows
            .iter()
            .any(|r| matches!(r.kind, LineKind::Added | LineKind::Removed));
        c.push_to_layer(
            layer,
            SceneNode::PushClip {
                x: area.x,
                y: area.y,
                w: area.w,
                h: area.h,
            },
        );
        let first = if row_h > 0.0 {
            (scroll / row_h).floor() as usize
        } else {
            0
        };
        for (at, row) in self.rows.iter().enumerate().skip(first) {
            let y = area.y + at as f32 * row_h - scroll;
            if y >= area.y + area.h {
                break;
            }
            if let Some(color) = row.kind.wash(theme) {
                c.push_to_layer(
                    layer,
                    SceneNode::Rect {
                        x: bounds.x + edge,
                        y,
                        w: (bounds.w - edge * 2.0).max(0.0),
                        h: row_h,
                        color,
                    },
                );
            }
            // A diff's marker ends in a column as wide as the widest sign,
            // the code after it at one x on every row: in a proportional
            // face (a theme may name one as its mono family) a `+`, a `-`
            // and a space would start the code at three x. The row stays
            // one text, shifted by what its sign is narrower than the
            // column; in the embedded mono face the shift is zero.
            let lead = match row.kind.marker(&row.text).filter(|_| diff) {
                Some((sign, _)) => marker_w - TextMeasurer::measure_styled(sign, &style, None).0,
                None => 0.0,
            };
            let room = (inner - lead).max(0.0);
            let shown = TextMeasurer::truncate_to_width(&row.text, &style, room);
            c.push_to_layer(
                layer,
                SceneNode::Text {
                    key: TextNodeKey::from_style(&shown, &style, None),
                    x: bounds.x + pad + lead,
                    y,
                    color: row.kind.ink(theme),
                },
            );
        }
        c.push_to_layer(layer, SceneNode::PopClip);
    }
}

#[cfg(test)]
mod tests;
