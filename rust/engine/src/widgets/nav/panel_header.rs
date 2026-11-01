//! Panel header: the column head of a screen or a pane. Title in the
//! title ramp (text-default tone), an optional blurb under it, an
//! optional count badge and trailing actions on the right. Its measures
//! and tones are the theme's: a panel's head (`theme.disclosure.header`)
//! is the kit's Notifications head in the core theme, a page's head
//! (`theme.disclosure.page`) the kit's Layout title; elsewhere both are
//! one `Xl` control tall plus the blurb. The blurb wraps in the width the
//! head is given, and the head is as tall as the blurb's lines.

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::theme::Theme;
use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use crate::widgets::buttons::Badge;
use crate::widgets::content::HeaderRecipe;
use crate::widgets::widget::Rect;

#[derive(Clone, Debug)]
pub struct PanelHeader {
    pub title: String,
    pub blurb: Option<String>,
    pub badge: Option<Badge>,
    /// Heads a panel's content (`theme.disclosure.header`, the kit's
    /// Notifications head in the core theme) rather than a page (the
    /// shell's head, as every theme drew it before the kit).
    pub framed: bool,
}

impl PanelHeader {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            blurb: None,
            badge: None,
            framed: false,
        }
    }

    /// Heads a panel's content: the theme's panel head.
    pub fn framed(mut self) -> Self {
        self.framed = true;
        self
    }

    pub fn blurb(mut self, blurb: impl Into<String>) -> Self {
        self.blurb = Some(blurb.into());
        self
    }

    pub fn badge(mut self, badge: Badge) -> Self {
        self.badge = Some(badge);
        self
    }

    /// The measures and tones this head draws with, `width` wide.
    fn recipe(&self, theme: &Theme, width: f32) -> HeaderRecipe {
        if self.framed {
            theme.disclosure.header
        } else {
            theme.disclosure.page.at(width)
        }
    }

    /// The title's style at an unbounded width (a page head's wide one).
    pub fn title_style(&self, theme: &Theme) -> TextStyle {
        Self::title_of(&self.recipe(theme, f32::INFINITY), theme)
    }

    fn title_of(r: &HeaderRecipe, theme: &Theme) -> TextStyle {
        theme.typography.run(r.title_type)
    }

    fn blurb_of(r: &HeaderRecipe, theme: &Theme) -> TextStyle {
        theme.typography.run(r.blurb_type)
    }

    /// Side padding of a page head inside the gutter (the shell lines its
    /// page up with it).
    pub fn pad(theme: &Theme) -> f32 {
        theme.disclosure.page.narrow.pad_x
    }

    /// True when this head spans the page column, its sides the page's
    /// gutter (`PageHeadRecipe::bleed`): a page head in a theme that
    /// says so, never a panel's.
    pub fn bleeds(&self, theme: &Theme) -> bool {
        !self.framed && theme.disclosure.page.bleed
    }

    /// Its left and right padding, `width` wide.
    pub fn pads(&self, width: f32, theme: &Theme) -> [f32; 2] {
        [self.recipe(theme, width).pad_x; 2]
    }

    /// The title row with its padding above and below: the band the
    /// title, the badge and the trailing actions center in.
    pub fn band(&self, width: f32, theme: &Theme) -> f32 {
        let r = self.recipe(theme, width);
        r.pad_y * 2.0 + r.row
    }

    /// Height with one blurb line, as at an unbounded width.
    pub fn height(&self, theme: &Theme) -> f32 {
        self.height_in(f32::INFINITY, [0.0; 2], theme)
    }

    /// Height `width` wide, the blurb wrapped in it.
    pub fn height_for(&self, width: f32, theme: &Theme) -> f32 {
        self.height_in(width, self.pads(width, theme), theme)
    }

    /// Height `width` wide, padded `pads` left and right: the band, plus
    /// the gap, the blurb's lines and the padding below when there is a
    /// blurb, plus the rule.
    pub fn height_in(&self, width: f32, pads: [f32; 2], theme: &Theme) -> f32 {
        let r = self.recipe(theme, width);
        let band = r.pad_y * 2.0 + r.row;
        let bounds = Rect::new(0.0, 0.0, width, band);
        match self.blurb_rect(bounds, pads, theme) {
            Some(b) => b.y + b.h + r.pad_y + r.rule,
            None => band + r.rule,
        }
    }

    /// Where the blurb draws in `bounds`, padded `pads`: under the title
    /// row, as wide as the head less its padding, as tall as its lines.
    pub fn blurb_rect(&self, bounds: Rect, pads: [f32; 2], theme: &Theme) -> Option<Rect> {
        let blurb = self.blurb.as_ref()?;
        let r = self.recipe(theme, bounds.w);
        let style = Self::blurb_of(&r, theme);
        let w = (bounds.w - pads[0] - pads[1]).max(0.0);
        let lines = if w.is_finite() {
            TextMeasurer::measure_styled(blurb, &style, Some(w)).1
        } else {
            TextMeasurer::measure_styled(blurb, &style, None).1
        };
        Some(Rect::new(
            bounds.x + pads[0],
            bounds.y + r.pad_y + r.row + r.blurb_gap,
            w,
            lines.max(style.line_height),
        ))
    }

    /// The rect trailing actions (buttons the owner draws) may use, at
    /// the right of the title row.
    pub fn trailing_rect(&self, bounds: Rect, theme: &Theme) -> Rect {
        let r = &self.recipe(theme, bounds.w);
        let pad = r.pad_x;
        let mut right = bounds.x + bounds.w - pad;
        if let Some(badge) = &self.badge {
            right -= badge.preferred_size(theme).0 + r.badge_gap;
        }
        Rect::new(
            bounds.x + pad,
            bounds.y,
            (right - bounds.x - pad).max(0.0),
            r.pad_y * 2.0 + r.row,
        )
    }

    /// Declares this head as `id` in a projection: a `Role::Label` named
    /// by its title, described by its blurb, its badge's text as `badge`.
    /// A head names what follows it and takes no action.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        let mut node = b.node(id, Role::Label, self.title.as_str());
        if let Some(parent) = parent {
            node.parent(parent);
        }
        if let Some(blurb) = &self.blurb {
            node.description(blurb.as_str());
        }
        if let Some(badge) = &self.badge {
            node.state("badge", badge.label.clone());
        }
        node
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        self.render_to_layer(c, LayerId::DEFAULT, bounds, theme);
    }

    pub fn render_to_layer(&self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        self.render_in(c, layer, bounds, self.pads(bounds.w, theme), theme);
    }

    /// Draws in `bounds`, padded `pads` left and right (the shell passes
    /// its gutter to a head that bleeds).
    pub fn render_in(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        bounds: Rect,
        pads: [f32; 2],
        theme: &Theme,
    ) {
        let r = &self.recipe(theme, bounds.w);
        let band = r.pad_y * 2.0 + r.row;
        let title = Self::title_of(r, theme);
        let mut right = bounds.x + bounds.w - pads[1];
        if let Some(badge) = &self.badge {
            let (bw, bh) = badge.preferred_size(theme);
            right -= bw;
            badge.render_to_layer(
                c,
                layer,
                Rect::new(right, bounds.y + (band - bh) / 2.0, bw, bh),
                theme,
            );
            right -= r.badge_gap;
        }
        let avail = (right - bounds.x - pads[0]).max(0.0);
        let shown = TextMeasurer::truncate_to_width(&self.title, &title, avail);
        c.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::from_style(&shown, &title, None),
                x: bounds.x + pads[0],
                y: bounds.y + TextMeasurer::vertical_center(&title, band),
                color: r.title.0,
            },
        );
        if let (Some(blurb), Some(at)) = (&self.blurb, self.blurb_rect(bounds, pads, theme)) {
            let style = Self::blurb_of(r, theme);
            c.push_to_layer(
                layer,
                SceneNode::Text {
                    key: TextNodeKey::from_style(blurb, &style, Some(at.w)),
                    x: at.x,
                    y: at.y,
                    color: r.blurb.0,
                },
            );
        }
        if r.rule > 0.0 {
            let h = self.height_in(bounds.w, pads, theme);
            c.push_to_layer(
                layer,
                SceneNode::Rect {
                    x: bounds.x,
                    y: bounds.y + h - r.rule,
                    w: bounds.w,
                    h: r.rule,
                    color: r.rule_color.0,
                },
            );
        }
    }
}
