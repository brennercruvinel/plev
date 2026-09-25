//! The generated surface of the ide (task-96): the intent line and the
//! host beside the fixed views, on a real repository and the real git
//! worker. Off by default, the workbench projects and draws as before; on,
//! the line is projected and drawn in a column at the right of the window,
//! an intent the rules read opens a verified surface through the one
//! dispatcher, and closing it leaves nothing of it: no node, no binding,
//! no state in the host.

// shared with the other ide tests: this binary uses part of the rig.
#[allow(dead_code)]
mod support;

use engine::compositor::Compositor;
use engine::theme::Theme;
use sem::{Actor, Role, SemId, Snapshot, Stage, Value};
use support::{Rig, completed};

fn id(text: &str) -> SemId {
    SemId::parse(text).expect("an id")
}

/// The workbench drawn into a fresh frame, as the window draws it.
fn frame(rig: &mut Rig) -> Compositor {
    let mut c = Compositor::new();
    c.begin_frame();
    rig.bench.render(&mut c);
    c
}

/// The ids bound in `c` under `prefix` (the id itself or below it).
fn bound_under(c: &Compositor, prefix: &str) -> Vec<String> {
    c.bindings()
        .map(|(id, _)| id.to_string())
        .filter(|id| id == prefix || id.starts_with(&format!("{prefix}.")))
        .collect()
}

/// The nodes of `snap` under `prefix`.
fn nodes_under(snap: &Snapshot, prefix: &str) -> Vec<String> {
    snap.nodes()
        .keys()
        .map(ToString::to_string)
        .filter(|id| id == prefix || id.starts_with(&format!("{prefix}.")))
        .collect()
}

/// The person writes `intent` in the line and proposes, each through the
/// dispatcher; the receipt of the proposal.
fn propose(rig: &mut Rig, intent: &str) -> sem::Receipt {
    let write = rig.ask(
        Actor::Human,
        "ide_surface.intent",
        "set_value",
        Some(intent.into()),
    );
    assert!(completed(&rig.receipt(write)).is_some());
    let asked = rig.ask(Actor::Human, "ide_surface.propose", "propose", None);
    rig.receipt(asked)
}

#[test]
fn off_by_default_the_workbench_is_what_it_was() {
    let mut rig = Rig::new(&["a.txt"], &[]);
    assert!(!rig.bench.hosts_surfaces());
    let whole = rig.look();
    let app = rig.bench.app_snapshot().expect("the projection validates");
    assert_eq!(whole, app, "no line, no surface: one projection");
    assert!(nodes_under(&whole, "ide_surface").is_empty());
    let c = frame(&mut rig);
    assert!(bound_under(&c, "ide_surface").is_empty());
    assert_eq!(
        rig.bench.workspace().vw,
        1280.0,
        "the window is the workspace's"
    );
}

#[test]
fn on_the_line_is_projected_and_drawn_beside_the_workspace() {
    let mut rig = Rig::new(&["a.txt"], &[]);
    rig.bench.host_surfaces();
    let whole = rig.look();
    let app = rig.bench.app_snapshot().expect("the projection validates");
    assert!(nodes_under(&app, "ide_surface").is_empty(), "the app alone");
    let node = |at: &str| whole.get(&id(at)).expect(at).clone();
    assert_eq!(node("ide_surface").role, Role::Region);
    assert_eq!(node("ide_surface.intent").role, Role::TextInput);
    assert_eq!(node("ide_surface.proposer").role, Role::Select);
    assert_eq!(
        whole.value(&"ide_surface.proposer:selected".parse().unwrap()),
        Some(&Value::Text("rules".into())),
        "the rules first"
    );
    assert!(!node("ide_surface.propose").enabled, "no intent yet");
    assert!(!node("ide_surface.close").enabled, "no surface yet");
    for kept in app.nodes().keys() {
        assert_eq!(whole.get(kept), app.get(kept), "{kept} as it was");
    }
    let c = frame(&mut rig);
    let theme = Theme::hoff();
    let ws = rig.bench.workspace().vw;
    assert_eq!(ws, 1280.0 - theme.size.card_w, "the column is a card wide");
    for at in ["ide_surface", "ide_surface.intent", "ide_surface.propose"] {
        let rect = c.binding(&id(at)).expect(at).rect;
        assert!(rect[0] >= ws, "{at} sits in the column: {rect:?}");
    }
}

#[test]
fn an_intent_the_rules_read_opens_a_surface_and_closing_it_leaves_nothing() {
    let mut rig = Rig::new(&["a.txt", "b.txt"], &["a.txt"]);
    rig.bench.host_surfaces();
    let before = rig.bench.app_snapshot().expect("validates");
    let receipt = propose(&mut rig, "the items");
    let outcome = completed(&receipt).unwrap_or_else(|| panic!("{receipt:?}"));
    assert_eq!(receipt.request.actor, Actor::Human);
    let surface = match outcome.result.get("surface") {
        Some(Value::Text(s)) => s.clone(),
        other => panic!("{other:?}"),
    };
    assert_eq!(
        rig.bench.surface_open().map(ToString::to_string),
        Some(surface.clone())
    );
    let open = rig.look();
    let shown = nodes_under(&open, &surface);
    assert!(shown.len() > 2, "the surface is projected: {shown:?}");
    assert_eq!(open.get(&id(&surface)).map(|n| n.role), Some(Role::Region));
    assert!(open.get(&id("ide_surface.close")).unwrap().enabled);
    let c = frame(&mut rig);
    assert!(
        !bound_under(&c, &surface).is_empty(),
        "the surface is drawn"
    );
    let wide = rig.bench.workspace().vw;
    assert!(
        wide < 1280.0 - Theme::hoff().size.card_w,
        "the surface takes room"
    );
    assert_eq!(
        rig.bench.app_snapshot().expect("validates"),
        before,
        "proposing moved nothing of the app"
    );

    let closed = rig.ask(Actor::Human, "ide_surface.close", "close", None);
    let receipt = rig.receipt(closed);
    let outcome = completed(&receipt).unwrap_or_else(|| panic!("{receipt:?}"));
    assert_eq!(
        outcome.result.get("closed"),
        Some(&Value::Text(surface.clone()))
    );
    let after = rig.look();
    assert!(nodes_under(&after, &surface).is_empty(), "no node of it");
    let c = frame(&mut rig);
    assert!(bound_under(&c, &surface).is_empty(), "no binding of it");
    let host = rig.bench.surfaces().expect("the host");
    assert!(host.is_empty(), "no surface in the host");
    assert!(host.changes(&id(&surface)).is_empty());
    assert!(host.policy(&id(&surface)).is_none());
    assert_eq!(rig.bench.surface_open(), None);
    assert_eq!(rig.bench.workspace().vw, 1280.0 - Theme::hoff().size.card_w);
    assert!(!after.get(&id("ide_surface.close")).unwrap().enabled);
}

#[test]
fn an_intent_the_rules_do_not_read_is_refused_with_why() {
    let mut rig = Rig::new(&["a.txt"], &[]);
    rig.bench.host_surfaces();
    let receipt = propose(&mut rig, "zzz qqq");
    let Some(Stage::Failed { error }) = receipt.stage() else {
        panic!("{receipt:?}");
    };
    assert!(error.contains("\"zzz\""), "{error}");
    let message = rig.value("ide_surface.status:message");
    assert_eq!(message, Value::Text(error.clone()), "the line says why");
    assert_eq!(rig.bench.surface_open(), None);
    assert!(rig.bench.surfaces().unwrap().is_empty());
}
