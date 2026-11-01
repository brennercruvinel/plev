//! The asset card (AssetCard/index.tsx) and the scene card (SceneCard/
//! index.tsx) with its skeleton (SceneCardSkeleton/index.tsx).

use super::{
    CatalogCard, CatalogKind, EDGE, boxed, glass, icon, palette, photo, scrim, text, tokens,
};
use crate::drawing::compositor::{Compositor, LayerId};
use crate::drawing::text::TextMeasurer;
use crate::styling::theme::Theme;
use crate::widgets::recipe::{rounded_rect, shadow_drops};
use crate::widgets::widget::Rect;

/// The asset card's three buttons in `b`: the add at the top right, the
/// download at the bottom left, the share (its icon and label, `px-3`) at
/// the bottom right.
pub(super) fn asset_actions(b: Rect, theme: &Theme) -> [Rect; 3] {
    let t = &tokens(theme).tile;
    let (n, inset) = (t.button, t.button_inset);
    let foot = b.bottom() - inset - n;
    let lw = TextMeasurer::measure_styled("Share", &t.label.style(theme), None).0;
    let share_w = t.label_pad * 2.0 + t.icon + t.label_gap + lw;
    [
        Rect::new(b.right() - inset - n, b.y + inset, n, n),
        Rect::new(b.x + inset, foot, n, n),
        Rect::new(b.right() - inset - share_w, foot, share_w, n),
    ]
}

/// The asset card: the panel wash (the hover's wash), the photo inset
/// `p-6`; under the pointer the scrim from the bottom, the raised add
/// button at the top right, the download and the share at the foot. The
/// button under the pointer hovers as the kit's: the add takes the panel
/// wash (`hover:bg-[#F8F7F7]`), the two on the photo the glass.
pub(super) fn asset(card: &CatalogCard, c: &mut Compositor, b: Rect, theme: &Theme) {
    let (t, k) = (&tokens(theme).tile, palette(theme));
    let hovered = card.press.hovered;
    let fill = if hovered { k.wash } else { k.panel };
    boxed(c, b, t.asset_radius, fill, k.line);
    let pad = t.asset_pad;
    photo(
        c,
        Rect::new(b.x + pad, b.y + pad, b.w - pad * 2.0, b.h - pad * 2.0),
        0.0,
        theme,
    );
    if !hovered {
        return;
    }
    scrim(c, b, t.scrim_h, t.scrim_alpha, t.asset_radius, theme);
    let [add, download, share] = asset_actions(b, theme);
    shadow_drops(c, LayerId::DEFAULT, add, t.button_radius, &t.raised);
    let add_fill = if card.hot == Some(0) {
        k.panel
    } else {
        k.paper
    };
    boxed(c, add, t.button_radius, add_fill, t.button_border);
    for (i, at) in [(1, download), (2, share)] {
        if card.hot == Some(i) {
            glass(c, at, t.button_radius, theme);
        }
    }
    let mid = (t.button - t.icon) / 2.0;
    icon(c, "plus", t.icon, k.ink, add.x + mid, add.y + mid);
    icon(
        c,
        "download",
        t.icon,
        k.on_image,
        download.x + mid,
        download.y + mid,
    );
    let sx = share.x + t.label_pad;
    icon(c, "forward", t.icon, k.on_image, sx, share.y + mid);
    text(
        c,
        "Share",
        &t.label.style(theme),
        sx + t.icon + t.label_gap,
        share.y + (t.button - t.label.line_height) / 2.0,
        k.on_image,
        None,
    );
}

/// The scene card's size in `w` of room: the frame `p-2`, the photo at
/// its ratio, the text `p-3` (two lines, or the skeleton's two bars).
pub(super) fn scene_size(w: f32, theme: &Theme) -> (f32, f32) {
    let t = &tokens(theme).tile;
    let w = t.scene_w.min(w);
    let image_h = (w - (t.scene_pad + EDGE) * 2.0) / t.image_ratio;
    let text_h = t.title.line_height + t.title_gap + t.category.line_height;
    let h = (t.scene_pad + EDGE) * 2.0 + image_h + t.image_gap + t.text_pad * 2.0 + text_h;
    (w, h)
}

/// The scene card: the paper frame (lifted under the pointer), the photo,
/// the type badge on it under the pointer, the title and the category;
/// the skeleton holds a panel wash for the photo and two bars.
pub(super) fn scene(card: &CatalogCard, c: &mut Compositor, b: Rect, theme: &Theme) {
    let (t, k) = (&tokens(theme).tile, palette(theme));
    let skeleton = card.kind == CatalogKind::SceneSkeleton;
    if card.press.hovered && !skeleton {
        shadow_drops(c, LayerId::DEFAULT, b, t.scene_radius, &t.lift);
    }
    boxed(c, b, t.scene_radius, k.paper, k.line);
    let inset = t.scene_pad + EDGE;
    let iw = b.w - inset * 2.0;
    let image = Rect::new(b.x + inset, b.y + inset, iw, iw / t.image_ratio);
    let tx = image.x + t.text_pad;
    let ty = image.bottom() + t.image_gap + t.text_pad;
    let tw = (iw - t.text_pad * 2.0).max(0.0);
    if skeleton {
        c.push(rounded_rect(
            image.x,
            image.y,
            image.w,
            image.h,
            t.image_radius,
            k.panel.0,
        ));
        // `mb-1 p-1` and `p-1` around the bars: the title's and the
        // category's lines.
        let rows = [ty, ty + t.title.line_height + t.title_gap];
        for (bar_w, y) in t.bars.iter().zip(rows) {
            c.push(rounded_rect(
                tx + t.bar_pad,
                y + t.bar_pad,
                bar_w.min(tw - t.bar_pad * 2.0).max(0.0),
                t.bar_h,
                t.bar_radius,
                k.wash.0,
            ));
        }
        return;
    }
    photo(c, image, t.image_radius, theme);
    let CatalogKind::Scene {
        title,
        category,
        video,
    } = &card.kind
    else {
        return;
    };
    if card.press.hovered {
        let at = Rect::new(
            image.x + t.badge_inset,
            image.y + t.badge_inset,
            t.badge,
            t.badge,
        );
        shadow_drops(c, LayerId::DEFAULT, at, t.badge_radius, &t.raised);
        boxed(c, at, t.badge_radius, k.paper, k.ring);
        let mid = (t.badge - t.icon) / 2.0;
        let name = if *video { "kit-clapperboard" } else { "image" };
        icon(c, name, t.icon, k.ink, at.x + mid, at.y + mid);
    }
    let title_style = t.title.style(theme);
    let shown = TextMeasurer::truncate_to_width(title, &title_style, tw);
    text(c, &shown, &title_style, tx, ty, k.ink, None);
    let cat_style = t.category.style(theme);
    let cat = TextMeasurer::truncate_to_width(category, &cat_style, tw);
    text(
        c,
        &cat,
        &cat_style,
        tx,
        ty + t.title.line_height + t.title_gap,
        k.muted,
        None,
    );
}
