//! The AccessKit tree as a projection of the semantic contract: the nodes,
//! roles, labels, states and actions come from a `sem::Snapshot`, the
//! same one agents and tests read, and the bounds from the compositor's
//! binding table (docs/ADR/semantic-binding-beside-the-scene.md). Nothing
//! is guessed from geometry: a pixel with no `SemId` has no node.
//!
//! The rules, each tested:
//!
//! - the root is the window, labeled with its title, and the snapshot's
//!   roots are its children. every node follows in tree order, children
//!   in projection order. only nodes reachable from the roots are
//!   emitted, so the update is always a valid tree.
//! - bounds are AccessKit's space: physical pixels, relative to the
//!   window's client area, y down. a binding is logical, so it is scaled
//!   by the window's scale factor, after the clip open at the bind is
//!   applied (the part a person can see and point at).
//! - a node without a binding this frame (scrolled out, virtualized, a
//!   container nothing binds) stays in the tree: its meaning, its id and
//!   its actions do not depend on being drawn. it takes the union of its
//!   bound descendants' bounds, or none. AccessKit has no offscreen
//!   state; a node with no bounds is how "not on screen" reads.
//! - an inert node (an open modal leaves the page inert) is hidden, as
//!   the `inert` attribute does on the web: it keeps its id and is not
//!   presented, and a request that still reaches it comes back `Inert`
//!   from the dispatcher.
//! - a `secret` node is a password field when it is a text input, and
//!   never has a value (the snapshot only holds its length anyway).
//! - state, by key: `text` of a text input, `value` of any node (a
//!   number with the node's range and step, or a text) and `selected` of
//!   a select are the value; the bool the node's own `toggle` action
//!   reads (else `checked` or `on`) is `toggled`; a bool `expanded` is
//!   `expanded`; a bool `selected` is `selected`; a text `selected` on a
//!   container names the key of its selected child (`todo.filter` holds
//!   `all`, so `todo.filter[all]` is selected and its siblings are not).
//! - actions: `Click` when the node has a primary action (see
//!   `action::primary`), `Focus` when it is focusable and not inert,
//!   `SetValue`, `Increment` and `Decrement` when it declares
//!   `set_value`, `increment` and `decrement`.
//! - focus is the snapshot's `focused`, or the window.

use accesskit::{Action, Node as AkNode, Rect, Toggled, Tree, TreeId, TreeUpdate};
use rustc_hash::{FxHashMap, FxHashSet};
use sem::{Node, Role, SemId, Snapshot, StateKey, Value};

use super::action;
use super::ids::{NodeIds, ROOT};
use super::role;
use crate::compositor::{Binding, Compositor, intersect_rects};

/// What the window adds around the app's meaning.
#[derive(Clone, Copy, Debug)]
pub struct Chrome<'a> {
    /// The window's title: the label of the root.
    pub title: &'a str,
    /// Physical pixels per logical pixel: the window's scale factor.
    pub scale: f64,
    /// The window's inner size in physical pixels: the root's bounds
    /// (`(0, 0)` leaves the root without bounds).
    pub size: (u32, u32),
}

/// What a screen reader was shown: the snapshot a tree was built from
/// and that tree's node ids. Its action requests are read against it.
#[derive(Debug, Default)]
pub struct Shown {
    pub snapshot: Snapshot,
    pub ids: NodeIds,
}

/// The full tree of `snapshot` inside the window `chrome` names, placed
/// by this frame's bindings in `compositor`, and the node ids it used
/// (what reads the screen reader's requests back, see `Shown`).
pub fn build(
    snapshot: &Snapshot,
    compositor: &Compositor,
    chrome: &Chrome<'_>,
) -> (TreeUpdate, NodeIds) {
    let order = tree_order(snapshot);
    let ids = NodeIds::assign(order.iter().map(|n| &n.id));
    let at: FxHashMap<&SemId, usize> = order.iter().enumerate().map(|(i, n)| (&n.id, i)).collect();
    let bounds = placement(snapshot, &order, &at, compositor, chrome.scale);

    let mut nodes = Vec::with_capacity(order.len() + 1);
    let mut root = AkNode::new(accesskit::Role::Window);
    root.set_label(chrome.title);
    root.set_children(children(snapshot.roots(), &ids));
    let (w, h) = chrome.size;
    if w > 0 && h > 0 {
        root.set_bounds(Rect::new(0.0, 0.0, f64::from(w), f64::from(h)));
    }
    nodes.push((ROOT, root));

    let mut described: Vec<AkNode> = Vec::with_capacity(order.len());
    for (node, placed) in order.iter().zip(&bounds) {
        let mut out = describe(node);
        out.set_children(children(snapshot.children(&node.id), &ids));
        if let Some(rect) = placed {
            out.set_bounds(*rect);
        }
        described.push(out);
    }
    select_children(snapshot, &order, &at, &mut described);
    for (node, out) in order.iter().zip(described) {
        if let Some(id) = ids.node(&node.id) {
            nodes.push((id, out));
        }
    }

    let focus = snapshot.focused().and_then(|f| ids.node(f)).unwrap_or(ROOT);
    let update = TreeUpdate {
        nodes,
        tree: Some(Tree {
            root: ROOT,
            toolkit_name: Some("plev".to_owned()),
            toolkit_version: Some(env!("CARGO_PKG_VERSION").to_owned()),
        }),
        tree_id: TreeId::ROOT,
        focus,
    };
    (update, ids)
}

/// Every node reachable from the roots, in tree order: each root in
/// projection order, then its descendants depth first, children in
/// projection order. A node is listed once even if a deserialized
/// snapshot loops.
pub(crate) fn tree_order(snapshot: &Snapshot) -> Vec<&Node> {
    let mut out = Vec::with_capacity(snapshot.len());
    let mut seen: FxHashSet<&SemId> = FxHashSet::default();
    let mut stack: Vec<&Node> = snapshot.roots().collect();
    stack.reverse();
    while let Some(node) = stack.pop() {
        if !seen.insert(&node.id) {
            continue;
        }
        out.push(node);
        let from = stack.len();
        stack.extend(snapshot.children(&node.id));
        if let Some(pushed) = stack.get_mut(from..) {
            pushed.reverse();
        }
    }
    out
}

fn children<'a>(nodes: impl Iterator<Item = &'a Node>, ids: &NodeIds) -> Vec<accesskit::NodeId> {
    nodes.filter_map(|n| ids.node(&n.id)).collect()
}

/// Each node's bounds in physical pixels: its own binding, clipped, or
/// the union of its children's (computed children first).
fn placement(
    snapshot: &Snapshot,
    order: &[&Node],
    at: &FxHashMap<&SemId, usize>,
    compositor: &Compositor,
    scale: f64,
) -> Vec<Option<Rect>> {
    let mut out: Vec<Option<Rect>> = order
        .iter()
        .map(|n| compositor.binding(&n.id).and_then(|b| physical(b, scale)))
        .collect();
    for i in (0..order.len()).rev() {
        let (Some(node), Some(None)) = (order.get(i), out.get(i)) else {
            continue;
        };
        let union = snapshot
            .children(&node.id)
            .filter_map(|c| at.get(&c.id).and_then(|&j| out.get(j).copied().flatten()))
            .reduce(|a, b| a.union(b));
        if let Some(slot) = out.get_mut(i) {
            *slot = union;
        }
    }
    out
}

/// The visible part of a binding (its rect inside its clip), scaled to
/// physical pixels; `None` when nothing of it is visible.
fn physical(b: &Binding, scale: f64) -> Option<Rect> {
    let [x, y, w, h] = match b.clip {
        Some(clip) => intersect_rects(b.rect, clip),
        None => b.rect,
    };
    if !(w > 0.0 && h > 0.0) {
        return None;
    }
    let [x, y, w, h] = [x, y, w, h].map(f64::from);
    Some(Rect::new(
        x * scale,
        y * scale,
        (x + w) * scale,
        (y + h) * scale,
    ))
}

/// One node's role, label, description, states and actions.
fn describe(n: &Node) -> AkNode {
    let secret = n.constraints.secret;
    let mut out = AkNode::new(role::of(n.role, secret));
    if !n.label.is_empty() {
        out.set_label(n.label.as_str());
    }
    if let Some(text) = &n.description {
        out.set_description(text.as_str());
    }
    if !n.enabled {
        out.set_disabled();
    }
    if n.inert {
        out.set_hidden();
    }
    let toggled = toggled_key(n);
    if let Some(Value::Bool(on)) = toggled.and_then(|k| n.state.get(k)) {
        out.set_toggled(Toggled::from(*on));
    }
    if let Some(Value::Bool(open)) = n.state.get("expanded") {
        out.set_expanded(*open);
    }
    if toggled.map(StateKey::as_str) != Some("selected") {
        if let Some(Value::Bool(on)) = n.state.get("selected") {
            out.set_selected(*on);
        }
    }
    if !secret {
        value(n, &mut out);
    }
    if action::primary(n).is_some() {
        out.add_action(Action::Click);
    }
    if n.focusable && !n.inert {
        out.add_action(Action::Focus);
    }
    for (name, act) in [
        ("set_value", Action::SetValue),
        ("increment", Action::Increment),
        ("decrement", Action::Decrement),
    ] {
        if n.actions.iter().any(|a| a.name.as_str() == name) {
            out.add_action(act);
        }
    }
    out
}

/// The key of the bool that says whether the node is on: the one its own
/// `toggle` action reads, else `checked` or `on`.
fn toggled_key(n: &Node) -> Option<&StateKey> {
    let read = n
        .actions
        .iter()
        .filter(|a| a.name.as_str() == "toggle")
        .flat_map(|a| &a.reads)
        .find(|r| r.node == n.id && matches!(n.state.get(&r.key), Some(Value::Bool(_))))
        .map(|r| &r.key);
    read.or_else(|| {
        ["checked", "on"].into_iter().find_map(|key| {
            n.state
                .get_key_value(key)
                .filter(|(_, v)| matches!(v, Value::Bool(_)))
                .map(|(k, _)| k)
        })
    })
}

/// The value a person hears: a number with its range and step, or a
/// text. Never called for a secret node.
fn value(n: &Node, out: &mut AkNode) {
    let text = match n.role {
        Role::TextInput => n.state.get("text"),
        Role::Select => n.state.get("selected"),
        _ => None,
    };
    match text.or_else(|| n.state.get("value")) {
        Some(Value::Text(text)) => out.set_value(text.as_str()),
        Some(Value::Num(v)) => numeric(n, out, *v),
        // the value of an int beyond 2^53 rounds to the nearest f64.
        Some(Value::Int(v)) => numeric(n, out, *v as f64),
        _ => {}
    }
}

fn numeric(n: &Node, out: &mut AkNode, v: f64) {
    out.set_numeric_value(v);
    if let Some((low, high)) = n.constraints.range {
        out.set_min_numeric_value(low);
        out.set_max_numeric_value(high);
    }
    if let Some(step) = n.constraints.step {
        out.set_numeric_value_step(step);
    }
}

/// A container whose text `selected` names one of its children by key
/// marks that child selected and its other children not (a child with a
/// bool `selected` of its own keeps it). A text that names no child (a
/// select holds its option's label) marks nothing.
fn select_children(
    snapshot: &Snapshot,
    order: &[&Node],
    at: &FxHashMap<&SemId, usize>,
    described: &mut [AkNode],
) {
    for node in order {
        let Some(Value::Text(key)) = node.state.get("selected") else {
            continue;
        };
        let chosen = node.id.key(key.as_str());
        if !snapshot.children(&node.id).any(|c| c.id == chosen) {
            continue;
        }
        for child in snapshot.children(&node.id) {
            if matches!(child.state.get("selected"), Some(Value::Bool(_))) {
                continue;
            }
            if let Some(out) = at.get(&child.id).and_then(|&i| described.get_mut(i)) {
                out.set_selected(child.id == chosen);
            }
        }
    }
}

#[cfg(test)]
mod tests;
