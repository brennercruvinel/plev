//! the operations on the shape of the tree: `Hide` and `Show` (a node out
//! of the tree and back, kept aside meanwhile), `Group` (siblings under a
//! new container) and `Reorder` (the order of a node's children).

use std::collections::BTreeSet;

use accord::SemId;

use super::tree::{self, Step};
use super::{Aside, Draft, Inverse, Op, TransformError, refused};
use crate::ir::IrNode;
use crate::token::Layout;

/// the parent of the node at `path` and its index there: `None` for the
/// root; refused for a repeat's item, which the repeat always has.
fn slot(d: &Draft, id: &SemId, path: &[Step]) -> Result<(Vec<Step>, usize), TransformError> {
    let Some((last, up)) = path.split_last() else {
        return Err(refused(format!(
            "`{id}` is the root: the surface is closed, not transformed"
        )));
    };
    match last {
        Step::Child(i) => Ok((up.to_vec(), *i)),
        Step::Item => {
            let list = tree::at(d.root(), up).map_or_else(|| id.to_string(), |n| n.sem.to_string());
            Err(refused(format!(
                "`{id}` is the item of the repeat of `{list}`: it makes every instance, so it \
                 goes with the list"
            )))
        }
    }
}

/// `Hide`: the node out of the tree, set aside with its parent and its
/// place. `at` is the place to keep (the inverse of a `Show` keeps the one
/// it had); its index by default. its inverse is `Show`.
pub(super) fn hide(
    d: &mut Draft,
    id: &SemId,
    at: Option<usize>,
) -> Result<Inverse, TransformError> {
    let Some(path) = tree::path(d.root(), id) else {
        if d.aside.contains_key(id) {
            return Err(refused(format!("`{id}` is hidden already")));
        }
        return Err(d.missing(id));
    };
    let (up, i) = slot(d, id, &path)?;
    let parent = tree::at_mut(&mut d.ir.surface.root, &up)
        .ok_or_else(|| TransformError::NotFound(id.clone()))?;
    let node = parent.children.remove(i);
    let kept = Aside {
        node,
        parent: parent.sem.clone(),
        at: at.unwrap_or(i),
    };
    d.aside.insert(id.clone(), kept);
    Ok(Inverse::Op(Op::Show { id: id.clone() }))
}

/// `Show`: a node set aside back in its parent, at its place (at the end
/// when the parent has fewer children now). its inverse sets it aside
/// again at the place it was kept with.
pub(super) fn show(d: &mut Draft, id: &SemId) -> Result<Inverse, TransformError> {
    if tree::find(d.root(), id).is_some() {
        return Err(refused(format!("`{id}` is not hidden")));
    }
    let Some(kept) = d.aside.get(id) else {
        if let Some(root) = d.aside_root(id) {
            return Err(refused(format!(
                "`{id}` is inside the hidden `{root}`: show `{root}`"
            )));
        }
        return Err(d.missing(id));
    };
    let (parent, at) = (kept.parent.clone(), kept.at);
    let Some(up) = tree::path(d.root(), &parent) else {
        let why = match d.aside_root(&parent) {
            Some(root) => format!("its place `{parent}` is hidden: show `{root}` first"),
            None => format!("its place `{parent}` is no longer a node of the surface"),
        };
        return Err(refused(why));
    };
    let Some(kept) = d.aside.remove(id) else {
        return Err(TransformError::NotFound(id.clone()));
    };
    let place = tree::at_mut(&mut d.ir.surface.root, &up)
        .ok_or_else(|| TransformError::NotFound(parent.clone()))?;
    if place.repeat.is_some() {
        return Err(refused(format!(
            "its place `{parent}` repeats a template: it holds no children of its own"
        )));
    }
    let i = at.min(place.children.len());
    place.children.insert(i, kept.node);
    Ok(Inverse::Aside { id: id.clone(), at })
}

/// the ids named twice in `ids`, refused.
fn once(ids: &[SemId]) -> Result<(), TransformError> {
    let mut seen = BTreeSet::new();
    match ids.iter().find(|id| !seen.insert(*id)) {
        Some(twice) => Err(refused(format!("`{twice}` is named twice"))),
        None => Ok(()),
    }
}

/// the path of the one parent every node of `ids` is a child of, and
/// their indices there in order.
fn siblings(d: &Draft, ids: &[SemId]) -> Result<(Vec<Step>, Vec<usize>), TransformError> {
    let mut parent: Option<Vec<Step>> = None;
    let mut at = Vec::new();
    for id in ids {
        let path = tree::path(d.root(), id).ok_or_else(|| d.missing(id))?;
        let (up, i) = slot(d, id, &path)?;
        match &parent {
            Some(p) if *p != up => {
                return Err(refused(format!(
                    "`{id}` is not a sibling of `{}`: a group takes one parent's children",
                    ids[0]
                )));
            }
            _ => parent = Some(up),
        }
        at.push(i);
    }
    at.sort_unstable();
    let parent = parent.ok_or_else(|| refused("a group takes one node at least"))?;
    Ok((parent, at))
}

/// `Group`: the siblings `ids` become the children of the new container
/// `into`, in their order, in the place of the first of them; the group
/// lays them out as their parent did when it is a stack. its inverse is
/// the parent before.
pub(super) fn group(d: &mut Draft, ids: &[SemId], into: &SemId) -> Result<Inverse, TransformError> {
    once(ids)?;
    d.free(into)?;
    let (up, at) = siblings(d, ids)?;
    let parent = tree::at_mut(&mut d.ir.surface.root, &up)
        .ok_or_else(|| TransformError::NotFound(into.clone()))?;
    let before = parent.clone();
    let mut group = IrNode::new(into.clone());
    if let Some(stack @ Layout::Stack { .. }) = parent.layout {
        group.layout = Some(stack);
    }
    for &i in at.iter().rev() {
        group.children.insert(0, parent.children.remove(i));
    }
    parent.children.insert(at[0], group);
    Ok(Inverse::Restore(before))
}

/// `Reorder`: `ids` first, in their order, then the other children of
/// `parent` as they were. its inverse is the whole order before.
pub(super) fn reorder(
    d: &mut Draft,
    parent: &SemId,
    ids: &[SemId],
) -> Result<Inverse, TransformError> {
    once(ids)?;
    let node = d.node_mut(parent)?;
    if let Some(repeat) = &node.repeat {
        return Err(refused(format!(
            "`{parent}` repeats a template: its order is the order of `{}`, the app's",
            repeat.each
        )));
    }
    let before: Vec<SemId> = node.children.iter().map(|c| c.sem.clone()).collect();
    if ids.is_empty() {
        return Err(refused("a reorder names one child at least"));
    }
    if let Some(stranger) = ids.iter().find(|id| !before.contains(id)) {
        return Err(refused(format!(
            "`{stranger}` is not a child of `{parent}`"
        )));
    }
    let mut rest = std::mem::take(&mut node.children);
    let mut first = Vec::with_capacity(rest.len());
    for id in ids {
        if let Some(i) = rest.iter().position(|c| c.sem == *id) {
            first.push(rest.remove(i));
        }
    }
    first.append(&mut rest);
    node.children = first;
    let after: Vec<&SemId> = node.children.iter().map(|c| &c.sem).collect();
    if after.iter().copied().eq(before.iter()) {
        return Err(refused(format!(
            "the children of `{parent}` are in that order already"
        )));
    }
    Ok(Inverse::Op(Op::Reorder {
        parent: parent.clone(),
        ids: before,
    }))
}
