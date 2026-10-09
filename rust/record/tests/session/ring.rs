//! the ring respects both limits, with the test's own clock: it keeps at
//! least the last `max_age` and not much more, it stays under
//! `max_bytes`, and what it keeps always opens at a full snapshot.

use record::{InputKind, Kind, Limits, Recorder, Session};
use accord::{
    Actor, Dispatcher, Execution, Operable, Outcome, Request, RequestId, Role, Semantics,
    SnapshotBuilder,
};

use crate::toys::{id, look};

type Fallible = Result<(), Box<dyn std::error::Error>>;

/// a counter that moves, and a job whose execution ends later.
struct Counter {
    n: i64,
}

impl Semantics for Counter {
    fn project(&self, b: &mut SnapshotBuilder) {
        b.node("count", Role::Group, "count").state("n", self.n);
        b.node("job", Role::Button, "run").action("run");
    }
}

impl Operable for Counter {
    fn execute(&mut self, _id: RequestId, _req: &Request) -> Execution {
        Execution::Started
    }
}

fn opens_keyed(session: &Session) -> bool {
    matches!(
        session.events().first().map(|e| &e.kind),
        Some(Kind::Snapshot { .. })
    )
}

#[test]
fn the_ring_keeps_the_last_max_age_and_opens_at_a_full_snapshot() -> Fallible {
    let limits = Limits {
        max_age: 5_000,
        max_bytes: usize::MAX,
        full_every: 1_000,
        ..Limits::default()
    };
    let rec = Recorder::new(limits);
    let mut counter = Counter { n: 0 };
    let step = 100;
    for now in (0..60_000).step_by(step) {
        if now % 300 == 0 {
            counter.n += 1;
        }
        rec.snapshot(now, &look(&counter));
        rec.input(now, InputKind::Key, None, None);

        let session = rec.snapshot_window();
        let (start, end) = session.span().ok_or("empty window")?;
        assert_eq!(end, now);
        assert!(opens_keyed(&session), "t={now}: the window opens unkeyed");
        assert!(
            start <= now.saturating_sub(limits.max_age),
            "t={now}: the window starts at {start}, after now - max_age"
        );
        let most = limits.max_age + limits.full_every + step as u64;
        assert!(
            now - start < most,
            "t={now}: the window spans {}",
            now - start
        );
        assert!(session.state_at(start)?.is_some());
    }
    let stats = rec.stats();
    assert!(stats.evicted_segments > 0);
    assert_eq!(stats.over_budget, 0);
    Ok(())
}

#[test]
fn the_ring_stays_under_its_byte_budget() -> Fallible {
    let limits = Limits {
        max_age: u64::MAX,
        max_bytes: 12_000,
        full_every: 500,
        ..Limits::default()
    };
    let rec = Recorder::new(limits);
    let mut counter = Counter { n: 0 };
    for now in (0..40_000).step_by(100) {
        counter.n += 1;
        rec.snapshot(now, &look(&counter));
        rec.input(
            now,
            InputKind::PointerMove,
            Some([1.0, 2.0]),
            Some(id("count")),
        );
        let stats = rec.stats();
        assert!(
            stats.bytes <= limits.max_bytes,
            "t={now}: {} bytes over {}",
            stats.bytes,
            limits.max_bytes
        );
        assert_eq!(stats.bytes, stats.semantic_bytes);
    }
    let session = rec.snapshot_window();
    assert!(opens_keyed(&session));
    let (start, end) = session.span().ok_or("empty window")?;
    assert!(start > 0, "the budget cut the oldest segments");
    assert_eq!(end, 39_900);
    let stats = rec.stats();
    assert!(stats.evicted_segments > 0);
    assert_eq!(stats.over_budget, 0);
    Ok(())
}

#[test]
fn a_budget_below_one_segment_rekeys_so_the_ring_can_cut() -> Fallible {
    let rec = Recorder::new(Limits {
        max_bytes: 1,
        ..Limits::default()
    });
    let counter = Counter { n: 7 };
    rec.snapshot(0, &look(&counter));
    for now in 1..50 {
        rec.input(now, InputKind::Key, None, None);
        let session = rec.snapshot_window();
        assert!(opens_keyed(&session), "t={now}");
        let kinds: Vec<&str> = session.events().iter().map(|e| e.kind.name()).collect();
        assert_eq!(kinds, ["snapshot", "input"], "t={now}");
        assert_eq!(session.span(), Some((now, now)));
    }
    let stats = rec.stats();
    assert_eq!(stats.segments, 1);
    assert!(stats.over_budget >= 49);
    assert!(stats.rekeys >= 49);
    Ok(())
}

#[test]
fn a_recording_without_snapshots_stays_bounded_event_by_event() -> Fallible {
    let rec = Recorder::new(Limits {
        max_age: 1_000,
        ..Limits::default()
    });
    for now in (0..10_000).step_by(10) {
        rec.input(now, InputKind::Wheel, None, None);
    }
    let session = rec.snapshot_window();
    assert_eq!(session.span(), Some((8_990, 9_990)));
    assert!(session.state_at(9_990)?.is_none(), "no snapshot, no state");
    assert!(rec.stats().evicted_unkeyed > 0);

    let rec = Recorder::new(Limits {
        max_bytes: 2_000,
        ..Limits::default()
    });
    for now in 0..1_000 {
        rec.input(now, InputKind::Wheel, None, None);
        assert!(rec.stats().bytes <= 2_000);
    }
    Ok(())
}

#[test]
fn a_receipt_cut_by_the_window_keeps_its_request() -> Fallible {
    let rec = Recorder::new(Limits {
        max_age: 1_000,
        full_every: 200,
        ..Limits::default()
    });
    let mut dispatcher = Dispatcher::new();
    dispatcher.add_sink(Box::new(rec.sink()));
    let mut counter = Counter { n: 0 };
    rec.snapshot(0, &look(&counter));
    let run = Request::observe(&look(&counter), Actor::Human, "job", "run", None)?;
    let job = dispatcher.dispatch(&mut counter, run, 0);
    for now in (100..5_000).step_by(100) {
        counter.n += 1;
        rec.snapshot(now, &look(&counter));
    }
    dispatcher.complete(job, Outcome::default(), 5_000)?;

    let session = rec.snapshot_window();
    let receipts = session.receipts_for(&id("job"), 5_000);
    let [cut] = receipts.as_slice() else {
        return Err(format!("expected one receipt, got {receipts:?}").into());
    };
    assert!(!cut.whole, "its first stages went with the window");
    let names: Vec<&str> = cut.receipt.stages.iter().map(|(s, _)| s.name()).collect();
    assert_eq!(names, ["completed"]);
    assert_eq!(cut.receipt.request.target, id("job"));
    assert_eq!(cut.receipt.id, job);
    Ok(())
}
