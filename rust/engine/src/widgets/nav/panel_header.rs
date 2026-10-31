//! Panel header: the column head of a screen or a pane. Title in the
//! title ramp (text-default tone), an optional blurb under it, an
//! optional count badge and trailing actions on the right. Its measures
//! and tones are the theme's (`theme.disclosure.header`): the kit's
//! Notifications head in the core theme, one `Xl` control tall plus the
//! blurb line elsewhere.

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::theme::Theme;
use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use crate::widgets::buttons::Badge;
use crate::widgets::widget::Rect;

#[derive(Clone, Debug)]
pub struct PanelHeader {
    pub title: String,
    pub blurb: Option<String>,
    pub badge: Option<Badge>,
}

impl PanelHeader {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            blurb: None,
            badge: None,
        }
    }

    pub fn blurb(mut self, blurb: impl Into<String>) -> Self {
        self.blurb = Some(blurb.into());
        self
    }

    pub fn badge(mut self, badge: Badge) -> Self {
        self.badge = Some(badge);
        self
    }

    pub fn title_style(theme: &Theme) -> TextStyle {
        let r = &theme.disclosure.header;
        theme
            .typography
            .exact(r.title_size, r.title_line_height, r.title_weight)
    }

    fn blurb_style(theme: &Theme) -> TextStyle {
        let r = &theme.disclosure.header;
        theme
            .typography
            .exact(r.blurb_size, r.blurb_line_height, r.blurb_weight)
    }

    /// Side padding (`theme.disclosure.header`).
    pub fn pad(theme: &Theme) -> f32 {
        theme.disclosure.header.pad_x
    }

    /// Header height: the title row, plus the blurb line, the gap above
    /// it and the space below it when there is a blurb, plus the rule.
    pub fn height(&self, theme: &Theme) -> f32 {
        let r = &theme.disclosure.header;
        let head = match self.blurb {
            Some(_) => r.row + Self::blurb_style(theme).line_height + r.blurb_gap + r.blurb_bottom,
            None => r.row,
        };
        head + r.rule
    }

    /// The rect trailing actions (buttons the owner draws) may use, at
    /// the right of the title row.
    pub fn trailing_rect(&self, bounds: Rect, theme: &Theme) -> Rect {
        let r = &theme.disclosure.header;
        let pad = Self::pad(theme);
        let mut right = bounds.x + bounds.w - pad;
        if let Some(badge) = &self.badge {
            right -= badge.preferred_size(theme).0 + r.badge_gap;
        }
        Rect::new(
            bounds.x + pad,
            bounds.y,
            (right - bounds.x - pad).max(0.0),
            r.row,
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
        let r = &theme.disclosure.header;
        let pad = Self::pad(theme);
        let row = r.row;
        let title = Self::title_style(theme);
        let mut right = bounds.x + bounds.w - pad;
        if let Some(badge) = &self.badge {
            let (bw, bh) = badge.preferred_size(theme);
            right -= bw;
            badge.render_to_layer(
                c,
                layer,
                Rect::new(right, bounds.y + (row - bh) / 2.0, bw, bh),
                theme,
            );
            right -= r.badge_gap;
        }
        let avail = (right - bounds.x - pad).max(0.0);
        let shown = TextMeasurer::truncate_to_width(&self.title, &title, avail);
        c.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::from_style(&shown, &title, None),
                x: bounds.x + pad,
                y: bounds.y + TextMeasurer::vertical_center(&title, row),
                color: r.title.0,
            },
        );
        if let Some(blurb) = &self.blurb {
            let style = Self::blurb_style(theme);
            c.push_to_layer(
                layer,
                SceneNode::Text {
                    key: TextNodeKey::from_style(blurb, &style, Some(bounds.w - pad * 2.0)),
                    x: bounds.x + pad,
                    y: bounds.y + row + r.blurb_gap,
                    color: r.blurb.0,
                },
            );
        }
        if r.rule > 0.0 {
            let h = self.height(theme);
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
