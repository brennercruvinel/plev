//! Zones: nesting and depth, self time, threads, the per-thread cap, the
//! off path, and a zone still open at a take.
//!
//! Zones are process wide, so every test holds `zones::test_lock` and
//! reads only the records of its own thread (or of the threads it
//! spawned): another test's frame tail may record on its own thread
//! while zones are on here.

use std::sync::MutexGuard;
use std::time::Duration;

use super::super::zones::{self, MAX_ZONES_PER_THREAD, ThreadZones, ZoneFrame, ZoneRecord};

/// Zones on for one test, off again (and drained) when it ends.
struct On {
    _lock: MutexGuard<'static, ()>,
}

impl On {
    fn new() -> Self {
        let lock = zones::test_lock();
        zones::enable(false);
        zones::enable(true);
        let _ = zones::take();
        On { _lock: lock }
    }
}

impl Drop for On {
    fn drop(&mut self) {
        zones::enable(false);
    }
}

fn this_thread() -> String {
    std::thread::current()
        .name()
        .map(str::to_owned)
        .unwrap_or_default()
}

/// The zones `frame` holds for the thread named `name`.
fn of<'a>(frame: &'a ZoneFrame, name: &str) -> &'a [ZoneRecord] {
    frame
        .threads
        .iter()
        .find(|t| t.name == name)
        .map_or(&[], |t| t.zones.as_slice())
}

fn mine(frame: &ZoneFrame) -> Vec<ZoneRecord> {
    of(frame, &this_thread()).to_vec()
}

fn record(name: &'static str, start_ns: u64, nanos: u64, depth: u32) -> ZoneRecord {
    ZoneRecord {
        name,
        start_ns,
        nanos,
        depth,
    }
}

#[test]
fn nested_zones_close_inner_first_with_their_depth() {
    let _on = On::new();
    {
        crate::zone!("outer");
        {
            crate::zone!("inner");
            crate::zone!("innermost");
        }
        crate::zone!("second");
    }
    let zones = mine(&zones::take());
    let seen: Vec<_> = zones.iter().map(|z| (z.name, z.depth)).collect();
    assert_eq!(
        seen,
        [("innermost", 2), ("inner", 1), ("second", 1), ("outer", 0)]
    );
    let (inner, outer) = (zones[1], zones[3]);
    assert!(outer.start_ns <= inner.start_ns && inner.end_ns() <= outer.end_ns());
    assert!(
        zones[2].start_ns >= inner.end_ns(),
        "second opens after inner"
    );
}

#[test]
fn self_time_subtracts_only_the_zones_directly_inside() {
    let frame = ZoneFrame {
        threads: vec![ThreadZones {
            name: "t".into(),
            // outer 0..100 holds child 10..60 (which holds leaf 20..50)
            // and child 70..90; closed inner first
            zones: vec![
                record("leaf", 20, 30, 2),
                record("child", 10, 50, 1),
                record("child", 70, 20, 1),
                record("outer", 0, 100, 0),
            ],
        }],
        dropped: 0,
    };
    let stats = frame.stats();
    let get = |name: &str| {
        let found = stats.iter().find(|s| s.name == name).copied();
        found.unwrap_or_else(|| panic!("no {name} in {stats:?}"))
    };
    let outer = get("outer");
    assert_eq!((outer.total_ns, outer.self_ns), (100, 30));
    let child = get("child");
    assert_eq!((child.calls, child.total_ns, child.self_ns), (2, 70, 40));
    assert_eq!((child.min_ns, child.max_ns), (20, 50));
    assert_eq!(get("leaf").self_ns, 30);
    let order: Vec<_> = stats.iter().map(|s| s.name).collect();
    assert_eq!(order, ["child", "leaf", "outer"], "most self time first");
}

#[test]
fn stats_sum_a_name_across_threads_and_a_missing_parent_subtracts_nothing() {
    let frame = ZoneFrame {
        threads: vec![
            ThreadZones {
                name: "a".into(),
                zones: vec![record("work", 0, 40, 0)],
            },
            ThreadZones {
                name: "b".into(),
                // its parent was still open at the take
                zones: vec![record("work", 5, 60, 1)],
            },
        ],
        dropped: 0,
    };
    let stats = frame.stats();
    assert_eq!(stats.len(), 1);
    let work = stats[0];
    assert_eq!((work.calls, work.total_ns, work.self_ns), (2, 100, 100));
    assert_eq!((work.min_ns, work.max_ns), (40, 60));
}

#[test]
fn every_thread_lands_in_the_take_under_its_name() {
    let _on = On::new();
    {
        crate::zone!("main side");
    }
    let worker = std::thread::Builder::new()
        .name("zones-test-worker".into())
        .spawn(|| {
            crate::zone!("worker side");
        })
        .map(|h| h.join());
    assert!(matches!(worker, Ok(Ok(()))), "the worker ran");
    let frame = zones::take();
    let names: Vec<_> = mine(&frame).iter().map(|z| z.name).collect();
    assert_eq!(names, ["main side"]);
    let theirs: Vec<_> = of(&frame, "zones-test-worker")
        .iter()
        .map(|z| z.name)
        .collect();
    assert_eq!(theirs, ["worker side"]);
    assert!(
        mine(&zones::take()).is_empty(),
        "a take starts the next frame"
    );
}

#[test]
fn a_thread_keeps_at_most_the_cap_and_counts_the_rest() {
    let _on = On::new();
    for _ in 0..MAX_ZONES_PER_THREAD + 5 {
        crate::zone!("many");
    }
    let frame = zones::take();
    assert_eq!(mine(&frame).len(), MAX_ZONES_PER_THREAD);
    assert!(frame.dropped >= 5, "{}", frame.dropped);
    assert_eq!(zones::take().dropped, 0, "a take resets the count");
}

#[test]
fn off_records_nothing() {
    let _lock = zones::test_lock();
    zones::enable(false);
    assert!(!zones::is_enabled());
    {
        crate::zone!("off");
        let _also = zones::enter("off too");
    }
    zones::enable(true);
    let frame = zones::take();
    zones::enable(false);
    assert!(mine(&frame).is_empty(), "{frame:?}");
}

#[test]
fn the_off_path_takes_no_lock() {
    let _lock = zones::test_lock();
    zones::enable(false);
    // with the registry held here, a zone that took any lock of the
    // module would block the thread below forever
    let (tx, rx) = std::sync::mpsc::channel();
    let finished = zones::holding_the_registry(|| {
        let spawned = std::thread::Builder::new()
            .name("zones-test-off".into())
            .spawn(move || {
                for _ in 0..1_000 {
                    crate::zone!("off");
                }
                let _ = tx.send(());
            });
        assert!(spawned.is_ok());
        rx.recv_timeout(Duration::from_secs(10)).is_ok()
    });
    assert!(finished, "a zone with zones off waited on the registry");
}

#[test]
fn turning_off_drops_what_was_recorded() {
    let on = On::new();
    {
        crate::zone!("dropped");
    }
    zones::enable(false);
    zones::enable(true);
    assert!(mine(&zones::take()).is_empty());
    drop(on);
}

#[test]
fn a_zone_open_at_the_take_lands_in_a_later_one() {
    let _on = On::new();
    let open = zones::enter("open");
    {
        crate::zone!("closed");
    }
    let first = mine(&zones::take());
    let names: Vec<_> = first.iter().map(|z| z.name).collect();
    assert_eq!(names, ["closed"]);
    drop(open);
    let later = mine(&zones::take());
    assert_eq!(later.len(), 1);
    assert_eq!((later[0].name, later[0].depth), ("open", 0));
    assert!(later[0].start_ns <= first[0].start_ns, "it opened first");
}
