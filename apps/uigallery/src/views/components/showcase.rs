//! The showcase mold of the hoff core kit's component pages, as its
//! ButtonsPage draws it: groups with a title and a bottom rule, cards in
//! rows, one state per card behind a chip that names it, the widget
//! centered in its card. A page keeps its widgets and its projection; this
//! lays its groups out and draws the frame around them. Every visual value
//! comes from the theme.

use engine::drawing::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::drawing::text::{TextMeasurer, TextStyle};
use engine::styling::theme::Theme;
use engine::widgets::prelude::{Rect, rounded_rect, rounded_rect_stroke};

/// A card is as tall as the kit page's `h-50`.
pub(crate) const CARD_H: f32 = 200.0;
/// Narrowest a card column may get before the row drops a column.
pub(crate) const CARD_MIN_W: f32 = 300.0;

/// Where one group lands: its title's top, its cards, the rule under it.
#[derive(Debug)]
pub(crate) struct GroupLayout {
    pub title_y: f32,
    pub cards: Vec<Rect>,
    pub rule_y: f32,
}

/// Group padding: the kit page's `p-12`, vertical only (the content rect
/// already carries the horizontal gutter).
fn pad(theme: &Theme) -> f32 {
    theme.spacing.xxl
}

/// Space between the group title and its cards (`mb-5`).
fn title_gap(theme: &Theme) -> f32 {
    theme.spacing.lg
}

fn title_style(theme: &Theme) -> TextStyle {
    theme.typography.exact(theme.typography.title_sm, 28.0, 500)
}

fn chip_style(theme: &Theme) -> TextStyle {
    theme.typography.exact(theme.typography.caption, 16.0, 500)
}

/// Columns a row actually gets at this width: the group's span until the
/// cards would get narrower than `CARD_MIN_W`.
fn columns(span: usize, w: f32, gap: f32) -> usize {
    (1..=span)
        .rev()
        .find(|n| (w - gap * (*n as f32 - 1.0)) / *n as f32 >= CARD_MIN_W)
        .unwrap_or(1)
}

/// The groups down `content`, each given as `(span, cards)`: the cards
/// per row at full width and how many cards it has.
pub(crate) fn layout(
    groups: impl IntoIterator<Item = (usize, usize)>,
    content: Rect,
    theme: &Theme,
) -> Vec<GroupLayout> {
    let pad = pad(theme);
    let gap = theme.spacing.sm;
    let title_h = title_style(theme).line_height;
    let mut y = content.y;
    let mut out = Vec::new();
    for (span, count) in groups {
        let title_y = y + pad;
        let cols = columns(span, content.w, gap);
        let card_w = (content.w - gap * (cols as f32 - 1.0)) / cols as f32;
        let first_row = title_y + title_h + title_gap(theme);
        let cards = (0..count)
            .map(|i| {
                let (col, row) = (i % cols, i / cols);
                Rect::new(
                    content.x + col as f32 * (card_w + gap),
                    first_row + row as f32 * (CARD_H + gap),
                    card_w,
                    CARD_H,
                )
            })
            .collect();
        let rows = count.div_ceil(cols);
        let rule_y = first_row + rows as f32 * CARD_H + (rows as f32 - 1.0) * gap + pad;
        out.push(GroupLayout {
            title_y,
            cards,
            rule_y,
        });
        y = rule_y;
    }
    out
}

/// The height the groups take from `content`'s top.
pub(crate) fn content_height(layout: &[GroupLayout], content: Rect) -> f32 {
    layout.last().map_or(0.0, |g| g.rule_y - content.y)
}

/// A widget of `size` centered in its card, on whole pixels.
pub(crate) fn centered((w, h): (f32, f32), card: Rect) -> Rect {
    Rect::new(
        (card.x + (card.w - w) / 2.0).round(),
        (card.y + (card.h - h) / 2.0).round(),
        w,
        h,
    )
}

/// A group's title at its layout's `title_y`.
pub(crate) fn title(c: &mut Compositor, text: &str, content: Rect, y: f32, theme: &Theme) {
    c.push(SceneNode::Text {
        key: TextNodeKey::from_style(text, &title_style(theme), None),
        x: content.x,
        y,
        color: theme.colors.text.0,
    });
}

/// A card's frame and the chip naming the states it shows; the page draws
/// the widget over it.
pub(crate) fn card(c: &mut Compositor, rect: Rect, chip: &str, theme: &Theme) {
    let style = chip_style(theme);
    let pad_x = theme.spacing.md;
    let pad_y = theme.spacing.xs;
    let inset = theme.spacing.sm;
    // The card: a divider-colored hairline at the card radius.
    c.push(rounded_rect_stroke(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        theme.shape.card,
        theme.colors.divider.0,
        1.0,
    ));
    // The chip naming the states.
    let (tw, _) = TextMeasurer::measure_styled(chip, &style, None);
    let at = Rect::new(
        rect.x + inset,
        rect.y + inset,
        tw + pad_x * 2.0,
        style.line_height + pad_y * 2.0,
    );
    c.push(rounded_rect(
        at.x,
        at.y,
        at.w,
        at.h,
        theme.shape.nav,
        theme.colors.bg_hover.0,
    ));
    c.push(SceneNode::Text {
        key: TextNodeKey::from_style(chip, &style, None),
        x: at.x + pad_x,
        y: at.y + pad_y,
        color: theme.colors.text.0,
    });
}

/// The rule under a group (`border-b`); a page leaves it off the last.
pub(crate) fn rule(c: &mut Compositor, content: Rect, y: f32, theme: &Theme) {
    c.push(SceneNode::Rect {
        x: content.x,
        y,
        w: content.w,
        h: 1.0,
        color: theme.colors.divider.0,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Narrow content: rows drop to one column instead of squeezing the
    /// cards under `CARD_MIN_W`.
    #[test]
    fn cards_stack_in_narrow_content() {
        let content = Rect::new(0.0, 0.0, 500.0, 900.0);
        let layout = layout([(2, 2), (3, 3)], content, &Theme::hoff_core());
        for group in &layout {
            for r in &group.cards {
                assert_eq!(r.x, content.x);
                assert!(r.w >= CARD_MIN_W);
            }
        }
    }

    /// Wide content: a two-card span sits side by side, a three-card span
    /// in threes, and the groups follow each other down the page.
    #[test]
    fn cards_fill_rows_in_wide_content() {
        let content = Rect::new(0.0, 0.0, 1200.0, 900.0);
        let layout = layout([(2, 2), (3, 3)], content, &Theme::hoff_core());
        let first = &layout[0].cards;
        assert_eq!(first[0].y, first[1].y);
        assert!(first[1].x > first[0].x);
        let second = &layout[1].cards;
        assert_eq!(second[0].y, second[2].y);
        assert!(layout[1].title_y > layout[0].rule_y);
        assert!(content_height(&layout, content) > 2.0 * CARD_H);
    }

    /// A widget sits in the middle of its card, on whole pixels.
    #[test]
    fn a_widget_is_centered_in_its_card() {
        let card = Rect::new(10.0, 20.0, 301.0, CARD_H);
        let r = centered((80.0, 36.0), card);
        assert_eq!((r.x, r.y), (121.0, 102.0));
        assert_eq!((r.w, r.h), (80.0, 36.0));
    }
}
