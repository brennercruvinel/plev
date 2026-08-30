//! One deck tile: commander art on top, name and meta below.
//!
//! Shared by the dashboard and the decks grid so both draw the same cover.
//! The tile is pure layout: it renders what `DeckSummary` carries and reports
//! nothing back - hit-testing is the screen's job, against the rect it placed
//! the tile at.

use engine::compositor::{Compositor, SceneNode};
use engine::theme::Theme;
use engine::ui::widgets::{Rect, rounded_rect, rounded_rect_stroke};
use spellbook_core::images;
use spellbook_core::ops::decks::DeckSummary;
use spellbook_core::types::ImageRef;

use super::{text, with_alpha};
use crate::art::ArtCache;

/// Height of the text block under the art.
pub const BODY_H: f32 = 118.0;

/// Tile height for a column width: the art keeps art_crop's 626x457 aspect,
/// the body is fixed.
pub fn tile_height(col_w: f32) -> f32 {
    col_w * (457.0 / 626.0) + BODY_H
}

/// Mana-pip colours, the same palette the web UI used (FILTER_COLORS).
pub fn pip_color(letter: char) -> [f32; 4] {
    let hex = match letter {
        'W' => 0xfffbd5,
        'U' => 0xaae0fa,
        'B' => 0xcbc2bf,
        'R' => 0xf9aa8f,
        'G' => 0x9bd3ae,
        _ => 0x8b8398, // C - truly colourless
    };
    [
        ((hex >> 16) & 0xff) as f32 / 255.0,
        ((hex >> 8) & 0xff) as f32 / 255.0,
        (hex & 0xff) as f32 / 255.0,
        1.0,
    ]
}

/// The art_crop cache path for a commander image, if it is cacheable.
fn crop_rel(image: &Option<ImageRef>) -> Option<String> {
    let rel = image.as_ref()?.rel.as_deref()?;
    Some(images::with_variant(rel, "art_crop"))
}

pub fn render(
    c: &mut Compositor,
    rect: Rect,
    deck: &DeckSummary,
    hovered: bool,
    art: &mut ArtCache,
    theme: &Theme,
) {
    let art_h = rect.h - BODY_H;

    // Card body.
    c.push(rounded_rect(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        theme.radius.lg,
        if hovered {
            theme.glass.surface_hover.0
        } else {
            theme.glass.surface.0
        },
    ));
    c.push(rounded_rect_stroke(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        theme.radius.lg,
        if hovered {
            theme.glass.edge.0
        } else {
            theme.glass.edge_soft.0
        },
        1.0,
    ));

    // Commander art, clipped to the tile's top corners. Partners split the
    // band in two.
    let images: Vec<Option<String>> = match &deck.commander_name_2 {
        Some(_) => vec![
            crop_rel(&deck.commander_image),
            crop_rel(&deck.commander_image_2),
        ],
        None => vec![crop_rel(&deck.commander_image)],
    };
    c.push(SceneNode::PushClip {
        x: rect.x,
        y: rect.y,
        w: rect.w,
        h: art_h,
    });
    let slot_w = rect.w / images.len() as f32;
    for (i, rel) in images.iter().enumerate() {
        let ix = rect.x + i as f32 * slot_w;
        match rel.as_deref().and_then(|r| art.get(r)) {
            Some(handle) => {
                // Cover: the crop is wider than the slot when two commanders
                // share the band; centre the overflow.
                let scale = (slot_w / 626.0).max(art_h / 457.0);
                let (dw, dh) = (626.0 * scale, 457.0 * scale);
                c.push(SceneNode::Image {
                    x: ix + (slot_w - dw) / 2.0,
                    y: rect.y + (art_h - dh) / 2.0,
                    w: dw,
                    h: dh,
                    image: handle,
                    corner_radius: 0.0,
                });
            }
            None => {
                c.push(SceneNode::Rect {
                    x: ix,
                    y: rect.y,
                    w: slot_w,
                    h: art_h,
                    color: theme.glass.surface_active.0,
                });
                let name = if i == 0 {
                    &deck.commander_name
                } else {
                    deck.commander_name_2.as_deref().unwrap_or("")
                };
                text(
                    c,
                    name,
                    12.0,
                    500,
                    ix + 10.0,
                    rect.y + art_h / 2.0 - 8.0,
                    theme.colors.text_dim.0,
                );
            }
        }
    }
    c.push(SceneNode::PopClip);

    // Tags ride on the art, top-left, as on the old cover labels.
    if let Some(tags) = &deck.tags {
        let mut tx = rect.x + 8.0;
        for tag in tags.split(',').map(str::trim).filter(|t| !t.is_empty()) {
            let w = tag.len() as f32 * 6.4 + 14.0;
            c.push(rounded_rect(
                tx,
                rect.y + 8.0,
                w,
                18.0,
                9.0,
                with_alpha(theme.colors.surface.0, 0.85),
            ));
            text(
                c,
                tag,
                10.0,
                600,
                tx + 7.0,
                rect.y + 11.0,
                theme.colors.text.0,
            );
            tx += w + 6.0;
        }
    }

    // Body: name, commander line, meta row.
    let bx = rect.x + 12.0;
    let mut by = rect.y + art_h + 10.0;
    text(c, &deck.name, 15.0, 600, bx, by, theme.colors.text.0);
    by += 22.0;
    let commander = match &deck.commander_name_2 {
        Some(second) => format!("{} + {}", deck.commander_name, second),
        None => deck.commander_name.clone(),
    };
    text(c, &commander, 12.0, 500, bx, by, theme.colors.text_dim.0);
    by += 18.0;
    if let Some(philosophy) = deck.philosophy.as_deref().filter(|p| !p.is_empty()) {
        text(
            c,
            philosophy,
            11.0,
            400,
            bx,
            by,
            theme.glass.text_placeholder.0,
        );
    }

    // Meta row: count pill, colour pips, win/loss.
    let my = rect.y + rect.h - 28.0;
    let ok = deck.total_cards == 100;
    let pill_c = if ok {
        theme.colors.success.0
    } else {
        theme.colors.danger.0
    };
    c.push(rounded_rect(
        bx,
        my,
        56.0,
        20.0,
        10.0,
        with_alpha(pill_c, 0.18),
    ));
    let label = format!("{}/100", deck.total_cards);
    text(c, &label, 11.0, 600, bx + 8.0, my + 4.0, pill_c);

    let mut px = bx + 66.0;
    let identity: Vec<char> = deck
        .color_identity
        .as_deref()
        .unwrap_or("")
        .chars()
        .filter(|c| "WUBRG".contains(*c))
        .collect();
    let identity = if identity.is_empty() {
        vec!['C']
    } else {
        identity
    };
    for letter in identity {
        c.push(rounded_rect(
            px,
            my + 2.0,
            16.0,
            16.0,
            8.0,
            pip_color(letter),
        ));
        let mut s = [0u8; 4];
        text(
            c,
            letter.encode_utf8(&mut s),
            10.0,
            700,
            px + 4.5,
            my + 4.5,
            [0.12, 0.10, 0.16, 1.0],
        );
        px += 20.0;
    }

    let wl = format!("{}V · {}D", deck.wins, deck.losses);
    text(
        c,
        &wl,
        11.0,
        500,
        rect.x + rect.w - wl.len() as f32 * 6.2 - 12.0,
        my + 4.0,
        theme.colors.text_dim.0,
    );
}
