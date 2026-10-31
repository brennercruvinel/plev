//! The explore card (ExploreCard/index.tsx), its small tag
//! (ExploreCardSm/index.tsx) and the slide item (CardsPage :80-100).

use super::{CatalogCard, CatalogKind, EDGE, boxed, icon, palette, photo, scrim, text, tokens};
use crate::drawing::compositor::{Compositor, LayerId};
use crate::drawing::text::TextMeasurer;
use crate::styling::theme::Theme;
use crate::widgets::recipe::shadow_drops;
use crate::widgets::widget::Rect;

/// The explore card: the photo in its rounded frame; under the pointer
/// the scrim, the title and three actions (like, download, share) at the
/// foot.
pub(super) fn card(card: &CatalogCard, c: &mut Compositor, b: Rect, theme: &Theme) {
    let (e, k) = (&tokens(theme).explore, palette(theme));
    photo(c, b, e.radius, theme);
    let CatalogKind::Explore { title } = &card.kind else {
        return;
    };
    if !card.press.hovered {
        return;
    }
    let t = &tokens(theme).tile;
    scrim(c, b, t.scrim_h, e.scrim_alpha, e.radius, theme);
    let bar_y = b.bottom() - e.bar_pad - e.button;
    let mut x = b.right() - e.bar_pad - e.button;
    let mid = (e.button - t.icon) / 2.0;
    for name in ["forward", "download", "heart"] {
        icon(c, name, t.icon, k.on_image, x + mid, bar_y + mid);
        x -= e.button + e.button_gap;
    }
    let style = t.title.style(theme);
    let room = (x + e.button - (b.x + e.bar_pad + e.title_pad * 2.0)).max(0.0);
    let shown = TextMeasurer::truncate_to_width(title, &style, room);
    text(
        c,
        &shown,
        &style,
        b.x + e.bar_pad + e.title_pad,
        bar_y + (e.button - t.title.line_height) / 2.0,
        k.on_image,
        None,
    );
}

/// The small tag: the thumbnail and two lines on the panel wash; under
/// the pointer the paper, the ring and a lift.
pub(super) fn tag(card: &CatalogCard, c: &mut Compositor, b: Rect, theme: &Theme) {
    let (e, k) = (&tokens(theme).explore, palette(theme));
    let CatalogKind::ExploreSm { title, description } = &card.kind else {
        return;
    };
    if card.press.hovered {
        shadow_drops(c, LayerId::DEFAULT, b, e.tag_radius, &e.lift);
        boxed(c, b, e.tag_radius, k.paper, k.ring);
    } else {
        boxed(c, b, e.tag_radius, k.panel, k.line);
    }
    let inset = e.tag_pad + EDGE;
    let thumb = Rect::new(b.x + inset, b.y + inset, e.thumb, e.thumb);
    photo(c, thumb, e.thumb_radius, theme);
    let tx = thumb.right() + e.text_gap;
    let tw = (b.right() - inset - tx).max(0.0);
    let lines = e.tag_title.line_height + e.line_gap + e.tag_line.line_height;
    let ty = thumb.y + (e.thumb - lines) / 2.0;
    let title_style = e.tag_title.style(theme);
    let shown = TextMeasurer::truncate_to_width(title, &title_style, tw);
    text(c, &shown, &title_style, tx, ty, k.ink, None);
    let line_style = e.tag_line.style(theme);
    let line = TextMeasurer::truncate_to_width(description, &line_style, tw);
    text(
        c,
        &line,
        &line_style,
        tx,
        ty + e.tag_title.line_height + e.line_gap,
        k.muted,
        None,
    );
}

/// The slide item: the photo in a bordered well, the panel wash at rest,
/// the paper and the ring under the pointer, the paper lifted when it is
/// the active one.
pub(super) fn slide(card: &CatalogCard, c: &mut Compositor, b: Rect, theme: &Theme) {
    let (e, k, t) = (&tokens(theme).explore, palette(theme), &tokens(theme).tile);
    if card.selected {
        shadow_drops(c, LayerId::DEFAULT, b, e.slide_radius, &t.raised);
        boxed(c, b, e.slide_radius, k.paper, k.line);
    } else if card.press.hovered {
        boxed(c, b, e.slide_radius, k.paper, k.ring);
    } else {
        boxed(c, b, e.slide_radius, k.panel, k.line);
    }
    let s = e.slide.min(b.w - EDGE * 2.0);
    photo(
        c,
        Rect::new(b.x + EDGE, b.y + EDGE, s, s),
        e.slide_radius - EDGE,
        theme,
    );
}
