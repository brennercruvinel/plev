//! The radio group: one choice out of a list, each row a disc and its
//! label. The kit has no radio; the row is its single-choice list line
//! (`components/Zoom`) and the disc its on/off control (`components/
//! Switch`), so every value is `theme.selection.radio` and the surfaces
//! are painted by the buttons' `paint_surface`. Each row presses through
//! the buttons' [`Press`].
//!
//! The rows stack, or, [`RadioLayout::Flow`], run along a line and wrap
//! when the width runs out; a label longer than its row truncates. The
//! disc alone is [`paint_radio`], which a card's list rows draw too.
//!
//! With a [`SemId`] ([`RadioGroup::sem`]) the render binds the group and
//! each row (`<id>[<index>]`), and [`RadioGroup::project`] declares a
//! `Role::Group` of `Role::Radio` rows ([`project_choices`], the toggle
//! group's projection too).

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::theme::Theme;
use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use crate::widgets::buttons::{Press, paint_surface};
use crate::widgets::recipe::{focus_ring_inside, rounded_rect};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

/// How the rows sit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RadioLayout {
    /// One row under the other, as wide as the group.
    #[default]
    Stack,
    /// Along a line, wrapping onto the next one.
    Flow,
}

/// Paint one radio disc in `slot` (centered): the well, and on a chosen
/// one the dark well and its dot; faded by `alpha`.
pub fn paint_radio(
    c: &mut Compositor,
    layer: LayerId,
    slot: Rect,
    checked: bool,
    alpha: f32,
    theme: &Theme,
) {
    let t = &theme.selection.radio;
    let disc = Rect::new(
        slot.x + (slot.w - t.disc) / 2.0,
        slot.y + (slot.h - t.disc) / 2.0,
        t.disc,
        t.disc,
    );
    let surface = if checked { &t.on } else { &t.off };
    paint_surface(c, layer, disc, t.disc / 2.0, surface, alpha, false, theme);
    if checked {
        let dot = disc.inset((t.disc - t.dot_d) / 2.0);
        paint_surface(c, layer, dot, t.dot_d / 2.0, &t.dot, alpha, false, theme);
    }
}

/// A group of choices as a projection: `id` a `Role::Group` named
/// `label` whose text `selected` is the chosen key (its index), and one
/// `Role::Radio` `<id>[<index>]` per item, named by it, with its
/// `checked` and a `select` that reads the group's `selected`. The radio
/// group and the toggle group project this. Returns the group's node.
#[track_caller]
pub(crate) fn project_choices<'b>(
    b: &'b mut SnapshotBuilder,
    id: &SemId,
    label: String,
    parent: Option<&SemId>,
    items: &[String],
    selected: usize,
    disabled: bool,
) -> NodeBuilder<'b> {
    let reads = format!("{id}:selected");
    for (i, item) in items.iter().enumerate() {
        let mut node = b.node(id.key(i.to_string()), Role::Radio, item.as_str());
        node.parent(id);
        node.state("checked", i == selected).enabled(!disabled);
        node.action("select").reads(reads.as_str());
    }
    let mut node = b.node(id, Role::Group, label);
    if let Some(parent) = parent {
        node.parent(parent);
    }
    node.state("selected", selected.to_string())
        .enabled(!disabled);
    node
}

#[derive(Clone, Debug)]
pub struct RadioGroup {
    /// One label per row; each is its row's accessible name.
    pub items: Vec<String>,
    pub selected: usize,
    pub disabled: bool,
    pub layout: RadioLayout,
    presses: Vec<Press>,
    focused: Option<usize>,
    sem: Option<SemId>,
}

impl RadioGroup {
    /// A group over `items`, the first chosen.
    pub fn new(items: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let items: Vec<String> = items.into_iter().map(Into::into).collect();
        Self {
            presses: vec![Press::default(); items.len()],
            items,
            selected: 0,
            disabled: false,
            layout: RadioLayout::Stack,
            focused: None,
            sem: None,
        }
    }

    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index.min(self.items.len().saturating_sub(1));
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn layout(mut self, layout: RadioLayout) -> Self {
        self.layout = layout;
        self
    }

    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    /// Choose `index`, as a click on its row does. A disabled group, an
    /// index out of range or the chosen one change nothing; returns
    /// whether the choice moved.
    pub fn select(&mut self, index: usize) -> bool {
        if self.disabled || index >= self.items.len() || index == self.selected {
            return false;
        }
        self.selected = index;
        true
    }

    pub fn hovered(&self) -> Option<usize> {
        self.presses.iter().position(|p| p.hovered)
    }

    /// Keyboard focus on row `index`, or on none; a disabled group
    /// refuses it.
    pub fn set_focused(&mut self, index: Option<usize>) {
        self.focused = index.filter(|i| *i < self.items.len() && !self.disabled);
    }

    pub fn focused(&self) -> Option<usize> {
        self.focused
    }

    pub fn text_style(theme: &Theme) -> TextStyle {
        let t = &theme.selection.radio;
        theme
            .typography
            .exact(t.font_size, t.line_height, t.font_weight)
    }

    /// The space a row takes besides its label.
    fn chrome(theme: &Theme) -> f32 {
        let t = &theme.selection.radio;
        t.row_pad_x * 2.0 + t.slot + t.slot_gap
    }

    fn row_w(&self, i: usize, theme: &Theme) -> f32 {
        let (w, _) = TextMeasurer::measure_styled(&self.items[i], &Self::text_style(theme), None);
        (w + Self::chrome(theme)).ceil()
    }

    /// The narrowest a row gets: its chrome and an ellipsis.
    pub fn min_width(&self, theme: &Theme) -> f32 {
        let (w, _) = TextMeasurer::measure_styled("…", &Self::text_style(theme), None);
        (w + Self::chrome(theme)).ceil()
    }

    /// The rows in `bounds`: stacked as wide as the group, or flowing
    /// along lines at their own width (clamped to the group's).
    pub fn row_rects(&self, bounds: Rect, theme: &Theme) -> Vec<Rect> {
        let t = &theme.selection.radio;
        let mut out = Vec::with_capacity(self.items.len());
        let (mut x, mut y) = (bounds.x, bounds.y);
        for i in 0..self.items.len() {
            match self.layout {
                RadioLayout::Stack => {
                    out.push(Rect::new(bounds.x, y, bounds.w, t.row_h));
                    y += t.row_h;
                }
                RadioLayout::Flow => {
                    let w = self.row_w(i, theme).min(bounds.w);
                    if x > bounds.x && x + w > bounds.right() {
                        x = bounds.x;
                        y += t.row_h;
                    }
                    out.push(Rect::new(x, y, w, t.row_h));
                    x += w;
                }
            }
        }
        out
    }

    /// The group's size within `max_w`: stacked rows as wide as the
    /// widest (clamped), or the flowed lines.
    pub fn size_within(&self, max_w: f32, theme: &Theme) -> (f32, f32) {
        let max_w = max_w.max(self.min_width(theme));
        let widest = (0..self.items.len())
            .map(|i| self.row_w(i, theme))
            .fold(0.0, f32::max);
        let probe = Rect::new(0.0, 0.0, widest.min(max_w), 0.0);
        let probe = match self.layout {
            RadioLayout::Stack => probe,
            RadioLayout::Flow => Rect::new(0.0, 0.0, max_w, 0.0),
        };
        let rows = self.row_rects(probe, theme);
        let w = rows.iter().map(|r| r.right()).fold(0.0, f32::max);
        let h = rows.iter().map(|r| r.bottom()).fold(0.0, f32::max);
        (w.ceil(), h)
    }

    /// The label of row `index` as drawn in `row`: whole when it fits,
    /// truncated with an ellipsis when it does not.
    pub fn shown_label(&self, index: usize, row: Rect, theme: &Theme) -> String {
        let Some(label) = self.items.get(index) else {
            return String::new();
        };
        let style = Self::text_style(theme);
        let room = (row.w - Self::chrome(theme)).max(0.0);
        if TextMeasurer::measure_styled(label, &style, None).0 <= room {
            label.clone()
        } else {
            TextMeasurer::truncate_to_width(label, &style, room)
        }
    }

    /// Pointer events: each row presses on its own; a click chooses it.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        let rects = self.row_rects(bounds, theme);
        let mut result = EventResult::IGNORED;
        let mut chosen = None;
        for (i, (press, rect)) in self.presses.iter_mut().zip(&rects).enumerate() {
            let r = press.handle(event, *rect, self.disabled);
            if r.clicked {
                chosen = Some(i);
            }
            result = result.merge(r);
        }
        if let Some(i) = chosen {
            self.select(i);
        }
        result
    }

    /// Declares the group as `id` ([`project_choices`]).
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        label: impl Into<String>,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        project_choices(
            b,
            id,
            label.into(),
            parent,
            &self.items,
            self.selected,
            self.disabled,
        )
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        let t = &theme.selection.radio;
        let rects = self.row_rects(bounds, theme);
        if let Some(id) = &self.sem {
            c.bind(id, [bounds.x, bounds.y, bounds.w, bounds.h]);
            for (i, r) in rects.iter().enumerate() {
                c.bind(&id.key(i.to_string()), [r.x, r.y, r.w, r.h]);
            }
        }
        let alpha = if self.disabled { t.disabled_alpha } else { 1.0 };
        let style = Self::text_style(theme);
        for (i, row) in rects.iter().enumerate() {
            if self.presses.get(i).is_some_and(|p| p.hovered) {
                let [r, g, b, a] = t.row_hover.0;
                c.push(rounded_rect(
                    row.x,
                    row.y,
                    row.w,
                    row.h,
                    t.row_radius,
                    [r, g, b, a * alpha],
                ));
            }
            let slot = Rect::new(
                row.x + t.row_pad_x,
                row.y + (row.h - t.slot) / 2.0,
                t.slot,
                t.slot,
            );
            paint_radio(c, LayerId::DEFAULT, slot, i == self.selected, alpha, theme);
            let shown = self.shown_label(i, *row, theme);
            let [r, g, b, a] = t.label.0;
            c.push(SceneNode::Text {
                key: TextNodeKey::from_style(&shown, &style, None),
                x: slot.right() + t.slot_gap,
                y: row.y + TextMeasurer::vertical_center(&style, row.h),
                color: [r, g, b, a * alpha],
            });
            if self.focused == Some(i) {
                c.push(focus_ring_inside(*row, t.row_radius, theme));
            }
        }
    }
}
