//! The kit's view controller (`components/ViewController`): two arrows
//! around a ruler of ticks, two of them major, on a pill: across to
//! orbit, upright to tilt (the back arrow at the bottom then, as the
//! kit's `flex-col-reverse` puts it). Every value is
//! `theme.navigation.tool.view`.

use accord::{Role, SemId, SnapshotBuilder};

use crate::drawing::compositor::{Compositor, LayerId};
use crate::styling::theme::Theme;
use crate::widgets::buttons::Press;
use crate::widgets::icons;
use crate::widgets::recipe::{rounded_rect, rounded_rect_stroke, shadow_drops};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

/// The ruler, in order: which ticks are major.
const RULER: [bool; 7] = [false, true, false, false, false, true, false];

/// The view controller.
#[derive(Clone, Debug)]
pub struct ViewController {
    pub upright: bool,
    back: Press,
    forward: Press,
    sem: Option<SemId>,
}

impl ViewController {
    pub fn new(upright: bool) -> Self {
        Self {
            upright,
            back: Press::default(),
            forward: Press::default(),
            sem: None,
        }
    }

    /// Its identity; the arrows are `<id>.back` and `<id>.forward`.
    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    fn ruler_len(theme: &Theme) -> f32 {
        let v = &theme.navigation.tool.view;
        RULER.len() as f32 * v.tick_w + (RULER.len() - 1) as f32 * v.tick_gap
    }

    /// Along: the paddings, both arrows and the ruler with the gaps
    /// between; across: the pill's short side.
    pub fn preferred_size(&self, theme: &Theme) -> (f32, f32) {
        let v = &theme.navigation.tool.view;
        let along = v.pad * 2.0 + v.arrow * 2.0 + v.gap * 2.0 + Self::ruler_len(theme);
        if self.upright {
            (v.thick, along)
        } else {
            (along, v.thick)
        }
    }

    /// The back arrow, the forward arrow and the ruler's start, in a
    /// pill at `bounds`' top-left.
    fn parts(&self, bounds: Rect, theme: &Theme) -> (Rect, Rect, f32) {
        let v = &theme.navigation.tool.view;
        let (w, h) = self.preferred_size(theme);
        let mid = (v.thick - v.arrow) / 2.0;
        if self.upright {
            let x = bounds.x + mid;
            let forward = Rect::new(x, bounds.y + v.pad, v.arrow, v.arrow);
            let back = Rect::new(x, bounds.y + h - v.pad - v.arrow, v.arrow, v.arrow);
            (back, forward, forward.y + v.arrow + v.gap)
        } else {
            let y = bounds.y + mid;
            let back = Rect::new(bounds.x + v.pad, y, v.arrow, v.arrow);
            let forward = Rect::new(bounds.x + w - v.pad - v.arrow, y, v.arrow, v.arrow);
            (back, forward, back.x + v.arrow + v.gap)
        }
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        let (back, forward, _) = self.parts(bounds, theme);
        let a = self.back.handle(event, back, false);
        let b = self.forward.handle(event, forward, false);
        EventResult {
            handled: a.handled || b.handled,
            changed: a.changed || b.changed,
            clicked: a.clicked || b.clicked,
        }
    }

    pub fn render_to_layer(&self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        let v = &theme.navigation.tool.view;
        let (w, h) = self.preferred_size(theme);
        let pill = Rect::new(bounds.x, bounds.y, w, h);
        let radius = v.thick / 2.0;
        let (back, forward, start) = self.parts(bounds, theme);
        if let Some(id) = &self.sem {
            c.bind_to_layer(layer, id, [pill.x, pill.y, pill.w, pill.h]);
            for (name, r) in [("back", back), ("forward", forward)] {
                if let Ok(arrow) = id.child(name) {
                    c.bind_to_layer(layer, &arrow, [r.x, r.y, r.w, r.h]);
                }
            }
        }
        shadow_drops(c, layer, pill, radius, &v.shadow);
        c.push_to_layer(
            layer,
            rounded_rect(pill.x, pill.y, pill.w, pill.h, radius, v.fill.0),
        );
        c.push_to_layer(
            layer,
            rounded_rect_stroke(
                pill.x,
                pill.y,
                pill.w,
                pill.h,
                radius,
                v.border.0,
                v.border_width,
            ),
        );
        let (back_icon, forward_icon) = if self.upright {
            ("arrow-down", "arrow-up")
        } else {
            ("arrow-left", "arrow-right")
        };
        for (icon, r, tone) in [
            (back_icon, back, v.back),
            (forward_icon, forward, v.forward),
        ] {
            if let Some(node) = icons::icon_at(icon, v.arrow, tone.0, r.x, r.y) {
                c.push_to_layer(layer, node);
            }
        }
        let mut at = start;
        for major in RULER {
            let len = if major { v.tick_long } else { v.tick_short };
            let color = if major { v.tick_major } else { v.tick };
            let (x, y, tw, th) = if self.upright {
                (pill.x + (w - len) / 2.0, at, len, v.tick_w)
            } else {
                (at, pill.y + (h - len) / 2.0, v.tick_w, len)
            };
            c.push_to_layer(layer, rounded_rect(x, y, tw, th, v.tick_w / 2.0, color.0));
            at += v.tick_w + v.tick_gap;
        }
    }

    /// A `Role::Group` named `label` with its two arrows as buttons.
    #[track_caller]
    pub fn project(
        &self,
        b: &mut SnapshotBuilder,
        id: &SemId,
        label: &str,
        parent: Option<&SemId>,
    ) {
        let mut node = b.node(id, Role::Group, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        let names = if self.upright {
            [("back", "Tilt down"), ("forward", "Tilt up")]
        } else {
            [("back", "Orbit left"), ("forward", "Orbit right")]
        };
        for (key, name) in names {
            if let Ok(arrow) = id.child(key) {
                let mut button = b.node(&arrow, Role::Button, name);
                button.parent(id);
                button.action("activate");
            }
        }
    }
}
