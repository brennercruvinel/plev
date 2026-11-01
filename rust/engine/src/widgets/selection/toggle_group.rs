//! The toggle group of the hoff core kit: one choice out of a few, as
//! the kit's segmented control (`components/Tabs/index.tsx`). A well
//! holds equal segments; a raised thumb slides under the chosen one.
//! Every value is `theme.selection.segmented`; the well and the thumb
//! are surfaces painted by the buttons' `paint_surface`, and each segment
//! presses through the buttons' [`Press`]: no second press model.
//!
//! The thumb's travel is a spring over the segment index, so the motion
//! is a proportion of the group, not a distance in pixels; with reduced
//! motion it lands at once (`compose::animation::reduced_motion`).
//!
//! The width is intrinsic (the widest label, padded by half a segment's
//! height on each side, times the count); given less, each label
//! truncates inside its segment, down to [`ToggleGroup::min_width`].
//!
//! With a [`SemId`] ([`ToggleGroup::sem`]) the render binds the group and
//! each segment (`<id>[<index>]`), and [`ToggleGroup::project`] declares a
//! `Role::Group` whose text `selected` names the chosen segment, each a
//! `Role::Radio` with its `checked` and a `select` that reads the group's
//! `selected` (the single toggle group of the WAI-ARIA patterns).

use crate::compose::animation::Spring;
use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::theme::{MotionPhysics, Theme};
use accord::{NodeBuilder, SemId, SnapshotBuilder};

use super::radio_group::project_choices;
use super::tokens::{SegmentMetrics, SegmentedTokens};
use crate::widgets::buttons::{Press, paint_surface};
use crate::widgets::recipe::focus_ring_inside;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

/// The truncation mark, measured for [`ToggleGroup::min_width`].
const ELLIPSIS: &str = "…";

/// The two steps of the kit's segment ladder.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SegmentSize {
    /// `h-8`, 13px: the default.
    #[default]
    Md,
    /// `h-7`, 12px: the kit's `isMedium`.
    Sm,
}

impl SegmentSize {
    pub fn metrics(self, theme: &Theme) -> SegmentMetrics {
        theme.selection.segmented.sizes[self as usize]
    }

    /// Label style, one source for measuring and drawing.
    pub fn text_style(self, theme: &Theme) -> TextStyle {
        theme.typography.run(self.metrics(theme).text)
    }

    /// A segment's side padding: half its height, a proportion of the
    /// segment rather than a length of its own.
    fn pad_x(self, theme: &Theme) -> f32 {
        self.metrics(theme).height / 2.0
    }
}

#[derive(Clone, Debug)]
pub struct ToggleGroup {
    /// One label per segment; each is its segment's accessible name.
    pub items: Vec<String>,
    /// The chosen segment.
    pub selected: usize,
    pub disabled: bool,
    pub size: SegmentSize,
    presses: Vec<Press>,
    /// Keyboard focus on one segment, driven by the owning view.
    focused: Option<usize>,
    thumb: Spring<f32>,
    sem: Option<SemId>,
}

impl ToggleGroup {
    /// A group over `items`, the first chosen.
    pub fn new(items: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let items: Vec<String> = items.into_iter().map(Into::into).collect();
        Self {
            presses: vec![Press::default(); items.len()],
            items,
            selected: 0,
            disabled: false,
            size: SegmentSize::Md,
            focused: None,
            thumb: Spring::new(0.0),
            sem: None,
        }
    }

    /// The chosen segment, with the thumb already under it.
    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index.min(self.items.len().saturating_sub(1));
        self.thumb = Spring::new(self.selected as f32);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn size(mut self, size: SegmentSize) -> Self {
        self.size = size;
        self
    }

    /// The theme's motion physics for the thumb's spring.
    pub fn with_motion(mut self, motion: &MotionPhysics) -> Self {
        self.thumb = self.thumb.with_motion(motion);
        self
    }

    /// The logical identity the render binds the group to; each segment
    /// binds as `<id>[<index>]`.
    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    /// Choose `index`, as a click on it does; the thumb slides there. A
    /// disabled group, an index out of range or the chosen one change
    /// nothing; returns whether the choice moved.
    pub fn select(&mut self, index: usize) -> bool {
        if self.disabled || index >= self.items.len() || index == self.selected {
            return false;
        }
        self.selected = index;
        self.thumb.set_target(index as f32);
        true
    }

    /// Advance the thumb. Returns `true` while it still moves.
    pub fn tick(&mut self, dt: f32) -> bool {
        self.thumb.tick(dt);
        self.thumb.is_animating()
    }

    pub fn is_animating(&self) -> bool {
        self.thumb.is_animating()
    }

    /// Where the thumb is, in segments from the first.
    pub fn thumb_position(&self) -> f32 {
        self.thumb.get()
    }

    pub fn hovered(&self) -> Option<usize> {
        self.presses.iter().position(|p| p.hovered)
    }

    /// Keyboard focus on the segment at `index`, or on none; a disabled
    /// group refuses it.
    pub fn set_focused(&mut self, index: Option<usize>) {
        self.focused = index.filter(|i| *i < self.items.len() && !self.disabled);
    }

    pub fn focused(&self) -> Option<usize> {
        self.focused
    }

    /// The well's border and padding, on each side.
    fn inset(theme: &Theme) -> f32 {
        let t = &theme.selection.segmented;
        t.edge + t.pad
    }

    /// The group's height: one segment in the well.
    pub fn height(&self, theme: &Theme) -> f32 {
        self.size.metrics(theme).height + Self::inset(theme) * 2.0
    }

    fn label_w(&self, label: &str, theme: &Theme) -> f32 {
        TextMeasurer::measure_styled(label, &self.size.text_style(theme), None).0
    }

    /// Intrinsic size: every segment as wide as the widest label.
    pub fn preferred_size(&self, theme: &Theme) -> (f32, f32) {
        let widest = self
            .items
            .iter()
            .map(|l| self.label_w(l, theme))
            .fold(0.0, f32::max);
        let seg = widest + self.size.pad_x(theme) * 2.0;
        (
            (seg * self.items.len() as f32 + Self::inset(theme) * 2.0).ceil(),
            self.height(theme),
        )
    }

    /// The narrowest width that still shows every label as an ellipsis.
    pub fn min_width(&self, theme: &Theme) -> f32 {
        let seg = self.label_w(ELLIPSIS, theme) + self.size.pad_x(theme) * 2.0;
        (seg * self.items.len() as f32 + Self::inset(theme) * 2.0).ceil()
    }

    /// The segments in `bounds`: equal shares of the well's inside.
    pub fn segment_rects(&self, bounds: Rect, theme: &Theme) -> Vec<Rect> {
        let inner = bounds.inset(Self::inset(theme));
        let n = self.items.len().max(1) as f32;
        let w = inner.w / n;
        (0..self.items.len())
            .map(|i| Rect::new(inner.x + i as f32 * w, inner.y, w, inner.h))
            .collect()
    }

    /// The label of segment `index` as drawn in `segment`: whole when it
    /// fits, truncated with an ellipsis when it does not.
    pub fn shown_label(&self, index: usize, segment: Rect, theme: &Theme) -> String {
        let Some(label) = self.items.get(index) else {
            return String::new();
        };
        let room = (segment.w - self.size.pad_x(theme) * 2.0).max(0.0);
        if self.label_w(label, theme) <= room {
            label.clone()
        } else {
            TextMeasurer::truncate_to_width(label, &self.size.text_style(theme), room)
        }
    }

    /// Pointer events: each segment presses on its own; a click (release
    /// inside) chooses it.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        let rects = self.segment_rects(bounds, theme);
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

    /// Declares the group as `id`: a `Role::Group` named `label` whose
    /// text `selected` is the chosen segment's key (its index), and one
    /// `Role::Radio` child `<id>[<index>]` per segment, named by its label,
    /// with its `checked` and a `select` that reads the group's
    /// `selected`, so a choice made on a state another actor left goes
    /// stale. It only reads the widget; the app runs `select`
    /// ([`ToggleGroup::select`]). Returns the group's node.
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
        let t = &theme.selection.segmented;
        let rects = self.segment_rects(bounds, theme);
        if let Some(id) = &self.sem {
            c.bind(id, [bounds.x, bounds.y, bounds.w, bounds.h]);
            for (i, r) in rects.iter().enumerate() {
                c.bind(&id.key(i.to_string()), [r.x, r.y, r.w, r.h]);
            }
        }
        let alpha = if self.disabled { t.disabled_alpha } else { 1.0 };
        let well = bounds.inset(t.edge);
        paint_surface(
            c,
            LayerId::DEFAULT,
            well,
            t.track_radius.min(well.h / 2.0),
            &t.track,
            alpha,
            false,
            theme,
        );
        if let Some(first) = rects.first() {
            let thumb = Rect::new(
                first.x + self.thumb.get() * first.w,
                first.y,
                first.w,
                first.h,
            );
            paint_surface(
                c,
                LayerId::DEFAULT,
                thumb,
                t.thumb_radius.min(thumb.h / 2.0),
                &t.thumb,
                alpha,
                false,
                theme,
            );
        }
        self.labels(c, &rects, t, alpha, theme);
        if let Some(r) = self.focused.and_then(|i| rects.get(i)) {
            c.push(focus_ring_inside(*r, t.thumb_radius.min(r.h / 2.0), theme));
        }
    }

    fn labels(
        &self,
        c: &mut Compositor,
        rects: &[Rect],
        t: &SegmentedTokens,
        alpha: f32,
        theme: &Theme,
    ) {
        let style = self.size.text_style(theme);
        for (i, rect) in rects.iter().enumerate() {
            let color = if i == self.selected {
                t.thumb.text
            } else if self.presses.get(i).is_some_and(|p| p.hovered) {
                t.text_hover
            } else {
                t.text
            };
            let [r, g, b, a] = color.0;
            let shown = self.shown_label(i, *rect, theme);
            let (w, _) = TextMeasurer::measure_styled(&shown, &style, None);
            c.push(SceneNode::Text {
                key: TextNodeKey::from_style(&shown, &style, None),
                x: rect.x + (rect.w - w) / 2.0,
                y: rect.y + TextMeasurer::vertical_center(&style, rect.h),
                color: [r, g, b, a * alpha],
            });
        }
    }
}
