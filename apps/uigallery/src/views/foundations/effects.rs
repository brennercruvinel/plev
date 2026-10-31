//! Effects page, the hoff core kit's DepthsPage: square cards, each a
//! surface lifted by one of the theme's shadow stacks (`theme.shadows`,
//! one card per elevation, and the button's own stack), named in its chip.
//! A second group keeps the compositor's other primitives, each in a card
//! of the same mold: linear gradients, backdrop blur (the theme's
//! `effects.blur_sigma`), the clip stack and tessellated vector shapes
//! (`engine::drawing::path::PathBuilder`). Everything is static, so an idle
//! page settles (render-on-demand contract).

use engine::drawing::compositor::{Compositor, LayerId, SceneNode};
use engine::drawing::path::PathBuilder;
use engine::styling::theme::{Elevation, ShadowSpec, Theme};
use engine::widgets::prelude::{
    EventResult, Rect, WidgetEvent, gradient_fill, rounded_rect, rounded_rect_stroke, shadow_drops,
    shadow_insets,
};

use super::super::components::showcase::{self, Grid, GroupLayout};

/// Cards to a row at full width (the kit's `span={4}`).
const SPAN: usize = 4;
/// The lifted surface takes this share of its card's side (the kit's
/// `size-32` in a card of about 240).
const SURFACE_SHARE: f32 = 0.5;
/// The compositor samples, one card each.
const SAMPLES: [&str; 4] = ["Gradient", "Backdrop blur", "Clip", "Paths"];

/// The depths, from the theme: a name and the css box-shadow list (first
/// layer on top; the theme lists its stacks bottom first).
fn depths(theme: &Theme) -> Vec<(&'static str, Vec<ShadowSpec>)> {
    let s = &theme.shadows;
    let css = |layers: &[ShadowSpec]| layers.iter().rev().copied().collect::<Vec<_>>();
    let stack = |e: Elevation| css(s.stack(e));
    let mut out = vec![
        ("Surface", Vec::new()),
        ("Raised", stack(Elevation::Raised)),
        ("Floating", stack(Elevation::Floating)),
        ("Overlay", stack(Elevation::Overlay)),
        ("Hint", stack(Elevation::Hint)),
        ("Card", css(&s.card)),
        ("Button", theme.button.primary.rest.shadow.clone()),
    ];
    // The glass themes light their surfaces from inside; the core kit
    // does not, and shows no card for it.
    if let Some(light) = theme.glass.keylight {
        out.push(("Key-light", vec![light]));
    }
    out
}

pub struct EffectsPage;

impl EffectsPage {
    pub fn new() -> Self {
        Self
    }

    /// The narrowest a card gets: half the theme's narrowest card.
    fn min_w(theme: &Theme) -> f32 {
        theme.size.card_min_w / 2.0
    }

    /// Depths, then the compositor samples, in square cards.
    fn layout(content: Rect, theme: &Theme) -> Vec<GroupLayout> {
        let side = showcase::card_width_at(SPAN, Self::min_w(theme), content, theme);
        let counts = [depths(theme).len(), SAMPLES.len()];
        let grids = counts.map(|n| Grid {
            span: SPAN,
            min_w: Self::min_w(theme),
            heights: vec![side; n],
        });
        showcase::layout_grids(grids, content, theme)
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        showcase::content_height(&Self::layout(content, theme), content)
    }

    /// Static gallery page: no widgets, nothing to hit-test.
    pub fn handle_event(&mut self, _event: &WidgetEvent, _content: Rect) -> EventResult {
        EventResult::IGNORED
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let layout = Self::layout(content, theme);
        if let [depth, samples] = layout.as_slice() {
            showcase::title(c, "Depths", content, depth.title_y, theme);
            for ((name, stack), card) in depths(theme).iter().zip(&depth.cards) {
                showcase::card(c, *card, name, theme);
                lifted(c, *card, stack, theme);
            }
            showcase::rule(c, content, depth.rule_y, theme);
            showcase::title(c, "Compositor", content, samples.title_y, theme);
            for (name, card) in SAMPLES.iter().zip(&samples.cards) {
                showcase::card(c, *card, name, theme);
                let stage = stage(*card, theme);
                match *name {
                    "Gradient" => gradients(c, stage, theme),
                    "Backdrop blur" => frosted(c, stage, theme),
                    "Clip" => clipped(c, stage, theme),
                    _ => shapes(c, stage, theme),
                }
            }
        }
    }
}

/// The square the sample draws in: centered under the chip band.
fn stage(card: Rect, theme: &Theme) -> Rect {
    let side = (card.w.min(card.h) * SURFACE_SHARE).round();
    let at = showcase::centered((side, side), card);
    Rect::new(at.x, at.y + theme.spacing.sm, side, side)
}

/// A paper surface with a hairline (`bg-[#FCFCFC] border border-[#ECECEC]`)
/// lifted by `stack`, drops under it and insets over it.
fn lifted(c: &mut Compositor, card: Rect, stack: &[ShadowSpec], theme: &Theme) {
    let s = stage(card, theme);
    let radius = theme.shape.card;
    shadow_drops(c, LayerId::DEFAULT, s, radius, stack);
    c.push(rounded_rect(
        s.x,
        s.y,
        s.w,
        s.h,
        radius,
        theme.colors.surface.0,
    ));
    shadow_insets(c, LayerId::DEFAULT, s, radius, stack);
    c.push(rounded_rect_stroke(
        s.x,
        s.y,
        s.w,
        s.h,
        radius,
        theme.colors.divider.0,
        theme.control.edge_width,
    ));
}

/// Two-stop linear gradients at four css angles, a quarter each.
fn gradients(c: &mut Compositor, s: Rect, theme: &Theme) {
    let k = &theme.colors;
    let pairs = [
        (k.accent, k.info, 0.0),
        (k.success, k.accent, 45.0),
        (k.warning, k.danger, 90.0),
        (k.info, k.success, 135.0),
    ];
    let gap = theme.spacing.xs;
    let half = (s.w - gap) / 2.0;
    for (i, (from, to, angle)) in pairs.into_iter().enumerate() {
        let x = s.x + (i % 2) as f32 * (half + gap);
        let y = s.y + (i / 2) as f32 * (half + gap);
        let mut node = gradient_fill(Rect::new(x, y, half, half), theme.radius.sm, from.0, to.0);
        if let SceneNode::GradientRect { angle_deg, .. } = &mut node {
            *angle_deg = angle;
        }
        c.push(node);
    }
}

/// Bands of color with a frosted pane over their middle: what is under the
/// pane blurs by the theme's `effects.blur_sigma`.
fn frosted(c: &mut Compositor, s: Rect, theme: &Theme) {
    let k = &theme.colors;
    let colors = [k.accent, k.success, k.warning, k.danger, k.info];
    let band = s.w / colors.len() as f32;
    for (i, color) in colors.iter().enumerate() {
        c.push(rounded_rect(
            s.x + i as f32 * band,
            s.y,
            band,
            s.h,
            0.0,
            color.0,
        ));
    }
    let pane = s.inset(s.w / 4.0);
    c.push(SceneNode::BackdropBlur {
        x: pane.x,
        y: pane.y,
        w: pane.w,
        h: pane.h,
        corner_radius: theme.radius.md,
        sigma: theme.effects.blur_sigma,
    });
    c.push(rounded_rect_stroke(
        pane.x,
        pane.y,
        pane.w,
        pane.h,
        theme.radius.md,
        theme.glass.edge.0,
        theme.control.edge_width,
    ));
}

/// Rows wider and taller than the frame, cut to it by the clip stack.
fn clipped(c: &mut Compositor, s: Rect, theme: &Theme) {
    c.push(SceneNode::PushClip {
        x: s.x,
        y: s.y,
        w: s.w,
        h: s.h,
    });
    let row = theme.spacing.lg;
    let gap = theme.spacing.xs;
    // Half a row above the frame and past its far edge: both cut.
    let mut y = s.y - row / 2.0;
    let mut i = 0;
    while y < s.bottom() {
        let fill = if i % 2 == 0 {
            theme.colors.bg_hover
        } else {
            theme.colors.border_active
        };
        c.push(rounded_rect(
            s.x + gap,
            y,
            s.w,
            row,
            theme.radius.sm,
            fill.0,
        ));
        y += row + gap;
        i += 1;
    }
    c.push(SceneNode::PopClip);
    c.push(rounded_rect_stroke(
        s.x,
        s.y,
        s.w,
        s.h,
        0.0,
        theme.colors.divider.0,
        theme.control.edge_width,
    ));
}

/// A circle, a star, a hexagon and a ring, tessellated, a quarter each.
fn shapes(c: &mut Compositor, s: Rect, theme: &Theme) {
    let k = &theme.colors;
    let colors = [k.accent, k.success, k.warning, k.info];
    let q = s.w / 2.0;
    for (i, color) in colors.iter().enumerate() {
        let cx = s.x + (i % 2) as f32 * q + q / 2.0;
        let cy = s.y + (i / 2) as f32 * q + q / 2.0;
        let r = q * 0.4;
        let path = match i {
            0 => PathBuilder::circle(cx, cy, r),
            1 => polygon(cx, cy, r, r * 0.45, 10),
            2 => polygon(cx, cy, r, r, 6),
            _ => ring(cx, cy, r, r * 0.55),
        };
        c.draw_path(path.fill(color.0));
    }
}

/// A closed polygon of `n` vertices alternating between the two radii
/// (equal radii: a regular polygon; unequal: a star), point up.
fn polygon(cx: f32, cy: f32, r_out: f32, r_in: f32, n: u32) -> PathBuilder {
    let mut b = PathBuilder::new();
    for i in 0..n {
        let a = std::f32::consts::TAU * (i as f32 / n as f32) - std::f32::consts::FRAC_PI_2;
        let r = if i % 2 == 0 { r_out } else { r_in };
        let (px, py) = (cx + r * a.cos(), cy + r * a.sin());
        b = if i == 0 {
            b.move_to(px, py)
        } else {
            b.line_to(px, py)
        };
    }
    b.close()
}

/// Filled ring: outer circle one way, inner circle back (even-odd fill).
fn ring(cx: f32, cy: f32, r_out: f32, r_in: f32) -> PathBuilder {
    let n = 64;
    let point = |r: f32, i: u32| {
        let a = std::f32::consts::TAU * (i as f32 / n as f32);
        (cx + r * a.cos(), cy + r * a.sin())
    };
    let (x, y) = point(r_out, 0);
    let mut b = PathBuilder::new().move_to(x, y);
    for i in 1..=n {
        let (x, y) = point(r_out, i);
        b = b.line_to(x, y);
    }
    for i in (0..=n).rev() {
        let (x, y) = point(r_in, i);
        b = b.line_to(x, y);
    }
    b.close()
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine::widgets::prelude::shadow_node;

    fn nodes(theme: &Theme, w: f32) -> Vec<SceneNode> {
        let mut c = Compositor::new();
        EffectsPage::new().render(&mut c, Rect::new(0.0, 0.0, w, 800.0), theme);
        c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec()
    }

    /// The shadows drawn are the theme's stacks, layer for layer: no blur
    /// the page made up. A layer whose spread eats the sample casts
    /// nothing, as in css (the large card's `-64px` on its 128 box).
    #[test]
    fn every_shadow_is_a_theme_stack() {
        for theme in [Theme::hoff_core(), Theme::hoff(), Theme::dark()] {
            let content = Rect::new(0.0, 0.0, 1000.0, 800.0);
            let drawn = nodes(&theme, content.w)
                .iter()
                .filter(|n| matches!(n, SceneNode::Shadow { .. }))
                .count();
            let cards = &EffectsPage::layout(content, &theme)[0].cards;
            let layers: usize = depths(&theme)
                .iter()
                .zip(cards)
                .map(|((_, stack), card)| {
                    let s = stage(*card, &theme);
                    stack
                        .iter()
                        .filter(|spec| shadow_node(s, theme.shape.card, spec).is_some())
                        .count()
                })
                .sum();
            assert_eq!(drawn, layers);
            assert!(layers > 0);
        }
    }

    #[test]
    fn the_compositor_samples_keep_every_primitive() {
        let theme = Theme::hoff_core();
        let all = nodes(&theme, 1000.0);
        let count = |m: fn(&SceneNode) -> bool| all.iter().filter(|n| m(n)).count();
        assert_eq!(count(|n| matches!(n, SceneNode::GradientRect { .. })), 4);
        assert_eq!(count(|n| matches!(n, SceneNode::BackdropBlur { .. })), 1);
        assert_eq!(count(|n| matches!(n, SceneNode::PushClip { .. })), 1);
        assert_eq!(count(|n| matches!(n, SceneNode::Path { .. })), 4);
    }

    /// Square cards inside the content from 280 to 1600 wide, and every
    /// sample inside its card.
    #[test]
    fn cards_fit_from_narrow_to_wide() {
        let theme = Theme::hoff_core();
        for w in (280..=1600).step_by(8) {
            let content = Rect::new(0.0, 0.0, w as f32, 800.0);
            for group in EffectsPage::layout(content, &theme) {
                for card in &group.cards {
                    assert!(card.right() <= content.right() + 0.5, "{w}: {card:?}");
                    assert!((card.w - card.h).abs() < 0.5);
                    let s = stage(*card, &theme);
                    assert!(s.bottom() <= card.bottom() && s.x >= card.x, "{w}: {s:?}");
                }
            }
        }
    }
}
