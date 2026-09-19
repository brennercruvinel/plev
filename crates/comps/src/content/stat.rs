//! Stat: a key/value readout. The value in the headline ramp over a
//! caption label, optionally with a unit after the value and a delta
//! badge. The stat card's numbers and every "N chunks / 1.3 GB" line in
//! an app are this one block, so they line up.

use engine::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::text::{TextMeasurer, TextStyle};
use engine::theme::{Intent, Theme};

use crate::action::Badge;
use crate::core::Rect;

/// Value size on the type ramp.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StatSize {
    /// Inline readout: base-m value, small label.
    Sm,
    /// Card readout: title value, caption label.
    #[default]
    Md,
    /// Hero readout: headline value, base-2r label.
    Lg,
}

#[derive(Clone, Debug)]
pub struct Stat {
    pub value: String,
    pub label: String,
    pub unit: Option<String>,
    /// Delta badge next to the value ("+12.4%"), tinted by its intent.
    pub delta: Option<(String, Intent)>,
    pub size: StatSize,
}

impl Stat {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            unit: None,
            delta: None,
            size: StatSize::default(),
        }
    }

    pub fn unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = Some(unit.into());
        self
    }

    pub fn delta(mut self, delta: impl Into<String>, intent: Intent) -> Self {
        self.delta = Some((delta.into(), intent));
        self
    }

    pub fn size(mut self, size: StatSize) -> Self {
        self.size = size;
        self
    }

    pub fn value_style(&self, theme: &Theme) -> TextStyle {
        let ty = &theme.typography;
        match self.size {
            StatSize::Sm => ty.base_m(),
            StatSize::Md => ty.title(),
            StatSize::Lg => ty.headline(),
        }
    }

    pub fn label_style(&self, theme: &Theme) -> TextStyle {
        let ty = &theme.typography;
        match self.size {
            StatSize::Sm => ty.small_r(),
            StatSize::Md => ty.caption_r(),
            StatSize::Lg => ty.base_2r(),
        }
    }

    /// Unit style: one ramp step under the value.
    fn unit_style(&self, theme: &Theme) -> TextStyle {
        let ty = &theme.typography;
        match self.size {
            StatSize::Sm => ty.caption_r(),
            StatSize::Md => ty.base_2r(),
            StatSize::Lg => ty.title(),
        }
    }

    fn delta_badge(&self) -> Option<Badge> {
        self.delta
            .as_ref()
            .map(|(text, intent)| Badge::tag(text.clone()).intent(*intent))
    }

    /// Intrinsic size from real text measurement: the widest of the
    /// value row and the label, the two rows plus the `xs` gap tall.
    pub fn preferred_size(&self, theme: &Theme) -> (f32, f32) {
        let value = self.value_style(theme);
        let label = self.label_style(theme);
        let (vw, _) = TextMeasurer::measure_styled(&self.value, &value, None);
        let mut row_w = vw;
        if let Some(unit) = &self.unit {
            let (uw, _) = TextMeasurer::measure_styled(unit, &self.unit_style(theme), None);
            row_w += theme.spacing.xs + uw;
        }
        if let Some(badge) = self.delta_badge() {
            row_w += theme.spacing.sm + badge.preferred_size(theme).0;
        }
        let (lw, _) = TextMeasurer::measure_styled(&self.label, &label, None);
        (
            row_w.max(lw).ceil(),
            value.line_height + theme.spacing.xs + label.line_height,
        )
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        self.render_to_layer(c, LayerId::DEFAULT, bounds, theme);
    }

    /// Draw at the top-left of `bounds`.
    pub fn render_to_layer(&self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        let value = self.value_style(theme);
        let label = self.label_style(theme);
        let (vw, _) = TextMeasurer::measure_styled(&self.value, &value, None);
        c.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::from_style(&self.value, &value, None),
                x: bounds.x,
                y: bounds.y,
                color: theme.colors.text.0,
            },
        );
        let mut x = bounds.x + vw;
        if let Some(unit) = &self.unit {
            let style = self.unit_style(theme);
            x += theme.spacing.xs;
            c.push_to_layer(
                layer,
                SceneNode::Text {
                    key: TextNodeKey::from_style(unit, &style, None),
                    x,
                    // Baseline-ish alignment: bottoms of the line boxes meet.
                    y: bounds.y + value.line_height - style.line_height,
                    color: theme.colors.text_mid.0,
                },
            );
            x += TextMeasurer::measure_styled(unit, &style, None).0;
        }
        if let Some(badge) = self.delta_badge() {
            let (_, bh) = badge.preferred_size(theme);
            x += theme.spacing.sm;
            badge.render_to_layer(
                c,
                layer,
                Rect::new(x, bounds.y + (value.line_height - bh) / 2.0, 0.0, 0.0),
                theme,
            );
        }
        c.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::from_style(&self.label, &label, None),
                x: bounds.x,
                y: bounds.y + value.line_height + theme.spacing.xs,
                color: theme.colors.text_dim.0,
            },
        );
    }
}
