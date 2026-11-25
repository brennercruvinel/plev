//! The Inspect view: the page's controls, and one of them from what it
//! means to what it cost.
//!
//! At the left, every control the page projects as a tree (the app's
//! semantic snapshot: a role and a label a line, under its parent); a
//! press on a line chooses that control, and the pointer over a line
//! rings it on the page. The picker in the panel's strip arms the pointer
//! the other way: the control under it on the page is ringed with its
//! role and size, and a press chooses it instead of reaching the page.
//!
//! At the right, the control chosen: its identity, role, state and
//! actions (the snapshot), the code that projected it, where it sits in
//! the frame and on which layer (the compositor's binding), and what that
//! layer did in the frame shown.

use accord::{Node, SemId, Snapshot};

use super::Act;
use super::paint::{Painter, alpha, time};
use crate::runtime::perf::FrameReport;
use crate::widgets::prelude::Rect;

/// The body's width from which the control sits beside the tree.
const WIDE: f32 = 760.0;
/// How far a level of the tree sits in from its parent, and how many
/// levels still move in.
const INDENT: f32 = 12.0;
const DEPTH_MOST: usize = 10;
/// Lines a notch of the wheel moves, at least.
const NOTCH_PX: f32 = 20.0;

/// A line of the tree.
struct Line {
    depth: usize,
    id: SemId,
    role: String,
    label: String,
}

pub(super) struct Inspect {
    /// The next press on the page chooses the control under it.
    armed: bool,
    /// The control under the pointer on the page, while armed.
    hover: Option<SemId>,
    picked: Option<SemId>,
    /// The first line of the tree shown.
    first: usize,
    /// Bring the control chosen into the tree's lines at the next draw.
    reveal: bool,
}

impl Inspect {
    pub fn new() -> Self {
        Self {
            armed: false,
            hover: None,
            picked: None,
            first: 0,
            reveal: false,
        }
    }

    /// Choose `id` as a press on it would.
    pub fn pick(&mut self, id: Option<SemId>) {
        self.reveal = id.is_some();
        self.picked = id;
        self.arm(false);
    }

    pub fn picked(&self) -> Option<&SemId> {
        self.picked.as_ref()
    }

    pub fn arm(&mut self, armed: bool) {
        self.armed = armed;
        self.hover = None;
    }

    pub fn is_armed(&self) -> bool {
        self.armed
    }

    /// The control under the pointer on the page is `id`; `true` when
    /// that is another one than before.
    pub fn hover(&mut self, id: Option<SemId>) -> bool {
        let changed = id != self.hover;
        self.hover = id;
        changed
    }

    /// The wheel over the tree: a positive `delta` goes down it. `true`
    /// when the tree moved.
    pub fn scroll(&mut self, delta: f32) -> bool {
        let lines = (delta.abs() / NOTCH_PX).ceil().max(1.0) as usize;
        let first = if delta > 0.0 {
            self.first.saturating_add(lines)
        } else {
            self.first.saturating_sub(lines)
        };
        let moved = first != self.first;
        self.first = first;
        moved
    }

    /// The view in `body`.
    pub fn draw(
        &mut self,
        p: &mut Painter<'_>,
        body: Rect,
        shown: Option<&FrameReport>,
        snapshot: Option<&Snapshot>,
    ) {
        let gap = p.gap();
        let (left, right) = if body.w >= WIDE {
            let w = ((body.w - gap * 3.0) * 0.46).round();
            p.rule_v(body.x + w + gap * 1.5, body.y, body.h);
            (
                Rect::new(body.x, body.y, w, body.h),
                Rect::new(
                    body.x + w + gap * 3.0,
                    body.y,
                    body.w - w - gap * 3.0,
                    body.h,
                ),
            )
        } else {
            let h = (body.h / 2.0).floor();
            (
                Rect::new(body.x, body.y, body.w, h - gap),
                Rect::new(body.x, body.y + h, body.w, body.h - h),
            )
        };
        self.tree(p, left, snapshot);
        self.control(p, right, shown, snapshot);
    }

    /// The page's controls, a line each.
    fn tree(&mut self, p: &mut Painter<'_>, area: Rect, snapshot: Option<&Snapshot>) {
        let (caption, strong, mono) = (p.caption(), p.strong(), p.mono());
        let gap = p.gap();
        let row = p.row() + 2.0;
        let lines = lines(p, snapshot);
        let note = match (snapshot.is_some(), lines.len()) {
            (true, n) => format!("{n} projected"),
            (false, n) => format!("{n} bound · no projection"),
        };
        let top = p.title("Controls", &note, area);
        if lines.is_empty() {
            let why = "This page binds no control to an id.";
            p.empty(
                Rect::new(area.x, top, area.w, area.bottom() - top),
                "scan",
                why,
            );
            return;
        }
        let room = (((area.bottom() - top) / row).floor() as usize).max(1);
        let last_first = lines.len().saturating_sub(room);
        if std::mem::take(&mut self.reveal)
            && let Some(at) = lines
                .iter()
                .position(|l| Some(&l.id) == self.picked.as_ref())
            && !(self.first..self.first + room).contains(&at)
        {
            self.first = at.saturating_sub(room / 2);
        }
        self.first = self.first.min(last_first);
        let info = p.theme.colors.info.0;
        let mut y = top;
        for line in lines.iter().skip(self.first).take(room) {
            let place = Rect::new(area.x, y, area.w, row);
            let hot = p.region(place, Act::Node(line.id.clone()));
            let on = Some(&line.id) == self.picked.as_ref();
            if on {
                p.fill(place, alpha(info, 0.12));
                p.fill(Rect::new(place.x, place.y, 2.0, place.h), info);
            } else if hot {
                p.wash(place, 1.0);
            }
            let text_y = y + (row - p.line()) / 2.0;
            let bound = p.c.binding(&line.id).is_some();
            let mut x = area.x + gap + line.depth.min(DEPTH_MOST) as f32 * INDENT;
            let role_ink = if bound { info } else { alpha(info, 0.5) };
            p.text(&line.role, x, text_y, area.right() - x, role_ink, &strong);
            x += p.width(&line.role, &strong) + gap;
            let ink = if bound { p.ink() } else { p.faint() };
            let label_w = p.width(&line.label, &caption);
            p.text(
                &line.label,
                x,
                text_y,
                area.right() - x - gap,
                ink,
                &caption,
            );
            x += label_w + gap * 1.5;
            // the id in what is left of the line, when it adds to the label
            let id = line.id.to_string();
            let id_w = p.width(&id, &mono);
            if x + id_w + gap <= area.right() {
                p.text_right(&id, area.right() - gap, text_y, p.faint(), &mono);
            }
            y += row;
        }
        if lines.len() > room {
            // where the lines shown are in the whole tree
            let whole = area.bottom() - top;
            let thumb = (whole * room as f32 / lines.len() as f32).max(row);
            let at = top + (whole - thumb) * self.first as f32 / last_first.max(1) as f32;
            p.pill(
                Rect::new(area.right() - 3.0, at, 3.0, thumb),
                alpha(p.ink(), 0.2),
            );
        }
    }

    /// The control chosen, or under the pointer while picking.
    fn control(
        &self,
        p: &mut Painter<'_>,
        area: Rect,
        shown: Option<&FrameReport>,
        snapshot: Option<&Snapshot>,
    ) {
        let (caption, num, mono) = (p.caption(), p.num(), p.mono());
        let gap = p.gap();
        let row = p.row();
        let Some(id) = self.hover.as_ref().or(self.picked.as_ref()) else {
            let why = if self.armed {
                "Picking: press a control on the page."
            } else {
                "Choose a control in the tree, or pick one on the page with the pointer in the strip."
            };
            p.empty(area, "mouse-pointer-2", why);
            return;
        };
        let node = snapshot.and_then(|s| s.get(id));
        let name = node.map_or_else(
            || "Control".to_owned(),
            |n| {
                if n.label.is_empty() {
                    format!("{:?}", n.role)
                } else {
                    format!("{:?} · {}", n.role, n.label)
                }
            },
        );
        let note = if self.hover.is_some() {
            "under the pointer"
        } else {
            "chosen"
        };
        let mut y = p.title(&name, note, area);

        // (name, value, in the mono face)
        let mut lines: Vec<(&str, String, bool)> = vec![("id", id.to_string(), true)];
        match node {
            Some(node) => {
                if let Some(description) = &node.description {
                    lines.push(("description", description.clone(), false));
                }
                if !node.state.is_empty() {
                    let state: Vec<String> = node
                        .state
                        .iter()
                        .map(|(key, value)| format!("{key} = {value:?}"))
                        .collect();
                    lines.push(("state", state.join(" · "), false));
                }
                if !node.actions.is_empty() {
                    let actions: Vec<String> =
                        node.actions.iter().map(|a| a.name.to_string()).collect();
                    lines.push(("actions", actions.join(" · "), false));
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
                    lines.push(("flags", flags.join(" · "), false));
                }
                if let Some(source) = &node.source {
                    let at = format!("{}:{}:{}", source.file, source.line, source.column);
                    lines.push(("source", at, true));
                }
            }
            None => lines.push(("meaning", "not in the app's snapshot".to_owned(), false)),
        }
        match p.c.binding(id).copied() {
            Some(binding) => {
                let [x, y, w, h] = binding.rect;
                lines.push(("size", format!("{w:.0} × {h:.0}"), false));
                lines.push(("position", format!("x {x:.0} · y {y:.0}"), false));
                lines.push(("order", format!("{} in its layer", binding.order), false));
                lines.push((
                    "layer",
                    layer_line(binding.layer.0, shown, p.layer.0),
                    false,
                ));
            }
            None => lines.push((
                "position",
                "not bound in the frame on screen".to_owned(),
                false,
            )),
        }

        let label_w = p.width("description", &caption) + gap * 2.0;
        for (label, value, code) in lines {
            if y + row > area.bottom() {
                break;
            }
            p.text(label, area.x, y, label_w, p.faint(), &caption);
            let style = if code { &mono } else { &num };
            p.text(
                &value,
                area.x + label_w,
                y,
                area.w - label_w,
                p.ink(),
                style,
            );
            y += row;
            p.fill(
                Rect::new(area.x, y - 1.0, area.w, 1.0),
                alpha(p.ink(), 0.04),
            );
        }
    }

    /// Over the page: a ring around the control chosen, and around the
    /// one the pointer is on (on the page while picking, or on its line
    /// of the tree) with its role and size beside it.
    pub fn overlay(&self, p: &mut Painter<'_>, page: Rect, snapshot: Option<&Snapshot>) {
        let from_tree = match &p.hot {
            Some(Act::Node(id)) => Some(id.clone()),
            _ => None,
        };
        let pointed = self.hover.clone().or(from_tree);
        let info = p.theme.colors.info.0;
        let rect_of = |p: &Painter<'_>, id: &SemId| {
            p.c.binding(id)
                .and_then(|b| b.visible_rect())
                .map(|[x, y, w, h]| Rect::new(x, y, w, h))
                .filter(|r| r.x < page.right() && r.y < page.bottom())
        };
        if let Some(id) = self
            .picked
            .as_ref()
            .filter(|id| Some(*id) != pointed.as_ref())
            && let Some(r) = rect_of(p, id)
        {
            p.outline(r, info, 1.5);
        }
        let Some(id) = pointed else {
            return;
        };
        let Some(r) = rect_of(p, &id) else {
            return;
        };
        p.chip(r, alpha(info, 0.14));
        p.outline(r, info, 1.5);

        // its role and size on a dark tag, over it or under it
        let role = snapshot
            .and_then(|s| s.get(&id))
            .map_or_else(|| "bound".to_owned(), |n| format!("{:?}", n.role));
        let (strong, num) = (p.strong(), p.num());
        let size = format!("{:.0} × {:.0}", r.w, r.h);
        let gap = p.gap();
        let w = p.width(&role, &strong) + gap + p.width(&size, &num) + gap * 2.0;
        let h = p.row() + 2.0;
        let x = r.x.min(page.right() - w - gap).max(page.x + gap);
        let over = r.y - h - 4.0;
        let y = if over >= page.y {
            over
        } else {
            (r.bottom() + 4.0).min(page.bottom() - h)
        };
        let tag = Rect::new(x, y, w, h);
        p.chip(tag, p.theme.colors.text.0);
        let ground = p.theme.colors.bg.0;
        let text_y = y + (h - p.line()) / 2.0;
        p.text(&role, x + gap, text_y, w, ground, &strong);
        p.text_right(&size, tag.right() - gap, text_y, alpha(ground, 0.75), &num);
    }
}

/// The tree's lines: the snapshot's nodes under their parents, or, for a
/// page that projects nothing, the ids the frame bound, in bind order,
/// each under the id it extends (`shell.sidebar.row[0]` under
/// `shell.sidebar`) and named by its last part.
fn lines(p: &Painter<'_>, snapshot: Option<&Snapshot>) -> Vec<Line> {
    let Some(snapshot) = snapshot else {
        return p
            .c
            .bindings()
            .map(|(id, _)| {
                let path = id.as_str();
                let last = path.rfind('.').map_or(0, |dot| dot + 1);
                Line {
                    depth: path.matches('.').count(),
                    id: id.clone(),
                    role: path[last..].to_owned(),
                    label: String::new(),
                }
            })
            .collect();
    };
    fn walk(snapshot: &Snapshot, node: &Node, depth: usize, out: &mut Vec<Line>) {
        out.push(Line {
            depth,
            id: node.id.clone(),
            role: format!("{:?}", node.role),
            label: node.label.clone(),
        });
        for child in snapshot.children(&node.id) {
            walk(snapshot, child, depth + 1, out);
        }
    }
    let mut out = Vec::with_capacity(snapshot.len());
    for root in snapshot.roots() {
        walk(snapshot, root, 0, &mut out);
    }
    out
}

/// What the control's layer did in the frame shown.
fn layer_line(id: u32, shown: Option<&FrameReport>, dev: u32) -> String {
    let name = if id == dev {
        format!("{id} (dev mode)")
    } else {
        id.to_string()
    };
    let Some(frame) = shown else {
        return name;
    };
    let Some(layer) = frame.layers.iter().find(|l| l.id.0 == id) else {
        return format!("{name} · not in frame {}", frame.index);
    };
    let state = if layer.redrawn { "redrawn" } else { "reused" };
    let gpu: u64 = frame
        .gpu
        .iter()
        .flat_map(|g| g.passes.iter())
        .filter(|pass| pass.layer.0 == id)
        .filter_map(|pass| pass.nanos)
        .sum();
    let mut line = format!(
        "{name} · z {} · {state} in frame {} · {} nodes · {} draws",
        layer.z_order, frame.index, layer.nodes, layer.draws
    );
    if gpu > 0 {
        line = format!("{line} · gpu {}", time(gpu));
    }
    line
}
