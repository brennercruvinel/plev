//! A sheet's option row: what it is (its label, a hint under it) at the
//! start, its value at the end. The row is intrinsic: side by side while
//! the label keeps the room its longest word needs beside the value, and
//! stacked where it would not (a phone, a larger text size, a longer
//! language), the value under the label and its hint, each as wide as
//! the row's text. No column is fixed: the value's own width and the
//! label's longest word decide.

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::theme::Theme;

/// One option row: what it is, a hint under it, its value at the end.
#[derive(Clone, Debug, PartialEq)]
pub struct SheetRow {
    pub label: String,
    pub hint: Option<String>,
    pub value: String,
}

impl SheetRow {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            hint: None,
            value: value.into(),
        }
    }

    pub fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}

/// How a row lays out in a panel of a given width.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct RowFit {
    /// The width the label and the hint wrap in.
    pub label_w: f32,
    /// The value goes under the label and the hint, as wide as they.
    pub stacked: bool,
    /// The value's width: its own beside the label, the row's stacked.
    pub value_w: f32,
}

fn text_h(text: &str, style: &TextStyle, w: f32) -> f32 {
    let (_, h) = TextMeasurer::measure_styled(text, style, Some(w.max(1.0)));
    h.max(style.line_height)
}

/// The widest word of `text`: the least it wraps in without a word cut.
fn longest_word(text: &str, style: &TextStyle) -> f32 {
    text.split_whitespace()
        .map(|w| TextMeasurer::measure_styled(w, style, None).0)
        .fold(0.0, f32::max)
        .ceil()
}

impl SheetRow {
    /// The row in a panel `w` wide: side by side while the label and the
    /// hint keep their longest word beside the value, stacked otherwise.
    pub(super) fn fit(&self, theme: &Theme, w: f32) -> RowFit {
        let scale = &theme.typography;
        let t = &theme.overlay.sheet;
        let text_w = (w - t.pad_x * 2.0).max(1.0);
        let value_w = TextMeasurer::measure_styled(&self.value, &scale.run(t.value), None)
            .0
            .ceil();
        let beside = text_w - value_w - t.value_gap;
        let mut need = longest_word(&self.label, &scale.run(t.label));
        if let Some(hint) = &self.hint {
            need = need.max(longest_word(hint, &scale.run(t.hint)));
        }
        if beside >= need.max(1.0) {
            RowFit {
                label_w: beside,
                stacked: false,
                value_w,
            }
        } else {
            RowFit {
                label_w: text_w,
                stacked: true,
                value_w: text_w,
            }
        }
    }

    /// The label and its hint, the gap between: the block that sits at
    /// the row's start.
    fn label_h(&self, theme: &Theme, fit: RowFit) -> f32 {
        let scale = &theme.typography;
        let t = &theme.overlay.sheet;
        let mut h = text_h(&self.label, &scale.run(t.label), fit.label_w);
        if let Some(hint) = &self.hint {
            h += t.gap + text_h(hint, &scale.run(t.hint), fit.label_w);
        }
        h
    }

    /// The value's height stacked under the label, the gap above it.
    fn value_below(&self, theme: &Theme, fit: RowFit) -> f32 {
        let t = &theme.overlay.sheet;
        let style = theme.typography.run(t.value);
        t.gap + text_h(&self.value, &style, fit.value_w)
    }

    /// The row's height in a panel `w` wide: its texts and its padding,
    /// never under the token's least.
    pub(super) fn height(&self, theme: &Theme, w: f32) -> f32 {
        let t = &theme.overlay.sheet;
        let fit = self.fit(theme, w);
        let mut block = self.label_h(theme, fit);
        if fit.stacked {
            block += self.value_below(theme, fit);
        }
        (t.pad_y * 2.0 + block).max(t.row_min_h)
    }

    /// The row's texts at `y` in a panel from `x` `w` wide, on `layer`:
    /// the block centered in the row's height.
    pub(super) fn draw(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        theme: &Theme,
        x: f32,
        w: f32,
        y: f32,
    ) {
        let scale = &theme.typography;
        let t = &theme.overlay.sheet;
        let fit = self.fit(theme, w);
        let h = self.height(theme, w);
        let mut text = |s: &str, style: &TextStyle, tw: f32, tx: f32, ty: f32, color: [f32; 4]| {
            let key = TextNodeKey::from_style(s, style, Some(tw.max(1.0)));
            c.push_to_layer(
                layer,
                SceneNode::Text {
                    key,
                    x: tx,
                    y: ty,
                    color,
                },
            );
        };
        let tx = x + t.pad_x;
        let mut block = self.label_h(theme, fit);
        if fit.stacked {
            block += self.value_below(theme, fit);
        }
        let ly = y + (h - block) / 2.0;
        let label = scale.run(t.label);
        text(&self.label, &label, fit.label_w, tx, ly, t.label_color.0);
        if let Some(hint) = &self.hint {
            let hy = ly + text_h(&self.label, &label, fit.label_w) + t.gap;
            text(
                hint,
                &scale.run(t.hint),
                fit.label_w,
                tx,
                hy,
                t.hint_color.0,
            );
        }
        let value = scale.run(t.value);
        if fit.stacked {
            let vy = ly + self.label_h(theme, fit) + t.gap;
            text(&self.value, &value, fit.value_w, tx, vy, t.value_color.0);
        } else {
            let vx = x + w - t.pad_x - fit.value_w;
            let vy = y + TextMeasurer::vertical_center(&value, h);
            text(
                &self.value,
                &value,
                fit.value_w + 1.0,
                vx,
                vy,
                t.value_color.0,
            );
        }
    }
}
