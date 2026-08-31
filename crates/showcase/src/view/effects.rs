//! Effects section: the compositor's visual primitives as design-system
//! tokens — analytic shadows (CSS-like blur radii, incl. the theme's own
//! `effects.shadow_sigma`), linear gradients, backdrop blur, the clip stack
//! and tessellated vector shapes (`engine::path::PathBuilder`). Absorbs the
//! `visual` and `sdf_shapes` engine examples; everything is static, so an
//! idle section settles (render-on-demand contract).

use engine::compositor::{
    Compositor, GradientRectParams, RoundedRectParams, SceneNode, ShadowParams, TextNodeKey,
};
use engine::path::PathBuilder;
use engine::text::TextStyle;
use engine::theme::Theme;
use engine::ui::widgets::{EventResult, Rect, WidgetEvent};

use super::{group_label, panel, text};

const GAP: f32 = 12.0;
const LABEL_H: f32 = 30.0;
const ROW_GAP: f32 = 28.0;
const SHADOW_CARD_W: f32 = 150.0;
const SHADOW_CARD_H: f32 = 90.0;
/// Extra horizontal room so wide shadows are not clipped by the next card.
const SHADOW_STRIDE: f32 = 190.0;
const GRADIENT_W: f32 = 150.0;
const GRADIENT_H: f32 = 70.0;
const BLUR_PANEL_H: f32 = 120.0;
const CLIP_PANEL_H: f32 = 140.0;
const SHAPE_BOX: f32 = 90.0;

pub struct EffectsSection;

/// Rects for everything the section draws, top to bottom.
struct Layout {
    labels: Vec<(&'static str, f32)>,
    /// (rect, blur radius, inset, caption).
    shadows: Vec<(Rect, f32, bool, String)>,
    /// (rect, angle).
    gradients: Vec<(Rect, f32)>,
    blur_panel: Rect,
    clip_panel: Rect,
    shapes: Vec<(Rect, &'static str)>,
    total_h: f32,
}

fn code(text: &str, size: f32) -> (TextStyle, String) {
    (
        TextStyle::new(size)
            .with_line_height(size * 1.4)
            .with_family("Inclusive Sans"),
        text.to_string(),
    )
}

impl EffectsSection {
    pub fn new() -> Self {
        Self
    }

    fn layout(content: Rect, theme: &Theme) -> Layout {
        let mut labels = Vec::new();
        let mut y = content.y;

        // Shadow cards flow left to right and wrap on narrow widths.
        labels.push(("ANALYTIC SHADOWS — NO BLUR PASS", y));
        let mut shadows = Vec::new();
        let mut x = content.x;
        let mut sy = y + LABEL_H + 24.0; // headroom so the blur is not cut above
        let mut shadow_specs: Vec<(f32, bool, String)> = [4.0f32, 12.0, 24.0, 48.0]
            .into_iter()
            .map(|blur| (blur, false, format!("blur {blur}")))
            .collect();
        shadow_specs.push((
            theme.effects.shadow_sigma * 2.0,
            false,
            format!("theme sigma {:.0}", theme.effects.shadow_sigma),
        ));
        shadow_specs.push((16.0, true, "inset".to_string()));
        for (blur, inset, caption) in shadow_specs {
            if x > content.x && x + SHADOW_CARD_W > content.x + content.w {
                x = content.x;
                sy += SHADOW_CARD_H + 48.0;
            }
            shadows.push((
                Rect::new(x, sy, SHADOW_CARD_W, SHADOW_CARD_H),
                blur,
                inset,
                caption,
            ));
            x += SHADOW_STRIDE;
        }
        y = sy + SHADOW_CARD_H + 40.0 + ROW_GAP;

        labels.push(("LINEAR GRADIENTS — CSS ANGLES", y));
        let gradients = [0.0f32, 45.0, 90.0, 135.0]
            .into_iter()
            .enumerate()
            .map(|(i, angle)| {
                (
                    Rect::new(
                        content.x + i as f32 * (GRADIENT_W + GAP),
                        y + LABEL_H,
                        GRADIENT_W,
                        GRADIENT_H,
                    ),
                    angle,
                )
            })
            .collect();
        y += LABEL_H + GRADIENT_H + 24.0 + ROW_GAP;

        labels.push(("BACKDROP BLUR — FROSTED GLASS", y));
        let blur_panel = Rect::new(content.x, y + LABEL_H, content.w, BLUR_PANEL_H);
        y += LABEL_H + BLUR_PANEL_H + ROW_GAP;

        labels.push(("CLIP STACK — CONTENT LARGER THAN THE PANEL", y));
        let clip_panel = Rect::new(content.x, y + LABEL_H, content.w, CLIP_PANEL_H);
        y += LABEL_H + CLIP_PANEL_H + ROW_GAP;

        labels.push(("VECTOR SHAPES — PATHBUILDER TESSELLATION", y));
        let shapes = ["circle", "ring", "star", "polygon"]
            .into_iter()
            .enumerate()
            .map(|(i, name)| {
                (
                    Rect::new(
                        content.x + i as f32 * (SHAPE_BOX + GAP),
                        y + LABEL_H,
                        SHAPE_BOX,
                        SHAPE_BOX,
                    ),
                    name,
                )
            })
            .collect();
        y += LABEL_H + SHAPE_BOX + 24.0;

        Layout {
            labels,
            shadows,
            gradients,
            blur_panel,
            clip_panel,
            shapes,
            total_h: y - content.y,
        }
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        Self::layout(content, theme).total_h + GAP
    }

    /// Static gallery page: no widgets, nothing to hit-test.
    pub fn handle_event(&mut self, _event: &WidgetEvent, _content: Rect) -> EventResult {
        EventResult::IGNORED
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let l = Self::layout(content, theme);
        for (label, y) in &l.labels {
            group_label(c, label, content.x, *y, theme);
        }

        // Analytic shadows (Evan Wallace approximation) under a glass card.
        for (rect, blur, inset, caption) in &l.shadows {
            c.draw_shadow(ShadowParams {
                x: rect.x,
                y: rect.y,
                w: rect.w,
                h: rect.h,
                corner_radius: theme.radius.md,
                blur_radius: *blur,
                offset: [0.0, 6.0],
                color: theme.effects.shadow_color.0,
                inset: *inset,
            });
            c.draw_rounded_rect(RoundedRectParams {
                x: rect.x,
                y: rect.y,
                w: rect.w,
                h: rect.h,
                color: theme.colors.bg_panel.0,
                corner_radius: theme.radius.md,
                border_width: 1.0,
                border_color: theme.glass.edge_soft.0,
            });
            let (style, caption) = code(caption, 12.0);
            c.draw_text(
                TextNodeKey::from_style(&caption, &style, None),
                rect.x + 14.0,
                rect.y + rect.h / 2.0 - 8.0,
                theme.colors.text.0,
            );
        }

        // Two-stop linear gradients at CSS angles (accent pairs per theme).
        let pairs = [
            (theme.colors.accent.0, theme.colors.info.0),
            (theme.colors.success.0, theme.colors.accent.0),
            (theme.colors.warning.0, theme.colors.danger.0),
            (theme.colors.info.0, theme.colors.success.0),
        ];
        for ((rect, angle), (from, to)) in l.gradients.iter().zip(pairs) {
            c.draw_gradient_rect(GradientRectParams {
                x: rect.x,
                y: rect.y,
                w: rect.w,
                h: rect.h,
                color: from,
                color2: to,
                angle_deg: *angle,
                corner_radius: theme.radius.sm,
                border_width: 0.0,
                border_color: [0.0; 4],
            });
            text(
                c,
                &format!("{angle:.0} deg"),
                12.0,
                400,
                rect.x,
                rect.y + rect.h + 6.0,
                theme.glass.text_faint.0,
            );
        }

        // Backdrop blur: a busy strip of gradient chips, then a frosted
        // panel over the middle. What is below the region blurs; what is
        // pushed after stays sharp.
        panel(c, l.blur_panel, theme);
        c.push(SceneNode::PushClip {
            x: l.blur_panel.x,
            y: l.blur_panel.y,
            w: l.blur_panel.w,
            h: l.blur_panel.h,
        });
        let chip_w = 64.0;
        let mut cx = l.blur_panel.x + 10.0;
        let mut i = 0;
        while cx + chip_w < l.blur_panel.x + l.blur_panel.w - 10.0 {
            let (from, to) = pairs[i % pairs.len()];
            c.draw_gradient_rect(GradientRectParams {
                x: cx,
                y: l.blur_panel.y + 18.0,
                w: chip_w,
                h: BLUR_PANEL_H - 36.0,
                color: from,
                color2: to,
                angle_deg: 90.0 + i as f32 * 30.0,
                corner_radius: theme.radius.sm,
                border_width: 0.0,
                border_color: [0.0; 4],
            });
            cx += chip_w + 10.0;
            i += 1;
        }
        let frost = Rect::new(
            l.blur_panel.x + l.blur_panel.w * 0.3,
            l.blur_panel.y + 14.0,
            l.blur_panel.w * 0.4,
            BLUR_PANEL_H - 28.0,
        );
        c.push(SceneNode::BackdropBlur {
            x: frost.x,
            y: frost.y,
            w: frost.w,
            h: frost.h,
            corner_radius: theme.radius.md,
            sigma: theme.effects.blur_sigma,
        });
        c.push(engine::ui::widgets::rounded_rect_stroke(
            frost.x,
            frost.y,
            frost.w,
            frost.h,
            theme.radius.md,
            theme.glass.edge.0,
            1.0,
        ));
        text(
            c,
            &format!("backdrop blur — sigma {:.0}", theme.effects.blur_sigma),
            13.0,
            500,
            frost.x + 16.0,
            frost.y + frost.h / 2.0 - 9.0,
            theme.colors.text.0,
        );
        c.push(SceneNode::PopClip);

        // Clip stack: oversized rows scissored to the panel bounds.
        panel(c, l.clip_panel, theme);
        c.push(SceneNode::PushClip {
            x: l.clip_panel.x,
            y: l.clip_panel.y,
            w: l.clip_panel.w,
            h: l.clip_panel.h,
        });
        let offset = 40.0; // static scroll offset: edges cut on both sides
        for row in 0..8 {
            let ry = l.clip_panel.y + 8.0 + row as f32 * 28.0 - offset;
            c.push(engine::ui::widgets::rounded_rect(
                l.clip_panel.x + 8.0,
                ry,
                l.clip_panel.w + 60.0, // wider than the panel on purpose
                22.0,
                theme.radius.sm,
                if row % 2 == 0 {
                    theme.glass.surface_active.0
                } else {
                    theme.glass.surface.0
                },
            ));
            text(
                c,
                &format!("row {row} — clipped to the panel bounds"),
                12.0,
                400,
                l.clip_panel.x + 16.0,
                ry + 4.0,
                theme.colors.text_dim.0,
            );
        }
        c.push(SceneNode::PopClip);

        // Vector shapes tessellated by lyon (engine::path::PathBuilder).
        let shape_colors = [
            theme.colors.accent.0,
            theme.colors.success.0,
            theme.colors.warning.0,
            theme.colors.info.0,
        ];
        for ((rect, name), color) in l.shapes.iter().zip(shape_colors) {
            let cx = rect.x + rect.w / 2.0;
            let cy = rect.y + rect.h / 2.0;
            let r = rect.w / 2.0 - 12.0;
            match *name {
                "circle" => c.draw_path(PathBuilder::circle(cx, cy, r).fill(color)),
                "ring" => draw_ring(c, cx, cy, r, r * 0.55, color),
                "star" => draw_star(c, cx, cy, r, r * 0.45, 5, color),
                _ => draw_polygon(c, cx, cy, r, 6, color),
            }
            text(
                c,
                name,
                12.0,
                400,
                rect.x + 12.0,
                rect.y + rect.h + 6.0,
                theme.glass.text_faint.0,
            );
        }
    }
}

/// Filled ring: outer circle one way, inner circle back (even-odd fill).
fn draw_ring(c: &mut Compositor, cx: f32, cy: f32, r_out: f32, r_in: f32, color: [f32; 4]) {
    let n = 64;
    let mut b = PathBuilder::new();
    for i in 0..=n {
        let a = std::f32::consts::TAU * (i as f32 / n as f32);
        b = if i == 0 {
            b.move_to(cx + r_out * a.cos(), cy + r_out * a.sin())
        } else {
            b.line_to(cx + r_out * a.cos(), cy + r_out * a.sin())
        };
    }
    for i in (0..=n).rev() {
        let a = std::f32::consts::TAU * (i as f32 / n as f32);
        b = b.line_to(cx + r_in * a.cos(), cy + r_in * a.sin());
    }
    c.draw_path(b.close().fill(color));
}

fn draw_star(
    c: &mut Compositor,
    cx: f32,
    cy: f32,
    r_out: f32,
    r_in: f32,
    points: u32,
    color: [f32; 4],
) {
    let mut b = PathBuilder::new();
    let total = points * 2;
    for i in 0..total {
        let a = std::f32::consts::TAU * (i as f32 / total as f32) - std::f32::consts::FRAC_PI_2;
        let r = if i % 2 == 0 { r_out } else { r_in };
        let (px, py) = (cx + r * a.cos(), cy + r * a.sin());
        b = if i == 0 {
            b.move_to(px, py)
        } else {
            b.line_to(px, py)
        };
    }
    c.draw_path(b.close().fill(color));
}

fn draw_polygon(c: &mut Compositor, cx: f32, cy: f32, r: f32, sides: u32, color: [f32; 4]) {
    let mut b = PathBuilder::new();
    for i in 0..sides {
        let a = std::f32::consts::TAU * (i as f32 / sides as f32) - std::f32::consts::FRAC_PI_2;
        let (px, py) = (cx + r * a.cos(), cy + r * a.sin());
        b = if i == 0 {
            b.move_to(px, py)
        } else {
            b.line_to(px, py)
        };
    }
    c.draw_path(b.close().fill(color));
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine::compositor::LayerId;

    fn narrow() -> Rect {
        Rect::new(288.0, 80.0, 472.0, 440.0)
    }

    fn wide() -> Rect {
        Rect::new(288.0, 80.0, 1272.0, 840.0)
    }

    #[test]
    fn renders_at_narrow_and_wide_without_overflow() {
        let theme = Theme::hoff();
        let section = EffectsSection::new();
        for content in [narrow(), wide()] {
            let l = EffectsSection::layout(content, &theme);
            for (rect, ..) in &l.shadows {
                assert!(rect.x + rect.w <= content.x + content.w + 0.5);
            }
            let mut c = Compositor::new();
            section.render(&mut c, content, &theme);
            let nodes = c.layer(LayerId::DEFAULT).unwrap().nodes();
            assert!(nodes.len() > 30, "thin scene: {} nodes", nodes.len());
            assert!(section.content_height(content, &theme) > content.h);
        }
    }

    #[test]
    fn scene_contains_every_effect_primitive() {
        let theme = Theme::hoff();
        let section = EffectsSection::new();
        let mut c = Compositor::new();
        section.render(&mut c, wide(), &theme);
        let nodes = c.layer(LayerId::DEFAULT).unwrap().nodes();
        let count = |m: fn(&SceneNode) -> bool| nodes.iter().filter(|n| m(n)).count();
        assert!(count(|n| matches!(n, SceneNode::Shadow { .. })) >= 6);
        assert!(count(|n| matches!(n, SceneNode::GradientRect { .. })) >= 8);
        assert!(count(|n| matches!(n, SceneNode::BackdropBlur { .. })) == 1);
        assert!(count(|n| matches!(n, SceneNode::PushClip { .. })) >= 2);
        assert!(count(|n| matches!(n, SceneNode::Path { .. })) == 4);
    }

    #[test]
    fn shadow_sigma_card_tracks_the_theme_token() {
        // Swapping themes must move the token card's blur (measured token,
        // not a constant): hoff and dark do not share shadow_sigma.
        let dark = Theme::dark();
        let l = EffectsSection::layout(wide(), &dark);
        let (_, blur, _, caption) = &l.shadows[4];
        assert_eq!(*blur, dark.effects.shadow_sigma * 2.0);
        assert!(caption.contains(&format!("{:.0}", dark.effects.shadow_sigma)));
    }
}
