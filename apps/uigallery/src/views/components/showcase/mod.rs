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
use engine::drawing::text::{OverflowWrap, TextMeasurer, TextStyle};
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

/// The band the title `text` takes in a group `width` wide: the lines it
/// wraps into there (a phone, a larger text, a longer language: never
/// cut), on a whole pixel, and the gap. One line at least.
pub(crate) fn title_band_of(text: &str, width: f32, theme: &Theme) -> f32 {
    let style = title_style(width, theme);
    let (_, h) = TextMeasurer::measure_styled(text, &style, Some(width));
    h.max(style.line_height).round() + title_gap(theme)
}

/// The group title's text style in a group `width` wide: the h6 role
/// (the kit's 20/28 medium), fluid with the width where the theme makes
/// it so; wrapped, a word breaks only when it cannot fit a line of its
/// own.
pub(crate) fn title_style(width: f32, theme: &Theme) -> TextStyle {
    theme
        .typography
        .role_at(TypeRole::H6, width)
        .with_overflow_wrap(OverflowWrap::BreakWord)
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

/// One group of cards: its title, how many to a row at full width
/// (`span`), the narrowest a card may get before the row drops a column
/// (`min_w`), and each card's height; a row is as tall as its tallest
/// card, and a row of more than one at least `floor`. The title takes
/// the lines it wraps into above the cards; a group without one takes no
/// room for it.
pub(crate) struct Grid {
    pub title: Option<String>,
    pub span: usize,
    pub min_w: f32,
    pub heights: Vec<f32>,
    pub floor: f32,
}

impl Grid {
    pub(crate) fn new(title: &str, span: usize, min_w: f32, heights: Vec<f32>) -> Self {
        Self {
            title: Some(title.to_owned()),
            span,
            min_w,
            heights,
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
    pub(crate) fn cards(title: &str, span: usize, heights: Vec<f32>, theme: &Theme) -> Self {
        Self::new(title, span, card_min_w(theme), heights).floor(card_min_h(theme))
    }

    /// The group without a title: its cards start at its padding.
    pub(crate) fn untitled(self) -> Self {
        Self {
            title: None,
            ..self
        }
    }
}

/// The groups down `content`, each given as `(title, span, cards)`, every card
/// as tall as what it holds: `widget_h(group, card, card_w)` is how tall
/// a card's widget is in a card `card_w` wide, centered clear of the chip
/// ([`card_height`]).
pub(crate) fn layout_measured<'a>(
    groups: impl IntoIterator<Item = (&'a str, usize, usize)>,
    content: Rect,
    theme: &Theme,
    widget_h: impl Fn(usize, usize, f32) -> f32,
) -> Vec<GroupLayout> {
    let sized = groups
        .into_iter()
        .enumerate()
        .map(|(g, (title, span, count))| {
            let w = card_width(span, content, theme);
            let heights = (0..count)
                .map(|c| card_height(widget_h(g, c, w), theme))
                .collect();
            (title, span, heights)
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

/// The groups down `content`, each `(title, span, heights)`: every card as tall
/// as its height (a [`card_height`] or a [`body_height`]), a row as tall
/// as its tallest card and, when it holds more than one, at least the
/// theme's least card height.
pub(crate) fn layout_sized<'a>(
    groups: impl IntoIterator<Item = (&'a str, usize, Vec<f32>)>,
    content: Rect,
    theme: &Theme,
) -> Vec<GroupLayout> {
    let grids = groups
        .into_iter()
        .map(|(title, span, heights)| Grid::cards(title, span, heights, theme));
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
    let grid = Grid::cards("", span, heights, theme).untitled();
    layout_grids([grid], content, theme)
}

/// The groups down `content`, each a [`Grid`]: its cards as narrow as its
/// `min_w` and as tall as its heights say, a row of more than one at
/// least its `floor`. The padding and the title grow with `content`'s
/// width; a title takes the lines it wraps into there, and pushes its
/// cards and every group below down by them.
pub(crate) fn layout_grids(
    groups: impl IntoIterator<Item = Grid>,
    content: Rect,
    theme: &Theme,
) -> Vec<GroupLayout> {
    let pad = pad(content.w, theme);
    let gap = theme.spacing.sm;
    let mut y = content.y;
    let mut out = Vec::new();
    for Grid {
        title,
        span,
        min_w,
        heights,
        floor,
    } in groups
    {
        let title_h = title.map_or(0.0, |t| title_band_of(&t, content.w, theme));
        let title_y = y + pad;
        let cols = columns_at(span, min_w, content.w, gap);
        let card_w = (content.w - gap * (cols as f32 - 1.0)) / cols as f32;
        let first_row = title_y + title_h;
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

/// A group's title at its layout's `title_y`, wrapped to the content's
/// width in the lines the layout gave it ([`title_band_of`]): whole on a
/// phone, at a larger text size, in a longer language.
pub(crate) fn title(c: &mut Compositor, text: &str, content: Rect, y: f32, theme: &Theme) {
    let style = title_style(content.w, theme);
    c.push(SceneNode::Text {
        key: TextNodeKey::from_style(text, &style, Some(content.w)),
        x: content.x,
        y,
        color: theme.colors.text.0,
    });
}

/// The kit's two cards (`components/Card/index.tsx`): the plain one,
/// and the `isGray` one (:52, `bg-[#F8F7F7]`) whose chip takes the
/// page's tone with an edge (:57-59, `bg-[#FCFCFC] border-[#ECECEC]`,
/// against the plain `bg-[#F1F1F1] border-transparent`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Tone {
    #[default]
    Plain,
    Gray,
}

/// A card's frame and, in its top corner, the chip naming the states it
/// shows; the page draws the widget over it.
pub(crate) fn card(c: &mut Compositor, rect: Rect, chip_text: &str, theme: &Theme) {
    card_in(c, rect, chip_text, Tone::Plain, theme);
}

/// [`card`] in `tone`: the gray one on the panel's tone under its frame.
pub(crate) fn card_in(c: &mut Compositor, rect: Rect, chip_text: &str, tone: Tone, theme: &Theme) {
    if tone == Tone::Gray {
        let fill = theme.colors.bg_panel.0;
        c.push(rounded_rect(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            theme.shape.card,
            fill,
        ));
    }
    card_frame(c, rect, theme);
    let inset = inset(theme);
    let text = chip_fit(chip_text, rect.w - inset * 2.0, theme);
    chip_in(c, rect.x + inset, rect.y + inset, &text, tone, theme);
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
    chip_in(c, x, y, text, Tone::Plain, theme)
}

/// [`chip`] in `tone`: on a gray card, the page's tone in the divider's
/// hairline.
fn chip_in(c: &mut Compositor, x: f32, y: f32, text: &str, tone: Tone, theme: &Theme) -> Rect {
    let at = chip_rect(x, y, text, theme);
    let radius = theme.shape.nav;
    let fill = match tone {
        Tone::Plain => theme.colors.bg_hover.0,
        Tone::Gray => theme.colors.bg.0,
    };
    c.push(rounded_rect(at.x, at.y, at.w, at.h, radius, fill));
    if tone == Tone::Gray {
        let edge = theme.colors.divider.0;
        c.push(rounded_rect_stroke(
            at.x, at.y, at.w, at.h, radius, edge, 1.0,
        ));
    }
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
mod tests;
