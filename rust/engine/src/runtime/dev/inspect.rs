//! The Inspect view: the page's controls, and one thing of the page from
//! what it means to what it cost.
//!
//! At the left, every control the page projects as a tree (the app's
//! semantic snapshot: a role and a label a line, under its parent, a
//! parent folding what it holds); a press on a line chooses that control,
//! and the pointer over a line rings it on the page.
//!
//! The picker in the panel's strip arms the pointer the other way, and it
//! picks anything the frame draws, not only what the page gave an id: the
//! thing under the pointer is ringed with what it is and its size, and a
//! press chooses it instead of reaching the page. What is under a point
//! is the bound control there (`Compositor::hit`) or the scene node there
//! (`Compositor::node_at`: an icon, a line of text, a card's ground),
//! whichever is the more particular: the node when it is smaller than the
//! control it sits in, the control when the node is its whole ground.
//!
//! At the right, the thing chosen (`facts`): a control's identity, state
//! and actions, the code that projected it, its place and what its layer
//! did in the frame shown; a node's kind, paint, type and place, and the
//! control it sits in, which a press on that line chooses.

use std::collections::HashSet;

use accord::{Node, SemId, Snapshot};

use super::Act;
use super::facts::{self, Fact};
use super::paint::{Painter, alpha};
use crate::drawing::compositor::{Compositor, LayerId};
use crate::runtime::perf::FrameReport;
use crate::widgets::prelude::Rect;

/// The body's width from which the thing chosen sits beside the tree.
const WIDE: f32 = 760.0;
/// How far a level of the tree sits in from its parent, and how many
/// levels still move in.
const INDENT: f32 = 12.0;
const DEPTH_MOST: usize = 10;
/// Lines a notch of the wheel moves, at least.
const NOTCH_PX: f32 = 20.0;
/// The fold mark's side, and the room it takes before a line's words.
const FOLD: f32 = 10.0;
const FOLD_ROOM: f32 = 14.0;
/// A node at least this much of the control it sits in is that control's
/// own ground: the control is what the pointer is on.
const GROUND_SHARE: f32 = 0.9;

/// What the pointer is on, or what is chosen.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum Target {
    /// A control the page bound to an id.
    Control(SemId),
    /// A scene node, by its place in its layer and what it was when
    /// chosen: the frame is built again every time, and a node there of
    /// another kind is not the one chosen.
    Node {
        layer: LayerId,
        index: usize,
        kind: &'static str,
    },
}

/// A line of the tree.
struct Line {
    depth: usize,
    id: SemId,
    role: String,
    label: String,
    /// It holds other lines, shown or folded.
    parent: bool,
}

pub(super) struct Inspect {
    /// The next press on the page chooses what is under it.
    armed: bool,
    /// What is under the pointer on the page, while armed.
    hover: Option<Target>,
    picked: Option<Target>,
    /// The first line of the tree shown.
    first: usize,
    /// Bring the control chosen into the tree's lines at the next draw.
    reveal: bool,
    /// The controls whose lines under them are folded away.
    folded: HashSet<SemId>,
}

fn rect([x, y, w, h]: [f32; 4]) -> Rect {
    Rect::new(x, y, w, h)
}

impl Target {
    /// What is at `(x, y)` of the frame `c`, the layer `skip` left out.
    pub fn at(c: &Compositor, x: f32, y: f32, skip: Option<LayerId>) -> Option<Target> {
        let control = c
            .hit(x, y)
            .and_then(|id| Some((id, c.binding(id)?.visible_rect()?)));
        let node = c.node_at(x, y, skip).and_then(|at| {
            let kind = c.node(at.layer, at.index)?.kind();
            Some((at, kind))
        });
        let area = |r: [f32; 4]| r[2] * r[3];
        match (control, node) {
            (Some((id, bound)), Some((at, _))) if area(at.rect) >= area(bound) * GROUND_SHARE => {
                Some(Target::Control(id.clone()))
            }
            (_, Some((at, kind))) => Some(Target::Node {
                layer: at.layer,
                index: at.index,
                kind,
            }),
            (Some((id, _)), None) => Some(Target::Control(id.clone())),
            (None, None) => None,
        }
    }

    /// Where it shows in the frame `c`; `None` once the frame no longer
    /// has it.
    fn rect(&self, c: &Compositor) -> Option<Rect> {
        match self {
            Target::Control(id) => c.binding(id)?.visible_rect().map(rect),
            Target::Node { layer, index, kind } => c
                .node(*layer, *index)
                .filter(|node| node.kind() == *kind)
                .and_then(|node| node.bounds())
                .map(rect),
        }
    }

    /// The control it is, or the one bound where it sits.
    fn owner(&self, c: &Compositor) -> Option<SemId> {
        match self {
            Target::Control(id) => Some(id.clone()),
            Target::Node { .. } => {
                let (x, y) = self.rect(c)?.center();
                c.hit(x, y).cloned()
            }
        }
    }

    /// What it is, in a word: a control's role, a node's kind.
    fn word(&self, snapshot: Option<&Snapshot>) -> String {
        match self {
            Target::Control(id) => snapshot
                .and_then(|s| s.get(id))
                .map_or_else(|| "Control".to_owned(), |n| format!("{:?}", n.role)),
            Target::Node { kind, .. } => (*kind).to_owned(),
        }
    }
}

impl Inspect {
    pub fn new() -> Self {
        Self {
            armed: false,
            hover: None,
            picked: None,
            first: 0,
            reveal: false,
            folded: HashSet::new(),
        }
    }

    /// Choose `target` as a press on it would.
    pub fn pick(&mut self, target: Option<Target>) {
        self.reveal = target.is_some();
        self.picked = target;
        self.arm(false);
    }

    pub fn picked(&self) -> Option<&Target> {
        self.picked.as_ref()
    }

    pub fn arm(&mut self, armed: bool) {
        self.armed = armed;
        self.hover = None;
    }

    pub fn is_armed(&self) -> bool {
        self.armed
    }

    /// The thing under the pointer on the page is `target`; `true` when
    /// that is another one than before.
    pub fn hover(&mut self, target: Option<Target>) -> bool {
        let changed = target != self.hover;
        self.hover = target;
        changed
    }

    /// Fold the lines under `id`, or show them again.
    pub fn fold(&mut self, id: SemId) {
        if !self.folded.remove(&id) {
            self.folded.insert(id);
        }
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
        self.chosen(p, right, shown, snapshot);
    }

    /// The page's controls, a line each.
    fn tree(&mut self, p: &mut Painter<'_>, area: Rect, snapshot: Option<&Snapshot>) {
        let (caption, strong, mono) = (p.caption(), p.strong(), p.mono());
        let gap = p.gap();
        let row = p.row() + 2.0;
        let lines = lines(p, snapshot, &self.folded);
        let note = match (snapshot.is_some(), lines.len()) {
            (true, n) => format!("{n} projected"),
            (false, n) => format!("{n} bound · no projection"),
        };
        if lines.is_empty() {
            // a page that gives nothing an id still draws: what its frame
            // holds, layer by layer, and the picker chooses any of it
            scene(p, area);
            return;
        }
        let top = p.title("Controls", &note, area);
        // the control chosen, or the one the node chosen sits in
        let owner = self.picked.as_ref().and_then(|target| target.owner(p.c));
        let direct = matches!(self.picked, Some(Target::Control(_)));
        let room = (((area.bottom() - top) / row).floor() as usize).max(1);
        let last_first = lines.len().saturating_sub(room);
        if std::mem::take(&mut self.reveal)
            && let Some(at) = lines.iter().position(|l| Some(&l.id) == owner.as_ref())
            && !(self.first..self.first + room).contains(&at)
        {
            self.first = at.saturating_sub(room / 2);
        }
        self.first = self.first.min(last_first);
        let info = p.theme.colors.info.0;
        let mut y = top;
        for line in lines.iter().skip(self.first).take(room) {
            let place = Rect::new(area.x, y, area.w, row);
            let hot = p.region(place, Act::Control(line.id.clone()));
            let on = Some(&line.id) == owner.as_ref();
            if on {
                let strength = if direct { 0.12 } else { 0.06 };
                p.fill(place, alpha(info, strength));
                p.fill(Rect::new(place.x, place.y, 2.0, place.h), info);
            } else if hot {
                p.wash(place, 1.0);
            }
            let text_y = y + (row - p.line()) / 2.0;
            let bound = p.c.binding(&line.id).is_some();
            let mut x = area.x + gap / 2.0 + line.depth.min(DEPTH_MOST) as f32 * INDENT;
            if line.parent {
                let folded = self.folded.contains(&line.id);
                let mark = Rect::new(x, y, FOLD_ROOM, row);
                let over = p.region(mark, Act::Fold(line.id.clone()));
                let icon = if folded {
                    "chevron-right"
                } else {
                    "chevron-down"
                };
                let ink = if over { p.ink() } else { p.mid() };
                p.icon_in(icon, mark, FOLD, ink);
            }
            x += FOLD_ROOM;
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

    /// The thing chosen, or the one under the pointer while picking.
    fn chosen(
        &self,
        p: &mut Painter<'_>,
        area: Rect,
        shown: Option<&FrameReport>,
        snapshot: Option<&Snapshot>,
    ) {
        let (caption, num, mono) = (p.caption(), p.num(), p.mono());
        let gap = p.gap();
        let row = p.row();
        let Some(target) = self.hover.as_ref().or(self.picked.as_ref()) else {
            let why = if self.armed {
                "Picking: press anything on the page."
            } else {
                "Choose a control in the tree, or pick anything on the page with the pointer in the strip."
            };
            p.empty(area, "mouse-pointer-2", why);
            return;
        };
        let note = if self.hover.is_some() {
            "under the pointer"
        } else {
            "chosen"
        };
        let dev = p.layer;
        let (name, lines): (String, Vec<Fact>) = match target {
            Target::Control(id) => (
                facts::control_name(id, snapshot),
                facts::control(id, p.c, snapshot, shown, dev),
            ),
            Target::Node { layer, index, kind } => {
                let node = p.c.node(*layer, *index).filter(|n| n.kind() == *kind);
                match node {
                    Some(node) => {
                        let owner = target.owner(p.c);
                        let lines =
                            facts::node(node, *layer, *index, p.c, owner.as_ref(), shown, dev);
                        (format!("{kind} node"), lines)
                    }
                    None => {
                        let y = p.title(&format!("{kind} node"), note, area);
                        let gone = "The frame no longer draws it there: the page was built again.";
                        p.text(gone, area.x, y, area.w, p.mid(), &caption);
                        return;
                    }
                }
            }
        };
        let mut y = p.title(&name, note, area);
        let label_w = p.width("border width", &caption) + gap * 2.0;
        let swatch = (p.line() - 4.0).round();
        for line in lines {
            if y + row > area.bottom() {
                break;
            }
            let place = Rect::new(area.x, y, area.w, row);
            let hot = line.act.clone().is_some_and(|act| p.region(place, act));
            if hot {
                p.wash(place, 1.0);
            }
            p.text(line.label, area.x, y, label_w, p.faint(), &caption);
            let mut x = area.x + label_w;
            if let Some(color) = line.swatch {
                let at = Rect::new(x, y + 2.0, swatch, swatch);
                p.chip(at.outset(1.0), p.rule());
                p.chip(at, color);
                x += swatch + gap;
            }
            let style = if line.code { &mono } else { &num };
            let ink = if line.act.is_some() {
                p.theme.colors.info.0
            } else {
                p.ink()
            };
            p.text(&line.value, x, y, area.right() - x, ink, style);
            y += row;
            p.fill(
                Rect::new(area.x, y - 1.0, area.w, 1.0),
                alpha(p.ink(), 0.04),
            );
        }
    }

    /// Over the page: a ring around the thing chosen, and around the one
    /// the pointer is on (on the page while picking, or on its line of
    /// the tree) with what it is and its size beside it.
    pub fn overlay(&self, p: &mut Painter<'_>, page: Rect, snapshot: Option<&Snapshot>) {
        let from_tree = match &p.hot {
            Some(Act::Control(id)) => Some(Target::Control(id.clone())),
            _ => None,
        };
        let pointed = self.hover.clone().or(from_tree);
        let info = p.theme.colors.info.0;
        let shows = |r: &Rect| r.x < page.right() && r.y < page.bottom();
        if let Some(target) = self
            .picked
            .as_ref()
            .filter(|t| Some(*t) != pointed.as_ref())
            && let Some(r) = target.rect(p.c).filter(shows)
        {
            p.outline(r, info, 1.5);
        }
        let Some(target) = pointed else {
            return;
        };
        let Some(r) = target.rect(p.c).filter(shows) else {
            return;
        };
        // a node also shows the control it sits in, fainter around it
        let owner = match &target {
            Target::Node { .. } => target.owner(p.c),
            Target::Control(_) => None,
        };
        if let Some(around) = owner
            .as_ref()
            .and_then(|id| Target::Control(id.clone()).rect(p.c))
            .filter(shows)
        {
            p.outline(around, alpha(info, 0.45), 1.0);
        }
        p.chip(r, alpha(info, 0.14));
        p.outline(r, info, 1.5);

        // what it is and its size on a dark tag, over it or under it
        let mut word = target.word(snapshot);
        if let Some(id) = &owner {
            let inside = Target::Control(id.clone()).word(snapshot);
            word = format!("{word} in {inside}");
        }
        let (strong, num) = (p.strong(), p.num());
        let size = format!("{:.0} × {:.0}", r.w, r.h);
        let gap = p.gap();
        let w = p.width(&word, &strong) + gap + p.width(&size, &num) + gap * 2.0;
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
        p.text(&word, x + gap, text_y, w, ground, &strong);
        p.text_right(&size, tag.right() - gap, text_y, alpha(ground, 0.75), &num);
    }
}

/// What the frame draws, for a page that binds no control: every visible
/// layer of the page with its nodes counted by kind.
fn scene(p: &mut Painter<'_>, area: Rect) {
    let (caption, num) = (p.caption(), p.num());
    let gap = p.gap();
    let row = p.row() + 2.0;
    let dev = p.layer;
    let mut rows: Vec<(String, String)> = Vec::new();
    let mut total = 0;
    for layer in p.c.layers().iter().filter(|l| l.visible && l.id != dev) {
        let mut kinds: Vec<(&'static str, usize)> = Vec::new();
        for node in layer.nodes() {
            match kinds.iter_mut().find(|(kind, _)| *kind == node.kind()) {
                Some((_, n)) => *n += 1,
                None => kinds.push((node.kind(), 1)),
            }
        }
        if kinds.is_empty() {
            continue;
        }
        total += layer.nodes().len();
        kinds.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
        let said: Vec<String> = kinds
            .iter()
            .map(|(kind, n)| format!("{n} {kind}"))
            .collect();
        rows.push((
            format!("layer {} · z {}", layer.id.0, layer.z_order),
            said.join(" · "),
        ));
    }
    let note = format!("{total} nodes · no control bound");
    let mut y = p.title("Scene", &note, area);
    let name_w = rows
        .iter()
        .map(|(name, _)| p.width(name, &caption))
        .fold(0.0, f32::max)
        + gap * 2.0;
    for (name, kinds) in rows {
        if y + row > area.bottom() {
            break;
        }
        let text_y = y + (row - p.line()) / 2.0;
        p.text(&name, area.x, text_y, name_w, p.mid(), &caption);
        p.text(
            &kinds,
            area.x + name_w,
            text_y,
            area.w - name_w,
            p.ink(),
            &num,
        );
        y += row;
        p.fill(
            Rect::new(area.x, y - 1.0, area.w, 1.0),
            alpha(p.ink(), 0.04),
        );
    }
    let hint = "The page gives none of it an id: pick any of it with the pointer in the strip.";
    if y + row * 2.0 <= area.bottom() {
        p.text(hint, area.x, y + gap, area.w, p.faint(), &caption);
    }
}

/// The tree's lines: the snapshot's nodes under their parents, the lines
/// under a folded one left out; or, for a page that projects nothing, the
/// ids the frame bound, in bind order, each under the id it extends
/// (`shell.sidebar.row[0]` under `shell.sidebar`) and named by its last
/// part.
fn lines(p: &Painter<'_>, snapshot: Option<&Snapshot>, folded: &HashSet<SemId>) -> Vec<Line> {
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
                    parent: false,
                }
            })
            .collect();
    };
    fn walk(
        snapshot: &Snapshot,
        node: &Node,
        depth: usize,
        folded: &HashSet<SemId>,
        out: &mut Vec<Line>,
    ) {
        let mut children = snapshot.children(&node.id).peekable();
        out.push(Line {
            depth,
            id: node.id.clone(),
            role: format!("{:?}", node.role),
            label: node.label.clone(),
            parent: children.peek().is_some(),
        });
        if folded.contains(&node.id) {
            return;
        }
        for child in children {
            walk(snapshot, child, depth + 1, folded, out);
        }
    }
    let mut out = Vec::with_capacity(snapshot.len());
    for root in snapshot.roots() {
        walk(snapshot, root, 0, folded, &mut out);
    }
    out
}
