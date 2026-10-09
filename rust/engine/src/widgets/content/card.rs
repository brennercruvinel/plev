//! HOFF cards: one glass shell, many previews.
//!
//! The reference deck (~60 variants) shares a single recipe, the live
//! post card's discreet lift (the card radius, the `glass.surface` white
//! wash, the soft top-lit edge, no frost, no shadow, no border), and
//! differs only in the preview. [`Card`] ships the shell plus the most
//! reusable preview families as [`CardVariant`]s: stat, profile, media,
//! list, chart and CTA.
//!
//! Every dimension is derived: paddings and gaps from `theme.spacing`,
//! controls from `theme.control`, radii from `theme.shape`, text from
//! the `theme.typography` ramp, and every preview's height from the
//! content it holds, so a card is as tall as what is in it and as wide
//! as the grid column it was given.
//!
//! Rendering uses the SDF pipeline (gradients, analytic shadows), images
//! and text only, plus the play icon; every element stacks correctly
//! within a single layer (quads, shadows, SDF, images, text).

use engine::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::gpu::image::ImageHandle;
use engine::text::{TextMeasurer, TextStyle};
use engine::theme::{ControlSize, IconSize, Theme};

use crate::core::{EventResult, Rect, WidgetEvent, with_alpha};
use crate::recipe::{glass_pill, inset_keylight};

/// One row of a [`CardVariant::List`] (expense-tracker / download rows).
#[derive(Clone, Debug)]
pub struct CardListRow {
    pub label: String,
    /// Right-aligned caption-sm value ("$128.00", "84%").
    pub trailing: String,
    /// Progress 0..=1 renders the gradient bar under the label.
    pub progress: Option<f32>,
    /// Active rows get the active border (HOFF selected state).
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
    /// Card 2/17: tick ruler, headline value, dot label, delta chip.
    Stat {
        value: String,
        label: String,
        /// Small chip at the top right ("+12.4%"); the bool picks the
        /// success (true) / danger (false) accent.
        delta: Option<(String, bool)>,
    },
    /// Follower card: avatar, name/username, bio, action pill.
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

/// The per-theme geometry every preview shares.
struct Metrics {
    /// Shell padding.
    pad: f32,
    /// The gap between stacked blocks.
    gap: f32,
    /// The small gap (dot to label, chip padding).
    gap_sm: f32,
    /// Legend / online dot diameter.
    dot: f32,
    /// Shell corner radius.
    radius: f32,
}

impl Metrics {
    fn of(theme: &Theme) -> Self {
        Self {
            pad: theme.spacing.lg,
            gap: theme.spacing.sm,
            gap_sm: theme.spacing.xs,
            dot: theme.spacing.md,
            radius: theme.shape.card,
        }
    }
}

/// A HOFF glass card. Construct with a [`CardVariant`], lay it out with
/// [`preferred_size`](Card::preferred_size), feed it events for the hover
/// state and render. Cards are presentation-first: a click anywhere in
/// the bounds reports [`EventResult::clicked`].
#[derive(Clone, Debug)]
pub struct Card {
    pub variant: CardVariant,
    /// Width the card lays out at; `None` takes `theme.size.card_w`.
    /// Grids set it from their column width.
    pub width: Option<f32>,
    hovered: bool,
    pressed: bool,
}

impl Card {
    pub fn new(variant: CardVariant) -> Self {
        Self {
            variant,
            width: None,
            hovered: false,
            pressed: false,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn is_hovered(&self) -> bool {
        self.hovered
    }

    fn width_for(&self, theme: &Theme) -> f32 {
        self.width
            .unwrap_or(theme.size.card_w)
            .max(theme.size.card_min_w)
    }

    /// Intrinsic size: the given (or default) width, height from the
    /// variant's content.
    pub fn preferred_size(&self, theme: &Theme) -> (f32, f32) {
        let w = self.width_for(theme);
        let m = Metrics::of(theme);
        let ty = &theme.typography;
        let h = match &self.variant {
            CardVariant::Stat { .. } => {
                // Ruler, headline, dot label, each separated by a gap.
                m.pad
                    + Self::ruler_h(theme)
                    + m.gap
                    + ty.headline().line_height
                    + m.gap_sm
                    + ty.base_2r().line_height
                    + m.pad
            }
            CardVariant::Profile { bio, .. } => {
                let inner = Self::profile_pad(theme);
                let avatar = Self::avatar_size(theme);
                let bio_w = w - inner * 2.0 - avatar - m.gap * 2.0;
                let (_, bio_h) = TextMeasurer::measure_styled(bio, &ty.body_2r(), Some(bio_w));
                inner + avatar + m.gap + bio_h.max(ty.body_2r().line_height) + inner
            }
            CardVariant::Media { .. } => {
                let (_, box_h) = Self::media_box(theme, w);
                m.pad
                    + Self::browser_dot(theme)
                    + m.gap
                    + box_h
                    + m.pad
                    + ty.title().line_height
                    + m.gap_sm
                    + ty.base_2r().line_height
                    + m.pad
            }
            CardVariant::List { rows, .. } => {
                let row_h = Self::list_row_h(theme);
                let n = rows.len() as f32;
                m.pad
                    + ty.title().line_height
                    + m.gap
                    + n * row_h
                    + (n - 1.0).max(0.0) * m.gap
                    + m.pad
            }
            CardVariant::Chart { .. } => {
                m.pad
                    + ty.base_m().line_height
                    + m.gap
                    + Self::chart_h(theme)
                    + m.gap
                    + Self::axis_h(theme)
                    + m.pad
            }
            CardVariant::Cta { body, .. } => {
                let inset = Self::cta_inset(theme);
                let style = ty.body();
                let (_, body_h) = TextMeasurer::measure_styled(body, &style, Some(w - inset * 2.0));
                m.pad
                    + ty.title().line_height
                    + m.gap
                    + body_h.max(style.line_height)
                    + theme.spacing.xl
                    + theme.control.height(ControlSize::Lg)
                    + theme.spacing.xxl
            }
        };
        (w, h.ceil())
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
        // Every card, deck and social, uses the one discreet post-card
        // shell (card radius, surface wash, soft edge, no frost/shadow).
        self.deck_shell(c, layer, bounds, theme);
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

    // -- Derived geometry -------------------------------------------------------

    /// Tick ruler height on the stat card: one `sm` step.
    fn ruler_h(theme: &Theme) -> f32 {
        theme.spacing.sm
    }

    /// The profile card's tighter inner padding (`md`).
    fn profile_pad(theme: &Theme) -> f32 {
        theme.spacing.md
    }

    /// Avatar: one `Md` control.
    fn avatar_size(theme: &Theme) -> f32 {
        theme.control.height(ControlSize::Md)
    }

    /// Browser chrome dot on the media card.
    fn browser_dot(theme: &Theme) -> f32 {
        theme.spacing.sm
    }

    /// Video box on the media card: half the card wide, 5:4 ish (the
    /// reference 176 by 124).
    fn media_box(theme: &Theme, card_w: f32) -> (f32, f32) {
        let w = (card_w - Metrics::of(theme).pad * 2.0) * 0.5;
        (w, w * 124.0 / 176.0)
    }

    /// List row: one `Lg` control plus the `xs` step above and below.
    fn list_row_h(theme: &Theme) -> f32 {
        theme.control.height(ControlSize::Lg) + theme.spacing.xs * 2.0
    }

    /// Chart plot height: three `Lg` controls.
    fn chart_h(theme: &Theme) -> f32 {
        theme.control.height(ControlSize::Lg) * 3.0
    }

    /// Baseline plus tick row under the chart.
    fn axis_h(theme: &Theme) -> f32 {
        theme.spacing.sm + theme.spacing.xs
    }

    /// CTA family column inset: the shell padding twice.
    fn cta_inset(theme: &Theme) -> f32 {
        Metrics::of(theme).pad * 2.0
    }

    // -- Shells ---------------------------------------------------------------

    /// The HOFF content-card shell, one discreet recipe for the whole
    /// deck, taken 1:1 from the live POST card (`Post.module.sass`: the
    /// card radius, `glass.surface` at rest and `surface_hover` hovered,
    /// NO `backdrop-filter`, NO `box-shadow`, NO border). A whisper
    /// lighter than the page, never a frosted panel, closed by the faint
    /// inset key-light. The reference's frost is reserved for pills,
    /// search and menus (real glass), not for cards of content.
    fn deck_shell(&self, c: &mut Compositor, layer: LayerId, b: Rect, theme: &Theme) {
        let glass = &theme.glass;
        let radius = Metrics::of(theme).radius;
        let (fill, edge) = if self.hovered {
            (glass.surface_hover.0, glass.edge.0)
        } else {
            (glass.surface.0, glass.edge_soft.0)
        };
        for node in glass_pill(b, radius, edge, theme.control.edge_width, fill) {
            c.push_to_layer(layer, node);
        }
        inset_keylight(c, layer, b, radius, theme);
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

    /// Legend / online dot: the surface-hover disc with the field-focus
    /// ring, or the solid knob gradient when `solid`.
    fn dot(&self, c: &mut Compositor, layer: LayerId, x: f32, y: f32, solid: bool, theme: &Theme) {
        let glass = &theme.glass;
        let d = Metrics::of(theme).dot;
        if solid {
            c.push_to_layer(
                layer,
                SceneNode::GradientRect {
                    x,
                    y,
                    w: d,
                    h: d,
                    color: glass.knob_gradient[0].0,
                    color2: glass.knob_gradient[1].0,
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
                    color: glass.surface_hover.0,
                    corner_radius: d / 2.0,
                    border_width: theme.control.edge_width,
                    border_color: glass.field_focus_border.0,
                },
            );
        }
    }

    /// Skeleton line: the HOFF "fake text" bar, as thick as the progress
    /// fill.
    #[allow(clippy::too_many_arguments)]
    fn skeleton(
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
    fn progress(&self, c: &mut Compositor, layer: LayerId, track: Rect, p: f32, theme: &Theme) {
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
        let m = Metrics::of(theme);
        let glass = &theme.glass;
        let text = theme.colors.text;
        let ty = &theme.typography;

        // Tick ruler (card 2): hairline spans at the surface-active tone,
        // one per `xl` step across the inner width.
        let ruler_h = Self::ruler_h(theme);
        let inner_w = b.w - m.pad * 2.0;
        let ticks = ((inner_w / theme.spacing.xl).floor() as usize).max(2);
        for i in 0..ticks {
            let tx = b.x + m.pad + inner_w * i as f32 / (ticks - 1) as f32;
            c.push_to_layer(
                layer,
                SceneNode::Rect {
                    x: tx,
                    y: b.y + m.pad,
                    w: theme.control.edge_width_strong,
                    h: ruler_h,
                    color: glass.surface_active.0,
                },
            );
        }

        // Headline value, then dot + label (base-2r, secondary).
        let headline = ty.headline();
        let value_y = b.y + m.pad + ruler_h + m.gap;
        self.text(
            c,
            layer,
            value,
            &headline,
            b.x + m.pad,
            value_y,
            text.0,
            None,
        );
        let label_style = ty.base_2r();
        let label_y = value_y + headline.line_height + m.gap_sm;
        self.dot(
            c,
            layer,
            b.x + m.pad,
            label_y + (label_style.line_height - m.dot) / 2.0,
            false,
            theme,
        );
        self.text(
            c,
            layer,
            label,
            &label_style,
            b.x + m.pad + m.dot + m.gap,
            label_y,
            theme.colors.text_mid.0,
            None,
        );

        // Delta chip: caption-sm in a tooltip-radius surface-hover chip,
        // success or danger colored.
        if let Some((delta, positive)) = delta {
            let style = ty.caption_sm();
            let (tw, _) = TextMeasurer::measure_styled(delta, &style, None);
            let chip_h = style.line_height + m.gap_sm * 2.0;
            let chip_w = tw + m.gap * 2.0;
            let chip = Rect::new(b.x + b.w - m.pad - chip_w, value_y, chip_w, chip_h);
            c.push_to_layer(
                layer,
                SceneNode::RoundedRect {
                    x: chip.x,
                    y: chip.y,
                    w: chip.w,
                    h: chip.h,
                    color: glass.surface_hover.0,
                    corner_radius: theme.shape.tooltip,
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
                chip.x + m.gap,
                chip.y + m.gap_sm,
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
        let m = Metrics::of(theme);
        let glass = &theme.glass;
        let text = theme.colors.text;
        let ty = &theme.typography;
        let pad = Self::profile_pad(theme);
        let size = Self::avatar_size(theme);
        let av = Rect::new(b.x + pad, b.y + pad, size, size);

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
            for node in glass_pill(
                av,
                av.w / 2.0,
                glass.edge.0,
                theme.control.edge_width_strong,
                glass.surface_active.0,
            ) {
                c.push_to_layer(layer, node);
            }
            if let Some(initial) = name.chars().next() {
                let s: String = initial.to_uppercase().collect();
                // Avatar initial: the title style, centered.
                let style = ty.title();
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
            let off = theme.control.edge_width * 2.0;
            self.dot(c, layer, av.x - off, av.y - off, true, theme);
        }

        // Name (base-2sm, primary) and username (caption-r, the meta
        // register text-tertiary, not the fainter placeholder tone, which
        // read gray-on-gray over the lifted card).
        let name_style = ty.base_2sm();
        let text_x = av.x + av.w + m.gap + m.gap_sm;
        self.text(c, layer, name, &name_style, text_x, b.y + pad, text.0, None);
        self.text(
            c,
            layer,
            username,
            &ty.caption_r(),
            text_x,
            b.y + pad + name_style.line_height + m.gap_sm / 2.0,
            theme.colors.text_dim.0,
            None,
        );

        // Action pill (FollowButton): a `Sm` glass button, caption-sm.
        let style = ty.caption_sm();
        let (tw, _) = TextMeasurer::measure_styled(action, &style, None);
        let pad_x = theme.control.pad_x(ControlSize::Sm);
        let bh = theme.control.height(ControlSize::Sm);
        let bw = (tw + pad_x * 2.0).max(bh * 2.0);
        let btn = Rect::new(b.x + b.w - pad - bw, b.y + pad, bw, bh);
        for node in glass_pill(
            btn,
            btn.h / 2.0,
            glass.edge.0,
            theme.control.edge_width_strong,
            glass.button.0,
        ) {
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

        // Bio: body-2r at the secondary body register, indented past the
        // avatar column.
        self.text(
            c,
            layer,
            bio,
            &ty.body_2r(),
            text_x,
            b.y + pad + size + m.gap,
            theme.colors.text_mid.0,
            Some(b.w - pad * 2.0 - size - m.gap * 2.0),
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
        let m = Metrics::of(theme);
        let glass = &theme.glass;
        let text = theme.colors.text;
        let ty = &theme.typography;

        // Browser dots: three, one `sm` step each, `xs` apart.
        let dot = Self::browser_dot(theme);
        for i in 0..3 {
            c.push_to_layer(
                layer,
                SceneNode::RoundedRect {
                    x: b.x + m.pad + i as f32 * (dot + m.gap_sm + m.gap_sm / 2.0),
                    y: b.y + m.pad,
                    w: dot,
                    h: dot,
                    color: glass.surface_active.0,
                    corner_radius: dot / 2.0,
                    border_width: 0.0,
                    border_color: [0.0; 4],
                },
            );
        }

        // Video box, centered, at the cluster radius.
        let (box_w, box_h) = Self::media_box(theme, b.w);
        let box_radius = theme.shape.cluster;
        let box_r = Rect::new(
            b.x + (b.w - box_w) / 2.0,
            b.y + m.pad + dot + m.gap,
            box_w,
            box_h,
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
                    corner_radius: box_radius,
                },
            );
        } else {
            for node in glass_pill(
                box_r,
                box_radius,
                glass.edge.0,
                theme.control.edge_width_strong,
                glass.surface.0,
            ) {
                c.push_to_layer(layer, node);
            }
            // Play: a ring one `Xl` control wide (faint text stroke) with
            // a glass core inside the rim.
            let ring_d = theme.control.height(ControlSize::Xl);
            let play = Rect::new(
                box_r.x + (box_r.w - ring_d) / 2.0,
                box_r.y + (box_r.h - ring_d) / 2.0,
                ring_d,
                ring_d,
            );
            c.push_to_layer(
                layer,
                SceneNode::RoundedRect {
                    x: play.x,
                    y: play.y,
                    w: play.w,
                    h: play.h,
                    color: [0.0; 4],
                    corner_radius: ring_d / 2.0,
                    border_width: theme.control.edge_width_strong,
                    border_color: glass.text_faint.0,
                },
            );
            let core = play.inset(theme.control.edge_width_strong * 2.0);
            for node in glass_pill(
                core,
                core.w / 2.0,
                glass.edge.0,
                theme.control.edge_width_strong,
                glass.surface_hover.0,
            ) {
                c.push_to_layer(layer, node);
            }
        }
        // Play icon + badge render above the image. The triangle is the
        // vector Lucide icon, not a text glyph: U+25B6 is not covered by
        // the embedded UI faces and would silently rasterize from a
        // system font.
        let play_icon = theme.control.icon(IconSize::Lg);
        if let Some(icon) = crate::icons::icon_at(
            "play",
            play_icon,
            text.0,
            box_r.x + (box_r.w - play_icon) / 2.0,
            box_r.y + (box_r.h - play_icon) / 2.0,
        ) {
            c.push_to_layer(layer, icon);
        }
        if let Some(badge) = badge {
            let style = ty.caption_sm();
            let (bw, _) = TextMeasurer::measure_styled(badge, &style, None);
            self.text(
                c,
                layer,
                badge,
                &style,
                box_r.x + box_r.w - m.gap - bw,
                box_r.y + m.gap,
                text.0,
                None,
            );
        }

        // Title + caption.
        let title_style = ty.title();
        let ty0 = box_r.y + box_r.h + m.pad;
        self.text(
            c,
            layer,
            title,
            &title_style,
            b.x + m.pad,
            ty0,
            text.0,
            None,
        );
        self.text(
            c,
            layer,
            caption,
            &ty.base_2r(),
            b.x + m.pad,
            ty0 + title_style.line_height + m.gap_sm,
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
        let m = Metrics::of(theme);
        let glass = &theme.glass;
        let text = theme.colors.text;
        let ty = &theme.typography;
        let title_style = ty.title();

        self.text(
            c,
            layer,
            title,
            &title_style,
            b.x + m.pad,
            b.y + m.pad,
            text.0,
            None,
        );

        let row_h = Self::list_row_h(theme);
        let radio_d = theme.control.box_size;
        let mut y = b.y + m.pad + title_style.line_height + m.gap;
        for row in rows {
            let r = Rect::new(b.x + m.pad, y, b.w - m.pad * 2.0, row_h);
            // Row: nav radius, strong rim (active = the active border),
            // a whisper of surface.
            c.push_to_layer(
                layer,
                SceneNode::RoundedRect {
                    x: r.x,
                    y: r.y,
                    w: r.w,
                    h: r.h,
                    color: with_alpha(glass.surface, glass.surface.0[3] / 2.0),
                    corner_radius: theme.shape.nav,
                    border_width: theme.control.edge_width_strong,
                    border_color: if row.active {
                        theme.colors.border_active.0
                    } else {
                        glass.edge_soft.0
                    },
                },
            );

            // Radio: the checkbox box size as a disc; active = a dark
            // glass disc with a white core.
            let radio = Rect::new(
                r.x + m.pad - theme.control.edge_width_strong,
                r.y + (r.h - radio_d) / 2.0,
                radio_d,
                radio_d,
            );
            if row.active {
                c.push_to_layer(
                    layer,
                    SceneNode::RoundedRect {
                        x: radio.x,
                        y: radio.y,
                        w: radio.w,
                        h: radio.h,
                        color: glass.tabs.0,
                        corner_radius: radio_d / 2.0,
                        border_width: theme.control.edge_width,
                        border_color: theme.colors.border_active.0,
                    },
                );
                let core = radio.inset(radio_d / 4.0);
                c.push_to_layer(
                    layer,
                    SceneNode::RoundedRect {
                        x: core.x,
                        y: core.y,
                        w: core.w,
                        h: core.h,
                        color: text.0,
                        corner_radius: core.w / 2.0,
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
                        corner_radius: radio_d / 2.0,
                        border_width: theme.control.edge_width_strong,
                        border_color: glass.field_focus_border.0,
                    },
                );
            }

            let content_x = radio.x + radio.w + m.gap + m.gap_sm;
            // Trailing value: caption-sm, primary, right aligned.
            let trailing_style = ty.caption_sm();
            let (tw, _) = TextMeasurer::measure_styled(&row.trailing, &trailing_style, None);
            let trailing_x = r.x + r.w - m.pad - tw;
            let content_w = (trailing_x - m.gap * 2.0 - content_x).max(0.0);
            match row.progress {
                Some(p) => {
                    // Download row: label up, gradient progress under.
                    let label_style = ty.caption_sm();
                    let block_h = label_style.line_height + m.gap_sm + theme.control.progress_track;
                    let top = r.y + (r.h - block_h) / 2.0;
                    self.text(
                        c,
                        layer,
                        &row.label,
                        &label_style,
                        content_x,
                        top,
                        theme.colors.text_mid.0,
                        None,
                    );
                    let track = Rect::new(
                        content_x,
                        top + label_style.line_height + m.gap_sm,
                        content_w,
                        theme.control.progress_track,
                    );
                    self.progress(c, layer, track, p, theme);
                }
                None => {
                    if row.label.is_empty() {
                        // Skeleton row (card 19's fake text): the active
                        // row reads as primary text, the others as the
                        // surface-active wash.
                        let (w, color) = if row.active {
                            (content_w * 0.4, text.0)
                        } else {
                            (content_w * 0.7, glass.surface_active.0)
                        };
                        self.skeleton(
                            c,
                            layer,
                            content_x,
                            r.y + (r.h - theme.control.progress_fill) / 2.0,
                            w,
                            color,
                            theme,
                        );
                    } else {
                        let style = ty.base_2m();
                        self.text(
                            c,
                            layer,
                            &row.label,
                            &style,
                            content_x,
                            r.y + TextMeasurer::vertical_center(&style, r.h),
                            theme.colors.text_mid.0,
                            None,
                        );
                    }
                }
            }

            self.text(
                c,
                layer,
                &row.trailing,
                &trailing_style,
                trailing_x,
                r.y + TextMeasurer::vertical_center(&trailing_style, r.h),
                text.0,
                None,
            );

            y += row_h + m.gap;
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
        let m = Metrics::of(theme);
        let glass = &theme.glass;
        let text = theme.colors.text;
        let ty = &theme.typography;

        // Legend: solid dot + value (base-m primary) + label (caption-r).
        let style = ty.base_m();
        self.dot(
            c,
            layer,
            b.x + m.pad,
            b.y + m.pad + (style.line_height - m.dot) / 2.0,
            true,
            theme,
        );
        let value_x = b.x + m.pad + m.dot + m.gap;
        self.text(c, layer, value, &style, value_x, b.y + m.pad, text.0, None);
        let (vw, _) = TextMeasurer::measure_styled(value, &style, None);
        let label_style = ty.caption_r();
        self.text(
            c,
            layer,
            label,
            &label_style,
            value_x + vw + m.gap + m.gap_sm,
            b.y + m.pad + (style.line_height - label_style.line_height) / 2.0,
            theme.colors.text_mid.0,
            None,
        );

        // Chart: groups of two bars (an `xs` half apart), group gap a
        // share of the width, bars at the micro radius.
        let chart = Rect::new(
            b.x + m.pad,
            b.y + m.pad + style.line_height + m.gap,
            b.w - m.pad * 2.0,
            Self::chart_h(theme),
        );
        let n = groups.len().max(1) as f32;
        let group_gap = chart.w * 0.045;
        let bar_gap = m.gap_sm / 2.0;
        let group_w = (chart.w - group_gap * (n - 1.0)) / n;
        let bar_w = ((group_w - bar_gap) / 2.0).max(1.0);
        let bar_radius = theme.shape.micro.min(bar_w / 2.0);
        for (i, (a, hb)) in groups.iter().enumerate() {
            let gx = chart.x + i as f32 * (group_w + group_gap);
            let heights = [a.clamp(0.0, 1.0), hb.clamp(0.0, 1.0)];
            for (j, t01) in heights.iter().enumerate() {
                let h = chart.h * t01;
                let x = gx + j as f32 * (bar_w + bar_gap);
                let y = chart.y + chart.h - h;
                if i == highlight && j == 1 {
                    // Highlight: gradient from the wash alpha to nothing,
                    // plus a floating cap above the bar.
                    c.push_to_layer(
                        layer,
                        SceneNode::GradientRect {
                            x,
                            y,
                            w: bar_w,
                            h,
                            color: with_alpha(text, glass.wash_hover_alpha),
                            color2: with_alpha(text, 0.0),
                            angle_deg: 180.0,
                            corner_radius: bar_radius,
                            border_width: 0.0,
                            border_color: [0.0; 4],
                        },
                    );
                    let cap_h = theme.control.edge_width_strong;
                    c.push_to_layer(
                        layer,
                        SceneNode::RoundedRect {
                            x,
                            y: y - m.gap_sm,
                            w: bar_w,
                            h: cap_h,
                            color: theme.colors.text_dim.0,
                            corner_radius: cap_h / 2.0,
                            border_width: 0.0,
                            border_color: [0.0; 4],
                        },
                    );
                } else {
                    let color = if j == 0 {
                        glass.surface_hover.0
                    } else {
                        glass.surface_active.0
                    };
                    c.push_to_layer(
                        layer,
                        SceneNode::RoundedRect {
                            x,
                            y,
                            w: bar_w,
                            h,
                            color,
                            corner_radius: bar_radius,
                            border_width: 0.0,
                            border_color: [0.0; 4],
                        },
                    );
                }
            }
        }
        // Baseline (surface-active, bleeding to the shell edge) + ticks.
        let line = theme.control.edge_width_strong;
        let base_y = chart.y + chart.h + m.gap_sm / 2.0;
        c.push_to_layer(
            layer,
            SceneNode::Rect {
                x: b.x + m.gap_sm,
                y: base_y,
                w: b.w - m.gap_sm * 2.0,
                h: line,
                color: glass.surface_active.0,
            },
        );
        let ticks = ((chart.w / theme.spacing.xxl).floor() as usize).max(2);
        for i in 0..ticks {
            let tx = chart.x + chart.w * i as f32 / (ticks - 1) as f32;
            c.push_to_layer(
                layer,
                SceneNode::Rect {
                    x: tx,
                    y: base_y + line + m.gap_sm,
                    w: line,
                    h: m.gap_sm + m.gap_sm / 2.0,
                    color: glass.text_placeholder.0,
                },
            );
        }
    }

    // The parameters mirror `CardVariant::Cta`'s fields, like the other
    // render_* dispatch helpers; a bag struct would only duplicate the enum.
    #[allow(clippy::too_many_arguments)]
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
        let m = Metrics::of(theme);
        let glass = &theme.glass;
        let text = theme.colors.text;
        let ty = &theme.typography;
        let inset = Self::cta_inset(theme);
        let x = b.x + inset;
        let w = b.w - inset * 2.0;

        // Title, then the body paragraph, then the CTA pill.
        let title_style = ty.title();
        self.text(c, layer, title, &title_style, x, b.y + m.pad, text.0, None);
        let style = ty.body();
        let (_, body_h) = TextMeasurer::measure_styled(body, &style, Some(w));
        let body_y = b.y + m.pad + title_style.line_height + m.gap;
        self.text(
            c,
            layer,
            body,
            &style,
            x,
            body_y,
            theme.colors.text_mid.0,
            Some(w),
        );

        // CTA pill: an `Lg` control with the `Xl` padding, base-m label,
        // faint edge ring + frozen glow.
        let bstyle = ty.base_m();
        let (tw, _) = TextMeasurer::measure_styled(button, &bstyle, None);
        let btn = Rect::new(
            x,
            body_y + body_h.max(style.line_height) + theme.spacing.xl,
            tw + theme.control.pad_x(ControlSize::Xl) * 2.0,
            theme.control.height(ControlSize::Lg),
        );
        let ring = with_alpha(glass.field_focus_border, glass.edge_soft.0[3]);
        for node in glass_pill(
            btn,
            btn.h / 2.0,
            ring,
            theme.control.edge_width,
            glass.button.0,
        ) {
            c.push_to_layer(layer, node);
        }
        // The orbiting conic glow, frozen as a faint top wash.
        let glow = btn.inset(theme.control.edge_width);
        c.push_to_layer(
            layer,
            SceneNode::GradientRect {
                x: glow.x,
                y: glow.y,
                w: glow.w,
                h: glow.h,
                color: glass.surface_hover.0,
                color2: with_alpha(text, 0.0),
                angle_deg: 180.0,
                corner_radius: glow.h / 2.0,
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
