//! The card's shell and the little pieces every family draws with: the
//! deck shell, a text run, the legend dot, a skeleton line, a progress
//! track.

use super::{Card, Metrics};
use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::theme::Theme;
use crate::widgets::recipe::{glass_pill, inset_keylight};
use crate::widgets::widget::{Rect, with_alpha};

impl Card {
    /// The HOFF content-card shell, one discreet recipe for the whole
    /// deck, taken 1:1 from the live POST card (`Post.module.sass`: the
    /// card radius, `glass.surface` at rest and `surface_hover` hovered,
    /// NO `backdrop-filter`, NO `box-shadow`, NO border). A whisper
    /// lighter than the page, never a frosted panel, closed by the faint
    /// inset key-light. The reference's frost is reserved for pills,
    /// search and menus (real glass), not for cards of content.
    pub(super) fn deck_shell(&self, c: &mut Compositor, layer: LayerId, b: Rect, theme: &Theme) {
        let glass = &theme.glass;
        let radius = Metrics::of(theme).radius;
        let (fill, edge) = if self.hovered {
            (glass.surface_hover.0, glass.edge.0)
        } else {
            (glass.surface.0, glass.edge_soft.0)
        };
        for node in glass_pill(b, radius, edge, theme.control.edge_width, fill, theme) {
            c.push_to_layer(layer, node);
        }
        inset_keylight(c, layer, b, radius, theme);
    }

    // -- Shared little pieces ---------------------------------------------------

    #[allow(clippy::too_many_arguments)]
    pub(super) fn text(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        s: &str,
        style: &TextStyle,
        x: f32,
        y: f32,
        color: [f32; 4],
        max_w: Option<f32>,
    ) {
        c.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::from_style(s, style, max_w),
                x,
                y,
                color,
            },
        );
    }

    /// How tall `s` is in `style` wrapped at `w`: one line at least.
    pub(super) fn wrapped_h(s: &str, style: &TextStyle, w: f32) -> f32 {
        let (_, h) = TextMeasurer::measure_styled(s, style, Some(w.max(0.0)));
        h.max(style.line_height)
    }

    /// `s` on one line no wider than `w`, cut with an ellipsis; nothing
    /// where not even the ellipsis fits.
    pub(super) fn fit(s: &str, style: &TextStyle, w: f32) -> String {
        let cut = TextMeasurer::truncate_to_width(s, style, w.max(0.0));
        if TextMeasurer::measure_styled(&cut, style, None).0 <= w {
            cut
        } else {
            String::new()
        }
    }

    /// Legend / online dot: the card's hollow dot (`theme.disclosure.
    /// card`), or its solid one when `solid`.
    pub(super) fn dot(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        x: f32,
        y: f32,
        solid: bool,
        theme: &Theme,
    ) {
        let marks = &theme.disclosure.card;
        let d = Metrics::of(theme).dot;
        if solid {
            c.push_to_layer(
                layer,
                SceneNode::GradientRect {
                    x,
                    y,
                    w: d,
                    h: d,
                    color: marks.dot_solid[0].0,
                    color2: marks.dot_solid[1].0,
                    angle_deg: 180.0,
                    corner_radius: d / 2.0,
                    border_width: 0.0,
                    border_color: [0.0; 4],
                },
            );
        } else {
            c.push_to_layer(
                layer,
                SceneNode::RoundedRect {
                    x,
                    y,
                    w: d,
                    h: d,
                    color: marks.dot_fill.0,
                    corner_radius: d / 2.0,
                    border_width: theme.control.edge_width,
                    border_color: marks.dot_ring.0,
                },
            );
        }
    }

    /// Skeleton line: the HOFF "fake text" bar, as thick as the progress
    /// fill.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn skeleton(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        x: f32,
        y: f32,
        w: f32,
        color: [f32; 4],
        theme: &Theme,
    ) {
        let h = theme.control.progress_fill;
        c.push_to_layer(
            layer,
            SceneNode::RoundedRect {
                x,
                y,
                w,
                h,
                color,
                corner_radius: h / 2.0,
                border_width: 0.0,
                border_color: [0.0; 4],
            },
        );
    }

    /// The HOFF progress track + gradient fill (the same recipe as
    /// `ProgressBar`, inline so the card stays a single layer push).
    pub(super) fn progress(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        track: Rect,
        p: f32,
        theme: &Theme,
    ) {
        let glass = &theme.glass;
        let text = theme.colors.text;
        let fill_h = theme.control.progress_fill.min(track.h);
        c.push_to_layer(
            layer,
            SceneNode::RoundedRect {
                x: track.x,
                y: track.y,
                w: track.w,
                h: track.h,
                color: [0.0; 4],
                corner_radius: track.h / 2.0,
                border_width: theme.control.edge_width_strong,
                border_color: glass.surface_active.0,
            },
        );
        let inset = (track.h - fill_h) / 2.0;
        let fill_w = (track.w - inset * 2.0) * p;
        if fill_w >= 1.0 {
            c.push_to_layer(
                layer,
                SceneNode::GradientRect {
                    x: track.x + inset,
                    y: track.y + inset,
                    w: fill_w,
                    h: fill_h,
                    color: with_alpha(text, 0.0),
                    color2: glass.text_faint.0,
                    angle_deg: 90.0,
                    corner_radius: fill_h / 2.0,
                    border_width: 0.0,
                    border_color: [0.0; 4],
                },
            );
        }
    }
}
