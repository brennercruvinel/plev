//! The Inspect view: one control of the page, from what it means to what
//! it cost. "Pick a control" arms the pointer: the control under it is
//! ringed, and a press chooses it instead of reaching the page. The view
//! then says the control's identity, role, state and actions (the app's
//! semantic snapshot), where it sits in the frame and on which layer (the
//! compositor's binding), what that layer did in the frame shown, and the
//! code that projected it.

use accord::{SemId, Snapshot};

use super::paint::{Painter, time};
use crate::drawing::compositor::{Compositor, LayerId};
use crate::runtime::perf::FrameReport;
use crate::styling::theme::Theme;
use crate::widgets::prelude::{Button, ButtonSize, ButtonVariant, Rect, WidgetEvent};

const IDLE: &str = "Pick a control";
const ARMED: &str = "Picking: press a control on the page";

pub(super) struct Inspect {
    pick: Button,
    /// The next press on the page chooses the control under it.
    armed: bool,
    /// The release of the press that chose is not the page's either.
    held: bool,
    hover: Option<SemId>,
    picked: Option<SemId>,
}

impl Inspect {
    pub fn new() -> Self {
        Self {
            pick: Button::new(IDLE)
                .size(ButtonSize::Sm)
                .variant(ButtonVariant::Outline),
            armed: false,
            held: false,
            hover: None,
            picked: None,
        }
    }

    /// Choose `id` as a press on it would.
    pub fn pick(&mut self, id: Option<SemId>) {
        self.picked = id;
        self.arm(false);
    }

    fn arm(&mut self, armed: bool) {
        self.armed = armed;
        self.hover = None;
        self.pick.label = if armed { ARMED } else { IDLE }.to_owned();
    }

    fn button(&self, body: Rect, theme: &Theme) -> Rect {
        let (w, h) = self.pick.preferred_size(theme);
        Rect::new(body.x, body.y, w, h)
    }

    /// A pointer event while the view shows: `(taken, changed)`. `page`
    /// is the page's share of the window, `body` the view's.
    pub fn event(
        &mut self,
        ev: &WidgetEvent,
        page: Rect,
        body: Rect,
        theme: &Theme,
        c: &Compositor,
    ) -> (bool, bool) {
        let r = self.pick.handle_event(ev, self.button(body, theme));
        let mut changed = r.changed || r.clicked;
        if r.clicked {
            self.arm(!self.armed);
        }
        let mut taken = false;
        match *ev {
            WidgetEvent::MouseMove { x, y } if self.armed && page.contains(x, y) => {
                let under = c.hit(x, y).cloned();
                if under != self.hover {
                    self.hover = under;
                    changed = true;
                }
                taken = true;
            }
            WidgetEvent::MouseDown { x, y } if self.armed && page.contains(x, y) => {
                self.picked = c.hit(x, y).cloned();
                self.arm(false);
                self.held = true;
                taken = true;
                changed = true;
            }
            WidgetEvent::MouseUp { .. } if self.held => {
                self.held = false;
                taken = true;
            }
            _ => {}
        }
        (taken, changed)
    }

    /// The view in `body`, and the rings over the page.
    pub fn draw(
        &self,
        p: &mut Painter<'_>,
        body: Rect,
        shown: Option<&FrameReport>,
        snapshot: Option<&Snapshot>,
        dev: LayerId,
    ) {
        let caption = p.caption();
        let gap = p.gap();
        let row = p.row();
        let button = self.button(body, p.theme);
        self.pick.render_to_layer(p.c, p.layer, button, p.theme);
        let accent = p.theme.colors.accent.0;
        for id in [self.hover.as_ref(), self.picked.as_ref()]
            .into_iter()
            .flatten()
        {
            if let Some([x, y, w, h]) = p.c.binding(id).and_then(|b| b.visible_rect()) {
                p.outline(Rect::new(x, y, w, h), accent);
            }
        }

        let mut y = button.bottom() + gap;
        let Some(id) = self.hover.as_ref().or(self.picked.as_ref()) else {
            let hint = "No control chosen. A control is what the page binds to an id it projects.";
            p.text(hint, body.x, y, body.w, p.mid(), &caption);
            return;
        };
        let mut lines: Vec<(&str, String)> = vec![("id", id.to_string())];
        match snapshot.and_then(|s| s.get(id)) {
            Some(node) => {
                lines.push(("role", format!("{:?}", node.role)));
                if !node.label.is_empty() {
                    lines.push(("label", node.label.clone()));
                }
                if !node.state.is_empty() {
                    let state: Vec<String> = node
                        .state
                        .iter()
                        .map(|(key, value)| format!("{key} = {value:?}"))
                        .collect();
                    lines.push(("state", state.join(" · ")));
                }
                if !node.actions.is_empty() {
                    let actions: Vec<String> =
                        node.actions.iter().map(|a| a.name.to_string()).collect();
                    lines.push(("actions", actions.join(" · ")));
                }
                let mut flags = Vec::new();
                if !node.enabled {
                    flags.push("disabled");
                }
                if node.inert {
                    flags.push("inert");
                }
                if node.focusable {
                    flags.push("focusable");
                }
                if !flags.is_empty() {
                    lines.push(("flags", flags.join(" · ")));
                }
                if let Some(source) = &node.source {
                    let at = format!("{}:{}:{}", source.file, source.line, source.column);
                    lines.push(("source", at));
                }
            }
            None => lines.push(("meaning", "not in the app's snapshot".to_owned())),
        }
        match p.c.binding(id).copied() {
            Some(binding) => {
                let [x, y, w, h] = binding.rect;
                lines.push((
                    "rect",
                    format!(
                        "x {x:.0} y {y:.0} · {w:.0} x {h:.0} · order {}",
                        binding.order
                    ),
                ));
                lines.push(("layer", layer_line(binding.layer, shown, dev)));
            }
            None => lines.push(("rect", "not bound in the frame on screen".to_owned())),
        }

        let label_w = p.width("actions", &caption) + gap * 2.0;
        for (label, value) in lines {
            if y + row > body.bottom() {
                break;
            }
            p.text(label, body.x, y, label_w, p.mid(), &caption);
            p.text(
                &value,
                body.x + label_w,
                y,
                body.w - label_w,
                p.ink(),
                &caption,
            );
            y += row;
        }
    }
}

/// What the control's layer did in the frame shown.
fn layer_line(id: LayerId, shown: Option<&FrameReport>, dev: LayerId) -> String {
    let name = if id == dev {
        format!("{} (dev mode)", id.0)
    } else {
        id.0.to_string()
    };
    let Some(frame) = shown else {
        return name;
    };
    let Some(layer) = frame.layers.iter().find(|l| l.id == id) else {
        return format!("{name} · not in frame {}", frame.index);
    };
    let state = if layer.redrawn { "redrawn" } else { "reused" };
    let gpu: u64 = frame
        .gpu
        .iter()
        .flat_map(|g| g.passes.iter())
        .filter(|pass| pass.layer == id)
        .filter_map(|pass| pass.nanos)
        .sum();
    let mut line = format!(
        "{name} · z {} · {state} in frame {} · {} nodes · {} glyphs · {} draws",
        layer.z_order, frame.index, layer.nodes, layer.glyphs, layer.draws
    );
    if gpu > 0 {
        line = format!("{line} · gpu {}", time(gpu));
    }
    line
}
