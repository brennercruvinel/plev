//! What the popover draws: the panel's surface, a line of its content
//! (the kit's zoom `Line`), and the kit's trigger (a value and a chevron
//! in a bordered pill, raised while its panel is open). Every value is
//! `theme.overlay.popover`.

use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::theme::Theme;
use crate::widgets::buttons::{Press, paint_surface};
use crate::widgets::content::kbd::Kbd;
use crate::widgets::icons;
use crate::widgets::recipe::{focus_ring_inside, rounded_rect};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

/// The panel's surface at `rect`: the drop shadow, the fill, the ring.
/// The content is the app's, drawn after, inside `rect` less the
/// panel's `pad`.
pub fn render_panel(c: &mut Compositor, layer: LayerId, rect: Rect, theme: &Theme) {
    let t = &theme.overlay.popover;
    let radius = t.radius.min(rect.h / 2.0).min(rect.w / 2.0);
    paint_surface(c, layer, rect, radius, &t.panel, 1.0, false, theme);
}

/// What a line of the panel shows: its label, its keys, its check, and
/// whether it is lit (under the pointer) or holds the keyboard's focus.
#[derive(Clone, Copy, Debug, Default)]
pub struct Line<'a> {
    pub label: &'a str,
    pub keys: Option<&'a str>,
    pub checked: bool,
    pub lit: bool,
    pub focused: bool,
}

/// One line of the panel at `rect` (`Zoom/index.tsx:14-35`): the lit fill,
/// the check in its slot (the slot kept unchecked), the label at the
/// start, the keys in a cap at the end, the focus ring inside.
pub fn render_line(c: &mut Compositor, layer: LayerId, rect: Rect, line: Line, theme: &Theme) {
    let scale = &theme.typography;
    let l = &theme.overlay.popover.line;
    let radius = l.radius.min(rect.h / 2.0);
    if line.lit || line.focused {
        c.push_to_layer(
            layer,
            rounded_rect(rect.x, rect.y, rect.w, rect.h, radius, l.hover_fill.0),
        );
    }
    let x = rect.x + l.pad_x;
    if line.checked {
        let at = (l.slot - l.check) / 2.0;
        let y = rect.y + (rect.h - l.check) / 2.0;
        if let Some(node) = icons::icon_at("check", l.check, l.check_color.0, x + at, y) {
            c.push_to_layer(layer, node);
        }
    }
    c.push_to_layer(
        layer,
        SceneNode::Text {
            key: TextNodeKey::from_style(line.label, &scale.run(l.label), None),
            x: x + l.slot + l.slot_gap,
            y: rect.y + (rect.h - scale.run(l.label).line_height) / 2.0,
            color: l.text.0,
        },
    );
    if let Some(keys) = line.keys {
        let cap = Kbd::new(keys);
        let (w, h) = cap.preferred_size(theme);
        let w = w.max(l.key_min_w);
        let at = Rect::new(
            rect.right() - l.pad_x - w,
            rect.y + (rect.h - h) / 2.0,
            w,
            h,
        );
        cap.render_to_layer(c, layer, at, theme);
    }
    if line.focused {
        c.push_to_layer(layer, focus_ring_inside(rect, radius, theme));
    }
}

/// The kit's popover trigger (`ZoomControl`): the value it shows and a
/// chevron that turns while the panel is open. Click fires on release
/// inside, as for a button; the owner toggles its [`super::Popover`].
#[derive(Clone, Debug)]
pub struct PopoverTrigger {
    pub label: String,
    press: Press,
    sem: Option<SemId>,
}

impl PopoverTrigger {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            press: Press::default(),
            sem: None,
        }
    }

    /// The logical identity the render binds its bounds to.
    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    pub fn is_hovered(&self) -> bool {
        self.press.hovered
    }

    /// Keyboard focus, driven by the owner.
    pub fn set_focused(&mut self, focused: bool) {
        self.press.focused = focused;
    }

    pub fn is_focused(&self) -> bool {
        self.press.focused
    }

    fn text_style(theme: &Theme) -> TextStyle {
        theme.typography.run(theme.overlay.popover.trigger.value)
    }

    /// Intrinsic size: the value measured, the chevron, the padding, at
    /// least the kit's `min-w-22`.
    pub fn preferred_size(&self, theme: &Theme) -> (f32, f32) {
        let t = &theme.overlay.popover.trigger;
        let (text_w, _) = TextMeasurer::measure_styled(&self.label, &Self::text_style(theme), None);
        let w = (text_w + t.gap + t.chevron + t.pad_x * 2.0).ceil();
        (w.max(t.min_w), t.height)
    }

    pub fn handle_event(&mut self, event: &WidgetEvent, bounds: Rect) -> EventResult {
        self.press.handle(event, bounds, false)
    }

    /// Declares the trigger as `id`: a `Role::Button` named by its value
    /// with an `activate` (the owner toggles the popover);
    /// [`super::Popover::project`] adds what the popover reads.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        let mut node = b.node(id, Role::Button, self.label.as_str());
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.action("activate");
        node
    }

    /// The trigger at `bounds`, raised while `open`: the value at the
    /// start, the chevron at the end (`justify-between`).
    pub fn render(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        bounds: Rect,
        open: bool,
        theme: &Theme,
    ) {
        if let Some(id) = &self.sem {
            c.bind_to_layer(layer, id, [bounds.x, bounds.y, bounds.w, bounds.h]);
        }
        let t = &theme.overlay.popover.trigger;
        let surface = if open {
            &t.open
        } else if self.press.hovered || self.press.pressed {
            &t.hover
        } else {
            &t.rest
        };
        let radius = t.radius.min(bounds.h / 2.0);
        let fg = paint_surface(
            c,
            layer,
            bounds,
            radius,
            surface,
            1.0,
            self.press.focused,
            theme,
        );

        let style = Self::text_style(theme);
        let text_w = (bounds.w - t.pad_x * 2.0 - t.gap - t.chevron).max(0.0);
        c.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::from_style(&self.label, &style, Some(text_w)),
                x: bounds.x + t.pad_x,
                y: bounds.y + TextMeasurer::vertical_center(&style, bounds.h),
                color: fg,
            },
        );
        // Half a turn while open (`group-data-open:rotate-180`).
        let chevron = if open { "chevron-up" } else { "chevron-down" };
        if let Some(node) = icons::icon_at(
            chevron,
            t.chevron,
            t.chevron_color.0,
            bounds.right() - t.pad_x - t.chevron,
            bounds.y + (bounds.h - t.chevron) / 2.0,
        ) {
            c.push_to_layer(layer, node);
        }
    }
}
