//! the rules as a transformer (task-70 phase 2): a change read by the
//! second grammar (`change.rs`) turned into operations by the surface's
//! structure and the registry, never by an app's names.
//!
//! - **the task** is what binds an action, a node or anything under it (a
//!   repeat whole: the item is the task). **context** is a block that
//!   shows state and binds no action anywhere under it.
//! - **less**: every block of context is hidden, from the root down
//!   through the containers that hold the task, never inside a repeat,
//!   never a split's child, and nothing when the surface binds no action;
//!   then, in each container that runs readouts and two controls or more
//!   in one stack, the controls are grouped (`<container>.actions`, which
//!   in a template carries its `[{item}]`); then the spacing one step
//!   denser.
//! - **compare**: a table of a list whose values all read as numbers, and
//!   which is not the keys of its node's children (a list of items holds
//!   ids, not measures), becomes a bar chart; a chart becomes a table. a
//!   list that repeats a template never swaps (its items' ids would leave
//!   the surface). nothing that fits is refused, saying why.
//! - **bigger**: every stat, and every text bound to an `int` or a `num`,
//!   one step up (the ones the app's words name, when they name some).
//! - **undo**: the last change asked, which the host knows.
//!
//! each operation is tried over the surface as the ones before left it,
//! and kept only when it applies; none applied is an error that says why.

use std::collections::BTreeSet;

use sem::{SemId, StateRef, Value, ValueKind};
use uir::{Adapted, Capabilities, ChartKind, Component, IrNode, Layout, Op, Step};

use crate::adapt::{Adapt, Adaptation, Transformer};
use crate::change::{Asked, CHANGES, read};
use crate::error::ProposeError;
use crate::list::is_items;
use crate::parts::Parts;
use crate::rules::Rules;
use crate::words::{fold, folded, split};

/// operations tried one after the other over a surface: the ones that
/// apply are kept, each over what the ones before made.
struct Plan<'a> {
    ask: &'a Adapt<'a>,
    now: Adapted,
    ops: Vec<Op>,
}

impl Plan<'_> {
    fn try_op(&mut self, op: Op) -> bool {
        match self.now.apply(&op, self.ask.by, self.ask.context()) {
            Ok((next, _)) => {
                self.now = next;
                self.ops.push(op);
                true
            }
            Err(_) => false,
        }
    }

    fn root(&self) -> IrNode {
        self.now.ir().surface.root.clone()
    }
}

/// `n` and everything under it, templates included.
fn subtree(n: &IrNode) -> Vec<&IrNode> {
    let mut out = Vec::new();
    let mut stack = vec![n];
    while let Some(n) = stack.pop() {
        out.push(n);
        if let Some(r) = &n.repeat {
            stack.push(&r.item);
        }
        stack.extend(n.children.iter().rev());
    }
    out
}

/// something under `n` binds an action: `n` is of the task.
fn acts(n: &IrNode) -> bool {
    subtree(n).iter().any(|m| !m.bind.actions.is_empty())
}

/// a block of context: it shows state and binds no action.
fn context(n: &IrNode) -> bool {
    !acts(n) && subtree(n).iter().any(|m| !m.bind.state.is_empty())
}

/// the blocks of context under `node`, down through what holds the task.
fn context_blocks(node: &IrNode, out: &mut Vec<SemId>) {
    let split = matches!(node.layout, Some(Layout::Split { .. }));
    for c in &node.children {
        if context(c) && !split {
            out.push(c.sem.clone());
        } else if acts(c) && c.repeat.is_none() {
            context_blocks(c, out);
        }
    }
}

/// a stack (or no layout, which stacks): the one a group copies.
fn stacks(node: &IrNode) -> bool {
    matches!(node.layout, None | Some(Layout::Stack { .. }))
}

/// the containers that run readouts and two controls or more, with the
/// controls to group.
fn mixed(node: &IrNode, out: &mut Vec<(SemId, Vec<SemId>)>) {
    let leaf = |c: &&IrNode| c.children.is_empty() && c.repeat.is_none();
    let controls: Vec<SemId> = node
        .children
        .iter()
        .filter(|c| leaf(c) && !c.bind.actions.is_empty())
        .map(|c| c.sem.clone())
        .collect();
    let readouts = node
        .children
        .iter()
        .filter(|c| leaf(c) && c.bind.actions.is_empty() && !c.bind.state.is_empty())
        .count();
    if controls.len() >= 2 && readouts >= 1 && stacks(node) && node.component.is_none() {
        out.push((node.sem.clone(), controls));
    }
    for c in &node.children {
        mixed(c, out);
    }
    if let Some(r) = &node.repeat {
        mixed(&r.item, out);
    }
}

/// a free id for a group of `parent`'s controls.
fn group_id(parent: &SemId, taken: &BTreeSet<SemId>) -> Option<SemId> {
    (1..100).find_map(|n| {
        let name = match n {
            1 => format!("{parent}.actions"),
            n => format!("{parent}.actions_{n}"),
        };
        SemId::parse(&name).ok().filter(|id| !taken.contains(id))
    })
}

/// the kind of the state `at` holds; a template's reference (with
/// `[{item}]`) reads the kind of the first item the registry has.
fn kind_of(caps: &Capabilities, at: &StateRef) -> Option<ValueKind> {
    if let Some(d) = caps.datum(at) {
        return Some(d.kind);
    }
    let text = at.node.as_str();
    let (pre, post) = text.split_once("[{item}]")?;
    caps.data()
        .iter()
        .find(|d| {
            let node = d.at.node.as_str();
            d.at.key == at.key
                && node.len() > pre.len() + post.len() + 2
                && node.starts_with(&format!("{pre}["))
                && node.ends_with(&format!("]{post}"))
        })
        .map(|d| d.kind)
}

/// the words a node of the surface goes by: its label, its last name,
/// the keys it binds.
fn node_words(n: &IrNode) -> BTreeSet<String> {
    let mut out = n.label.as_deref().map(folded).unwrap_or_default();
    let bare = n.sem.as_str().split('[').next().unwrap_or_default();
    out.extend(split(bare.rsplit('.').next().unwrap_or_default()).map(fold));
    for at in n.bind.state.values() {
        out.extend(split(at.key.as_str()).map(fold));
    }
    out
}

/// true when `words` is empty or one of them is a word of `n`.
fn named(words: &[(String, String)], n: &IrNode) -> bool {
    let own = node_words(n);
    words.is_empty() || words.iter().any(|(_, w)| own.contains(w))
}

fn less(plan: &mut Plan<'_>) -> Result<(), String> {
    let start = plan.ops.len();
    let root = plan.root();
    let mut hide = Vec::new();
    if acts(&root) {
        context_blocks(&root, &mut hide);
    }
    for id in hide {
        plan.try_op(Op::Hide { id });
    }
    let mut runs = Vec::new();
    mixed(&plan.root(), &mut runs);
    for (parent, ids) in runs {
        if let Some(into) = group_id(&parent, &plan.now.ids()) {
            plan.try_op(Op::Group { ids, into });
        }
    }
    plan.try_op(Op::Density { step: Step::Denser });
    if plan.ops.len() == start {
        return Err(
            "nothing to take away: no block here shows only context, no row mixes readouts \
             with controls, and every spacing is as dense as it gets"
                .to_owned(),
        );
    }
    Ok(())
}

/// a list the app shows whose values all read as numbers and which is
/// not a list of items.
fn numbers(ask: &Adapt<'_>, at: &StateRef) -> bool {
    let values: Vec<&String> = match ask.app.value(at) {
        Some(Value::List(v)) => v.iter().collect(),
        _ => return false,
    };
    !values.is_empty()
        && values.iter().all(|v| v.trim().parse::<f64>().is_ok())
        && !is_items(ask.app, at)
}

fn compare(plan: &mut Plan<'_>, words: &[(String, String)]) -> Result<(), String> {
    let root = plan.root();
    let mut swaps = Vec::new();
    let mut repeats = Vec::new();
    for n in subtree(&root) {
        if let Some(r) = &n.repeat {
            repeats.push(format!("`{}` repeats its items over `{}`", n.sem, r.each));
        }
        let to = match (
            n.component,
            n.bind.state.get("rows"),
            n.bind.state.get("data"),
        ) {
            (Some(Component::Table), Some(at), _) if numbers(plan.ask, at) => {
                Component::Chart(ChartKind::Bar)
            }
            (Some(Component::Chart(_)), _, Some(_)) => Component::Table,
            _ => continue,
        };
        if named(words, n) {
            swaps.push(Op::SwapView {
                id: n.sem.clone(),
                component: to,
            });
        }
    }
    let mut swapped = false;
    for op in swaps {
        swapped |= plan.try_op(op);
    }
    if swapped {
        return Ok(());
    }
    let why = if repeats.is_empty() {
        "the surface shows no list of numbers".to_owned()
    } else {
        format!(
            "{} (as a table or a chart its items' ids would leave the surface), and no list of \
             numbers is shown: each item's numbers are values of their own",
            repeats.join("; ")
        )
    };
    Err(format!(
        "nothing here can be swapped for a table or a chart to compare: {why}"
    ))
}

fn bigger(plan: &mut Plan<'_>, words: &[(String, String)]) -> Result<(), String> {
    let root = plan.root();
    let caps = plan.ask.caps;
    let number = |at: &StateRef| matches!(kind_of(caps, at), Some(ValueKind::Int | ValueKind::Num));
    let targets: Vec<&IrNode> = subtree(&root)
        .into_iter()
        .filter(|n| match n.component {
            Some(Component::Stat) => true,
            Some(Component::Text) => n.bind.state.get("text").is_some_and(number),
            _ => false,
        })
        .collect();
    if targets.is_empty() {
        return Err("the surface shows no number to make bigger".to_owned());
    }
    let chosen: Vec<SemId> = targets
        .iter()
        .filter(|n| named(words, n))
        .map(|n| n.sem.clone())
        .collect();
    if chosen.is_empty() {
        return Err("no number here goes by those words".to_owned());
    }
    let mut grew = false;
    for id in chosen {
        grew |= plan.try_op(Op::Emphasize { id });
    }
    if grew {
        Ok(())
    } else {
        Err("every number here is as big as it gets".to_owned())
    }
}

/// every word the app and the surface shown go by.
fn known(ask: &Adapt<'_>) -> BTreeSet<String> {
    let mut all: BTreeSet<String> = Parts::of(ask.caps, ask.app, &ask.policy.author)
        .words()
        .into_iter()
        .collect();
    for n in subtree(&ask.current.ir().surface.root) {
        all.extend(node_words(n));
    }
    all
}

impl Transformer for Rules {
    fn transform(&mut self, ask: &Adapt<'_>) -> Result<Adaptation, ProposeError> {
        let asked: Asked = read(ask.intent);
        let not = || ProposeError::NotAChange {
            intent: ask.intent.trim().to_owned(),
            changes: CHANGES.to_owned(),
        };
        if !asked.is_change() {
            return Err(not());
        }
        let known = known(ask);
        if asked.words.iter().any(|(_, w)| !known.contains(w)) {
            return Err(not());
        }
        if asked.undo {
            return Ok(Adaptation::Undo);
        }
        let mut plan = Plan {
            ask,
            now: ask.current.clone(),
            ops: Vec::new(),
        };
        let mut why = Vec::new();
        if asked.less {
            why.extend(less(&mut plan).err());
        }
        if asked.compare {
            why.extend(compare(&mut plan, &asked.words).err());
        }
        if asked.bigger {
            why.extend(bigger(&mut plan, &asked.words).err());
        }
        if plan.ops.is_empty() {
            return Err(ProposeError::NoChange(why.join("; ")));
        }
        Ok(Adaptation::Ops(plan.ops))
    }
}
