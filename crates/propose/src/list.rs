//! the lists of an app, found by structure: a `List` or a `Set` state of
//! a node whose values are the keys of that node's children of one kind
//! (`<prefix>[<value>]`), all of them and nothing else. over the showcase
//! todo that is `todo:items` and its `todo.item[<id>]`; over the ide's
//! workbench, `changes:files` and its `changes.file[<path>]` (the
//! `changes:staged` set holds only some of those children: a selection of
//! the items, not their list).
//!
//! each item shows and accepts what every item declares alike, the way
//! the verifier checks a template: a text that is a frame ref (a media
//! node and an instant, `parts::is_frame_ref`) is an image; any other
//! `text`, `int` or `num` state is a text; a bool with an action of no
//! parameter that reads that bool and nothing else is a checkbox bound to
//! both; every other action of no parameter the author may ask for, and
//! not destructive, is a button.

use std::collections::{BTreeMap, BTreeSet};

use sem::{ActionDecl, Node, SemId, Snapshot, StateRef, Value, ValueKind};
use uir::{Capabilities, CapabilityRef, ITEM};

use crate::words::{fold, split};

/// a list of the app's items.
#[derive(Clone, Debug)]
pub(crate) struct List {
    /// the state it is: `todo:items`.
    pub each: StateRef,
    /// an item's id with the item in its key's place: `todo.item[{item}]`.
    pub item: SemId,
    /// the state's key and the items' name.
    pub words: BTreeSet<String>,
    /// the frames each item shows (its frame ref states), before the texts.
    pub frames: Vec<StateRef>,
    /// what each item shows as text.
    pub texts: Vec<StateRef>,
    /// each bool an item flips, and the action that flips it.
    pub checks: Vec<(StateRef, CapabilityRef)>,
    /// every other action an item takes with no parameter, with its label.
    pub buttons: Vec<(CapabilityRef, String)>,
}

/// the prefix and the key (unescaped) of an id that ends with a key
/// segment: `todo.item[3]` is `("todo.item", "3")`.
fn last_key(id: &str) -> Option<(&str, String)> {
    let mut open = None;
    let mut escaped = false;
    let mut last = None;
    for (at, c) in id.char_indices() {
        match open {
            None if c == '[' => open = Some(at),
            None => {}
            Some(_) if escaped => escaped = false,
            Some(_) if c == '\\' => escaped = true,
            Some(start) if c == ']' => {
                last = Some((start, at));
                open = None;
            }
            Some(_) => {}
        }
    }
    let (start, end) = last?;
    if start == 0 || end + 1 != id.len() {
        return None;
    }
    let mut key = String::new();
    let mut escaped = false;
    for c in id.get(start + 1..end)?.chars() {
        if !escaped && c == '\\' {
            escaped = true;
        } else {
            key.push(c);
            escaped = false;
        }
    }
    Some((id.get(..start)?, key))
}

/// every list of `app` whose items show or accept something; an action
/// counts when `usable` says the author may bind it; a text `frame` says
/// is a frame ref shows as an image.
pub(crate) fn lists(
    caps: &Capabilities,
    app: &Snapshot,
    usable: &impl Fn(&Node, &ActionDecl) -> bool,
    frame: &impl Fn(&Value) -> bool,
) -> Vec<List> {
    let mut out = Vec::new();
    for node in app.nodes().values() {
        if node.constraints.secret {
            continue;
        }
        let mut kinds: BTreeMap<&str, BTreeMap<String, &Node>> = BTreeMap::new();
        for child in app.children(&node.id) {
            if let Some((prefix, key)) = last_key(child.id.as_str()) {
                kinds.entry(prefix).or_default().insert(key, child);
            }
        }
        for (key, value) in &node.state {
            let values: Vec<&str> = match value {
                Value::List(v) => v.iter().map(String::as_str).collect(),
                Value::Set(s) => s.iter().map(String::as_str).collect(),
                _ => continue,
            };
            let each = StateRef::new(node.id.clone(), key.clone());
            if values.is_empty() || caps.datum(&each).is_none() {
                continue;
            }
            let wanted: BTreeSet<&str> = values.iter().copied().collect();
            for (prefix, children) in &kinds {
                let have: BTreeSet<&str> = children.keys().map(String::as_str).collect();
                if have != wanted {
                    continue;
                }
                let Ok(item) = SemId::parse(prefix).map(|p| p.key(ITEM)) else {
                    continue;
                };
                let members: Vec<&Node> = values
                    .iter()
                    .filter_map(|v| children.get(*v).copied())
                    .collect();
                let list = List::of(each.clone(), item, &members, caps, usable, frame);
                let shows = !list.frames.is_empty() || !list.texts.is_empty();
                if shows || !list.checks.is_empty() || !list.buttons.is_empty() {
                    out.push(list);
                }
            }
        }
    }
    out
}

impl List {
    /// the ids of the items the list holds now, in its order: the ones a
    /// template over it names.
    pub fn members(&self, app: &Snapshot) -> Vec<SemId> {
        let prefix = self
            .item
            .as_str()
            .strip_suffix("[{item}]")
            .unwrap_or_default();
        let Ok(prefix) = SemId::parse(prefix) else {
            return Vec::new();
        };
        let values: Vec<&str> = match app.value(&self.each) {
            Some(Value::List(v)) => v.iter().map(String::as_str).collect(),
            Some(Value::Set(s)) => s.iter().map(String::as_str).collect(),
            _ => Vec::new(),
        };
        values.into_iter().map(|v| prefix.key(v)).collect()
    }

    fn of(
        each: StateRef,
        item: SemId,
        members: &[&Node],
        caps: &Capabilities,
        usable: &impl Fn(&Node, &ActionDecl) -> bool,
        frame: &impl Fn(&Value) -> bool,
    ) -> List {
        let slot = |key: &sem::StateKey| StateRef::new(item.clone(), key.clone());
        let cap = |decl: &ActionDecl| CapabilityRef {
            target: item.clone(),
            action: decl.name.clone(),
        };
        // an action every item declares alike, which the author may bind
        let alike = |decl: &ActionDecl| {
            members.iter().all(|m| {
                m.actions.iter().any(|b| {
                    let same = b.name == decl.name && b.param == decl.param;
                    same && b.policy == decl.policy && usable(m, b)
                })
            })
        };
        let first = members.first();
        let bare: Vec<&ActionDecl> = first
            .map(|n| {
                n.actions
                    .iter()
                    .filter(|a| a.param.is_none() && alike(a))
                    .collect()
            })
            .unwrap_or_default();
        // the action of every item that reads `key` of it and nothing else
        let flip = |key: &sem::StateKey, taken: &BTreeSet<sem::ActionName>| {
            bare.iter().copied().find(|a| {
                !taken.contains(&a.name)
                    && members.iter().all(|m| {
                        let own = StateRef::new(m.id.clone(), key.clone());
                        let decl = m.actions.iter().find(|b| b.name == a.name);
                        decl.is_some_and(|b| b.reads == [own])
                    })
            })
        };
        let mut list = List {
            words: split(each.key.as_str()).map(fold).collect(),
            frames: Vec::new(),
            texts: Vec::new(),
            checks: Vec::new(),
            buttons: Vec::new(),
            each,
            item: item.clone(),
        };
        // the items' own name: the last name before the item's key
        let prefix = item.as_str().strip_suffix("[{item}]").unwrap_or_default();
        let name = prefix.rsplit('.').next().unwrap_or_default();
        let name = name.split('[').next().unwrap_or_default();
        list.words.extend(split(name).map(fold));
        let Some(first) = first else {
            return list;
        };
        let mut flips = BTreeSet::new();
        for (key, value) in &first.state {
            let kind = value.kind();
            let shared = members.iter().all(|m| {
                let at = StateRef::new(m.id.clone(), key.clone());
                m.state.get(key).is_some_and(|v| v.kind() == kind) && caps.datum(&at).is_some()
            });
            if !shared {
                continue;
            }
            match kind {
                ValueKind::Text if frame(value) => list.frames.push(slot(key)),
                ValueKind::Text | ValueKind::Int | ValueKind::Num => list.texts.push(slot(key)),
                ValueKind::Bool => {
                    if let Some(a) = flip(key, &flips) {
                        flips.insert(a.name.clone());
                        list.checks.push((slot(key), cap(a)));
                    }
                }
                ValueKind::List | ValueKind::Set => {}
            }
        }
        for a in bare.into_iter().filter(|a| !flips.contains(&a.name)) {
            let label = a
                .label
                .clone()
                .unwrap_or_else(|| a.name.as_str().replace('_', " "));
            list.buttons.push((cap(a), label));
        }
        list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_last_key_of_an_id_is_its_item() {
        assert_eq!(
            last_key("todo.item[3]"),
            Some(("todo.item", "3".to_owned()))
        );
        assert_eq!(
            last_key("changes.file[src/a\\]b.rs]"),
            Some(("changes.file", "src/a]b.rs".to_owned()))
        );
        assert_eq!(last_key("a[x].b[y]"), Some(("a[x].b", "y".to_owned())));
        assert_eq!(last_key("a[x].b"), None);
        assert_eq!(last_key("todo"), None);
    }
}
