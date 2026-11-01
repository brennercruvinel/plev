//! The showcase mold of the hoff core kit's component pages, as its
//! ButtonsPage draws it: groups with a title and a bottom rule, cards in
//! rows, one state per card behind a chip that names it, the widget
//! centered in its card. A page keeps its widgets and its projection; this
//! lays its groups out and draws the frame around them. Every measure is
//! the theme's, and the group's padding and title are fluid with the
//! width the group gets (the kit's `p-12` and 20/28 wide, `p-5` and 16/24
//! narrow, on a line between them rather than a step: ADR 0073). A card
//! is as tall as what it holds; a row of more than one keeps the theme's
//! least card height (the kit's `h-50`) so its stages line up.

use engine::drawing::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::drawing::text::{TextMeasurer, TextStyle};
use engine::styling::theme::{Theme, TypeRole, fluid};
use engine::widgets::prelude::{Rect, rounded_rect, rounded_rect_stroke};

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

/// The least height of a card in a row of more than one: the theme's
/// (the kit's `h-50`).
pub(crate) fn card_min_h(theme: &Theme) -> f32 {
    theme.size.card_min_h
}

/// Narrowest a card column may get before the row drops a column: the
/// theme's least card width.
pub(crate) fn card_min_w(theme: &Theme) -> f32 {
    theme.size.card_min_w
}

/// Group padding, above and below (the content rect already carries the
/// horizontal gutter), for a group `width` wide: the kit's `p-5` (`lg`)
/// in narrow content up to its `p-12` (`xxl`) in wide, on a line between
/// the widths the type roles grow over, on whole pixels.
pub(crate) fn pad(width: f32, theme: &Theme) -> f32 {
    let range = theme.typography.roles.range;
    fluid(width, range, [theme.spacing.lg, theme.spacing.xxl]).round()
}

/// Space between the group title and its cards (`mb-5`).
pub(crate) fn title_gap(theme: &Theme) -> f32 {
    theme.spacing.lg
}

/// The band a group's title takes above its cards in a group `width`
/// wide: its line, on a whole pixel so the cards under it land on one,
/// and the gap.
pub(crate) fn title_band(width: f32, theme: &Theme) -> f32 {
    title_style(width, theme).line_height.round() + title_gap(theme)
}

/// The group title's text style in a group `width` wide: the h6 role
/// (the kit's 20/28 medium), fluid with the width where the theme makes
/// it so.
pub(crate) fn title_style(width: f32, theme: &Theme) -> TextStyle {
    theme.typography.role_at(TypeRole::H6, width)
}

/// The chip label's text style.
pub(crate) fn chip_style(theme: &Theme) -> TextStyle {
    theme
        .typography
        .exact(theme.typography.caption, CHIP_LINE_H, CHIP_WEIGHT)
}

/// The inset of everything in a card from its edges: the chip's corner
/// (the kit's `top-2 left-2`) and the room a widget gets across.
pub(crate) fn inset(theme: &Theme) -> f32 {
    theme.spacing.sm
}

/// The width a widget may take in a card `card_w` wide: the card less
/// its [`inset`] on both sides.
pub(crate) fn room(card_w: f32, theme: &Theme) -> f32 {
    (card_w - inset(theme) * 2.0).max(0.0)
}

/// Columns a row actually gets at this width: the group's span until the
/// cards would get narrower than `min_w`.
pub(crate) fn columns_at(span: usize, min_w: f32, w: f32, gap: f32) -> usize {
    (1..=span)
        .rev()
        .find(|n| (w - gap * (*n as f32 - 1.0)) / *n as f32 >= min_w)
        .unwrap_or(1)
}

/// One group of cards: how many to a row at full width (`span`), the
/// narrowest a card may get before the row drops a column (`min_w`), and
/// each card's height; a row is as tall as its tallest card, and a row
/// of more than one at least `floor`. A group without a title takes no
/// room for one.
pub(crate) struct Grid {
    pub span: usize,
    pub min_w: f32,
    pub heights: Vec<f32>,
    pub titled: bool,
    pub floor: f32,
}

impl Grid {
    pub(crate) fn new(span: usize, min_w: f32, heights: Vec<f32>) -> Self {
        Self {
            span,
            min_w,
            heights,
            titled: true,
            floor: 0.0,
        }
    }

    /// The group whose rows of more than one card are at least `floor`
    /// tall; a card alone in its row keeps its own height.
    pub(crate) fn floor(self, floor: f32) -> Self {
        Self { floor, ..self }
    }

    /// The showcase's group of `span` cards to a row, as tall as
    /// `heights`: the theme's least card width and, in a row of more than
    /// one, its least card height.
    pub(crate) fn cards(span: usize, heights: Vec<f32>, theme: &Theme) -> Self {
        Self::new(span, card_min_w(theme), heights).floor(card_min_h(theme))
    }

    /// The group without a title: its cards start at its padding.
    pub(crate) fn untitled(self) -> Self {
        Self {
            titled: false,
            ..self
        }
    }
}

/// The groups down `content`, each given as `(span, cards)`, every card
/// as tall as what it holds: `widget_h(group, card, card_w)` is how tall
/// a card's widget is in a card `card_w` wide, centered clear of the chip
/// ([`card_height`]).
pub(crate) fn layout_measured(
    groups: impl IntoIterator<Item = (usize, usize)>,
    content: Rect,
    theme: &Theme,
    widget_h: impl Fn(usize, usize, f32) -> f32,
) -> Vec<GroupLayout> {
    let sized = groups.into_iter().enumerate().map(|(g, (span, count))| {
        let w = card_width(span, content, theme);
        let heights = (0..count)
            .map(|c| card_height(widget_h(g, c, w), theme))
            .collect();
        (span, heights)
    });
    layout_sized(sized, content, theme)
}

/// A card's width in a group of `span` cards per row at `content`'s
/// width: what a page lays a widget out in before it knows how tall the
/// card must be.
pub(crate) fn card_width(span: usize, content: Rect, theme: &Theme) -> f32 {
    card_width_at(span, card_min_w(theme), content, theme)
}

/// [`card_width`] for a group whose cards may get as narrow as `min_w`:
/// what a square card is tall.
pub(crate) fn card_width_at(span: usize, min_w: f32, content: Rect, theme: &Theme) -> f32 {
    let gap = theme.spacing.sm;
    let cols = columns_at(span, min_w, content.w, gap);
    (content.w - gap * (cols as f32 - 1.0)) / cols as f32
}

/// The band the chip takes at the top of a card; a centered widget keeps
/// as much clear at the bottom.
fn chip_band(theme: &Theme) -> f32 {
    inset(theme) * 2.0 + chip_style(theme).line_height + theme.spacing.xs * 2.0
}

/// The height of a card whose widget is `widget_h` tall: the widget
/// centered with the chip's band clear above and below it.
pub(crate) fn card_height(widget_h: f32, theme: &Theme) -> f32 {
    chip_band(theme) * 2.0 + widget_h
}

/// The height of a card whose [`body`] holds `body_h`: the chip's band
/// above it and the inset below.
pub(crate) fn body_height(body_h: f32, theme: &Theme) -> f32 {
    chip_band(theme) + inset(theme) + body_h
}

/// The groups down `content`, each `(span, heights)`: every card as tall
/// as its height (a [`card_height`] or a [`body_height`]), a row as tall
/// as its tallest card and, when it holds more than one, at least the
/// theme's least card height.
pub(crate) fn layout_sized(
    groups: impl IntoIterator<Item = (usize, Vec<f32>)>,
    content: Rect,
    theme: &Theme,
) -> Vec<GroupLayout> {
    let grids = groups
        .into_iter()
        .map(|(span, heights)| Grid::cards(span, heights, theme));
    layout_grids(grids, content, theme)
}

/// One group without a title, `span` cards to a row, as tall as
/// `heights` says ([`layout_sized`]): a page that is one set of cards,
/// its title the page's own.
pub(crate) fn layout_untitled(
    span: usize,
    heights: Vec<f32>,
    content: Rect,
    theme: &Theme,
) -> Vec<GroupLayout> {
    let grid = Grid::cards(span, heights, theme).untitled();
    layout_grids([grid], content, theme)
}

/// The groups down `content`, each a [`Grid`]: its cards as narrow as its
/// `min_w` and as tall as its heights say, a row of more than one at
/// least its `floor`. The padding and the title grow with `content`'s
/// width.
pub(crate) fn layout_grids(
    groups: impl IntoIterator<Item = Grid>,
    content: Rect,
    theme: &Theme,
) -> Vec<GroupLayout> {
    let pad = pad(content.w, theme);
    let gap = theme.spacing.sm;
    let title_h = title_band(content.w, theme);
    let mut y = content.y;
    let mut out = Vec::new();
    for Grid {
        span,
        min_w,
        heights,
        titled,
        floor,
    } in groups
    {
        let title_y = y + pad;
        let cols = columns_at(span, min_w, content.w, gap);
        let card_w = (content.w - gap * (cols as f32 - 1.0)) / cols as f32;
        let first_row = title_y + if titled { title_h } else { 0.0 };
        let mut cards = Vec::with_capacity(heights.len());
        // The heights of the rows above and their count, kept apart so
        // rows of one height land where `first_row + row * (h + gap)`
        // puts them.
        let mut above = 0.0;
        let mut rows = 0;
        let floor = if cols > 1 { floor } else { 0.0 };
        for row in heights.chunks(cols) {
            let h = row.iter().copied().fold(floor, f32::max);
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

/// A card's body: the card less its chip band at the top and its
/// [`inset`] on the other sides, where a card that holds more than one
/// centered widget (a trigger and its open panel) lays it out.
pub(crate) fn body(card: Rect, theme: &Theme) -> Rect {
    let band = chip_band(theme);
    let pad = inset(theme);
    Rect::new(
        card.x + pad,
        card.y + band,
        (card.w - pad * 2.0).max(0.0),
        (card.h - band - pad).max(0.0),
    )
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

/// A group's title at its layout's `title_y`, on the one line the layout
/// gives it: cut with an ellipsis where it is wider than the content (a
/// phone, a larger text size); its projection keeps the whole title.
pub(crate) fn title(c: &mut Compositor, text: &str, content: Rect, y: f32, theme: &Theme) {
    let style = title_style(content.w, theme);
    let text = TextMeasurer::truncate_to_width(text, &style, content.w);
    c.push(SceneNode::Text {
        key: TextNodeKey::from_style(&text, &style, None),
        x: content.x,
        y,
        color: theme.colors.text.0,
    });
}

/// A card's frame and, in its top corner, the chip naming the states it
/// shows; the page draws the widget over it.
pub(crate) fn card(c: &mut Compositor, rect: Rect, chip_text: &str, theme: &Theme) {
    card_frame(c, rect, theme);
    let inset = inset(theme);
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

    /// `count` cards of a 36 px widget in each group: the button's.
    fn buttons(groups: &[(usize, usize)], content: Rect, theme: &Theme) -> Vec<GroupLayout> {
        layout_measured(groups.iter().copied(), content, theme, |_, _, _| 36.0)
    }

    /// Narrow content: rows drop to one column instead of squeezing the
    /// cards under the theme's least card width.
    #[test]
    fn cards_stack_in_narrow_content() {
        let theme = Theme::hoff_core();
        let content = Rect::new(0.0, 0.0, 500.0, 900.0);
        let layout = buttons(&[(2, 2), (3, 3)], content, &theme);
        for group in &layout {
            for r in &group.cards {
                assert_eq!(r.x, content.x);
                assert!(r.w >= card_min_w(&theme));
            }
        }
    }

    /// Wide content: a two-card span sits side by side, a three-card span
    /// in threes, and the groups follow each other down the page.
    #[test]
    fn cards_fill_rows_in_wide_content() {
        let theme = Theme::hoff_core();
        let content = Rect::new(0.0, 0.0, 1200.0, 900.0);
        let layout = buttons(&[(2, 2), (3, 3)], content, &theme);
        let first = &layout[0].cards;
        assert_eq!(first[0].y, first[1].y);
        assert!(first[1].x > first[0].x);
        let second = &layout[1].cards;
        assert_eq!(second[0].y, second[2].y);
        assert!(layout[1].title_y > layout[0].rule_y);
        assert!(content_height(&layout, content) > 2.0 * card_min_h(&theme));
    }

    /// A card is as tall as what it holds: alone in its row, a 36 px
    /// widget's card is the chip's band twice and the widget, nothing
    /// more; in a row of several the row keeps the theme's least card
    /// height so the stages line up; a tall card grows its row only.
    #[test]
    fn a_card_is_as_tall_as_what_it_holds() {
        let theme = Theme::hoff_core();
        let floor = card_min_h(&theme);
        let alone = buttons(&[(3, 2)], Rect::new(0.0, 0.0, 320.0, 900.0), &theme);
        let snug = card_height(36.0, &theme);
        assert!(snug < floor, "a button's card needs less than h-50");
        assert!(alone[0].cards.iter().all(|r| r.h == snug));
        let content = Rect::new(0.0, 0.0, 1200.0, 900.0);
        let wide = buttons(&[(3, 3)], content, &theme);
        assert!(wide[0].cards.iter().all(|r| r.h == floor));
        let tall = card_height(400.0, &theme);
        let sized = layout_sized([(3, vec![snug, tall, snug, snug])], content, &theme);
        let cards = &sized[0].cards;
        assert_eq!((cards[0].h, cards[1].h), (tall, tall), "the tallest card");
        assert_eq!(cards[3].h, floor, "the next row keeps the floor");
        assert_eq!(cards[3].y, cards[0].y + tall + theme.spacing.sm);
        assert_eq!(card_width(3, content, &theme), cards[0].w);
        // A body holds its content under the chip's band, its inset clear
        // below: the body of a card `body_height(h)` tall is `h` tall.
        let card = Rect::new(0.0, 0.0, 300.0, body_height(120.0, &theme));
        assert_eq!(body(card, &theme).h, 120.0);
    }

    /// A chip longer than its card is cut with an ellipsis inside it.
    #[test]
    fn a_long_chip_is_cut_to_its_card() {
        let theme = Theme::hoff_core();
        let inset = inset(&theme);
        let card = Rect::new(0.0, 0.0, 160.0, card_min_h(&theme));
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
        let card = Rect::new(10.0, 20.0, 301.0, 200.0);
        let r = centered((80.0, 36.0), card);
        assert_eq!((r.x, r.y), (121.0, 102.0));
        assert_eq!((r.w, r.h), (80.0, 36.0));
    }

    /// From 280 to 1600 the group's padding and title grow with the
    /// content's width without a step: from `lg` and the narrow title to
    /// `xxl` and h6, never more between two widths 8 px apart than the
    /// line allows (rounded up to a pixel), and never back down.
    #[test]
    fn the_pad_and_the_title_are_fluid_with_the_width() {
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            let [from, to] = theme.typography.roles.range;
            let slope = (theme.spacing.xxl - theme.spacing.lg) / (to - from);
            assert_eq!(pad(280.0, &theme), theme.spacing.lg);
            assert_eq!(pad(1600.0, &theme), theme.spacing.xxl);
            let h6 = theme.typography.role(TypeRole::H6);
            assert_eq!(title_style(1600.0, &theme).font_size, h6.font_size);
            let mut prev = (pad(272.0, &theme), title_style(272.0, &theme));
            for w in (280..=1600).step_by(8) {
                let w = w as f32;
                let (p, t) = (pad(w, &theme), title_style(w, &theme));
                assert!(
                    p >= prev.0 && p - prev.0 <= (slope * 8.0).ceil(),
                    "{w}: pad"
                );
                assert!(t.font_size >= prev.1.font_size, "{w}: title");
                assert!(t.font_size - prev.1.font_size <= 1.0, "{w}: title step");
                assert!(t.line_height - prev.1.line_height <= 1.0, "{w}: title line");
                // Every card of a group stays in the content, whatever
                // the width: nothing cut at either side.
                let content = Rect::new(0.0, 0.0, w, 900.0);
                for g in buttons(&[(3, 5), (2, 3), (1, 1)], content, &theme) {
                    for r in &g.cards {
                        assert!(r.x >= 0.0 && r.right() <= w + 0.5, "{w}: cut");
                    }
                    assert!(g.cards.iter().all(|r| r.bottom() <= g.rule_y), "{w}");
                }
                prev = (p, t);
            }
        }
    }

    /// A group without a title starts its cards at its padding: no room
    /// kept for a title it does not draw.
    #[test]
    fn an_untitled_group_keeps_no_room_for_a_title() {
        let theme = Theme::hoff_core();
        let content = Rect::new(0.0, 0.0, 800.0, 900.0);
        let grid = || Grid::new(3, card_min_w(&theme), vec![120.0; 3]);
        let titled = &layout_grids([grid()], content, &theme)[0];
        let bare = &layout_grids([grid().untitled()], content, &theme)[0];
        assert_eq!(bare.cards[0].y, bare.title_y);
        assert_eq!(bare.title_y, pad(content.w, &theme));
        let title_h = title_band(content.w, &theme);
        assert_eq!(title_h.fract(), 0.0, "the cards on a whole pixel");
        assert_eq!(titled.cards[0].y - bare.cards[0].y, title_h);
        assert_eq!(titled.rule_y - bare.rule_y, title_h);
    }
}
