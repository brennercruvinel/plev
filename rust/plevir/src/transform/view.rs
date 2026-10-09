//! the operations on what shows a datum and what the surface offers:
//! `SwapView` (the same datum by another component) and `Promote` (a
//! button for an action the app declares and the surface does not show).

use sem::{ActionName, Actor, Actors, SemId};

use super::tree;
use super::{Draft, Inverse, Op, TransformError, refused};
use crate::caps::{ActionCap, Capabilities};
use crate::ir::{CapabilityRef, IrNode};
use crate::item::Pattern;
use crate::vocab::{Component, spec_of};

/// the name of a component in a message, a chart with its kind.
pub(super) fn name(c: Component) -> String {
    match c {
        Component::Chart(kind) => format!("{} chart", kind.as_str()),
        other => other.name().to_owned(),
    }
}

/// the components that show one datum alike, each under its own name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Family {
    /// one value: a text's `text`, a stat's `value`, a badge's `text`.
    Value,
    /// one bool, flipped by `toggle`: a checkbox's `checked`, a switch's
    /// `on`, a toggle chip's `selected`.
    Flag,
    /// a choice among options: a select's and a tab strip's `selected`
    /// and `options`, `select` to choose.
    Choice,
    /// a list: a table's `rows`, a chart's `data`.
    Rows,
}

/// the family of `c` and the name it shows the family's datum by.
fn family(c: Component) -> Option<(Family, &'static str)> {
    Some(match c {
        Component::Text | Component::Badge => (Family::Value, "text"),
        Component::Stat => (Family::Value, "value"),
        Component::Checkbox => (Family::Flag, "checked"),
        Component::Switch => (Family::Flag, "on"),
        Component::Chip => (Family::Flag, "selected"),
        Component::Select | Component::Tabs => (Family::Choice, "selected"),
        Component::Table => (Family::Rows, "rows"),
        Component::Chart(_) => (Family::Rows, "data"),
        _ => return None,
    })
}

/// `SwapView`: the node shown by `to`, which shows what it shows under
/// its own name; every other binding kept, so nothing the node shows or
/// sends is dropped. only within a family; a list that repeats a template
/// is refused, since its template's ids would leave the surface without a
/// `Hide`. what the verifier checks next is whether the datum's kind fits
/// `to` and whether `to` is named. its inverse is the component before.
pub(super) fn swap(d: &mut Draft, id: &SemId, to: Component) -> Result<Inverse, TransformError> {
    let node = d.node_mut(id)?;
    let target = name(to);
    let Some(from) = node.component else {
        return Err(refused(format!(
            "`{id}` is a layout container: no {target} shows what it holds"
        )));
    };
    let source = name(from);
    if from == to {
        return Err(refused(format!("`{id}` is a {target} already")));
    }
    if node.repeat.is_some() {
        return Err(refused(format!(
            "`{id}` repeats a template: as a {target} the ids of its items would leave the \
             surface without a `Hide`"
        )));
    }
    let fits = match (family(from), family(to)) {
        (Some((a, old)), Some((b, new))) if a == b => Some((old, new)),
        _ => None,
    };
    let Some((old, new)) = fits else {
        return Err(refused(format!(
            "no {target} shows what a {source} shows: a swap keeps the datum and every binding"
        )));
    };
    if from == Component::Chip && node.bind.actions.contains_key("activate") {
        return Err(refused(format!(
            "`{id}` is a chip that sends `activate`, a button: a {target} would drop it"
        )));
    }
    if let Some(at) = node.bind.state.remove(old) {
        node.bind.state.insert(new.to_owned(), at);
    }
    let spec = to.spec();
    if let Some(prop) = node
        .bind
        .state
        .keys()
        .find(|p| spec.state_prop(p).is_none())
    {
        return Err(refused(format!(
            "a {target} shows no `{prop}`, which `{id}` binds"
        )));
    }
    if let Some(event) = node.bind.actions.keys().find(|e| spec.event(e).is_none()) {
        return Err(refused(format!(
            "a {target} sends no `{event}`, which `{id}` binds"
        )));
    }
    node.component = Some(to);
    Ok(Inverse::Op(Op::SwapView {
        id: id.clone(),
        component: from,
    }))
}

/// true when `actors` lets `actor` ask, the mapping `sem::check` uses.
fn allows(actors: &Actors, actor: &Actor) -> bool {
    match actor {
        Actor::Human => actors.human,
        Actor::Agent { .. } => actors.agent,
        Actor::Assistive => actors.assistive,
        Actor::Test { .. } => actors.test,
    }
}

/// the node under `root` that sends `cap`: by the same reference, or by a
/// template's that `cap`'s target is an item of.
fn sender<'a>(root: &'a IrNode, cap: &CapabilityRef) -> Option<&'a IrNode> {
    tree::nodes(root).into_iter().find(|n| {
        n.bind.actions.values().any(|c| {
            c.action == cap.action
                && (c.target == cap.target
                    || Pattern::of(c.target.as_str())
                        .is_some_and(|p| p.matches(cap.target.as_str())))
        })
    })
}

/// the declaration of `action` on `target`: the app's own, or, for the
/// template of an item (`silences.cut[{item}]`, the item once), the one
/// of the first node of the registry that fits it and declares the
/// action, with the template as its target. the verifier binds a
/// template the same way and then checks that every node that fits
/// declares it alike, so a promote never widens what it binds.
fn declared(caps: &Capabilities, target: &SemId, action: &ActionName) -> Option<ActionCap> {
    if let Some(cap) = caps.action(target, action) {
        return Some(cap.clone());
    }
    let pattern = Pattern::of(target.as_str())?;
    let member = caps
        .nodes()
        .filter(|id| pattern.matches(id.as_str()))
        .find_map(|id| caps.action(id, action))?;
    Some(ActionCap {
        target: target.clone(),
        ..member.clone()
    })
}

/// `Promote`: a button for `action` on `target`, labelled as the app
/// labels it, at the end of the container `into`, its id `into` followed
/// by the action. `target` may be the template of an item: the button then
/// asks each item's own action (`declared`). refused when the app
/// declares no such action, when its policy keeps it from `by`, when it
/// takes a parameter (a button's `activate` sends none), and when the
/// surface shows it already (in its tree, or hidden: then it is `Show`).
/// the verifier then checks that whoever wrote the surface may bind it,
/// that it is not destructive without a confirmation, and, for a
/// template, that every item declares it alike. its inverse is `into`
/// before.
pub(super) fn promote(
    d: &mut Draft,
    (target, action): (&SemId, &ActionName),
    into: &SemId,
    by: &Actor,
    caps: &Capabilities,
) -> Result<Inverse, TransformError> {
    let Some(declared) = declared(caps, target, action) else {
        return Err(refused(format!(
            "the app declares no `{action}` on `{target}`"
        )));
    };
    if !allows(&declared.policy.actors, by) {
        return Err(refused(format!(
            "the app keeps `{action}` on `{target}` from {by}: it is not promoted for them"
        )));
    }
    if let Some(param) = declared.param {
        return Err(refused(format!(
            "a promoted control is a button, and `{action}` takes a {}",
            param.kind.as_str()
        )));
    }
    let cap = CapabilityRef {
        target: target.clone(),
        action: action.clone(),
    };
    if let Some(shown) = sender(d.root(), &cap) {
        return Err(refused(format!(
            "`{}` shows `{action}` on `{target}` already",
            shown.sem
        )));
    }
    if let Some((root, _)) = d
        .aside
        .iter()
        .find(|(_, kept)| sender(&kept.node, &cap).is_some())
    {
        return Err(refused(format!(
            "`{action}` on `{target}` is shown in the hidden `{root}`: show it"
        )));
    }
    let id = into
        .child(action.as_str())
        .map_err(|e| refused(format!("no id for the promoted control: {e}")))?;
    d.free(&id)?;
    let label = if declared.label.trim().is_empty() {
        action.as_str().to_owned()
    } else {
        declared.label.clone()
    };
    let place = d.node_mut(into)?;
    if place.repeat.is_some() || !spec_of(place.component).container {
        let what = place.component.map_or("container".to_owned(), name);
        return Err(refused(format!(
            "`{into}` is a {what} that holds no children of its own: promote into a container"
        )));
    }
    let before = place.clone();
    let mut button = IrNode::new(id);
    button.component = Some(Component::Button);
    button.label = Some(label);
    button.bind.actions.insert("activate".to_owned(), cap);
    place.children.push(button);
    Ok(Inverse::Restore(before))
}
