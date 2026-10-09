//! the surface in the semantic contract, projected from the IR: the
//! surface is semantic by construction, so no widget's `project` is asked.
//!
//! - the surface is a `Region` named by its title, its tree under it;
//! - each node takes the role `uir::role_of` gives, its label (an instance
//!   is named by its item; a text or a badge by what it shows), its
//!   description, and the state it binds, read from the snapshot the
//!   surface shows;
//! - each bound event is an action of the node, named by the event
//!   (`activate`, `set_value`, `submit`, `toggle`, `select`, `confirm`,
//!   `cancel`), with the parameter, the policy and the `enabled` of the
//!   capability it asks for, and its reads: each state the capability
//!   reads, as the surface shows it when a node of the surface binds it
//!   (`todo_panel.input:text` for `todo.input:text`), as the app holds
//!   it otherwise. an agent that observed the surface has observed what
//!   its acts read wherever the surface shows it, and the surface routes
//!   the act, its observation mapped back, to the capability
//!   (`Surface::route`);
//! - a node with the id of one of the app's own nodes (a fixed screen
//!   written as IR) is the app's: the app already declares it, with its
//!   own actions, which are the ones the IR may bind on it (the verifier's
//!   identity rule);
//! - a separator has no role and is left out; a closed dialog is
//!   projected, its subtree is not.
//!
//! a read the surface does not show names the app's state, so the
//! surface projects into the builder that holds the app's own projection:
//! one snapshot, one contract.

use sem::{SemId, Snapshot, SnapshotBuilder, StateRef};
use uir::Component;

use crate::draw::{closed, shown_text};
use crate::expand::Node;
use crate::widget::declared;

/// the name a node takes in the contract.
fn label(node: &Node, shown: &Snapshot) -> String {
    if let Some(item) = &node.item {
        return item.clone();
    }
    match node.component {
        Some(Component::Text | Component::Badge) => shown_text(node, shown),
        _ => node.label.clone().unwrap_or_default(),
    }
}

/// which nodes of `tree` the surface projects: each one with a role that
/// is not the app's own and not inside a closed dialog.
pub(crate) fn projected(tree: &[Node], shown: &Snapshot) -> Vec<bool> {
    let mut out = vec![false; tree.len()];
    let mut hidden = vec![false; tree.len()];
    for (at, node) in tree.iter().enumerate() {
        let skipped = hidden.get(at).copied().unwrap_or(true);
        if skipped || closed(node, shown) {
            for &child in &node.children {
                if let Some(h) = hidden.get_mut(child) {
                    *h = true;
                }
            }
        }
        let own =
            uir::role_of(node.component, &node.bind).is_some() && shown.get(&node.id).is_none();
        if let Some(p) = out.get_mut(at) {
            *p = !skipped && own;
        }
    }
    out
}

/// the state of the surface that shows the app's `at`: the first
/// projected node that binds it, under the name it binds it by.
pub(crate) fn shown_as(tree: &[Node], projected: &[bool], at: &StateRef) -> Option<StateRef> {
    tree.iter()
        .zip(projected)
        .filter(|&(_, &p)| p)
        .find_map(|(node, _)| {
            let (prop, _) = node.bind.state.iter().find(|&(_, r)| r == at)?;
            let key = sem::StateKey::parse(prop).ok()?;
            Some(StateRef::new(node.id.clone(), key))
        })
}

/// projects the surface `id` titled `title`, unrolled as `tree` over
/// `shown`, into `b`. `inert` makes the whole surface inert (an overlay
/// holds the page); `focused` is the node that has the keyboard.
pub(crate) fn project(
    b: &mut SnapshotBuilder,
    (id, title): (&SemId, &str),
    tree: &[Node],
    shown: &Snapshot,
    inert: bool,
    focused: Option<&SemId>,
) {
    if shown.get(id).is_none() {
        b.node(id, sem::Role::Region, title).inert(inert);
    }
    let projected = projected(tree, shown);
    for (node, _) in tree.iter().zip(&projected).filter(|&(_, &p)| p) {
        let Some(role) = uir::role_of(node.component, &node.bind) else {
            continue;
        };
        let parent = node.parent.and_then(|p| tree.get(p)).map_or(id, |p| &p.id);
        let mut n = b.node(&node.id, role, label(node, shown));
        n.parent(parent);
        if let Some(d) = &node.description {
            n.description(d.as_str());
        }
        for (prop, at) in &node.bind.state {
            if let Some(value) = shown.value(at) {
                n.state(prop.as_str(), value.clone());
            }
        }
        // a control that sends a value takes the constraints of the node
        // it sends it to (range, step, options, max length, secret).
        let sends = ["set_value", "select"]
            .into_iter()
            .filter_map(|e| node.bind.actions.get(e))
            .find_map(|cap| declared(shown, cap));
        if let Some((target, _)) = sends {
            n.constraints(target.constraints.clone());
        }
        let caps: Vec<_> = node
            .bind
            .actions
            .values()
            .map(|cap| declared(shown, cap))
            .collect();
        if !caps.is_empty() {
            n.enabled(caps.iter().flatten().any(|(_, d)| d.enabled));
        }
        for ((event, _), cap) in node.bind.actions.iter().zip(caps) {
            let mut a = n.action(event.as_str());
            let Some((_, d)) = cap else {
                a.enabled(false);
                continue;
            };
            if let Some(text) = &d.label {
                a.label(text.as_str());
            }
            a.enabled(d.enabled).policy(d.policy);
            if let Some(param) = d.param {
                a.param(param);
            }
            for read in &d.reads {
                a.reads(shown_as(tree, &projected, read).unwrap_or_else(|| read.clone()));
            }
        }
    }
    if let Some(f) = focused {
        b.focus(f);
    }
}
