//! Stat: a key/value readout. The value over a label, optionally with a
//! unit after the value and a delta badge. The stat card's numbers and
//! every "N chunks / 1.3 GB" line in an app are this one block, so they
//! line up. Every visual value is the theme's `navigation.stat` recipe.
//!
//! The stat answers to the width it gets: where the value row and the
//! delta do not fit side by side, the delta wraps under the value, and
//! a label wider than the bounds truncates. [`Stat::project`] declares
//! the readout as a label named by its label, its value (with the unit)
//! as its value and the delta as its description.

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::theme::{Intent, Theme};
use crate::widgets::nav::{StatLevel, StatRecipe};
use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use crate::widgets::buttons::Badge;
use crate::widgets::widget::Rect;

/// Value size on the type ramp.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StatSize {
    /// Inline readout.
    Sm,
    /// Card readout.
    #[default]
    Md,
    /// Hero readout.
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
    sem: Option<SemId>,
}

impl Stat {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            unit: None,
            delta: None,
            size: StatSize::default(),
            sem: None,
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

    /// The logical identity the render binds the readout to.
    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    /// Declares the readout as `id`: a `Role::Label` named by its label,
    /// its value and unit as the `value` a reader hears, its delta as the
    /// description. It only reads the widget. Returns the node for the
    /// app to add to.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        let mut node = b.node(id, Role::Label, self.label.as_str());
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.state("value", self.spoken_value());
        if let Some((delta, _)) = &self.delta {
            node.description(delta.as_str());
        }
        node
    }

    /// The value with its unit, as a reader hears it.
    pub fn spoken_value(&self) -> String {
        match &self.unit {
            Some(unit) => format!("{} {unit}", self.value),
            None => self.value.clone(),
        }
    }

    fn recipe(theme: &Theme) -> &StatRecipe {
        &theme.navigation.stat
    }

    fn level(&self, theme: &Theme) -> StatLevel {
        let r = Self::recipe(theme);
        match self.size {
            StatSize::Sm => r.sm,
            StatSize::Md => r.md,
            StatSize::Lg => r.lg,
        }
    }

    pub fn value_style(&self, theme: &Theme) -> TextStyle {
        self.level(theme).value.style(&theme.typography)
    }

    pub fn label_style(&self, theme: &Theme) -> TextStyle {
        self.level(theme).label.style(&theme.typography)
    }

    fn unit_style(&self, theme: &Theme) -> TextStyle {
        self.level(theme).unit.style(&theme.typography)
    }

    fn delta_badge(&self) -> Option<Badge> {
        self.delta
            .as_ref()
            .map(|(text, intent)| Badge::tag(text.clone()).intent(*intent))
    }

    /// The value and its unit, side by side.
    fn value_row_w(&self, theme: &Theme) -> f32 {
        let r = Self::recipe(theme);
        let (vw, _) = TextMeasurer::measure_styled(&self.value, &self.value_style(theme), None);
        match &self.unit {
            Some(unit) => {
                let (uw, _) = TextMeasurer::measure_styled(unit, &self.unit_style(theme), None);
                vw + r.unit_gap + uw
            }
            None => vw,
        }
    }

    /// Whether the delta wraps under the value at `width`.
    pub fn delta_wraps(&self, width: f32, theme: &Theme) -> bool {
        let Some(badge) = self.delta_badge() else {
            return false;
        };
        let r = Self::recipe(theme);
        self.value_row_w(theme) + r.delta_gap + badge.preferred_size(theme).0 > width
    }

    /// Intrinsic size from real text measurement: the widest of the
    /// value row and the label, the two rows plus the gap tall.
    pub fn preferred_size(&self, theme: &Theme) -> (f32, f32) {
        let r = Self::recipe(theme);
        let mut row_w = self.value_row_w(theme);
        if let Some(badge) = self.delta_badge() {
            row_w += r.delta_gap + badge.preferred_size(theme).0;
        }
        let (lw, _) = TextMeasurer::measure_styled(&self.label, &self.label_style(theme), None);
        (row_w.max(lw).ceil(), self.height_at(f32::INFINITY, theme))
    }

    /// The narrowest the stat draws whole in: the value row (the label
    /// truncates, and a delta wrapped under the value is cut to the
    /// stat's width, its whole text kept in the projection).
    pub fn min_width(&self, theme: &Theme) -> f32 {
        self.value_row_w(theme).ceil()
    }

    /// The height at `width`: one row more where the delta wraps.
    pub fn height_at(&self, width: f32, theme: &Theme) -> f32 {
        let r = Self::recipe(theme);
        let value = self.value_style(theme).line_height;
        let label = self.label_style(theme).line_height;
        let wrapped = match self.delta_badge() {
            Some(badge) if self.delta_wraps(width, theme) => {
                badge.preferred_size(theme).1 + r.row_gap
            }
            _ => 0.0,
        };
        value + r.row_gap + wrapped + label
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        self.render_to_layer(c, LayerId::DEFAULT, bounds, theme);
    }

    /// Draw at the top-left of `bounds`.
    pub fn render_to_layer(&self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        let r = Self::recipe(theme);
        if let Some(id) = &self.sem {
            let h = self.height_at(bounds.w, theme);
            c.bind_to_layer(layer, id, [bounds.x, bounds.y, bounds.w, h]);
        }
        let value = self.value_style(theme);
        let label = self.label_style(theme);
        let (vw, _) = TextMeasurer::measure_styled(&self.value, &value, None);
        c.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::from_style(&self.value, &value, None),
                x: bounds.x,
                y: bounds.y,
                color: r.value.0,
            },
        );
        let mut x = bounds.x + vw;
        if let Some(unit) = &self.unit {
            let style = self.unit_style(theme);
            x += r.unit_gap;
            c.push_to_layer(
                layer,
                SceneNode::Text {
                    key: TextNodeKey::from_style(unit, &style, None),
                    x,
                    // Baseline-ish alignment: bottoms of the line boxes meet.
                    y: bounds.y + value.line_height - style.line_height,
                    color: r.unit.0,
                },
            );
            x += TextMeasurer::measure_styled(unit, &style, None).0;
        }
        let mut label_y = bounds.y + value.line_height + r.row_gap;
        if let Some(badge) = self.delta_badge() {
            let (_, bh) = badge.preferred_size(theme);
            let at = if self.delta_wraps(bounds.w, theme) {
                let at = Rect::new(bounds.x, label_y, bounds.w.max(0.0), 0.0);
                label_y += bh + r.row_gap;
                at
            } else {
                let y = bounds.y + (value.line_height - bh) / 2.0;
                Rect::new(x + r.delta_gap, y, 0.0, 0.0)
            };
            badge.render_to_layer(c, layer, at, theme);
        }
        let shown = TextMeasurer::truncate_to_width(&self.label, &label, bounds.w.max(0.0));
        c.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::from_style(&shown, &label, None),
                x: bounds.x,
                y: label_y,
                color: r.label.0,
            },
        );
    }
}
