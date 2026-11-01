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
//! Each family draws in a file of its own (`stat`, `profile`, `media`,
//! `list`, `chart`, `cta`), the shell and the shared pieces in `parts`.
//!
//! Rendering uses the SDF pipeline (gradients, analytic shadows), images
//! and text only, plus the play icon; every element stacks correctly
//! within a single layer (quads, shadows, SDF, images, text).

mod chart;
mod cta;
mod list;
mod media;
mod parts;
mod profile;
mod stat;

use crate::drawing::compositor::{Compositor, LayerId};
use crate::drawing::gpu::image::ImageHandle;
use crate::drawing::text::TextMeasurer;
use crate::styling::theme::TypeRole;
use crate::styling::theme::{ControlSize, Theme};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

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
    /// variant's content ([`Card::height_for`] that width).
    pub fn preferred_size(&self, theme: &Theme) -> (f32, f32) {
        let w = self.width_for(theme);
        (w, self.height_for(w, theme))
    }

    /// How tall the card is `w` wide: its blocks stacked, the texts that
    /// wrap (the profile's bio, the call to action's body) wrapped in the
    /// width they get there, the media box at its aspect. A grid gives
    /// each card its column and asks this, rather than a fixed height.
    pub fn height_for(&self, w: f32, theme: &Theme) -> f32 {
        let m = Metrics::of(theme);
        let ty = &theme.typography;
        let h = match &self.variant {
            CardVariant::Stat { .. } => {
                // Ruler, headline, dot label, each separated by a gap.
                m.pad
                    + Self::ruler_h(theme)
                    + m.gap
                    + ty.role(TypeRole::H3).line_height
                    + m.gap_sm
                    + ty.role(TypeRole::Body).line_height
                    + m.pad
            }
            CardVariant::Profile { bio, .. } => {
                let inner = Self::profile_pad(theme);
                let avatar = Self::avatar_size(theme);
                let bio_w = (w - inner * 2.0 - avatar - m.gap * 2.0).max(0.0);
                let (_, bio_h) =
                    TextMeasurer::measure_styled(bio, &ty.role(TypeRole::Paragraph), Some(bio_w));
                inner + avatar + m.gap + bio_h.max(ty.role(TypeRole::Paragraph).line_height) + inner
            }
            CardVariant::Media { .. } => {
                let (_, box_h) = Self::media_box(theme, w);
                m.pad
                    + Self::browser_dot(theme)
                    + m.gap
                    + box_h
                    + m.pad
                    + ty.role(TypeRole::Title).line_height
                    + m.gap_sm
                    + ty.role(TypeRole::Body).line_height
                    + m.pad
            }
            CardVariant::List { rows, .. } => {
                let row_h = Self::list_row_h(theme);
                let n = rows.len() as f32;
                m.pad
                    + ty.role(TypeRole::Title).line_height
                    + m.gap
                    + n * row_h
                    + (n - 1.0).max(0.0) * m.gap
                    + m.pad
            }
            CardVariant::Chart { .. } => {
                m.pad
                    + ty.run(TypeRole::BodyLg.medium()).line_height
                    + m.gap
                    + Self::chart_h(theme)
                    + m.gap
                    + Self::axis_h(theme)
                    + m.pad
            }
            CardVariant::Cta { body, .. } => {
                let inset = Self::cta_inset(theme);
                let style = ty.role(TypeRole::BodyLg);
                let (_, body_h) =
                    TextMeasurer::measure_styled(body, &style, Some((w - inset * 2.0).max(0.0)));
                m.pad
                    + ty.role(TypeRole::Title).line_height
                    + m.gap
                    + body_h.max(style.line_height)
                    + theme.spacing.xl
                    + theme.control.height(ControlSize::Lg)
                    + theme.spacing.xxl
            }
        };
        h.ceil()
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
}
