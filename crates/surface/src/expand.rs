//! a verified surface over the live state of the app: every repeat
//! unrolled into one instance per item, each instance with the item in
//! the place of `[{item}]` in its ids and in its bindings.
//!
//! the tree is flat, in pre-order, the root at index 0: layout, drawing,
//! events and the projection all walk it in that order, so the order a
//! person sees, the focus order and the projection order are one.

use std::collections::BTreeSet;

use sem::{SemId, Snapshot, StateRef, Value};
use uir::{Bind, CapabilityRef, Component, ITEM, IrNode, Layout, Props, Size};

/// one node of the unrolled tree: the IR node, or one instance of a
/// template, with its bindings resolved to the item.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Node {
    pub id: SemId,
    pub component: Option<Component>,
    pub label: Option<String>,
    pub description: Option<String>,
    pub props: Props,
    pub layout: Option<Layout>,
    pub size: Size,
    /// the bindings with the item in place: what the node shows and the
    /// capability each event asks for.
    pub bind: Bind,
    /// the item an instance root is named by; `None` everywhere else.
    pub item: Option<String>,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
}

impl Node {
    /// the component's state `prop` as the app holds it in `shown`.
    pub fn value<'a>(&self, prop: &str, shown: &'a Snapshot) -> Option<&'a Value> {
        self.bind.state.get(prop).and_then(|at| shown.value(at))
    }
}

/// the tree `root` makes over `app`: each repeat runs over the live value
/// of its `List` or `Set`. a `List` that holds an item twice gives one
/// instance, the first (two would share an id); a value that is gone or
/// of another kind gives none.
pub(crate) fn expand(root: &IrNode, app: &Snapshot) -> Vec<Node> {
    let mut out = Vec::new();
    push(root, None, None, app, &mut out);
    out
}

/// the items a repeat runs over, in order and once each.
pub(crate) fn items(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::List(list)) => {
            let mut seen = BTreeSet::new();
            list.iter()
                .filter(|item| seen.insert(item.as_str()))
                .cloned()
                .collect()
        }
        Some(Value::Set(set)) => set.iter().cloned().collect(),
        _ => Vec::new(),
    }
}

/// pushes `n` (and its subtree) under `parent`; the verifier bounds the
/// depth, so the recursion is bounded too.
fn push(
    n: &IrNode,
    parent: Option<usize>,
    item: Option<&str>,
    app: &Snapshot,
    out: &mut Vec<Node>,
) -> usize {
    let at = out.len();
    out.push(Node {
        id: instance(&n.sem, item),
        component: n.component,
        label: n.label.clone(),
        description: n.description.clone(),
        props: n.props,
        layout: n.layout,
        size: n.size,
        bind: resolve(&n.bind, item),
        item: None,
        parent,
        children: Vec::new(),
    });
    let mut children = Vec::new();
    for child in &n.children {
        children.push(push(child, Some(at), item, app, out));
    }
    if let Some(repeat) = &n.repeat {
        for key in items(app.value(&repeat.each)) {
            let child = push(&repeat.item, Some(at), Some(&key), app, out);
            if let Some(node) = out.get_mut(child) {
                node.item = Some(key);
            }
            children.push(child);
        }
    }
    if let Some(node) = out.get_mut(at) {
        node.children = children;
    }
    at
}

/// the bindings of a template node for one item.
fn resolve(bind: &Bind, item: Option<&str>) -> Bind {
    let mut out = Bind::default();
    for (prop, at) in &bind.state {
        let at = StateRef::new(instance(&at.node, item), at.key.clone());
        out.state.insert(prop.clone(), at);
    }
    for (event, cap) in &bind.actions {
        let cap = CapabilityRef {
            target: instance(&cap.target, item),
            action: cap.action.clone(),
        };
        out.actions.insert(event.clone(), cap);
    }
    out
}

/// `id` with its `[{item}]` segment keyed by `item`
/// (`review.file[{item}].stage` over `src/lib.rs` is
/// `review.file[src/lib.rs].stage`); `id` itself outside a template or
/// when it carries no item.
pub(crate) fn instance(id: &SemId, item: Option<&str>) -> SemId {
    let Some(item) = item else {
        return id.clone();
    };
    let text = id.as_str();
    let Some((start, end)) = keys(text)
        .into_iter()
        .find(|&(s, e)| text.get(s + 1..e - 1) == Some(ITEM))
    else {
        return id.clone();
    };
    let (Some(prefix), Some(suffix)) = (text.get(..start), text.get(end..)) else {
        return id.clone();
    };
    // the prefix is an id of its own (a key follows a segment); `key`
    // escapes the item the way the canonical form wants it.
    let keyed = match SemId::parse(prefix) {
        Ok(prefix) => prefix.key(item),
        Err(_) => return id.clone(),
    };
    SemId::parse(&format!("{keyed}{suffix}")).unwrap_or_else(|e| {
        log::warn!("surface: {id} over {item:?} is no id: {e}");
        id.clone()
    })
}

/// the byte ranges of the key segments of a canonical id, brackets
/// included. inside a key, `\` escapes the next character.
fn keys(id: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut open = None;
    let mut escaped = false;
    for (at, c) in id.char_indices() {
        match open {
            None if c == '[' => open = Some(at),
            None => {}
            Some(_) if escaped => escaped = false,
            Some(_) if c == '\\' => escaped = true,
            Some(start) if c == ']' => {
                out.push((start, at + 1));
                open = None;
            }
            Some(_) => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(text: &str) -> SemId {
        SemId::parse(text).unwrap()
    }

    #[test]
    fn an_instance_takes_the_item_in_the_place_of_the_slot() {
        let stage = id("review.file[{item}].stage");
        assert_eq!(
            instance(&stage, Some("src/lib.rs")).as_str(),
            "review.file[src/lib.rs].stage"
        );
        assert_eq!(
            instance(&id("changes.file[{item}]"), Some("a]b\\c")).as_str(),
            "changes.file[a\\]b\\\\c]"
        );
        assert_eq!(instance(&stage, None), stage);
        let fixed = id("review.message");
        assert_eq!(instance(&fixed, Some("x")), fixed);
    }

    #[test]
    fn a_list_repeats_each_item_once_in_order() {
        let list = Value::List(vec!["b".into(), "a".into(), "b".into()]);
        assert_eq!(items(Some(&list)), ["b", "a"]);
        let set = Value::Set(["z".to_owned(), "y".to_owned()].into());
        assert_eq!(items(Some(&set)), ["y", "z"]);
        assert!(items(Some(&Value::Text("a".into()))).is_empty());
        assert!(items(None).is_empty());
    }
}
