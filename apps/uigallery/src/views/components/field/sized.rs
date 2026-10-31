//! Cards as tall as a form widget needs, over the showcase's grid: the
//! least the form pages need until the showcase's own `fit` lands (front
//! C), then this goes. The columns, the titles and the padding are the
//! showcase's (`showcase::layout`); only a row taller than `h-50`, and the
//! rows under it, move.

use engine::styling::theme::Theme;
use engine::widgets::prelude::Rect;

use super::super::showcase::{self, CARD_H, GroupLayout};

/// A card's width in a group of `span` cards per row at `content`'s
/// width.
pub(crate) fn card_width(span: usize, content: Rect, theme: &Theme) -> f32 {
    showcase::layout([(span, 1)], content, theme)
        .first()
        .and_then(|g| g.cards.first())
        .map_or(content.w, |r| r.w)
}

/// The band the chip takes at the top of a card; a centered widget keeps
/// as much clear at the bottom.
fn chip_band(theme: &Theme) -> f32 {
    theme.spacing.sm * 2.0 + showcase::chip_style(theme).line_height + theme.spacing.xs * 2.0
}

/// The height of a card whose widget is `widget_h` tall: `h-50`, or
/// taller where the widget, centered, would reach the chip.
pub(crate) fn card_height(widget_h: f32, theme: &Theme) -> f32 {
    CARD_H.max(chip_band(theme) * 2.0 + widget_h)
}

/// The showcase's layout with each card's least height: a row as tall as
/// its tallest card, every row and group under it moved down by what it
/// grew.
pub(crate) fn layout_sized(
    groups: impl IntoIterator<Item = (usize, Vec<f32>)>,
    content: Rect,
    theme: &Theme,
) -> Vec<GroupLayout> {
    let groups: Vec<(usize, Vec<f32>)> = groups.into_iter().collect();
    let grid = showcase::layout(groups.iter().map(|(s, h)| (*s, h.len())), content, theme);
    let mut grown = 0.0;
    grid.into_iter()
        .zip(&groups)
        .map(|(g, (_, heights))| {
            let title_y = g.title_y + grown;
            let first = g.cards.first().map_or(g.title_y, |r| r.y);
            let cols = g.cards.iter().take_while(|r| r.y == first).count().max(1);
            let mut cards = Vec::with_capacity(g.cards.len());
            for (row, frames) in heights.chunks(cols).zip(g.cards.chunks(cols)) {
                let h = row.iter().copied().fold(CARD_H, f32::max);
                for frame in frames {
                    cards.push(Rect::new(frame.x, frame.y + grown, frame.w, h));
                }
                grown += h - CARD_H;
            }
            GroupLayout {
                title_y,
                cards,
                rule_y: g.rule_y + grown,
            }
        })
        .collect()
}
