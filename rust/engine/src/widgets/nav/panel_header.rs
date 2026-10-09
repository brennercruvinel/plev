//! Panel header: the column head of a screen or a pane. Title in the
//! title ramp (text-default tone), an optional blurb under it, an
//! optional count badge and trailing actions on the right. One `Xl`
//! control tall plus the blurb line when there is one.

use engine::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::text::{TextMeasurer, TextStyle};
use engine::theme::{ControlSize, Theme};

use crate::action::Badge;
use crate::core::Rect;

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
        theme.typography.title()
    }

    fn blurb_style(theme: &Theme) -> TextStyle {
        theme.typography.base_2r()
    }

    /// Side padding: the `md` step.
    pub fn pad(theme: &Theme) -> f32 {
        theme.spacing.md
    }

    /// Header height: the `Xl` control for the title row, plus the blurb
    /// line and the `xs` gap when there is a blurb.
    pub fn height(&self, theme: &Theme) -> f32 {
        let row = theme.control.height(ControlSize::Xl);
        match self.blurb {
            Some(_) => row + Self::blurb_style(theme).line_height + theme.spacing.xs,
            None => row,
        }
    }

    /// The rect trailing actions (buttons the owner draws) may use, at
    /// the right of the title row.
    pub fn trailing_rect(&self, bounds: Rect, theme: &Theme) -> Rect {
        let row = theme.control.height(ControlSize::Xl);
        let pad = Self::pad(theme);
        let mut right = bounds.x + bounds.w - pad;
        if let Some(badge) = &self.badge {
            right -= badge.preferred_size(theme).0 + theme.spacing.sm;
        }
        Rect::new(
            bounds.x + pad,
            bounds.y,
            (right - bounds.x - pad).max(0.0),
            row,
        )
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        self.render_to_layer(c, LayerId::DEFAULT, bounds, theme);
    }

    pub fn render_to_layer(&self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        let pad = Self::pad(theme);
        let row = theme.control.height(ControlSize::Xl);
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
            right -= theme.spacing.sm;
        }
        let avail = (right - bounds.x - pad).max(0.0);
        let shown = TextMeasurer::truncate_to_width(&self.title, &title, avail);
        c.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::from_style(&shown, &title, None),
                x: bounds.x + pad,
                y: bounds.y + TextMeasurer::vertical_center(&title, row),
                color: theme.glass.text_default.0,
            },
        );
        if let Some(blurb) = &self.blurb {
            let style = Self::blurb_style(theme);
            c.push_to_layer(
                layer,
                SceneNode::Text {
                    key: TextNodeKey::from_style(blurb, &style, Some(bounds.w - pad * 2.0)),
                    x: bounds.x + pad,
                    y: bounds.y + row + theme.spacing.xs,
                    color: theme.colors.text_dim.0,
                },
            );
        }
    }
}
