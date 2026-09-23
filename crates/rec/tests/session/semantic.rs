//! the semantic track event by event: what is stored whole, what as
//! changes, what not at all; the cadence re-key; ordered time; origins;
//! a malformed change list read back as a typed error.

use std::sync::Arc;

use rec::{Event, InputKind, Kind, Limits, ReadError, Recorder, Session};
use sem::{Change, Dispatcher, Request, Role, Semantics, SnapshotBuilder, Stage};

use crate::apps::{Git, id, look};

type Fallible = Result<(), Box<dyn std::error::Error>>;

/// a list whose items come and go, with a title that changes in place.
struct Todo {
    title: String,
    items: Vec<&'static str>,
}

impl Semantics for Todo {
    fn project(&self, b: &mut SnapshotBuilder) {
        b.node("todo", Role::List, "todo")
            .state("title", self.title.as_str());
        for item in &self.items {
            b.node(format!("todo.item[{item}]").as_str(), Role::ListItem, *item)
                .parent("todo");
        }
    }
}

fn kinds(session: &Session) -> Vec<&'static str> {
    session.events().iter().map(|e| e.kind.name()).collect()
}

#[test]
fn a_snapshot_is_stored_whole_as_changes_or_not_at_all() -> Fallible {
    let rec = Recorder::new(Limits {
        full_every: 1_000,
        ..Limits::default()
    });
    let mut todo = Todo {
        title: "home".to_owned(),
        items: vec!["milk"],
    };
    rec.snapshot(0, &look(&todo)); // first: whole
    rec.snapshot(10, &look(&todo)); // unchanged: nothing
    todo.title = "work".to_owned();
    rec.snapshot(20, &look(&todo)); // a state changed in place: changes
    todo.items.push("mail");
    rec.snapshot(30, &look(&todo)); // a node added: whole
    todo.items.retain(|i| *i != "milk");
    rec.snapshot(40, &look(&todo)); // a node removed: changes
    todo.items.reverse();
    rec.snapshot(50, &look(&todo)); // one item left: unchanged
    todo.title = "later".to_owned();
    rec.snapshot(1_100, &look(&todo)); // the cadence is due: whole
    rec.snapshot_full(1_200, &look(&todo)); // asked for: whole

    let session = rec.snapshot_window();
    assert_eq!(
        kinds(&session),
        [
            "snapshot", "changes", "snapshot", "changes", "snapshot", "snapshot"
        ]
    );
    let stats = rec.stats();
    assert_eq!(stats.snapshots_full, 4);
    assert_eq!(stats.snapshots_changed, 2);
    assert_eq!(stats.snapshots_unchanged, 2);
    assert_eq!(stats.segments, 4);
    assert!(stats.semantic_bytes > 0);

    // the whole snapshot at 30 keeps the diff that explains it.
    let Some(Event {
        kind: Kind::Snapshot { changes, .. },
        ..
    }) = session.events().get(2)
    else {
        return Err("no snapshot at 30".into());
    };
    assert_eq!(changes, &[Change::NodeAdded(id("todo.item[mail]"))]);

    let at_45 = session.state_at(45)?.ok_or("no state")?;
    let ids: Vec<&str> = at_45.nodes().iter().map(|n| n.id.as_str()).collect();
    assert_eq!(ids, ["todo", "todo.item[mail]"]);
    assert_eq!(
        at_45.revision()?,
        look(&todo_at("work", &["mail"])).revision()
    );
    Ok(())
}

fn todo_at(title: &str, items: &[&'static str]) -> Todo {
    Todo {
        title: title.to_owned(),
        items: items.to_vec(),
    }
}

#[test]
fn an_event_after_the_cadence_rekeys_the_unchanged_state() -> Fallible {
    let rec = Recorder::new(Limits {
        full_every: 100,
        ..Limits::default()
    });
    let todo = todo_at("home", &["milk"]);
    rec.snapshot(0, &look(&todo));
    rec.input(50, InputKind::Key, None, None);
    rec.input(150, InputKind::Key, None, None); // due: re-key first
    rec.input(160, InputKind::Key, None, None);

    let session = rec.snapshot_window();
    assert_eq!(
        kinds(&session),
        ["snapshot", "input", "snapshot", "input", "input"]
    );
    let opening = |at: usize| match session.events().get(at).map(|e| &e.kind) {
        Some(Kind::Snapshot { snapshot, changes }) if changes.is_empty() => Some(snapshot.clone()),
        _ => None,
    };
    let (first, rekey) = (opening(0).ok_or("no key")?, opening(2).ok_or("no rekey")?);
    assert!(
        Arc::ptr_eq(&first, &rekey),
        "an unchanged state shares its snapshot"
    );
    assert_eq!(rec.stats().rekeys, 1);
    assert_eq!(session.keyframe_at(155).map(|e| e.t), Some(150));
    Ok(())
}

#[test]
fn time_never_goes_back_and_events_before_a_snapshot_have_no_state() -> Fallible {
    let rec = Recorder::default();
    rec.input(100, InputKind::PointerDown, Some([1.0, 2.0]), None);
    rec.input(90, InputKind::PointerUp, Some([1.0, 2.0]), None);
    rec.snapshot(120, &look(&todo_at("home", &[])));

    let session = rec.snapshot_window();
    let times: Vec<u64> = session.events().iter().map(|e| e.t).collect();
    assert_eq!(times, [100, 100, 120]);
    assert_eq!(rec.stats().clamped, 1);
    assert!(session.state_at(110)?.is_none());
    assert!(session.state_at(120)?.is_some());
    assert_eq!(session.between(100, 100).len(), 2);
    assert_eq!(session.span(), Some((100, 120)));
    Ok(())
}

#[test]
fn two_dispatchers_never_mix_their_request_ids() -> Fallible {
    let rec = Recorder::default();
    let (left, right) = (rec.sink(), rec.sink());
    assert_ne!(left.origin(), right.origin());
    let mut one = Dispatcher::new();
    one.add_sink(Box::new(left));
    let mut two = Dispatcher::new();
    two.add_sink(Box::new(right));
    let mut a = Git::new(&["A"], "one");
    let mut b = Git::new(&["B"], "two");
    rec.snapshot(0, &look(&a));
    let ask = |git: &Git| {
        Request::observe(
            &look(git),
            sem::Actor::Human,
            "commit.submit",
            "submit",
            None,
        )
    };
    let (ask_a, ask_b) = (ask(&a)?, ask(&b)?);
    one.dispatch(&mut a, ask_a, 10);
    two.dispatch(&mut b, ask_b, 20);

    let receipts = rec.snapshot_window().receipts(20);
    assert_eq!(receipts.len(), 2, "both request 1, told apart by origin");
    assert!(receipts.iter().all(|r| r.receipt.id.as_u64() == 1));
    assert!(
        receipts
            .iter()
            .all(|r| r.receipt.stage().map(Stage::name) == Some("completed"))
    );
    assert_ne!(receipts[0].origin, receipts[1].origin);
    assert_eq!(receipts[1].receipt.request.target, id("commit.submit"));
    Ok(())
}

#[test]
fn a_change_list_the_recorder_never_writes_reads_back_as_an_error() -> Fallible {
    let todo = todo_at("home", &["milk"]);
    let base = Arc::new(look(&todo));
    let snapshot = Event {
        t: 0,
        kind: Kind::Snapshot {
            snapshot: base,
            changes: Vec::new(),
        },
    };
    let added = Event {
        t: 1,
        kind: Kind::Changes(vec![Change::NodeAdded(id("todo.item[mail]"))]),
    };
    let unknown = Event {
        t: 1,
        kind: Kind::Changes(vec![Change::NodeRemoved(id("nowhere"))]),
    };
    let session = Session::from_events(vec![snapshot.clone(), added]);
    assert_eq!(
        session.state_at(1),
        Err(ReadError::Structural("node_added"))
    );
    let session = Session::from_events(vec![snapshot, unknown]);
    assert_eq!(
        session.state_at(1),
        Err(ReadError::UnknownNode(id("nowhere")))
    );
    assert!(
        session.state_at(0)?.is_some(),
        "the snapshot alone still reads"
    );
    Ok(())
}
