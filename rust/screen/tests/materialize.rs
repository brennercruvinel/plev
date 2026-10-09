//! Materializing each fixture of task-68 projects the ids, the roles and
//! the actions of its IR, the instances of a repeat included, by the live
//! value of the list; and a node keeps its widget while its id stays.
//!
//! The expected projection is read from the IR here, by the test's own
//! walk, not by the crate's.

// shared with the other surface tests: this binary uses the fixtures, the
// page and the app.
#[allow(dead_code)]
mod support;

use std::collections::BTreeMap;

use sem::{Actor, Request, Role, SemId, Snapshot, Value};
use support::{App, CASES, IDE, REVIEW, SHOWCASE, TODO, id, ir, page, snapshot, verified};
use surface::Surface;
use uir::{Capabilities, CapabilityRef, GenerationPolicy, IrNode, role_of, verify};

/// What the IR says the page holds for one node.
struct Expect {
    id: SemId,
    role: Role,
    label: Option<String>,
    /// each event it sends, and the capability that runs.
    actions: BTreeMap<String, CapabilityRef>,
}

/// `text` with its `[{item}]` keyed by `item` (the fixtures' items hold
/// no `]` or `\`).
fn keyed(text: &str, item: Option<&str>) -> SemId {
    match item {
        Some(item) => id(&text.replace("[{item}]", &format!("[{item}]"))),
        None => id(text),
    }
}

fn expected(node: &IrNode, app: &Snapshot, item: Option<&str>, out: &mut Vec<Expect>) {
    if let Some(role) = role_of(node.component, &node.bind) {
        let actions = node
            .bind
            .actions
            .iter()
            .map(|(event, cap)| {
                let target = keyed(cap.target.as_str(), item);
                let cap = CapabilityRef {
                    target,
                    action: cap.action.clone(),
                };
                (event.clone(), cap)
            })
            .collect();
        out.push(Expect {
            id: keyed(node.sem.as_str(), item),
            role,
            label: node.label.clone(),
            actions,
        });
    }
    for child in &node.children {
        expected(child, app, item, out);
    }
    if let Some(repeat) = &node.repeat {
        let items = match app.value(&repeat.each) {
            Some(Value::List(list)) => list.clone(),
            Some(Value::Set(set)) => set.iter().cloned().collect(),
            _ => Vec::new(),
        };
        for it in items {
            let first = out.len();
            expected(&repeat.item, app, Some(&it), out);
            // an instance is named by its item
            if let Some(root) = out.get_mut(first) {
                root.label = Some(it.clone());
            }
        }
    }
}

#[test]
fn every_fixture_projects_the_ids_roles_and_actions_of_its_ir() {
    for (ir_name, snap_name) in CASES {
        let app = snapshot(snap_name);
        let surface = Surface::materialize(verified(ir_name, &app), &app);
        let page = page(&app, &surface);
        let mut want = Vec::new();
        expected(&ir(ir_name).surface.root, &app, None, &mut want);
        assert!(want.len() > 3, "{ir_name}");

        for e in &want {
            let node = page
                .get(&e.id)
                .unwrap_or_else(|| panic!("{ir_name}: {} is not projected", e.id));
            assert_eq!(node.role, e.role, "{ir_name}: {}", e.id);
            if app.get(&e.id).is_some() {
                // a node of the app's own (the fixed commit form written
                // as IR): the app declares it, and the capability the IR
                // binds on it is its own action.
                for cap in e.actions.values() {
                    assert_eq!(cap.target, e.id, "{ir_name}");
                    assert!(node.actions.iter().any(|a| a.name == cap.action));
                }
                continue;
            }
            if let Some(label) = &e.label {
                assert_eq!(&node.label, label, "{ir_name}: {}", e.id);
            }
            let events: Vec<&str> = e.actions.keys().map(String::as_str).collect();
            let names: Vec<&str> = node.actions.iter().map(|a| a.name.as_str()).collect();
            assert_eq!(names, events, "{ir_name}: {}", e.id);
            for (event, cap) in &e.actions {
                let own = node.actions.iter().find(|a| a.name.as_str() == event);
                let theirs = app
                    .get(&cap.target)
                    .and_then(|n| n.actions.iter().find(|a| a.name == cap.action));
                let (Some(own), Some(theirs)) = (own, theirs) else {
                    panic!("{ir_name}: {} {event} or {cap:?} is missing", e.id);
                };
                // the node's action reads what the capability reads: as
                // the surface shows it when a node of the surface binds
                // it, as the app holds it otherwise; and takes what the
                // capability takes
                assert_eq!(own.reads.len(), theirs.reads.len(), "{ir_name}: {}", e.id);
                for (mine, read) in own.reads.iter().zip(&theirs.reads) {
                    if mine != read {
                        assert!(app.get(&mine.node).is_none(), "{ir_name}: {mine}");
                        assert_eq!(page.value(mine), app.value(read), "{ir_name}: {mine}");
                    }
                }
                assert_eq!(own.param, theirs.param, "{ir_name}: {} {event}", e.id);
                assert_eq!(own.policy, theirs.policy, "{ir_name}: {} {event}", e.id);
                // and an act on it is the capability, by the same actor
                let actor = Actor::Test {
                    name: "materialize".into(),
                };
                let req = Request::observe(&page, actor.clone(), &e.id, event.as_str(), None)
                    .unwrap_or_else(|err| panic!("{ir_name}: {err}"));
                let routed = surface.route(req);
                assert_eq!(routed.target, cap.target, "{ir_name}: {} {event}", e.id);
                assert_eq!(routed.action, cap.action, "{ir_name}: {} {event}", e.id);
                assert_eq!(routed.actor, actor);
                // the observation names the app's state again
                let refs: Vec<_> = routed.observed.values.iter().map(|(at, _)| at).collect();
                assert_eq!(refs, theirs.reads.iter().collect::<Vec<_>>(), "{ir_name}");
            }
        }

        // nothing beyond the app and the IR: every other node of the page
        // is one the IR holds, or the surface's region.
        for node in page.nodes().values() {
            let known = app.get(&node.id).is_some()
                || want.iter().any(|e| e.id == node.id)
                || node.id == *surface.id();
            assert!(known, "{ir_name}: {} comes from nowhere", node.id);
        }
        let region = page.get(surface.id()).expect("the region");
        assert_eq!(region.role, Role::Region);
        assert_eq!(region.label, ir(ir_name).surface.title);
    }
}

/// The review of the ide repeats over `changes:files`: one instance per
/// file, named by it, in the list's order, and the list is read again
/// when it moves.
/// The todo surface shows `todo.input:text` in its field, so its add
/// button reads the field, and an agent that observed only the surface
/// has observed what the add reads.
#[test]
fn an_action_reads_the_state_where_the_surface_shows_it() {
    let app = snapshot(SHOWCASE);
    let surface = Surface::materialize(verified(TODO, &app), &app);
    let page = page(&app, &surface);
    let reads = |at: &str, action: &str| -> Vec<String> {
        let node = page.get(&id(at)).expect("projected");
        let a = node
            .actions
            .iter()
            .find(|a| a.name.as_str() == action)
            .expect("the action");
        a.reads.iter().map(ToString::to_string).collect()
    };
    assert_eq!(
        reads("todo_panel.add", "activate"),
        ["todo_panel.input:text"]
    );
    assert_eq!(
        reads("todo_panel.input", "submit"),
        ["todo_panel.input:text"]
    );
    // the filter's state is shown nowhere on the surface: the app's
    assert_eq!(
        reads("todo_panel.filter.active", "activate"),
        ["todo.filter:selected"]
    );
}

#[test]
fn a_repeat_follows_the_live_list() {
    let mut app = App::over(&snapshot(IDE));
    let now = app.snapshot();
    let mut surface = Surface::materialize(verified(REVIEW, &now), &now);
    let instances = |s: &Surface| -> Vec<String> {
        s.nodes()
            .map(SemId::as_str)
            .filter(|t| t.starts_with("review.file[") && t.ends_with(']'))
            .map(str::to_owned)
            .collect()
    };
    assert_eq!(
        instances(&surface),
        [
            "review.file[src/lexer.rs]",
            "review.file[notes.md]",
            "review.file[src/parser.rs]"
        ]
    );
    let shown = page(&now, &surface);
    let stage = shown
        .get(&id("review.file[notes.md].stage"))
        .expect("stage");
    assert_eq!(stage.role, Role::Button);
    assert_eq!(
        shown
            .get(&id("review.file[notes.md]"))
            .map(|n| n.label.as_str()),
        Some("notes.md")
    );

    // a file leaves the list and another is listed twice: one instance
    // per item, the first
    let files = ["notes.md", "src/parser.rs", "notes.md"].map(str::to_owned);
    app.set("changes:files", Value::List(files.to_vec()));
    assert!(surface.sync(&app.snapshot()));
    assert_eq!(
        instances(&surface),
        ["review.file[notes.md]", "review.file[src/parser.rs]"]
    );
    let shown = page(&app.snapshot(), &surface);
    assert!(shown.get(&id("review.file[src/lexer.rs].stage")).is_none());
    // the same revision again moves nothing
    assert!(!surface.sync(&app.snapshot()));
}

/// A node keeps its retained widget while its id stays: the field that
/// has the keyboard keeps it through a sync that moved other state.
#[test]
fn a_node_keeps_its_widget_while_its_id_stays() {
    let mut app = App::over(&snapshot(IDE));
    let now = app.snapshot();
    let mut surface = Surface::materialize(verified(REVIEW, &now), &now);
    let message = id("review.message");
    assert!(surface.focus(&message));
    assert_eq!(surface.focused(), Some(&message));
    app.set("changes:files", Value::List(vec!["notes.md".to_owned()]));
    assert!(surface.sync(&app.snapshot()));
    assert_eq!(
        surface.focused(),
        Some(&message),
        "the field kept its widget"
    );
    // the page says so: the focus is the surface's field
    assert_eq!(page(&app.snapshot(), &surface).focused(), Some(&message));
}

/// Only a verified IR materializes. `Surface::materialize` takes a
/// `VerifiedIr`, which only `uir::verify` builds (the `compile_fail`
/// doctest of the crate shows an `Ir` does not type-check there); an IR
/// the verifier refuses gives no value to materialize at all.
#[test]
fn an_ir_the_verifier_refuses_has_nothing_to_materialize() {
    let ide = Capabilities::from_snapshot(&snapshot(IDE));
    let refused = verify(&ir(TODO), &ide, &GenerationPolicy::default());
    let Err(errors) = refused else {
        panic!("the todo surface cannot verify against the ide");
    };
    assert!(!errors.is_empty());
    // the same IR over its own app verifies, and only that value opens
    let todo = snapshot(SHOWCASE);
    let surface = Surface::materialize(verified(TODO, &todo), &todo);
    assert_eq!(surface.id().as_str(), "todo_panel");
}
