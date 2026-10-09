//! A composite installed is an action of the app (task-70 phase 3),
//! declared after the app's own projection on the nodes it runs on: on
//! every item of its collection, reading what its steps read, on when its
//! first step is, with the narrowest policy of its steps; so a sequence
//! through an action kept from agents is a composite an agent cannot ask.

use offers::compose::Composer;
use accord::{
    ActionDecl, ActionName, Actor, Rejection, Request, SemId, Semantics, Snapshot, SnapshotBuilder,
    StateRef, Verdict, check,
};

use super::desk::{Bench, Desk, PERSON_ONLY, agent, person};

fn sid(text: &str) -> SemId {
    SemId::parse(text).unwrap()
}

/// The desk projected, then the composer's composites on it.
fn projected(desk: &Desk, composer: &Composer) -> Snapshot {
    let mut b = SnapshotBuilder::new();
    desk.project(&mut b);
    composer.declare(&mut b);
    b.finish().expect("the desk projects with its composites")
}

fn decl<'a>(snap: &'a Snapshot, node: &str, action: &str) -> Option<&'a ActionDecl> {
    snap.get(&sid(node))?
        .actions
        .iter()
        .find(|a| a.name.as_str() == action)
}

/// `actions` by `actor` on docs 1 and 2, then the offer accepted.
fn installed(actor: &Actor, actions: &[&str]) -> (Bench, Composer) {
    let mut bench = Bench::new(3);
    let mut composer = Composer::new(2);
    bench.dispatcher.add_sink(Box::new(composer.listener()));
    for doc in [1, 2] {
        for action in actions {
            bench.act(actor.clone(), &format!("desk.doc[{doc}]"), action);
        }
    }
    assert!(composer.offer(|_| true).is_some(), "offered");
    let app = bench.desk.snapshot();
    composer.accept(&app).expect("installed");
    (bench, composer)
}

#[test]
fn the_composite_is_declared_on_every_item_reading_what_its_steps_read() {
    let (bench, composer) = installed(&person(), &["review", "approve", "publish"]);
    let snap = projected(&bench.desk, &composer);
    let name = "review_then_approve_then_publish";
    for doc in 1..=3 {
        let at = format!("desk.doc[{doc}]");
        let composite = decl(&snap, &at, name).unwrap_or_else(|| panic!("on {at}"));
        let status = StateRef::parse(&format!("{at}:status")).unwrap();
        assert_eq!(composite.reads, [status]);
        assert_eq!(
            composite.label.as_deref(),
            Some("review, then approve, then publish")
        );
        assert!(composite.policy.actors.agent, "every step lets agents ask");
    }
    // docs 1 and 2 are published: the review is on, so the composite is;
    // its later steps are checked when they come
    assert!(decl(&snap, "desk.doc[3]", name).unwrap().enabled);
    assert!(decl(&snap, "desk.doc[1]", name).unwrap().enabled);
    // the steps it runs, on the node it is asked of
    let composite = &composer.installed()[0];
    let calls = composite.calls(&sid("desk.doc[3]")).expect("an item");
    let named: Vec<(String, &str)> = calls
        .iter()
        .map(|(id, a)| (id.to_string(), a.as_str()))
        .collect();
    assert_eq!(
        named,
        [
            ("desk.doc[3]".to_owned(), "review"),
            ("desk.doc[3]".to_owned(), "approve"),
            ("desk.doc[3]".to_owned(), "publish"),
        ]
    );
    assert!(composite.calls(&sid("desk.refresh")).is_none());
    let action = ActionName::parse(name).unwrap();
    assert!(composer.composite(&sid("desk.doc[3]"), &action).is_some());
}

/// the privacy promise: a sequence through an action kept from agents is
/// a composite for a person only, and an agent's request for it is
/// refused before any step runs.
#[test]
fn a_step_kept_from_agents_keeps_the_composite_from_them() {
    let (bench, composer) = installed(&person(), &["review", "sign"]);
    let snap = projected(&bench.desk, &composer);
    let composite = decl(&snap, "desk.doc[3]", "review_then_sign").expect("declared");
    assert_eq!(composite.policy.actors, PERSON_ONLY);
    let asked =
        Request::observe(&snap, agent("bot"), "desk.doc[3]", "review_then_sign", None).unwrap();
    assert_eq!(
        check(&asked, &snap),
        Verdict::Rejected(Rejection::NotAuthorized {
            actor: agent("bot")
        })
    );
    let asked = Request::observe(&snap, person(), "desk.doc[3]", "review_then_sign", None).unwrap();
    assert_eq!(check(&asked, &snap), Verdict::Accepted);
}

#[test]
fn a_node_that_declares_the_name_keeps_its_own() {
    let (bench, composer) = installed(&person(), &["review", "approve"]);
    let mut b = SnapshotBuilder::new();
    bench.desk.project(&mut b);
    b.reopen(&sid("desk.doc[3]"))
        .expect("declared")
        .action("review_then_approve")
        .label("the app's own");
    composer.declare(&mut b);
    let snap = b.finish().expect("no action declared twice");
    let own = decl(&snap, "desk.doc[3]", "review_then_approve").unwrap();
    assert_eq!(own.label.as_deref(), Some("the app's own"));
}
