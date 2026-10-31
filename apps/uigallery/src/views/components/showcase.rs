//! The showcase mold of the hoff core kit's component pages, as its
//! ButtonsPage draws it: groups with a title and a bottom rule, cards in
//! rows, one state per card behind a chip that names it, the widget
//! centered in its card. A page keeps its widgets and its projection; this
//! lays its groups out and draws the frame around them. The values are
//! the ButtonsPage's as they were, named here; moving them onto
//! typography tokens is left to the audit of the 47 pages.

use engine::drawing::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::drawing::text::{TextMeasurer, TextStyle};
use engine::styling::theme::Theme;
use engine::widgets::prelude::{Rect, rounded_rect, rounded_rect_stroke};

/// A card is as tall as the kit page's `h-50`.
pub(crate) const CARD_H: f32 = 200.0;
/// Narrowest a card column may get before the row drops a column.
pub(crate) const CARD_MIN_W: f32 = 300.0;
/// The group title's line height and weight, at the theme's `title_sm`
/// size.
pub(crate) const TITLE_LINE_H: f32 = 28.0;
pub(crate) const TITLE_WEIGHT: u16 = 500;
/// The chip label's line height and weight, at the theme's `caption`
/// size.
pub(crate) const CHIP_LINE_H: f32 = 16.0;
pub(crate) const CHIP_WEIGHT: u16 = 500;

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

/// The group title's text style.
pub(crate) fn title_style(theme: &Theme) -> TextStyle {
    theme
        .typography
        .exact(theme.typography.title_sm, TITLE_LINE_H, TITLE_WEIGHT)
}

/// The chip label's text style.
pub(crate) fn chip_style(theme: &Theme) -> TextStyle {
    theme
        .typography
        .exact(theme.typography.caption, CHIP_LINE_H, CHIP_WEIGHT)
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
    let sized = groups
        .into_iter()
        .map(|(span, count)| (span, vec![CARD_H; count]));
    layout_sized(sized, content, theme)
}

/// A card's width in a group of `span` cards per row at `content`'s
/// width: what a page lays a widget out in before it knows how tall the
/// card must be.
pub(crate) fn card_width(span: usize, content: Rect, theme: &Theme) -> f32 {
    let gap = theme.spacing.sm;
    let cols = columns(span, content.w, gap);
    (content.w - gap * (cols as f32 - 1.0)) / cols as f32
}

/// The band the chip takes at the top of a card; a centered widget keeps
/// as much clear at the bottom.
fn chip_band(theme: &Theme) -> f32 {
    theme.spacing.sm * 2.0 + chip_style(theme).line_height + theme.spacing.xs * 2.0
}

/// The height of a card whose widget is `widget_h` tall: `h-50`, or
/// taller where the widget, centered, would reach the chip.
pub(crate) fn card_height(widget_h: f32, theme: &Theme) -> f32 {
    CARD_H.max(chip_band(theme) * 2.0 + widget_h)
}

/// [`layout`] with each card's least height (`CARD_H`, or a
/// [`card_height`]): a row is as tall as its tallest card.
pub(crate) fn layout_sized(
    groups: impl IntoIterator<Item = (usize, Vec<f32>)>,
    content: Rect,
    theme: &Theme,
) -> Vec<GroupLayout> {
    let pad = pad(theme);
    let gap = theme.spacing.sm;
    let title_h = title_style(theme).line_height;
    let mut y = content.y;
    let mut out = Vec::new();
    for (span, heights) in groups {
        let title_y = y + pad;
        let cols = columns(span, content.w, gap);
        let card_w = (content.w - gap * (cols as f32 - 1.0)) / cols as f32;
        let first_row = title_y + title_h + title_gap(theme);
        let mut cards = Vec::with_capacity(heights.len());
        // The heights of the rows above and their count, kept apart so
        // rows of `h-50` land where `first_row + row * (CARD_H + gap)`
        // put them.
        let mut above = 0.0;
        let mut rows = 0;
        for row in heights.chunks(cols) {
            let h = row.iter().copied().fold(CARD_H, f32::max);
            let top = first_row + (above + rows as f32 * gap);
            for col in 0..row.len() {
                cards.push(Rect::new(
                    content.x + col as f32 * (card_w + gap),
                    top,
                    card_w,
                    h,
                ));
            }
            above += h;
            rows += 1;
        }
        let rule_y = first_row + above + (rows as f32 - 1.0) * gap + pad;
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

/// A card's frame and, in its top corner, the chip naming the states it
/// shows; the page draws the widget over it.
pub(crate) fn card(c: &mut Compositor, rect: Rect, chip_text: &str, theme: &Theme) {
    card_frame(c, rect, theme);
    let inset = theme.spacing.sm;
    let text = chip_fit(chip_text, rect.w - inset * 2.0, theme);
    chip(c, rect.x + inset, rect.y + inset, &text, theme);
}

/// A chip's text cut with an ellipsis where the chip, padding included,
/// would be wider than `w`; the text itself where it fits.
pub(crate) fn chip_fit(text: &str, w: f32, theme: &Theme) -> String {
    let room = w - theme.spacing.md * 2.0;
    TextMeasurer::truncate_to_width(text, &chip_style(theme), room)
}

/// The card alone: a divider-colored hairline at the card radius.
pub(crate) fn card_frame(c: &mut Compositor, rect: Rect, theme: &Theme) {
    c.push(rounded_rect_stroke(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        theme.shape.card,
        theme.colors.divider.0,
        1.0,
    ));
}

/// The chip's rect with its top-left at `(x, y)`, sized by its text.
pub(crate) fn chip_rect(x: f32, y: f32, text: &str, theme: &Theme) -> Rect {
    let style = chip_style(theme);
    let (tw, _) = TextMeasurer::measure_styled(text, &style, None);
    Rect::new(
        x,
        y,
        tw + theme.spacing.md * 2.0,
        style.line_height + theme.spacing.xs * 2.0,
    )
}

/// A chip with its top-left at `(x, y)`: the pill and its label. Returns
/// its rect.
pub(crate) fn chip(c: &mut Compositor, x: f32, y: f32, text: &str, theme: &Theme) -> Rect {
    let at = chip_rect(x, y, text, theme);
    c.push(rounded_rect(
        at.x,
        at.y,
        at.w,
        at.h,
        theme.shape.nav,
        theme.colors.bg_hover.0,
    ));
    c.push(SceneNode::Text {
        key: TextNodeKey::from_style(text, &chip_style(theme), None),
        x: at.x + theme.spacing.md,
        y: at.y + theme.spacing.xs,
        color: theme.colors.text.0,
    });
    at
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

    /// A card grows past `h-50` for a tall widget, and its row with it;
    /// rows of `h-50` land where the fixed layout puts them.
    #[test]
    fn a_tall_card_grows_its_row_only() {
        let theme = Theme::hoff_core();
        let content = Rect::new(0.0, 0.0, 1200.0, 900.0);
        let tall = card_height(400.0, &theme);
        assert!(tall > 400.0);
        assert_eq!(card_height(40.0, &theme), CARD_H);
        let sized = layout_sized([(3, vec![CARD_H, tall, CARD_H, CARD_H])], content, &theme);
        let cards = &sized[0].cards;
        assert_eq!(
            (cards[0].h, cards[1].h),
            (tall, tall),
            "the row is the tallest card"
        );
        assert_eq!(cards[3].y, cards[0].y + tall + theme.spacing.sm);
        let fixed = layout([(3, 4), (2, 2)], content, &theme);
        let same = layout_sized(
            [(3, vec![CARD_H; 4]), (2, vec![CARD_H; 2])],
            content,
            &theme,
        );
        for (a, b) in fixed.iter().zip(&same) {
            assert_eq!(
                (a.title_y, &a.cards, a.rule_y),
                (b.title_y, &b.cards, b.rule_y)
            );
        }
        assert_eq!(card_width(3, content, &theme), cards[0].w);
    }

    /// A chip longer than its card is cut with an ellipsis inside it.
    #[test]
    fn a_long_chip_is_cut_to_its_card() {
        let theme = Theme::hoff_core();
        let inset = theme.spacing.sm;
        let card = Rect::new(0.0, 0.0, 160.0, CARD_H);
        let text = chip_fit(
            "Default, Click on, Focus, Hover, Typing",
            card.w - inset * 2.0,
            &theme,
        );
        assert!(text.ends_with('\u{2026}'));
        let at = chip_rect(card.x + inset, card.y + inset, &text, &theme);
        assert!(at.right() <= card.right() - inset + 0.5);
        assert_eq!(chip_fit("Typed", card.w - inset * 2.0, &theme), "Typed");
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
