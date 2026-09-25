//! Every page of the gallery as a screen reader meets it (task-75 phase
//! 1): the tree the shell's bridge builds from the page's projection,
//! placed by one drawn frame, checked for what a reader needs. Every
//! control has a name and a role that takes focus; Tab and the reader
//! walk the controls in the same order, and the page takes the focus the
//! reader asks for; every action the tree offers reads back as a request
//! the dispatcher admits from assistive technology.
//!
//! Nothing here runs a request: the Files page asks the platform's own
//! dialogs and the Surface page a model, and a test opens neither. What
//! the dispatcher does with a request is `sem`'s, tested there and in the
//! todo page's own screen reader tests; here the tree and the contract
//! are held to each other.
//!
//! What the pages do not give a reader yet is listed, not fixed: the
//! pages that project no meaning (the component pages, task-75 phase 2b)
//! and what phase 2a left with its reason. Each list is exact, so a page
//! that starts projecting, or a new gap, shows here. Phase 2a emptied the
//! others: an item's actions its own node does not reach are controls the
//! tree presents inside it (`accessibility::present`), a select is
//! pressed open and set by its value, the pages project the focus they
//! take, and a press is offered only while its action is on.
#![cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]

use engine::accessibility::accesskit::{
    Action, ActionData, ActionRequest, Live, Node as AkNode, NodeId, Role as AkRole, TreeId,
    TreeUpdate,
};
use engine::accessibility::{self, Assist, Chrome, Shown, focus};
use engine::compositor::Compositor;
use sem::{Actor, Request, Role, SemId, StateKey, Value};
use showcase::view::{PAGES, ShowcaseView};

/// The pages that project their meaning, in sidebar order ("Toggle
/// Group" shows the todo page's example). Every other page is the
/// window alone to a reader.
const PROJECTING: [&str; 6] = [
    "Toggle Group",
    "Todo",
    "Surface",
    "Silences",
    "Files",
    "Capture",
];

/// The actions a reader can ask of a node: what the tree offers.
const OFFERED: [Action; 5] = [
    Action::Click,
    Action::Focus,
    Action::SetValue,
    Action::Increment,
    Action::Decrement,
];

const CHROME: Chrome<'static> = Chrome {
    title: "plev, design system showcase",
    scale: 2.0,
    size: (2560, 1600),
};

/// A page as the shell shows it to a reader: the view on the page, one
/// frame drawn, and the tree of that frame.
struct Seen {
    view: ShowcaseView,
    update: TreeUpdate,
    shown: Shown,
}

fn see(title: &str) -> Seen {
    let mut view = ShowcaseView::new(1280.0, 800.0);
    view.jump_to_page(title);
    assert_eq!(view.page_title(), title);
    let mut c = Compositor::new();
    view.render(&mut c);
    let snapshot = view.semantics().unwrap_or_default();
    let (update, ids) = accessibility::build(&snapshot, &c, &CHROME);
    Seen {
        view,
        update,
        shown: Shown { snapshot, ids },
    }
}

impl Seen {
    fn node(&self, id: NodeId) -> Option<&AkNode> {
        self.update
            .nodes
            .iter()
            .find(|(n, _)| *n == id)
            .map(|(_, n)| n)
    }

    /// Every node as a reader walks the tree: depth first from the
    /// window, children in order, with its `SemId`.
    fn walk(&self) -> Vec<(NodeId, &SemId, &AkNode)> {
        let mut out = Vec::new();
        let mut stack = vec![accessibility::ROOT];
        while let Some(id) = stack.pop() {
            let Some(node) = self.node(id) else {
                panic!("{id:?} is a child and not in the update");
            };
            if let Some(sem) = self.shown.ids.sem(id) {
                out.push((id, sem, node));
            }
            stack.extend(node.children().iter().rev());
        }
        out
    }

    fn sem(&self, id: &SemId) -> &sem::Node {
        match self.shown.snapshot.get(id) {
            Some(n) => n,
            None => panic!("{id} is in the tree and not in the snapshot"),
        }
    }

    /// The item and the action a control the tree presents stands for;
    /// `None` for a node of the snapshot.
    fn presented(&self, id: &SemId) -> Option<(&SemId, &str)> {
        self.shown.ids.presented(id)
    }
}

fn projecting() -> impl Iterator<Item = Seen> {
    PROJECTING.iter().map(|title| see(title))
}

/// Every node once, the window first, each other node the child of one
/// node, the focus among them: a full update AccessKit accepts.
fn assert_valid(title: &str, u: &TreeUpdate) {
    assert_eq!(u.nodes.first().map(|(n, _)| *n), Some(accessibility::ROOT));
    let mut listed: Vec<NodeId> = u.nodes.iter().map(|(n, _)| *n).collect();
    listed.sort();
    listed.dedup();
    assert_eq!(listed.len(), u.nodes.len(), "{title}: a node twice");
    let mut parented: Vec<NodeId> = u
        .nodes
        .iter()
        .flat_map(|(_, n)| n.children().iter().copied())
        .collect();
    parented.sort();
    let children = parented.len();
    parented.dedup();
    assert_eq!(parented.len(), children, "{title}: a node with two parents");
    assert_eq!(children, u.nodes.len() - 1, "{title}: a node unreachable");
    assert!(listed.binary_search(&u.focus).is_ok(), "{title}: focus");
}

#[test]
fn every_page_is_a_valid_tree_and_six_project_their_meaning() {
    let mut projected = Vec::new();
    for page in PAGES {
        let seen = see(page.title);
        assert_valid(page.title, &seen.update);
        if seen.shown.snapshot.is_empty() {
            assert_eq!(seen.update.nodes.len(), 1, "{}: the window", page.title);
        } else {
            projected.push(page.title);
        }
    }
    assert_eq!(projected, PROJECTING);
    assert_eq!(
        PAGES.len() - projected.len(),
        76,
        "the pages a reader meets empty"
    );
}

#[test]
fn every_control_has_a_name_and_takes_focus_and_no_group_is_empty() {
    for seen in projecting() {
        let title = seen.view.page_title();
        for (_, id, n) in seen.walk() {
            if seen.presented(id).is_some() {
                assert!(
                    n.label().is_some_and(|l| !l.trim().is_empty()),
                    "{title}: {id} is a presented control without a name"
                );
                continue;
            }
            let node = seen.sem(id);
            let control = node.role.is_interactive() || n.supports_action(Action::Focus);
            if control {
                assert!(
                    n.label().is_some_and(|l| !l.trim().is_empty()),
                    "{title}: {id} is a control without a name"
                );
            }
            if node.role.is_interactive() && node.enabled && !node.inert {
                assert!(
                    n.supports_action(Action::Focus),
                    "{title}: {id} is a {} that takes no focus",
                    node.role.as_str()
                );
            }
            if n.role() == AkRole::Group {
                assert!(
                    n.label().is_some() || n.description().is_some(),
                    "{title}: {id} is a group a reader stops on and hears nothing"
                );
            }
        }
    }
}

/// The focus a page takes and does not project: the reader moved it,
/// the page agreed, and the tree still says the window has it. The pages
/// project `focused` for most of their controls; not for the todo's
/// filter tabs, the quick todo's filter buttons and checkboxes, the
/// proposer selects, and the Silences page's detect and apply all.
const FOCUS_NOT_PROJECTED: &[&str] = &[
    "silences.apply_all",
    "silences.detect",
    "silences_surface.proposer",
    "surface_page.proposer",
    "todo.filter[active]",
    "todo.filter[all]",
    "todo.filter[completed]",
    "todo_panel.filter.active",
    "todo_panel.filter.all",
    "todo_panel.filter.completed",
    "todo_panel.item[1].done",
    "todo_panel.item[2].done",
    "todo_panel.item[3].done",
];

#[test]
fn tab_and_the_reader_walk_the_same_order_and_the_page_takes_the_focus() {
    let mut unprojected: Vec<String> = Vec::new();
    for seen in projecting() {
        let title = seen.view.page_title();
        let walked: Vec<&SemId> = seen
            .walk()
            .into_iter()
            .filter(|(_, _, n)| {
                n.supports_action(Action::Focus) && !n.is_disabled() && !n.is_hidden()
            })
            .map(|(_, id, _)| id)
            .collect();
        let tab = focus::order(&seen.shown.snapshot);
        assert_eq!(tab, walked, "{title}: Tab and the reader disagree");
        for id in tab {
            let mut fresh = see(title);
            assert!(fresh.view.assist_focus(id), "{title}: {id} refused");
            let now = fresh.view.semantics().unwrap_or_default();
            if now.focused() != Some(id) {
                unprojected.push(id.to_string());
            }
        }
    }
    unprojected.sort();
    unprojected.dedup();
    assert_eq!(unprojected, FOCUS_NOT_PROJECTED);
}

/// The value a reader's `SetValue` carries in this test: the node's own,
/// so the request is the one a reader would make without changing it.
fn same_value(n: &AkNode) -> ActionData {
    match n.numeric_value() {
        Some(v) => ActionData::NumericValue(v),
        None => ActionData::Value(n.value().unwrap_or("").into()),
    }
}

/// The presses a page offers while the action behind them is disabled:
/// the node is enabled, so the tree offers the press, and the dispatcher
/// refuses it (the todo field's `submit` while the field is empty; task-60
/// made the field's press its `submit`, the VoiceOver path of the page's
/// own tests). A reader hears nothing happen.
const OFFERED_DISABLED: &[&str] = &[
    "Surface: todo.input submit",
    "Surface: todo_panel.input submit",
    "Todo: todo.input submit",
    "Toggle Group: todo.input submit",
];

#[test]
fn every_action_the_tree_offers_is_a_request_the_dispatcher_admits() {
    let mut refused = Vec::new();
    for seen in projecting() {
        let title = seen.view.page_title();
        for (at, id, n) in seen.walk() {
            for action in OFFERED.into_iter().filter(|a| n.supports_action(*a)) {
                let req = ActionRequest {
                    action,
                    target_tree: TreeId::ROOT,
                    target_node: at,
                    data: (action == Action::SetValue).then(|| same_value(n)),
                };
                match accessibility::translate(&req, &seen.shown) {
                    Ok(Assist::Request(r)) => {
                        if !admitted(title, &seen, &r) {
                            refused.push(format!("{title}: {} {}", r.target, r.action));
                        }
                    }
                    Ok(Assist::Focus(to)) => assert_eq!(&to, id, "{title}"),
                    Err(e) => panic!("{title}: {id} offers {action:?} and it maps to nothing: {e}"),
                }
            }
        }
    }
    refused.sort();
    assert_eq!(refused, OFFERED_DISABLED);
}

/// The request is the reader's, on the node it names, of an action the
/// node declares and opens to assistive technology; `false` when the
/// action is disabled on a node the tree tells enabled.
fn admitted(title: &str, seen: &Seen, r: &Request) -> bool {
    assert_eq!(r.actor, Actor::Assistive);
    let node = seen.sem(&r.target);
    let Some(decl) = node.actions.iter().find(|d| d.name == r.action) else {
        panic!("{title}: {} does not declare {}", r.target, r.action);
    };
    assert!(
        decl.policy.actors.assistive,
        "{title}: {} {} is offered to a reader and refused to one",
        r.target, r.action
    );
    decl.enabled || told_disabled(seen, &r.target)
}

/// Whether the tree tells the reader the node is disabled.
fn told_disabled(seen: &Seen, id: &SemId) -> bool {
    seen.shown
        .ids
        .node(id)
        .and_then(|n| seen.node(n))
        .is_some_and(AkNode::is_disabled)
}

/// The actions the pages declare that no action of the tree reaches. Phase
/// 1 listed 22: a row's `remove` beside its `toggle`, the three actions
/// of a cut, a select's `open`, `close` and `select`, and the file drop.
/// Phase 2a reaches all but the drop: a row's and a cut's actions are
/// controls the tree presents inside them, a select is pressed open (and
/// shut, while open) and set by its value. The drop stays: a reader does
/// not drag a file, and `files.open` reaches the same files.
const UNREACHABLE: &[&str] = &["Files: files.drop drop_files"];

/// The node with its list `expanded` the other way: a select's press is
/// `open` while shut and `close` while open, so both are reached by it.
fn flipped(node: &sem::Node) -> sem::Node {
    let mut other = node.clone();
    let open = matches!(node.state.get("expanded"), Some(Value::Bool(true)));
    if let Ok(key) = StateKey::parse("expanded") {
        other.state.insert(key, Value::Bool(!open));
    }
    other
}

#[test]
fn the_actions_no_reader_can_reach_are_the_known_ones() {
    let mut unreachable = Vec::new();
    for seen in projecting() {
        let title = seen.view.page_title();
        for (_, id, n) in seen.walk() {
            if seen.presented(id).is_some() {
                continue;
            }
            let node = seen.sem(id);
            let other = flipped(node);
            let mut reached: Vec<&str> = [
                (
                    Action::Click,
                    accessibility::primary(node).map(|d| d.name.as_str()),
                ),
                (
                    Action::Click,
                    accessibility::primary(&other).map(|d| d.name.as_str()),
                ),
                (
                    Action::SetValue,
                    accessibility::setter(node).map(|d| d.name.as_str()),
                ),
                (Action::Increment, Some("increment")),
                (Action::Decrement, Some("decrement")),
            ]
            .into_iter()
            .filter(|(a, _)| n.supports_action(*a))
            .filter_map(|(_, name)| name)
            .collect();
            // the controls the tree presents inside it, each its action
            for child in n.children() {
                let owned = seen
                    .shown
                    .ids
                    .sem(*child)
                    .and_then(|c| seen.presented(c))
                    .filter(|(item, _)| *item == id);
                if let Some((_, action)) = owned {
                    reached.push(action);
                }
            }
            for decl in &node.actions {
                if !reached.contains(&decl.name.as_str()) {
                    unreachable.push(format!("{title}: {id} {}", decl.name));
                }
            }
        }
    }
    unreachable.sort();
    assert_eq!(unreachable, UNREACHABLE);
}

/// Every status line of every page is a polite live region, and an
/// alert an assertive one, that says its words: the message a status
/// keeps (`silences.status`, `surface_page.status`, the Files and
/// Capture lines), or its label (the todo's counter, the Silences
/// counts). What announces a toast is `app/access.rs`, tested there.
#[test]
fn every_status_line_is_a_live_region_that_says_its_words() {
    let mut lines = Vec::new();
    for seen in projecting() {
        let title = seen.view.page_title();
        for (_, id, n) in seen.walk() {
            if seen.presented(id).is_some() {
                assert_eq!(n.live(), None, "{title}: {id} is not live");
                continue;
            }
            let node = seen.sem(id);
            let politeness = match node.role {
                Role::Status => Live::Polite,
                Role::Alert => Live::Assertive,
                _ => {
                    assert_eq!(n.live(), None, "{title}: {id} is not live");
                    continue;
                }
            };
            assert_eq!(n.live(), Some(politeness), "{title}: {id}");
            let words = match node.state.get("message") {
                Some(Value::Text(m)) if !m.is_empty() => Some(m.as_str()),
                Some(_) => None,
                None => Some(node.label.as_str()),
            };
            assert_eq!(n.value(), words, "{title}: {id} says its words");
            lines.push(id.to_string());
        }
    }
    lines.sort();
    lines.dedup();
    assert_eq!(
        lines,
        [
            "capture.status",
            "files.status",
            "silences.counts",
            "silences.status",
            "silences_surface.status",
            "surface_page.status",
            "todo.counter",
        ]
    );
}
