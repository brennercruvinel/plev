//! HOFF cards: one glass shell, many previews.
//!
//! The reference deck (~60 variants) shares a single recipe — graphite
//! glass shell (radius 32, `rgba(40,40,40,.8)`), top-lit edge border,
//! deep drop shadow — and differs only in the preview. [`Card`] ships the
//! shell plus the most reusable preview families as [`CardVariant`]s:
//! stat, profile, media, list, chart and CTA.
//!
//! Rendering uses the SDF pipeline (gradients, analytic shadows), images
//! and text only — no path icons — so every element stacks correctly
//! within a single layer (quads → shadows → SDF → images → text).

use crate::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::gpu::image::ImageHandle;
use crate::text::{TextMeasurer, TextStyle};
use crate::theme::{Theme, TypographyScale};

use super::{EventResult, Rect, WidgetEvent, glass_pill, with_alpha};

/// HOFF card default width.
pub const CARD_W: f32 = 368.0;
const PAD: f32 = 16.0;
/// Details column inset (inner 16 + details 16).
const DETAILS: f32 = 32.0;

/// One row of a [`CardVariant::List`] (expense-tracker / download rows).
#[derive(Clone, Debug)]
pub struct CardListRow {
    pub label: String,
    /// Right-aligned caption-sm value ("$128.00", "84%").
    pub trailing: String,
    /// Progress 0..=1 renders the 4px gradient bar under the label.
    pub progress: Option<f32>,
    /// Active rows get the 40%-white border (HOFF selected state).
    pub active: bool,
}

impl CardListRow {
    pub fn new(label: impl Into<String>, trailing: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            trailing: trailing.into(),
            progress: None,
            active: false,
        }
    }

    pub fn progress(mut self, value: f32) -> Self {
        self.progress = Some(value.clamp(0.0, 1.0));
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }
}

/// Preview families from the HOFF card deck.
#[derive(Clone, Debug)]
pub enum CardVariant {
    /// Card 2/17: tick ruler, 32px headline, dot label, delta chip.
    Stat {
        value: String,
        label: String,
        /// Small chip at the top right ("+12.4%"); the bool picks the
        /// green (true) / red (false) accent.
        delta: Option<(String, bool)>,
    },
    /// Follower card: avatar, name/username, two-line bio, action pill.
    Profile {
        name: String,
        username: String,
        bio: String,
        action: String,
        online: bool,
        avatar: Option<ImageHandle>,
    },
    /// Card 25: browser window, video box with badge + play, caption.
    Media {
        title: String,
        caption: String,
        badge: Option<String>,
        image: Option<ImageHandle>,
    },
    /// Cards 8/19: title + rows (radio, label/progress, trailing value).
    List {
        title: String,
        rows: Vec<CardListRow>,
    },
    /// Card 23: legend + comparative bar chart; `groups` holds pairs of
    /// 0..=1 heights, `highlight` selects the gradient group.
    Chart {
        value: String,
        label: String,
        groups: Vec<(f32, f32)>,
        highlight: usize,
    },
    /// The shell's own footer family: title, body, CTA pill.
    Cta {
        title: String,
        body: String,
        button: String,
    },
}

/// A HOFF glass card. Construct with a [`CardVariant`], lay it out with
/// [`preferred_size`](Card::preferred_size), feed it events for the hover
/// state and render. Cards are presentation-first: a click anywhere in
/// the bounds reports [`EventResult::clicked`].
#[derive(Clone, Debug)]
pub struct Card {
    pub variant: CardVariant,
    pub width: f32,
    hovered: bool,
    pressed: bool,
}

impl Card {
    pub fn new(variant: CardVariant) -> Self {
        Self {
            variant,
            width: CARD_W,
            hovered: false,
            pressed: false,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn is_hovered(&self) -> bool {
        self.hovered
    }

    /// Intrinsic size: fixed width, height from the variant's layout
    /// (body text measured for the CTA family).
    pub fn preferred_size(&self) -> (f32, f32) {
        let h = match &self.variant {
            CardVariant::Stat { .. } => 118.0,
            CardVariant::Profile { .. } => 128.0,
            CardVariant::Media { .. } => 248.0,
            CardVariant::List { rows, .. } => {
                PAD + 34.0 + rows.len() as f32 * (58.0 + 8.0) - 8.0 + PAD
            }
            CardVariant::Chart { .. } => 232.0,
            CardVariant::Cta { body, .. } => {
                let style = TypographyScale::hoff().body();
                let (_, body_h) =
                    TextMeasurer::measure_styled(body, &style, Some(self.width - DETAILS * 2.0));
                PAD + 24.0 + 8.0 + body_h.max(24.0) + 24.0 + 48.0 + 32.0
            }
        };
        (self.width, h)
    }

    pub fn handle_event(&mut self, event: &WidgetEvent, bounds: Rect) -> EventResult {
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let inside = bounds.contains(x, y);
                if inside != self.hovered {
                    self.hovered = inside;
                    EventResult::changed()
                } else {
                    EventResult::IGNORED
                }
            }
            WidgetEvent::MouseDown { x, y } => {
                if bounds.contains(x, y) {
                    self.pressed = true;
                    EventResult::changed()
                } else {
                    EventResult::IGNORED
                }
            }
            WidgetEvent::MouseUp { x, y } => {
                if !self.pressed {
                    return EventResult::IGNORED;
                }
                self.pressed = false;
                if bounds.contains(x, y) {
                    EventResult::clicked()
                } else {
                    EventResult::changed()
                }
            }
            WidgetEvent::Scroll { .. } => EventResult::IGNORED,
        }
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        self.render_to_layer(c, LayerId::DEFAULT, bounds, theme);
    }

    pub fn render_to_layer(&self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        // Social-style cards (Profile) use the 20-radius .02 surface; the
        // deck shells use radius 32 + rgba(40,40,40,.8).
        match &self.variant {
            CardVariant::Profile { .. } => self.social_shell(c, layer, bounds, theme),
            _ => self.deck_shell(c, layer, bounds, theme),
        }
        match &self.variant {
            CardVariant::Stat {
                value,
                label,
                delta,
            } => self.render_stat(c, layer, bounds, theme, value, label, delta.as_ref()),
            CardVariant::Profile {
                name,
                username,
                bio,
                action,
                online,
                avatar,
            } => self.render_profile(
                c, layer, bounds, theme, name, username, bio, action, *online, *avatar,
            ),
            CardVariant::Media {
                title,
                caption,
                badge,
                image,
            } => self.render_media(
                c,
                layer,
                bounds,
                theme,
                title,
                caption,
                badge.as_deref(),
                *image,
            ),
            CardVariant::List { title, rows } => {
                self.render_list(c, layer, bounds, theme, title, rows)
            }
            CardVariant::Chart {
                value,
                label,
                groups,
                highlight,
            } => self.render_chart(c, layer, bounds, theme, value, label, groups, *highlight),
            CardVariant::Cta {
                title,
                body,
                button,
            } => self.render_cta(c, layer, bounds, theme, title, body, button),
        }
    }

    // -- Shells ---------------------------------------------------------------

    /// Deck shell: deep drop shadow + edge-light + rgba(40,40,40,.8).
    fn deck_shell(&self, c: &mut Compositor, layer: LayerId, b: Rect, theme: &Theme) {
        let radius = theme.radius.xl;
        // 0 32px 24px -16px rgba(0,0,0,.40): the -16 spread shrinks the
        // casting rect.
        c.push_to_layer(
            layer,
            SceneNode::Shadow {
                x: b.x + 16.0,
                y: b.y + 16.0,
                w: b.w - 32.0,
                h: b.h - 32.0,
                corner_radius: radius,
                blur_radius: 24.0,
                offset: [0.0, 32.0],
                color: [0.0, 0.0, 0.0, 0.40],
            },
        );
        // Card overlay: the page surface one notch more opaque
        // (rgba(40,40,40,.8) under HOFF, where surface is .7).
        let fill = {
            let s = theme.colors.surface.0;
            [s[0], s[1], s[2], (s[3] + 0.1).min(1.0)]
        };
        for node in glass_pill(b, radius, theme.glass.edge_soft.0, 1.5, fill) {
            c.push_to_layer(layer, node);
        }
    }

    /// Social card shell: radius 20, .02 white glass (.05 hovered),
    /// stronger edge-light on hover.
    fn social_shell(&self, c: &mut Compositor, layer: LayerId, b: Rect, theme: &Theme) {
        let glass = &theme.glass;
        let radius = theme.radius.lg;
        let fill = if self.hovered {
            glass.surface_hover.0
        } else {
            glass.surface.0
        };
        let edge = if self.hovered {
            glass.edge.0
        } else {
            glass.edge_soft.0
        };
        for node in glass_pill(b, radius, edge, 1.0, fill) {
            c.push_to_layer(layer, node);
        }
    }

    // -- Shared little pieces ---------------------------------------------------

    #[allow(clippy::too_many_arguments)]
    fn text(
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

    /// 12px legend/online dot: white 5% disc with a 25% ring, or the
    /// solid knob gradient when `solid`.
    fn dot(&self, c: &mut Compositor, layer: LayerId, x: f32, y: f32, solid: bool, theme: &Theme) {
        let glass = &theme.glass;
        if solid {
            c.push_to_layer(
                layer,
                SceneNode::GradientRect {
                    x,
                    y,
                    w: 12.0,
                    h: 12.0,
                    color: glass.knob_gradient[0].0,
                    color2: glass.knob_gradient[1].0,
                    angle_deg: 180.0,
                    corner_radius: 6.0,
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
                    w: 12.0,
                    h: 12.0,
                    color: glass.surface_hover.0,
                    corner_radius: 6.0,
                    border_width: 1.0,
                    border_color: glass.field_focus_border.0,
                },
            );
        }
    }

    /// Skeleton line: the HOFF 4px "fake text" bar.
    #[allow(clippy::too_many_arguments)]
    fn skeleton(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        x: f32,
        y: f32,
        w: f32,
        alpha: f32,
        theme: &Theme,
    ) {
        let t = theme.colors.text.0;
        c.push_to_layer(
            layer,
            SceneNode::RoundedRect {
                x,
                y,
                w,
                h: 4.0,
                color: [t[0], t[1], t[2], alpha],
                corner_radius: 1.0,
                border_width: 0.0,
                border_color: [0.0; 4],
            },
        );
    }

    // -- Variants ---------------------------------------------------------------

    #[allow(clippy::too_many_arguments)]
    fn render_stat(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        b: Rect,
        theme: &Theme,
        value: &str,
        label: &str,
        delta: Option<&(String, bool)>,
    ) {
        let glass = &theme.glass;
        let text = theme.colors.text;

        // Tick ruler (card 2): 1.5x9 spans at 10% white.
        let mut tx = b.x + PAD;
        let tick_color = with_alpha(text, 0.10);
        while tx < b.x + b.w - PAD {
            c.push_to_layer(
                layer,
                SceneNode::Rect {
                    x: tx,
                    y: b.y + PAD,
                    w: 1.5,
                    h: 9.0,
                    color: tick_color,
                },
            );
            tx += (b.w - PAD * 2.0) / 14.0;
        }

        let ramp = TypographyScale::hoff();
        // Headline 32/1.25/500 $text-primary (no mixin: the gradient
        // headline of the reference cards).
        self.text(
            c,
            layer,
            value,
            &TextStyle::new(32.0).with_line_height(40.0).with_weight(500),
            b.x + PAD,
            b.y + PAD + 9.0 + 12.0,
            text.0,
            None,
        );
        // Dot + label (base-2r, secondary).
        let label_y = b.y + PAD + 9.0 + 12.0 + 40.0 + 4.0;
        self.dot(c, layer, b.x + PAD, label_y + 4.0, false, theme);
        self.text(
            c,
            layer,
            label,
            &ramp.base_2r(),
            b.x + PAD + 20.0,
            label_y,
            theme.colors.text_mid.0,
            None,
        );

        // Delta chip: caption-sm in an 8-radius 5% chip, accent-colored.
        if let Some((delta, positive)) = delta {
            let style = ramp.caption_sm();
            let (tw, _) = TextMeasurer::measure_styled(delta, &style, None);
            let chip_w = tw + 16.0;
            let chip = Rect::new(
                b.x + b.w - PAD - chip_w,
                b.y + PAD + 9.0 + 12.0,
                chip_w,
                24.0,
            );
            c.push_to_layer(
                layer,
                SceneNode::RoundedRect {
                    x: chip.x,
                    y: chip.y,
                    w: chip.w,
                    h: chip.h,
                    color: glass.surface_hover.0,
                    corner_radius: theme.radius.sm,
                    border_width: 0.0,
                    border_color: [0.0; 4],
                },
            );
            let accent = if *positive {
                theme.colors.success.0
            } else {
                theme.colors.danger.0
            };
            self.text(
                c,
                layer,
                delta,
                &style,
                chip.x + 8.0,
                chip.y + 4.0,
                accent,
                None,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_profile(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        b: Rect,
        theme: &Theme,
        name: &str,
        username: &str,
        bio: &str,
        action: &str,
        online: bool,
        avatar: Option<ImageHandle>,
    ) {
        let glass = &theme.glass;
        let text = theme.colors.text;
        let pad = 12.0;
        let av = Rect::new(b.x + pad, b.y + pad, 44.0, 44.0);

        // Avatar: image, or a monochrome glass disc with an initial.
        if let Some(image) = avatar {
            c.push_to_layer(
                layer,
                SceneNode::Image {
                    x: av.x,
                    y: av.y,
                    w: av.w,
                    h: av.h,
                    image,
                    corner_radius: av.w / 2.0,
                },
            );
        } else {
            for node in glass_pill(av, av.w / 2.0, glass.edge.0, 1.5, glass.surface_active.0) {
                c.push_to_layer(layer, node);
            }
            if let Some(initial) = name.chars().next() {
                let s: String = initial.to_uppercase().collect();
                // Avatar initial: 18/24/500 (no mixin).
                let style = TextStyle::new(18.0).with_line_height(24.0).with_weight(500);
                let (tw, _) = TextMeasurer::measure_styled(&s, &style, None);
                self.text(
                    c,
                    layer,
                    &s,
                    &style,
                    av.x + (av.w - tw) / 2.0,
                    av.y + TextMeasurer::vertical_center(&style, av.h),
                    text.0,
                    None,
                );
            }
        }
        // Online dot at the avatar's top-left corner (knob gradient).
        if online {
            self.dot(c, layer, av.x - 2.0, av.y - 2.0, true, theme);
        }

        // Name (.95, base-2sm) and username (caption-r, .4).
        let ramp = TypographyScale::hoff();
        self.text(
            c,
            layer,
            name,
            &ramp.base_2sm(),
            av.x + av.w + 12.0,
            b.y + pad + 2.0,
            text.0,
            None,
        );
        self.text(
            c,
            layer,
            username,
            &ramp.caption_r(),
            av.x + av.w + 12.0,
            b.y + pad + 24.0,
            glass.text_faint.0,
            None,
        );

        // Action pill (FollowButton): glass button, caption-sm .70.
        let style = ramp.caption_sm();
        let (tw, _) = TextMeasurer::measure_styled(action, &style, None);
        let bw = (tw + 32.0).max(88.0);
        let btn = Rect::new(b.x + b.w - pad - bw, b.y + pad + 2.0, bw, 40.0);
        for node in glass_pill(btn, btn.h / 2.0, glass.edge.0, 1.5, glass.button.0) {
            c.push_to_layer(layer, node);
        }
        self.text(
            c,
            layer,
            action,
            &style,
            btn.x + (btn.w - tw) / 2.0,
            btn.y + TextMeasurer::vertical_center(&style, btn.h),
            theme.colors.text_mid.0,
            None,
        );

        // Bio: body-2r at .4, clamped to the column, padded-left 56.
        self.text(
            c,
            layer,
            bio,
            &ramp.body_2r(),
            b.x + pad + 44.0 + 12.0,
            b.y + pad + 44.0 + 6.0,
            glass.text_faint.0,
            Some(b.w - pad * 2.0 - 56.0),
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn render_media(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        b: Rect,
        theme: &Theme,
        title: &str,
        caption: &str,
        badge: Option<&str>,
        image: Option<ImageHandle>,
    ) {
        let glass = &theme.glass;
        let text = theme.colors.text;

        // Browser dots: 3 x 8px, gap 6.
        for i in 0..3 {
            c.push_to_layer(
                layer,
                SceneNode::RoundedRect {
                    x: b.x + PAD + i as f32 * 14.0,
                    y: b.y + PAD,
                    w: 8.0,
                    h: 8.0,
                    color: glass.surface_active.0,
                    corner_radius: 4.0,
                    border_width: 0.0,
                    border_color: [0.0; 4],
                },
            );
        }

        // Video box 176x124 r10, centered.
        let box_r = Rect::new(
            b.x + (b.w - 176.0) / 2.0,
            b.y + PAD + 8.0 + 12.0,
            176.0,
            124.0,
        );
        if let Some(image) = image {
            c.push_to_layer(
                layer,
                SceneNode::Image {
                    x: box_r.x,
                    y: box_r.y,
                    w: box_r.w,
                    h: box_r.h,
                    image,
                    corner_radius: 10.0,
                },
            );
        } else {
            // bg white 3% + edge 10%.
            for node in glass_pill(box_r, 10.0, glass.edge.0, 1.5, with_alpha(text, 0.03)) {
                c.push_to_layer(layer, node);
            }
            // Play: 60px ring (white 40%) + glass core. The triangle is a
            // text glyph so it also survives above images (text pass).
            let play = Rect::new(
                box_r.x + (box_r.w - 60.0) / 2.0,
                box_r.y + (box_r.h - 60.0) / 2.0,
                60.0,
                60.0,
            );
            c.push_to_layer(
                layer,
                SceneNode::RoundedRect {
                    x: play.x,
                    y: play.y,
                    w: play.w,
                    h: play.h,
                    color: [0.0; 4],
                    corner_radius: 30.0,
                    border_width: 1.5,
                    border_color: with_alpha(text, 0.40),
                },
            );
            let core = Rect::new(play.x + 2.5, play.y + 2.5, 55.0, 55.0);
            for node in glass_pill(core, 27.5, glass.edge.0, 1.5, glass.surface_hover.0) {
                c.push_to_layer(layer, node);
            }
        }
        // Play glyph + badge render above the image (text pass).
        let ramp = TypographyScale::hoff();
        self.text(
            c,
            layer,
            "\u{25B6}",
            &TextStyle::new(20.0).with_line_height(24.0),
            box_r.x + box_r.w / 2.0 - 8.0,
            box_r.y + box_r.h / 2.0 - 12.0,
            text.0,
            None,
        );
        if let Some(badge) = badge {
            self.text(
                c,
                layer,
                badge,
                &ramp.caption_sm(),
                box_r.x + box_r.w - 8.0 - 24.0,
                box_r.y + 8.0,
                text.0,
                None,
            );
        }

        // Title + caption.
        let ty = box_r.y + box_r.h + PAD;
        self.text(c, layer, title, &ramp.title(), b.x + PAD, ty, text.0, None);
        self.text(
            c,
            layer,
            caption,
            &ramp.base_2r(),
            b.x + PAD,
            ty + 28.0,
            theme.colors.text_mid.0,
            None,
        );
    }

    fn render_list(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        b: Rect,
        theme: &Theme,
        title: &str,
        rows: &[CardListRow],
    ) {
        let glass = &theme.glass;
        let text = theme.colors.text;
        let ramp = TypographyScale::hoff();

        self.text(
            c,
            layer,
            title,
            &ramp.title(),
            b.x + PAD,
            b.y + PAD,
            text.0,
            None,
        );

        let mut y = b.y + PAD + 34.0;
        for row in rows {
            let r = Rect::new(b.x + PAD, y, b.w - PAD * 2.0, 58.0);
            // Row: r12, 1.5px border (active = 40% white), bg white 1%.
            c.push_to_layer(
                layer,
                SceneNode::RoundedRect {
                    x: r.x,
                    y: r.y,
                    w: r.w,
                    h: r.h,
                    color: with_alpha(text, 0.01),
                    corner_radius: theme.radius.md,
                    border_width: 1.5,
                    border_color: if row.active {
                        with_alpha(text, 0.40)
                    } else {
                        glass.edge_soft.0
                    },
                },
            );

            // Radio 18px: active = dark glass with a white core.
            let radio = Rect::new(r.x + 14.5, r.y + 20.0, 18.0, 18.0);
            if row.active {
                c.push_to_layer(
                    layer,
                    SceneNode::RoundedRect {
                        x: radio.x,
                        y: radio.y,
                        w: radio.w,
                        h: radio.h,
                        color: [18.0 / 255.0, 18.0 / 255.0, 18.0 / 255.0, 0.3],
                        corner_radius: 9.0,
                        border_width: 1.0,
                        border_color: with_alpha(text, 0.40),
                    },
                );
                c.push_to_layer(
                    layer,
                    SceneNode::RoundedRect {
                        x: radio.x + 4.5,
                        y: radio.y + 4.5,
                        w: 9.0,
                        h: 9.0,
                        color: text.0,
                        corner_radius: 4.5,
                        border_width: 0.0,
                        border_color: [0.0; 4],
                    },
                );
            } else {
                c.push_to_layer(
                    layer,
                    SceneNode::RoundedRect {
                        x: radio.x,
                        y: radio.y,
                        w: radio.w,
                        h: radio.h,
                        color: [0.0; 4],
                        corner_radius: 9.0,
                        border_width: 1.5,
                        border_color: glass.field_focus_border.0,
                    },
                );
            }

            let content_x = radio.x + radio.w + 12.0;
            match row.progress {
                Some(p) => {
                    // Download row: label up, gradient progress under.
                    self.text(
                        c,
                        layer,
                        &row.label,
                        &ramp.caption_sm(),
                        content_x,
                        r.y + 10.0,
                        theme.colors.text_mid.0,
                        None,
                    );
                    let track = Rect::new(content_x, r.y + 32.0, r.w - 160.0, 12.0);
                    c.push_to_layer(
                        layer,
                        SceneNode::RoundedRect {
                            x: track.x,
                            y: track.y,
                            w: track.w,
                            h: track.h,
                            color: [0.0; 4],
                            corner_radius: 6.0,
                            border_width: 1.5,
                            border_color: glass.surface_active.0,
                        },
                    );
                    if p > 0.01 {
                        c.push_to_layer(
                            layer,
                            SceneNode::GradientRect {
                                x: track.x + 4.0,
                                y: track.y + 4.0,
                                w: (track.w - 8.0) * p,
                                h: 4.0,
                                color: with_alpha(text, 0.0),
                                color2: with_alpha(text, 0.40),
                                angle_deg: 90.0,
                                corner_radius: 2.0,
                                border_width: 0.0,
                                border_color: [0.0; 4],
                            },
                        );
                    }
                }
                None => {
                    if row.label.is_empty() {
                        // Skeleton row (card 19's fake text).
                        self.skeleton(
                            c,
                            layer,
                            content_x,
                            r.y + 27.0,
                            if row.active { 68.0 } else { 118.0 },
                            if row.active { 0.95 } else { 0.10 },
                            theme,
                        );
                    } else {
                        self.text(
                            c,
                            layer,
                            &row.label,
                            &ramp.base_2m(),
                            content_x,
                            r.y + 19.0,
                            theme.colors.text_mid.0,
                            None,
                        );
                    }
                }
            }

            // Trailing value: caption-sm, primary.
            let style = ramp.caption_sm();
            let (tw, _) = TextMeasurer::measure_styled(&row.trailing, &style, None);
            self.text(
                c,
                layer,
                &row.trailing,
                &style,
                r.x + r.w - 18.5 - tw,
                r.y + TextMeasurer::vertical_center(&style, r.h),
                text.0,
                None,
            );

            y += 58.0 + 8.0;
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_chart(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        b: Rect,
        theme: &Theme,
        value: &str,
        label: &str,
        groups: &[(f32, f32)],
        highlight: usize,
    ) {
        let text = theme.colors.text;

        // Legend: solid dot + value (base-m primary) + label (caption-r).
        let ramp = TypographyScale::hoff();
        self.dot(c, layer, b.x + PAD, b.y + PAD + 6.0, true, theme);
        let style = ramp.base_m();
        self.text(
            c,
            layer,
            value,
            &style,
            b.x + PAD + 20.0,
            b.y + PAD,
            text.0,
            None,
        );
        let (vw, _) = TextMeasurer::measure_styled(value, &style, None);
        self.text(
            c,
            layer,
            label,
            &ramp.caption_r(),
            b.x + PAD + 20.0 + vw + 12.0,
            b.y + PAD + 4.0,
            theme.colors.text_mid.0,
            None,
        );

        // Chart: groups of two bars (gap 2), ~4.5% group gap, r2 bars.
        let chart = Rect::new(b.x + PAD, b.y + PAD + 36.0, b.w - PAD * 2.0, 140.0);
        let n = groups.len().max(1) as f32;
        let group_gap = chart.w * 0.045;
        let group_w = (chart.w - group_gap * (n - 1.0)) / n;
        let bar_w = (group_w - 2.0) / 2.0;
        for (i, (a, hb)) in groups.iter().enumerate() {
            let gx = chart.x + i as f32 * (group_w + group_gap);
            let heights = [a.clamp(0.0, 1.0), hb.clamp(0.0, 1.0)];
            for (j, t01) in heights.iter().enumerate() {
                let h = chart.h * t01;
                let x = gx + j as f32 * (bar_w + 2.0);
                let y = chart.y + chart.h - h;
                if i == highlight && j == 1 {
                    // Highlight: gradient 180deg white 40% -> 0 at .5
                    // opacity, plus the floating 2px cap above the bar.
                    c.push_to_layer(
                        layer,
                        SceneNode::GradientRect {
                            x,
                            y,
                            w: bar_w,
                            h,
                            color: with_alpha(text, 0.20),
                            color2: with_alpha(text, 0.0),
                            angle_deg: 180.0,
                            corner_radius: 2.0,
                            border_width: 0.0,
                            border_color: [0.0; 4],
                        },
                    );
                    c.push_to_layer(
                        layer,
                        SceneNode::RoundedRect {
                            x,
                            y: y - 4.0,
                            w: bar_w,
                            h: 2.0,
                            color: with_alpha(text, 0.50),
                            corner_radius: 1.0,
                            border_width: 0.0,
                            border_color: [0.0; 4],
                        },
                    );
                } else {
                    let alpha = if j == 0 { 0.05 } else { 0.10 };
                    c.push_to_layer(
                        layer,
                        SceneNode::RoundedRect {
                            x,
                            y,
                            w: bar_w,
                            h,
                            color: with_alpha(text, alpha),
                            corner_radius: 2.0,
                            border_width: 0.0,
                            border_color: [0.0; 4],
                        },
                    );
                }
            }
        }
        // Baseline (10% white, bleeding to the card edge) + #4C4C4C ticks.
        c.push_to_layer(
            layer,
            SceneNode::Rect {
                x: b.x + 4.0,
                y: chart.y + chart.h + 2.0,
                w: b.w - 8.0,
                h: 1.5,
                color: with_alpha(text, 0.10),
            },
        );
        let ticks = 9;
        for i in 0..ticks {
            let tx = chart.x + chart.w * i as f32 / (ticks - 1) as f32;
            c.push_to_layer(
                layer,
                SceneNode::Rect {
                    x: tx,
                    y: chart.y + chart.h + 7.0,
                    w: 1.5,
                    h: 6.0,
                    color: [
                        0x4C as f32 / 255.0,
                        0x4C as f32 / 255.0,
                        0x4C as f32 / 255.0,
                        1.0,
                    ],
                },
            );
        }
    }

    fn render_cta(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        b: Rect,
        theme: &Theme,
        title: &str,
        body: &str,
        button: &str,
    ) {
        let glass = &theme.glass;
        let text = theme.colors.text;
        let x = b.x + DETAILS;
        let w = b.w - DETAILS * 2.0;

        // title 20/1.2/500 primary, mb 8.
        let ramp = TypographyScale::hoff();
        self.text(c, layer, title, &ramp.title(), x, b.y + PAD, text.0, None);
        // =body (16/1.5, letter-spacing 0.025em) secondary, mb 24.
        let style = ramp.body();
        let (_, body_h) = TextMeasurer::measure_styled(body, &style, Some(w));
        self.text(
            c,
            layer,
            body,
            &style,
            x,
            b.y + PAD + 24.0 + 8.0,
            theme.colors.text_mid.0,
            Some(w),
        );

        // CTA pill: pad 12 32, base-m label, 25% edge ring + frozen glow.
        let bstyle = ramp.base_m();
        let (tw, _) = TextMeasurer::measure_styled(button, &bstyle, None);
        let btn = Rect::new(
            x,
            b.y + PAD + 24.0 + 8.0 + body_h.max(24.0) + 24.0,
            tw + 64.0,
            48.0,
        );
        // border 1px rgba(248,248,248,.25) at opacity .25.
        let ring = with_alpha(glass.field_focus_border, 0.25 * 0.25);
        for node in glass_pill(btn, btn.h / 2.0, ring, 1.0, glass.button.0) {
            c.push_to_layer(layer, node);
        }
        // The orbiting conic glow, frozen as a faint top wash.
        c.push_to_layer(
            layer,
            SceneNode::GradientRect {
                x: btn.x + 1.0,
                y: btn.y + 1.0,
                w: btn.w - 2.0,
                h: btn.h - 2.0,
                color: with_alpha(text, 0.05),
                color2: with_alpha(text, 0.0),
                angle_deg: 180.0,
                corner_radius: (btn.h - 2.0) / 2.0,
                border_width: 0.0,
                border_color: [0.0; 4],
            },
        );
        self.text(
            c,
            layer,
            button,
            &bstyle,
            btn.x + (btn.w - tw) / 2.0,
            btn.y + TextMeasurer::vertical_center(&bstyle, btn.h),
            theme.colors.text_mid.0,
            None,
        );
    }
}
