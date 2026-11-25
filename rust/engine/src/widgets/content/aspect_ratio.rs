//! Aspect ratio: a proportion that turns the width a widget receives into
//! its height (the kit's `aspect-square`), and the frame the kit draws
//! around a thumbnail of that proportion (its AssetCard well: a padded,
//! bordered, rounded square that darkens from the bottom on hover).
//!
//! The proportion is geometry, not a size: [`AspectRatio::height`] follows
//! the width it is given and [`AspectRatio::fit`] the largest rect of the
//! proportion in a box, so a frame is as tall as its column is wide at
//! any width. The frame's values are `theme.disclosure.aspect`'s.
//!
//! A frame is looked at, not operated: [`AspectFrame::project`] declares
//! it a `Role::Image` named by its label and described by its proportion.

use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use crate::drawing::compositor::Compositor;
use crate::styling::color::Color;
use crate::styling::theme::Theme;
use crate::widgets::icons;
use crate::widgets::recipe::{gradient_fill, rounded_rect, rounded_rect_stroke};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

/// Width over height.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AspectRatio {
    w: f32,
    h: f32,
}

impl AspectRatio {
    /// The kit's `aspect-square`.
    pub const SQUARE: Self = Self { w: 1.0, h: 1.0 };

    /// `w` to `h` (16 to 9); a side that is not positive and finite reads
    /// as 1.
    pub fn new(w: f32, h: f32) -> Self {
        let side = |v: f32| if v.is_finite() && v > 0.0 { v } else { 1.0 };
        Self {
            w: side(w),
            h: side(h),
        }
    }

    pub fn ratio(&self) -> f32 {
        self.w / self.h
    }

    /// The height at width `w`.
    pub fn height(&self, w: f32) -> f32 {
        w.max(0.0) * self.h / self.w
    }

    /// The largest rect of the proportion in `bounds`, centered.
    pub fn fit(&self, bounds: Rect) -> Rect {
        let (w, h) = if self.height(bounds.w) <= bounds.h {
            (bounds.w, self.height(bounds.w))
        } else {
            (bounds.h.max(0.0) * self.ratio(), bounds.h.max(0.0))
        };
        Rect::new(
            bounds.x + (bounds.w - w) / 2.0,
            bounds.y + (bounds.h - h) / 2.0,
            w,
            h,
        )
    }

    /// `16:9`, `1:1`: the proportion as people write it.
    pub fn label(&self) -> String {
        format!("{}:{}", self.w, self.h)
    }
}

/// The kit's well around a thumbnail of a proportion.
#[derive(Clone, Debug)]
pub struct AspectFrame {
    label: String,
    ratio: AspectRatio,
    glyph: Option<&'static str>,
    hovered: bool,
    sem: Option<SemId>,
}

impl AspectFrame {
    pub fn new(label: impl Into<String>, ratio: AspectRatio) -> Self {
        Self {
            label: label.into(),
            ratio,
            glyph: None,
            hovered: false,
            sem: None,
        }
    }

    /// A placeholder glyph where the image goes.
    pub fn glyph(mut self, name: &'static str) -> Self {
        self.glyph = Some(name);
        self
    }

    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    pub fn ratio(&self) -> AspectRatio {
        self.ratio
    }

    pub fn is_hovered(&self) -> bool {
        self.hovered
    }

    /// The frame's height at width `w`: its proportion's.
    pub fn height(&self, w: f32) -> f32 {
        self.ratio.height(w)
    }

    /// The pointer over it darkens it from the bottom.
    pub fn handle_event(&mut self, event: &WidgetEvent, bounds: Rect) -> EventResult {
        let WidgetEvent::MouseMove { x, y } = *event else {
            return EventResult::IGNORED;
        };
        let inside = bounds.contains(x, y);
        if inside == self.hovered {
            return EventResult::IGNORED;
        }
        self.hovered = inside;
        EventResult::changed()
    }

    /// The well, the glyph and, hovered, the shade, in `bounds` (give it
    /// [`AspectFrame::height`] of its width, or [`AspectRatio::fit`]).
    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        let r = &theme.disclosure.aspect;
        if let Some(id) = &self.sem {
            c.bind(id, [bounds.x, bounds.y, bounds.w, bounds.h]);
        }
        let surface = if self.hovered { &r.hover } else { &r.rest };
        let radius = r.radius.min(bounds.w / 2.0).min(bounds.h / 2.0);
        c.push(rounded_rect(
            bounds.x,
            bounds.y,
            bounds.w,
            bounds.h,
            radius,
            surface.fill.0,
        ));
        if let Some(name) = self.glyph {
            let inner = bounds.inset(r.pad);
            let size = inner.w.min(inner.h) * r.glyph;
            if size > 0.0
                && let Some(node) = icons::icon_at(
                    name,
                    size,
                    r.glyph_color.0,
                    inner.x + (inner.w - size) / 2.0,
                    inner.y + (inner.h - size) / 2.0,
                )
            {
                c.push(node);
            }
        }
        if self.hovered {
            let h = r.shade_h.min(bounds.h);
            let clear = Color([r.shade.0[0], r.shade.0[1], r.shade.0[2], 0.0]);
            c.push_clip(bounds.x, bounds.y, bounds.w, bounds.h);
            c.push(gradient_fill(
                Rect::new(bounds.x, bounds.bottom() - h, bounds.w, h),
                radius,
                clear.0,
                r.shade.0,
            ));
            c.pop_clip();
        }
        if r.border > 0.0 {
            c.push(rounded_rect_stroke(
                bounds.x,
                bounds.y,
                bounds.w,
                bounds.h,
                radius,
                surface.border.0,
                r.border,
            ));
        }
    }

    /// Declares the frame as `id`: a `Role::Image` named by its label and
    /// described by its proportion.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        let mut node = b.node(id, Role::Image, self.label.as_str());
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.description(self.ratio.label());
        node
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_height_follows_the_width_and_fit_keeps_the_proportion() {
        assert_eq!(AspectRatio::SQUARE.height(262.0), 262.0);
        let wide = AspectRatio::new(16.0, 9.0);
        assert_eq!(wide.height(320.0), 180.0);
        assert_eq!(wide.label(), "16:9");
        let tall = AspectRatio::new(3.0, 4.0).fit(Rect::new(0.0, 0.0, 400.0, 200.0));
        assert_eq!((tall.w, tall.h), (150.0, 200.0));
        assert_eq!(tall.x, 125.0, "centered");
        let bad = AspectRatio::new(0.0, f32::NAN);
        assert_eq!(bad.ratio(), 1.0);
        assert_eq!(wide.height(-5.0), 0.0);
    }

    #[test]
    fn a_frame_shades_on_hover_and_projects_as_an_image() {
        let bounds = Rect::new(0.0, 0.0, 200.0, 200.0);
        let mut frame = AspectFrame::new("Cube", AspectRatio::SQUARE).glyph("square");
        assert!(
            frame
                .handle_event(&WidgetEvent::MouseMove { x: 50.0, y: 50.0 }, bounds)
                .changed
        );
        assert!(frame.is_hovered());
        assert_eq!(
            frame.handle_event(&WidgetEvent::MouseDown { x: 50.0, y: 50.0 }, bounds),
            EventResult::IGNORED
        );
        let id = SemId::parse("frames.cube").unwrap();
        let mut b = SnapshotBuilder::new();
        frame.project(&mut b, &id, None);
        let seen = b.finish().unwrap();
        let node = seen.get(&id).unwrap();
        assert_eq!(node.role, Role::Image);
        assert_eq!(node.description.as_deref(), Some("1:1"));
    }
}
