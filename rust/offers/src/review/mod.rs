//! the review of items (task-96): "review these three commits", "revise
//! estes três commits".
//!
//! a word of review ("review", "revisar", "revise", "revisa", "revisão")
//! takes the intent down this path, before the grammar of the other
//! parts, which it leaves as it was: every intent with such a word was a
//! `NoMatch` before. the rest of the intent is read here: a number counts
//! the items ("three", "três", "3"; without one, all of them), a word of
//! order says where they are taken from, which is the top ("last",
//! "recent", "últimos"), the fillers of the grammar are skipped, and every
//! other word must name the items.
//!
//! the items, found by structure: the children of one node whose ids end
//! in a key under one prefix (`stacks.commit[<sha>]`,
//! `changes.file[<path>]`), in projection order, named by the item's name
//! (`commit`) and by their parent's label and name (`stacks`). there is no
//! list state to repeat over, and a review of "these three" is of these
//! three: each one is a card of its own, fixed on the surface, never a
//! repeat that would follow the list as it moves.
//!
//! each item is a card with what the registry holds of it: a head with
//! the text that names it and, on the same line, a button per action with
//! no parameter that the author may ask for and that is not destructive
//! (a commit's `show_diff`); then its other texts (a frame ref as an
//! image), its lists as a table each (a commit's files) and its numbers
//! as a stat each. what those actions read outside the item (the
//! `diff:target` a `show_diff` reads) is the region they open, a card
//! shown once beside the items: its texts, its numbers, and last its
//! lists, whose height the token `dropdown_max_h` bounds, which scroll
//! inside it. the items on the left and the region on the right, in a
//! split; the items alone in a column when there is no region.
//!
//! a text of the region those actions read, holding the id of a node of
//! the app or nothing (`diff:target`, `stacks.commit[<sha>]`), says which
//! item the region shows in the dispatcher's words, not a person's: it is
//! not shown (task-96, phase b). a region with no text of its own opens
//! with its label ("diff"), in the typography of what names a card, and
//! binding nothing; a new form keeps the id the current surface gave it.
//!
//! a list a node names `lines` is a `code` (task-97), not a table: the
//! name a code shows its lines by. when the node also has a list named
//! `kinds`, the name a code shows the kind of each line by, it is bound
//! to the same code and not shown on its own: the diff's lines, each
//! with its kind, in one block. the app that projects them speaks the
//! vocabulary; every other list stays a table.
//!
//! the refusals, each with why: a word that names no item of the app
//! (`NoMatch`, with the words of the items), nothing to review, no word
//! that says which items when the app has more than one kind, fewer items
//! than the number asked for (`Review`).

use std::collections::{BTreeMap, BTreeSet};

use accord::{ActionDecl, Node, SemId, Snapshot, StateKey, StateRef, Value, ValueKind};
use plevir::{
    Align, Axis, CapabilityRef, Component, ControlSize, Elevation, Ir, IrNode, Layout, Props, Size,
    SizeToken, SpacingToken, Surface, TypographyRole, VerifiedIr,
};

use crate::Ask;
use crate::error::ProposeError;
use crate::ids::{Anchor, Namer, anchors};
use crate::list::last_key;
use crate::parts::{allows, is_frame_ref};
use crate::rules::stack;
use crate::words::{fold, folded, is, is_filler, split};

/// the words that ask for a review, english then portuguese, as written.
const REVIEW: &[&str] = &[
    "review",
    "reviewing",
    "revisar",
    "revise",
    "revisa",
    "revisão",
    "rever",
    "reveja",
];

/// the words of where a review takes its items from: the top, as the app
/// projects them.
const ORDER: &[&str] = &[
    "last", "latest", "recent", "newest", "top", "first", "ultimo", "ultima", "recente",
    "primeiro", "primeira",
];

/// the numbers a review counts its items by, in words, both languages.
const NUMBERS: [&[&str]; 10] = [
    &["one", "um", "uma"],
    &["two", "dois", "duas"],
    &["three", "três"],
    &["four", "quatro"],
    &["five", "cinco"],
    &["six", "seis"],
    &["seven", "sete"],
    &["eight", "oito"],
    &["nine", "nove"],
    &["ten", "dez"],
];

/// a review, as the intent asks for it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Review {
    /// how many items, from the top; all of them when `None`.
    pub count: Option<usize>,
    /// the words left to name the items: (as written, folded).
    pub words: Vec<(String, String)>,
}

/// the number `word` (folded) says, in digits or in words.
fn number(word: &str) -> Option<usize> {
    if let Ok(n) = word.parse::<usize>() {
        return Some(n);
    }
    NUMBERS
        .iter()
        .position(|names| is(word, names))
        .map(|at| at + 1)
}

/// the review `intent` asks for; `None` when no word of it asks for one,
/// and the rules read it as they always did.
pub(crate) fn read(intent: &str) -> Option<Review> {
    let mut asked = false;
    let mut review = Review::default();
    for written in split(intent) {
        let word = fold(written);
        if is(&word, REVIEW) {
            asked = true;
        } else if let Some(n) = number(&word) {
            review.count.get_or_insert(n);
        } else if !is(&word, ORDER) && !is_filler(&word) {
            review.words.push((written.to_owned(), word));
        }
    }
    asked.then_some(review)
}

/// items of one kind: the children of one node keyed under one prefix.
struct Kind<'a> {
    parent: &'a Node,
    /// the items' own name, the last name of the prefix: `commit`.
    name: String,
    /// the item's name and the parent's label and name, folded.
    words: BTreeSet<String>,
    members: Vec<&'a Node>,
}

/// the last name of `id`, without its keys: `commit` of `stacks.commit`.
fn last_name(id: &str) -> &str {
    let bare = id.split('[').next().unwrap_or(id);
    bare.rsplit('.').next().unwrap_or(bare)
}

/// every kind of item of `app`, in projection order.
fn kinds(app: &Snapshot) -> Vec<Kind<'_>> {
    let mut out = Vec::new();
    for parent in app.nodes().values() {
        if parent.constraints.secret {
            continue;
        }
        let mut by_prefix: BTreeMap<&str, Vec<&Node>> = BTreeMap::new();
        let mut order = Vec::new();
        for child in app.children(&parent.id) {
            if let Some((prefix, _)) = last_key(child.id.as_str()) {
                if !by_prefix.contains_key(prefix) {
                    order.push(prefix);
                }
                by_prefix.entry(prefix).or_default().push(child);
            }
        }
        for prefix in order {
            let members = by_prefix.remove(prefix).unwrap_or_default();
            let name = fold(last_name(prefix));
            let mut words = folded(&parent.label);
            words.extend(split(last_name(parent.id.as_str())).map(fold));
            words.insert(name.clone());
            out.push(Kind {
                parent,
                name,
                words,
                members,
            });
        }
    }
    out
}

/// the kind the words name, or why there is none.
fn chosen<'a>(review: &Review, kinds: Vec<Kind<'a>>) -> Result<Kind<'a>, ProposeError> {
    if kinds.is_empty() {
        return Err(ProposeError::Review(
            "there is nothing to review here: no items of one kind under one node".to_owned(),
        ));
    }
    if review.words.is_empty() {
        let names: Vec<String> = kinds.iter().map(|k| k.name.clone()).collect();
        return match <[Kind; 1]>::try_from(kinds) {
            Ok([only]) => Ok(only),
            Err(_) => Err(ProposeError::Review(format!(
                "say which items to review: {}",
                names.join(", ")
            ))),
        };
    }
    let every: BTreeSet<&String> = kinds.iter().flat_map(|k| &k.words).collect();
    let unknown: Vec<String> = review
        .words
        .iter()
        .filter(|(_, w)| !every.contains(w))
        .map(|(written, _)| written.clone())
        .collect();
    let app: Vec<String> = every.into_iter().cloned().collect();
    let named = |k: &Kind| review.words.iter().all(|(_, w)| k.words.contains(w));
    match kinds.into_iter().find(named) {
        Some(kind) if unknown.is_empty() => Ok(kind),
        _ => {
            let words = if unknown.is_empty() {
                review.words.iter().map(|(w, _)| w.clone()).collect()
            } else {
                unknown
            };
            Err(ProposeError::NoMatch { words, app })
        }
    }
}

/// what a proposal of a review builds with.
struct Build<'a> {
    namer: Namer,
    max: usize,
    ask: &'a Ask<'a>,
}

/// the first `max` characters of `text`.
fn clip(text: &str, max: usize) -> String {
    text.chars().take(max).collect()
}

/// a state's key as a label: `cut_secs` reads "cut secs".
fn label(key: &str) -> String {
    key.replace('_', " ")
}

/// whether `value` names a node of `app` by its id, or nothing: what a
/// region holds to say which item it shows.
fn names_a_node(app: &Snapshot, value: &Value) -> bool {
    match value {
        Value::Text(text) => {
            text.is_empty() || SemId::parse(text).is_ok_and(|id| app.get(&id).is_some())
        }
        _ => false,
    }
}

/// the id of the text that opens the card of `region` on `current`: the
/// first child of the node whose children show `region`'s state, when it
/// is a text that binds nothing. such a node has no anchor and holds
/// nothing, so the namer never gives its id out (`ids.rs`): keeping it
/// cannot give one id twice.
fn kept_heading(current: Option<&Ir>, region: &SemId) -> Option<SemId> {
    let mut stack = vec![&current?.surface.root];
    while let Some(n) = stack.pop() {
        let shows = n
            .children
            .iter()
            .any(|c| c.bind.state.values().any(|at| at.node == *region));
        match n.children.first() {
            Some(first)
                if shows && first.component == Some(Component::Text) && first.bind.is_empty() =>
            {
                return Some(first.sem.clone());
            }
            _ => stack.extend(n.children.iter()),
        }
    }
    None
}

impl Build<'_> {
    /// a node bound to `anchor`, named `name` and labelled `text`.
    fn bound(&mut self, mut node: IrNode, anchor: Anchor, name: &str, text: String) -> IrNode {
        let (id, text) = self.namer.bound(&anchor, name, text);
        node.sem = id;
        if !text.is_empty() {
            node.label = Some(clip(&text, self.max));
        }
        node
    }

    fn container(&mut self, name: &str, label: Option<String>, children: Vec<IrNode>) -> IrNode {
        let mut held = BTreeSet::new();
        for c in &children {
            held.extend(anchors(c));
        }
        let label = label.map(|l| clip(&l, self.max));
        let (id, label) = self.namer.container(&held, name, label);
        let mut node = IrNode::new(id);
        node.label = label;
        node.children = children;
        node
    }

    /// what the registry holds of `node`'s state, in its order.
    fn data<'n>(&self, node: &'n Node) -> Vec<(&'n accord::StateKey, &'n Value)> {
        node.state
            .iter()
            .filter(|(key, _)| {
                let at = StateRef::new(node.id.clone(), (*key).clone());
                self.ask.caps.datum(&at).is_some()
            })
            .collect()
    }

    /// `node`'s state by sense: each text (the first larger, as what names
    /// it, and a frame ref as an image), each number as a stat, each list
    /// as a table, its `lines` as a code with their `kinds` (every list
    /// bounded in height when `bounded`). a text of `pointers` that names
    /// a node of the app, or nothing, is left out.
    fn senses(&mut self, node: &Node, bounded: bool, pointers: &BTreeSet<StateKey>) -> Senses {
        let mut out = Senses::default();
        let data = self.data(node);
        let list = |name: &str| {
            data.iter()
                .find(|(key, value)| key.as_str() == name && value.kind() == ValueKind::List)
                .map(|(key, _)| StateRef::new(node.id.clone(), (*key).clone()))
        };
        let lines = list("lines");
        let kinds = lines.as_ref().and_then(|_| list("kinds"));
        let bound = || Size {
            max: Some(SizeToken::DropdownMaxH),
            ..Size::default()
        };
        for (key, value) in data {
            let at = StateRef::new(node.id.clone(), key.clone());
            let name = format!("{}.{key}", node.id);
            let mut n = IrNode::new(node.id.clone());
            let anchor = Anchor::State(at.clone());
            match value.kind() {
                // which item the region shows, by its id: the dispatcher's
                ValueKind::Text if pointers.contains(key) && names_a_node(self.ask.app, value) => {}
                ValueKind::Text if is_frame_ref(self.ask.app, value) => {
                    n.component = Some(Component::Image);
                    n.props.control_size = Some(ControlSize::Xl);
                    n.bind.state.insert("source".to_owned(), at);
                    out.texts
                        .push(self.bound(n, anchor, &name, label(key.as_str())));
                }
                ValueKind::Text => {
                    n.component = Some(Component::Text);
                    let role = match out.texts.len() {
                        0 => TypographyRole::Base2sm,
                        _ => TypographyRole::CaptionR,
                    };
                    n.props.typography = Some(role);
                    n.bind.state.insert("text".to_owned(), at);
                    out.texts.push(self.bound(n, anchor, &name, String::new()));
                }
                // the kinds are the code's, never a list of their own
                ValueKind::List if kinds.as_ref() == Some(&at) => {}
                ValueKind::List if lines.as_ref() == Some(&at) => {
                    n.component = Some(Component::Code);
                    n.bind.state.insert("lines".to_owned(), at);
                    if let Some(kinds) = &kinds {
                        n.bind.state.insert("kinds".to_owned(), kinds.clone());
                    }
                    if bounded {
                        n.size = bound();
                    }
                    // anchored as the current surface's node is, by its
                    // first state, so a new form keeps its id
                    let anchor = crate::ids::anchor(&n).unwrap_or(anchor);
                    out.lists
                        .push(self.bound(n, anchor, &name, label(key.as_str())));
                }
                ValueKind::List | ValueKind::Set => {
                    n.component = Some(Component::Table);
                    n.bind.state.insert("rows".to_owned(), at);
                    if bounded {
                        n.size = bound();
                    }
                    out.lists
                        .push(self.bound(n, anchor, &name, label(key.as_str())));
                }
                ValueKind::Int | ValueKind::Num => {
                    n.component = Some(Component::Stat);
                    n.props.typography = Some(TypographyRole::BaseM);
                    n.bind.state.insert("value".to_owned(), at);
                    out.numbers
                        .push(self.bound(n, anchor, &name, label(key.as_str())));
                }
                ValueKind::Bool => {}
            }
        }
        out
    }

    /// whether the author may bind `decl` of `node` on a button: in the
    /// registry, no parameter, allowed to the author, not destructive.
    fn usable(&self, node: &Node, decl: &ActionDecl) -> bool {
        decl.param.is_none()
            && !decl.policy.destructive
            && allows(&decl.policy.actors, &self.ask.policy.author)
            && self.ask.caps.action(&node.id, &decl.name).is_some()
    }

    /// a small button per action of `node` the author may ask for.
    fn buttons(&mut self, node: &Node) -> Vec<IrNode> {
        let usable: Vec<&ActionDecl> = node
            .actions
            .iter()
            .filter(|d| self.usable(node, d))
            .collect();
        let mut out = Vec::new();
        for decl in usable {
            let cap = CapabilityRef {
                target: node.id.clone(),
                action: decl.name.clone(),
            };
            let mut button = IrNode::new(node.id.clone());
            button.component = Some(Component::Button);
            button.props.control_size = Some(ControlSize::Xs);
            button
                .bind
                .actions
                .insert("activate".to_owned(), cap.clone());
            let text = decl
                .label
                .clone()
                .unwrap_or_else(|| label(decl.name.as_str()));
            let name = format!("{}.{}", node.id, decl.name);
            out.push(self.bound(button, Anchor::Action(cap), &name, text));
        }
        out
    }

    /// one item as a card: a head with what names it and its actions on
    /// one line, then its other texts, its lists and its numbers.
    fn item(&mut self, node: &Node) -> IrNode {
        let Senses {
            mut texts,
            lists,
            numbers,
        } = self.senses(node, false, &BTreeSet::new());
        let mut head = Vec::new();
        if !texts.is_empty() {
            let mut name = texts.remove(0);
            name.size = Size {
                flex: Some(1),
                ..Size::default()
            };
            head.push(name);
        }
        head.extend(self.buttons(node));
        let mut children = Vec::new();
        match head.len() {
            0 => {}
            1 => children.extend(head),
            _ => {
                let name = format!("{}.head", node.id);
                let mut row = self.container(&name, None, head);
                row.layout = stack(Axis::Horizontal, SpacingToken::Sm, Some(Align::Center));
                children.push(row);
            }
        }
        children.extend(texts);
        children.extend(lists);
        children.extend(numbers);
        let mut card = self.container(node.id.as_str(), Some(node.label.clone()), children);
        card.component = Some(Component::Card);
        card.props.padding = Some(SpacingToken::Md);
        card.layout = stack(Axis::Vertical, SpacingToken::Xs, None);
        card
    }

    /// the region the items' actions read outside them (`read`, the keys
    /// they read of it), as a card: its texts, or its label when it has
    /// none of its own, its numbers, and its lists bounded in height, last.
    fn region(&mut self, node: &Node, read: &BTreeSet<StateKey>) -> IrNode {
        let Senses {
            mut texts,
            lists,
            numbers,
        } = self.senses(node, true, read);
        if texts.is_empty() {
            texts.push(self.heading(node));
        }
        let children = texts.into_iter().chain(numbers).chain(lists).collect();
        let mut card = self.container(node.id.as_str(), Some(node.label.clone()), children);
        card.component = Some(Component::Card);
        card.props.padding = Some(SpacingToken::Md);
        card.layout = stack(Axis::Vertical, SpacingToken::Sm, None);
        card
    }

    /// what opens the card of a region with no text of its own: its label,
    /// in the typography of what names a card, binding nothing. the id the
    /// current surface gave it, or one of its own.
    fn heading(&mut self, node: &Node) -> IrNode {
        let current = self.ask.current.map(VerifiedIr::ir);
        let id = kept_heading(current, &node.id).unwrap_or_else(|| {
            // it holds and binds nothing: a fresh id, as a container that
            // held nothing before takes one
            let name = format!("{}.heading", node.id);
            self.namer.container(&BTreeSet::new(), &name, None).0
        });
        let mut text = IrNode::new(id);
        text.component = Some(Component::Text);
        text.label = Some(clip(&node.label, self.max));
        text.props.typography = Some(TypographyRole::Base2sm);
        text
    }
}

/// a node's state as a review shows it, by sense.
#[derive(Default)]
struct Senses {
    texts: Vec<IrNode>,
    numbers: Vec<IrNode>,
    lists: Vec<IrNode>,
}

/// the nodes the actions of `members` read outside them, in the order
/// they are first read, each once: the region those actions open, with
/// the keys they read of it.
fn regions<'a>(
    app: &'a Snapshot,
    members: &[&Node],
    usable: impl Fn(&Node, &ActionDecl) -> bool,
) -> Vec<(&'a Node, BTreeSet<StateKey>)> {
    let own: BTreeSet<&SemId> = members.iter().map(|m| &m.id).collect();
    let mut out: Vec<(&Node, BTreeSet<StateKey>)> = Vec::new();
    for m in members {
        for decl in m.actions.iter().filter(|d| usable(m, d)) {
            for read in decl.reads.iter().filter(|r| !own.contains(&r.node)) {
                if let Some((_, keys)) = out.iter_mut().find(|(n, _)| n.id == read.node) {
                    keys.insert(read.key.clone());
                } else if let Some(node) = app.get(&read.node).filter(|n| !n.constraints.secret) {
                    out.push((node, BTreeSet::from([read.key.clone()])));
                }
            }
        }
    }
    out
}

/// the review `review` asks for over `ask`: a verified surface is still
/// `propose_verified`'s to decide.
pub(crate) fn propose(review: &Review, ask: &Ask<'_>) -> Result<Ir, ProposeError> {
    let kind = chosen(review, kinds(ask.app))?;
    let count = review.count.unwrap_or(kind.members.len());
    if count == 0 || count > kind.members.len() {
        return Err(ProposeError::Review(format!(
            "{count} {} asked for, and {} has {}",
            kind.name,
            kind.parent.label,
            kind.members.len()
        )));
    }
    let members: Vec<&Node> = kind.members.iter().take(count).copied().collect();
    let current = ask.current.map(|v| v.ir());
    let mut build = Build {
        namer: Namer::new(current, ask.app),
        max: ask.policy.max_text,
        ask,
    };
    let usable = |n: &Node, d: &ActionDecl| build.usable(n, d);
    let regions = regions(ask.app, &members, usable);
    let items: Vec<IrNode> = members.iter().map(|m| build.item(m)).collect();
    let name = last_name(kind.parent.id.as_str()).to_owned();
    let mut list = build.container(&name, Some(kind.parent.label.clone()), items);
    list.layout = stack(Axis::Vertical, SpacingToken::Md, None);
    let mut shown: Vec<IrNode> = regions
        .iter()
        .map(|(node, read)| build.region(node, read))
        .collect();
    let side = match shown.len() {
        0 => None,
        1 => shown.pop(),
        _ => {
            let mut all = build.container("regions", None, shown);
            all.layout = stack(Axis::Vertical, SpacingToken::Md, None);
            Some(all)
        }
    };
    let (children, layout) = match side {
        None => (vec![list], stack(Axis::Vertical, SpacingToken::Md, None)),
        Some(side) => {
            let split = Layout::Split {
                axis: Axis::Horizontal,
                ratio: 50,
            };
            (vec![list, side], Some(split))
        }
    };
    let (id, label) = build.namer.root(None);
    let mut root = IrNode::new(id);
    root.component = Some(Component::Card);
    root.label = label.map(|l| clip(&l, build.max));
    root.props = Props {
        padding: Some(SpacingToken::Lg),
        elevation: Some(Elevation::Raised),
        ..Props::default()
    };
    root.layout = layout;
    root.children = children;
    let title = match current {
        Some(ir) => ir.surface.title.clone(),
        None => clip(ask.intent.trim(), build.max),
    };
    Ok(Ir::new(Surface {
        id: build.namer.surface().clone(),
        title,
        root,
    }))
}

#[cfg(test)]
mod tests;
