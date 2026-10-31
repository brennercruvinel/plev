//! The AccessKit tree as a projection of the semantic contract: the nodes,
//! roles, labels, states and actions come from a `accord::Snapshot`, the
//! same one agents and tests read, and the bounds from the compositor's
//! binding table (docs/decisoes/semantica/0035-semantic-binding-beside-the-scene.md). Nothing
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
//! - a group with no label, no description, no action and no focus is a
//!   `GenericContainer`: structure the adapters leave out, its children
//!   lifted into its place, so a reader never stops on an empty "group".
//!   a group named only by the key of its own id (`todo_panel.item[1]`
//!   named "1", how a generated surface names an instance of a repeat) is
//!   named by the labels inside it, the words a person sees in it.
//! - state, by key: `text` of a text input, `value` of any node (a
//!   number with the node's range and step, or a text) and `selected` of
//!   a select are the value; the bool the node's own `toggle` action
//!   reads (else `checked` or `on`) is `toggled`; a bool `expanded` is
//!   `expanded`; a bool `selected` is `selected`; a text `selected` on a
//!   container names the key of its selected child (`todo.filter` holds
//!   `all`, so `todo.filter[all]` is selected and its siblings are not),
//!   and so does a table's `selected_row` (the comps table's key for it).
//! - a status is a polite live region and an alert an assertive one (a
//!   secret one is neither), and what a reader hears when it changes is
//!   the node's value: its own, else its `message`, else its label (see
//!   `live`); a message is its name too, since some adapters announce the
//!   name. The focus never moves for it.
//! - actions: `Click` when the node has a primary action (see
//!   `action::primary`) and both are on (a press the app would refuse is
//!   not offered: a reader would hear nothing happen; the node keeps
//!   telling whether it is disabled), `Focus` when it is focusable and not inert,
//!   `SetValue` when it has a setter (`action::setter`: its `set_value`,
//!   or a select's `select`), `Increment` and `Decrement` when it
//!   declares `increment` and `decrement`.
//! - an item of a collection holds, as controls of their own, the actions
//!   its own node does not reach (`present`): its toggle as a checkbox,
//!   every other action without a parameter as a button. They are in the
//!   tree only, with no bounds, and a click on one is the item's action.
//! - focus is the snapshot's `focused`, or the window.

use accesskit::{Action, Live, Node as AkNode, Rect, Toggled, Tree, TreeId, TreeUpdate};
use accord::{Node, Role, SemId, Snapshot, StateKey, Value};
use rustc_hash::{FxHashMap, FxHashSet};

use super::action;
use super::ids::{NodeIds, ROOT};
use super::present::{self, Kind, Presented};
use super::role;
use crate::drawing::compositor::{Binding, Compositor, intersect_rects};

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
    let presented: Vec<Vec<Presented<'_>>> = order
        .iter()
        .map(|n| present::presented_in(n, snapshot))
        .collect();
    let mut ids = NodeIds::assign(
        order
            .iter()
            .map(|n| &n.id)
            .chain(presented.iter().flatten().map(|p| &p.id)),
    );
    for (node, list) in order.iter().zip(&presented) {
        for p in list {
            ids.stand_for(p.id.clone(), node.id.clone(), p.decl.name.clone());
        }
    }
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
    for ((node, placed), list) in order.iter().zip(&bounds).zip(&presented) {
        let mut out = describe(node);
        if let Some(words) = instance_name(snapshot, node) {
            out.set_label(words);
        }
        let mut kids = children(snapshot.children(&node.id), &ids);
        kids.extend(list.iter().filter_map(|p| ids.node(&p.id)));
        out.set_children(kids);
        if let Some(rect) = placed {
            out.set_bounds(*rect);
        }
        described.push(out);
    }
    select_children(snapshot, &order, &at, &mut described);
    for ((node, out), list) in order.iter().zip(described).zip(&presented) {
        if let Some(id) = ids.node(&node.id) {
            nodes.push((id, out));
        }
        for p in list {
            if let Some(id) = ids.node(&p.id) {
                nodes.push((id, describe_presented(node, p)));
            }
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

/// The AccessKit role of `n`: its contract role (`role::of`), but for a
/// group that names nothing, says nothing and does nothing. That one is
/// only structure, a `GenericContainer`, which every adapter of the lock
/// leaves out of the platform tree and lifts its children into its place
/// (accesskit_consumer 0.35.0 and 0.36.0 filters.rs:31-34), so a reader
/// does not stop on an empty "group".
fn ak_role(n: &Node) -> accesskit::Role {
    let bare = n.role == Role::Group
        && n.label.is_empty()
        && n.description.is_none()
        && n.actions.is_empty()
        && !n.focusable;
    if bare {
        accesskit::Role::GenericContainer
    } else {
        role::of(n.role, n.constraints.secret)
    }
}

/// The name of a group named only by the key of its own id: the words of
/// the labels right inside it, in order (a generated surface names an
/// instance of a repeat by its item's key, and the IR interpolates no
/// label: "1" says nothing a person sees). `None` for any other node, and
/// for such a group with no label inside.
fn instance_name(snapshot: &Snapshot, n: &Node) -> Option<String> {
    if n.role != Role::Group || n.label.is_empty() {
        return None;
    }
    let keyed = n.id.parent_path().map(|p| p.key(n.label.as_str()));
    if keyed.as_ref() != Some(&n.id) {
        return None;
    }
    let words: Vec<&str> = snapshot
        .children(&n.id)
        .filter(|c| c.role == Role::Label && !c.label.is_empty())
        .map(|c| c.label.as_str())
        .collect();
    (!words.is_empty()).then(|| words.join(" "))
}

/// One node's role, label, description, states and actions.
fn describe(n: &Node) -> AkNode {
    let secret = n.constraints.secret;
    let mut out = AkNode::new(ak_role(n));
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
    let holds_toggle = present::holds_toggle(n);
    let toggled = toggled_key(n);
    if let Some(Value::Bool(on)) = toggled.and_then(|k| n.state.get(k))
        && !holds_toggle
    {
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
        live(n, &mut out);
    }
    if action::primary(n).is_some_and(|d| d.enabled) && n.enabled && !holds_toggle {
        out.add_action(Action::Click);
    }
    if n.focusable && !n.inert {
        out.add_action(Action::Focus);
    }
    if action::setter(n).is_some() {
        out.add_action(Action::SetValue);
    }
    for (name, act) in [
        ("increment", Action::Increment),
        ("decrement", Action::Decrement),
    ] {
        if n.actions.iter().any(|a| a.name.as_str() == name) {
            out.add_action(act);
        }
    }
    out
}

/// A control presented inside the item `item` (`present`): a checkbox
/// checked by the bool the toggle reads, or a button; named, off while
/// its action or the item is off, pressed by a click while on. No focus
/// of its own: a reader walks to it, and a click on it is the item's
/// action.
fn describe_presented(item: &Node, p: &Presented<'_>) -> AkNode {
    let role = match p.kind {
        Kind::Checkbox => accesskit::Role::CheckBox,
        Kind::Button => accesskit::Role::Button,
    };
    let mut out = AkNode::new(role);
    out.set_label(p.label.as_str());
    if p.kind == Kind::Checkbox
        && let Some(Value::Bool(on)) = toggled_key(item).and_then(|k| item.state.get(k))
    {
        out.set_toggled(Toggled::from(*on));
    }
    if p.decl.enabled && item.enabled {
        out.add_action(Action::Click);
    } else {
        out.set_disabled();
    }
    out
}

/// The key of the bool that says whether the node is on: the one its own
/// `toggle` action reads, else `checked` or `on`.
pub(crate) fn toggled_key(n: &Node) -> Option<&StateKey> {
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

/// A status is a polite live region and an alert an assertive one: a
/// screen reader says what changed in it, and the focus stays where it
/// is (nothing here touches the update's focus). What it says is its
/// value, the one the node already has, else its `message` when it keeps
/// its words in one (a status named "status" whose line is the message),
/// else its label (a toast, a counter). An empty message says nothing.
///
/// Value and name both: macOS announces a live node's value
/// (accesskit_macos 0.26.1 event.rs:35-44, 236-239, 298-306), Windows and
/// AT-SPI its name (accesskit_windows 0.32.1 adapter.rs:225-232, 271-281;
/// accesskit_atspi_common 0.18.1 adapter.rs:71-76, node.rs:543-556). So
/// the words are the name as well (task-75 phase 2a): a status whose words
/// are its label already had them there, and one that keeps them in a
/// `message` under a fixed label (`silences.status`, "status"; the
/// proposing row's, "proposal") is named by the message while it says
/// one, and by its label while it says nothing. The projection keeps its
/// label and its message: this is how a reader is told, not what the
/// node is.
fn live(n: &Node, out: &mut AkNode) {
    let politeness = match n.role {
        Role::Status => Live::Polite,
        Role::Alert => Live::Assertive,
        _ => return,
    };
    out.set_live(politeness);
    if out.value().is_some() || out.numeric_value().is_some() {
        return;
    }
    let words = match n.state.get("message") {
        Some(Value::Text(message)) => message.as_str(),
        Some(_) => "",
        None => n.label.as_str(),
    };
    if !words.is_empty() {
        out.set_value(words);
        out.set_label(words);
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
/// select holds its option's label) marks nothing. A table says it in
/// `selected_row` (`engine::widgets::content::Table::project`, task-75 phase 2b).
fn select_children(
    snapshot: &Snapshot,
    order: &[&Node],
    at: &FxHashMap<&SemId, usize>,
    described: &mut [AkNode],
) {
    for node in order {
        let says = match node.role {
            Role::Table => "selected_row",
            _ => "selected",
        };
        let Some(Value::Text(key)) = node.state.get(says) else {
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
