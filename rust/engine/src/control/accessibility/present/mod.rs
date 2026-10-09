//! The controls the tree presents inside an item of a collection (task-75
//! phase 2a): the item's actions its own click does not reach, each as a
//! control a reader can press.
//!
//! No adapter of the lock implements AccessKit's custom actions
//! (accesskit_macos 0.26.1, accesskit_windows 0.32.1 and
//! accesskit_atspi_common 0.18.1: no occurrence in their sources), so a
//! row that declares a toggle and a remove, or a cut that declares a
//! preview, an apply and a reject, keeps all but its click from a reader.
//! The page does not project them as controls of its own: they would be
//! capabilities of their own for an agent and in the registry a generated
//! surface is verified against, twice what the page offers. They are the
//! reader's presentation of the item's actions, and the tree is where the
//! adapters' limits are known:
//!
//! - an item (a list item, a row, a tree item) whose click is its
//!   `toggle` holds the toggle as a checkbox, named by the action's label
//!   or by the key the toggle reads ("completed"), checked by that key;
//!   the item itself then carries no toggled state and no click (the
//!   macOS adapter makes a list item a group and would say its toggled as
//!   a bare value, accesskit_macos 0.26.1 node.rs:48, 323-326);
//! - every other action of the item without a parameter, and that no
//!   action of the item's own node reaches (its click, a value, a step),
//!   is a button inside it, named by the action's label or by its name
//!   ("remove", "apply cut"), off while the action is off.
//!
//! A presented control is `<item>[<action>]`, the form of a comps modal's
//! buttons, and is in the tree only: a click on it reads back as the
//! item's action, observed over the snapshot the reader was shown
//! (`action::translate`), through the app's one dispatcher like any
//! other request of a reader. An id the snapshot already holds is the
//! page's own control, and nothing is presented in its place. A presented
//! control has no bounds: nothing binds a rect to an action, and the
//! item's rect would win a reader's hit test from the item, children
//! first (accesskit_consumer 0.36.0 node.rs:346-373).

use accord::{ActionDecl, Node, Role, SemId, Snapshot};

use super::action;
use super::tree::toggled_key;

/// What a presented control is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// The item's `toggle`, checked by the bool it reads.
    Checkbox,
    /// Any other action of the item.
    Button,
}

/// A control the tree presents inside an item, standing for one of the
/// item's actions.
#[derive(Clone, Debug, PartialEq)]
pub struct Presented<'a> {
    /// `<item>[<action>]`: in the tree, never in the snapshot.
    pub id: SemId,
    pub kind: Kind,
    /// What a reader hears.
    pub label: String,
    /// The item's action a click on it asks for.
    pub decl: &'a ActionDecl,
}

/// Whether `node` is an item of a collection.
fn is_item(node: &Node) -> bool {
    matches!(node.role, Role::ListItem | Role::Row | Role::TreeItem)
}

/// Whether the item `node` holds its toggle as a checkbox: its click is
/// its `toggle`.
pub fn holds_toggle(node: &Node) -> bool {
    is_item(node) && action::primary(node).is_some_and(|d| d.name.as_str() == "toggle")
}

/// An action's name as words: `apply_cut` is "apply cut".
fn words(name: &str) -> String {
    name.replace('_', " ")
}

/// The controls the tree presents inside `node`, in the order its actions
/// are declared, the toggle's checkbox first. Empty for anything but an
/// item, and for an item whose own node reaches every action.
pub fn presented(node: &Node) -> Vec<Presented<'_>> {
    if !is_item(node) {
        return Vec::new();
    }
    let mut out = Vec::new();
    let primary = action::primary(node);
    if holds_toggle(node)
        && let Some(decl) = primary
    {
        let key = toggled_key(node).map(|k| k.as_str().to_owned());
        out.push(Presented {
            id: node.id.key(decl.name.as_str()),
            kind: Kind::Checkbox,
            label: decl
                .label
                .clone()
                .or(key)
                .unwrap_or_else(|| words("toggle")),
            decl,
        });
    }
    let reached = |d: &ActionDecl| {
        primary.is_some_and(|p| p.name == d.name)
            || matches!(d.name.as_str(), "set_value" | "increment" | "decrement")
    };
    for decl in node.actions.iter().filter(|d| d.param.is_none()) {
        if reached(decl) {
            continue;
        }
        out.push(Presented {
            id: node.id.key(decl.name.as_str()),
            kind: Kind::Button,
            label: decl
                .label
                .clone()
                .unwrap_or_else(|| words(decl.name.as_str())),
            decl,
        });
    }
    out
}

/// `presented`, but for an id `snapshot` already holds: that one is the
/// page's own control, and nothing is presented in its place.
pub(crate) fn presented_in<'a>(node: &'a Node, snapshot: &Snapshot) -> Vec<Presented<'a>> {
    let mut out = presented(node);
    out.retain(|p| snapshot.get(&p.id).is_none());
    out
}

#[cfg(test)]
mod tests;
