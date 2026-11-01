//! The pricing card (PriceCard/index.tsx): the plan's title on the card's
//! wash (the featured plan's on the image, lifted), and the paper panel:
//! the description, the price box (the currency, the amount, the unit,
//! the plan's button) and the features, each after a check.

use super::{CatalogCard, CatalogKind, EDGE, Plan, boxed, icon, palette, text, tokens};
use crate::drawing::compositor::{Compositor, LayerId};
use crate::drawing::text::TextMeasurer;
use crate::styling::theme::Theme;
use crate::widgets::buttons::{Button, ButtonSize, ButtonVariant};
use crate::widgets::recipe::{gradient_fill, rounded_rect, shadow_drops, shadow_insets};
use crate::widgets::widget::Rect;

/// The plan's button: the kit's labels, the dark one for the plan to take
/// (`isSecondary`), the pale one elsewhere (:57-66).
pub(super) fn button(plan: Plan) -> Button {
    let (label, variant) = match plan {
        Plan::Free => ("You're on Creator", ButtonVariant::Primary),
        Plan::Creator => ("Current Plan", ButtonVariant::Primary),
        Plan::Studio => ("Get Studio", ButtonVariant::Secondary),
    };
    Button::new(label)
        .variant(variant)
        .disabled(plan == Plan::Free)
}

/// Where the card's parts land.
pub(super) struct Layout {
    pub panel: Rect,
    pub description: Rect,
    pub price_box: Rect,
    pub button: Rect,
    pub features: Rect,
}

/// The description's and the features' wrap width in a card `w` wide.
fn inner_w(w: f32, theme: &Theme) -> f32 {
    let p = &tokens(theme).price;
    (w - (EDGE + p.pad) * 2.0 - p.pad * 2.0).max(0.0)
}

fn parts(card: &CatalogCard) -> (&str, &[String]) {
    match &card.kind {
        CatalogKind::Price {
            description,
            features,
            ..
        } => (description, features),
        _ => ("", &[]),
    }
}

pub(super) fn layout(card: &CatalogCard, b: Rect, theme: &Theme) -> Layout {
    let p = &tokens(theme).price;
    let (description, features) = parts(card);
    let title_row = p.title_pad[1] * 2.0 + p.title.line_height;
    let wrap = inner_w(b.w, theme);
    let desc_h = TextMeasurer::measure_styled(description, &p.description.style(theme), Some(wrap))
        .1
        .max(p.description.line_height);
    let feature = p.feature.style(theme);
    let text_w = (wrap - p.check - p.check_pad[0] - p.check_pad[1]).max(0.0);
    let feats: f32 = features
        .iter()
        .map(|f| {
            TextMeasurer::measure_styled(f, &feature, Some(text_w))
                .1
                .max(p.feature.line_height)
        })
        .sum::<f32>()
        + p.feature_gap * features.len().saturating_sub(1) as f32;
    let row_h = p
        .amount
        .line_height
        .max(p.currency.line_height + p.currency_pad[1] * 2.0)
        .max(p.unit.line_height * 2.0 + p.unit_pad * 2.0);
    let btn_h = ButtonSize::Md.height(theme);
    let box_h = EDGE * 2.0 + p.pad * 2.0 + row_h + p.button_gap + btn_h;
    // The panel: `p-3`, the description `mb-3 p-3`, the box, the features
    // `mt-3 p-3 pt-2`.
    let x = b.x + EDGE + p.pad;
    let mut y = b.y + title_row + EDGE + p.pad;
    let description_at = Rect::new(x + p.pad, y + p.pad, wrap, desc_h);
    y += desc_h + p.pad * 3.0;
    let price_box = Rect::new(x, y, b.w - (EDGE + p.pad) * 2.0, box_h);
    let button = Rect::new(
        x + EDGE + p.pad,
        y + EDGE + p.pad + row_h + p.button_gap,
        (price_box.w - (EDGE + p.pad) * 2.0).max(0.0),
        btn_h,
    );
    y += box_h + p.features_gap;
    let features_at = Rect::new(x + p.pad, y + p.feature_gap, wrap, feats);
    y = features_at.bottom() + p.pad + p.pad + EDGE;
    Layout {
        panel: Rect::new(b.x, b.y + title_row, b.w, y - (b.y + title_row)),
        description: description_at,
        price_box,
        button,
        features: features_at,
    }
}

pub(super) fn size(card: &CatalogCard, w: f32, theme: &Theme) -> (f32, f32) {
    let w = tokens(theme).price.width.min(w);
    let l = layout(card, Rect::new(0.0, 0.0, w, 0.0), theme);
    (w, l.panel.bottom())
}

pub(super) fn render(card: &CatalogCard, c: &mut Compositor, b: Rect, theme: &Theme) {
    let CatalogKind::Price {
        plan,
        title,
        description,
        price,
        features,
    } = &card.kind
    else {
        return;
    };
    let (p, k) = (&tokens(theme).price, palette(theme));
    let l = layout(card, b, theme);
    let card_rect = Rect::new(b.x, b.y, b.w, l.panel.bottom() - b.y);
    // The featured plan sits on the image, lifted; the others on the wash.
    let title_color = if *plan == Plan::Creator {
        shadow_drops(c, LayerId::DEFAULT, card_rect, p.radius, &p.lift);
        let [top, bottom] = k.feature;
        c.push(gradient_fill(card_rect, p.radius, top.0, bottom.0));
        k.on_image
    } else {
        c.push(rounded_rect(
            card_rect.x,
            card_rect.y,
            card_rect.w,
            card_rect.h,
            p.radius,
            k.wash.0,
        ));
        k.ink
    };
    text(
        c,
        title,
        &p.title.style(theme),
        b.x + p.title_pad[0],
        b.y + p.title_pad[1],
        title_color,
        None,
    );
    boxed(c, l.panel, p.radius, k.paper, k.ring);
    text(
        c,
        description,
        &p.description.style(theme),
        l.description.x,
        l.description.y,
        k.ink,
        Some(l.description.w),
    );

    // The price box: its lift, the panel wash, its inset highlight.
    let bx = l.price_box;
    shadow_drops(c, LayerId::DEFAULT, bx, p.radius, &p.price_shadow);
    boxed(c, bx, p.radius, k.panel, k.ring);
    shadow_insets(c, LayerId::DEFAULT, bx, p.radius, &p.price_shadow);
    let row_y = bx.y + EDGE + p.pad;
    let mut x = bx.x + EDGE + p.pad;
    let currency = p.currency.style(theme);
    text(
        c,
        "$",
        &currency,
        x + p.currency_pad[0],
        row_y + p.currency_pad[1],
        k.muted,
        None,
    );
    x += TextMeasurer::measure_styled("$", &currency, None).0 + p.currency_pad[0] * 2.0;
    let amount = p.amount.style(theme);
    text(c, price, &amount, x, row_y, k.ink, None);
    x += TextMeasurer::measure_styled(price, &amount, None).0;
    text(
        c,
        "USD /\nmonth",
        &p.unit.style(theme),
        x + p.unit_pad,
        row_y + p.unit_pad,
        k.muted,
        None,
    );
    if let Some(button) = &card.button {
        if let Some(id) = card.sem.as_ref().and_then(|id| id.child("plan").ok()) {
            c.bind(&id, [l.button.x, l.button.y, l.button.w, l.button.h]);
        }
        button.render(c, l.button, theme);
    }

    // The features, each after a check.
    let feature = p.feature.style(theme);
    let text_x = l.features.x + p.check_pad[0] + p.check + p.check_pad[1];
    let text_w = (l.features.right() - text_x).max(0.0);
    let mut y = l.features.y;
    for f in features {
        let h = TextMeasurer::measure_styled(f, &feature, Some(text_w))
            .1
            .max(p.feature.line_height);
        icon(
            c,
            "check",
            p.check,
            k.ink,
            l.features.x + p.check_pad[0],
            y + (p.feature.line_height - p.check) / 2.0,
        );
        text(c, f, &feature, text_x, y, k.ink, Some(text_w));
        y += h + p.feature_gap;
    }
}
