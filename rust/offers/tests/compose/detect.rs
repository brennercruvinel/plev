//! The detector over a synthetic history (task-70 phase 3): the desk's
//! own dispatcher, heard live by a `Listener`, as the Silences page hears
//! the gallery's. Repeated N times it finds the sequence; another actor
//! in the middle, another order, a refused step, a parameter, a wait for
//! a person, and it does not.

use std::sync::{Arc, Mutex};

use accord::{ActionName, SemId, Stage, Value};
use offers::compose::{Detector, Found, Listener, Shape, Step, Target};

use super::desk::{Bench, SECRET, agent, person};

fn sid(text: &str) -> SemId {
    SemId::parse(text).unwrap()
}

fn step(action: &str, target: Target) -> Step {
    Step {
        action: ActionName::parse(action).unwrap(),
        target,
    }
}

fn item(action: &str) -> Step {
    step(action, Target::Item(sid("desk.doc")))
}

fn node(action: &str, id: &str) -> Step {
    step(action, Target::Node(sid(id)))
}

/// The desk with `docs` documents, heard by a detector that wants
/// `repeats` in a row.
fn heard(docs: u64, repeats: usize) -> (Bench, Arc<Mutex<Detector>>) {
    let mut bench = Bench::new(docs);
    let detector = Arc::new(Mutex::new(Detector::new(repeats)));
    bench
        .dispatcher
        .add_sink(Box::new(Listener::new(detector.clone())));
    (bench, detector)
}

fn found(detector: &Arc<Mutex<Detector>>) -> Option<Found> {
    detector.lock().unwrap().found()
}

fn thesis() -> Shape {
    Shape {
        steps: vec![item("review"), item("approve"), item("publish")],
    }
}

#[test]
fn a_sequence_repeated_n_times_on_items_is_found() {
    let (mut bench, detector) = heard(4, 3);
    bench.by_hand(&person(), 1);
    bench.by_hand(&person(), 2);
    assert_eq!(found(&detector), None, "twice is not three times");
    bench.by_hand(&person(), 3);
    let got = found(&detector).expect("found");
    assert_eq!(got.actor, person());
    assert_eq!(got.shape, thesis());
    assert_eq!(got.repeats, 3);
    assert_eq!(bench.desk.status(3), "published");
    bench.by_hand(&person(), 4);
    assert_eq!(found(&detector).map(|f| f.repeats), Some(4));
}

#[test]
fn a_node_that_is_the_same_every_run_stays_exact() {
    let (mut bench, detector) = heard(3, 2);
    for doc in [1, 2] {
        bench.act(person(), "desk.refresh", "refresh");
        bench.act(person(), &format!("desk.doc[{doc}]"), "review");
    }
    let got = found(&detector).expect("found");
    assert_eq!(
        got.shape.steps,
        vec![node("refresh", "desk.refresh"), item("review")]
    );
}

#[test]
fn another_actor_in_the_middle_is_not_a_repetition() {
    let (mut bench, detector) = heard(4, 2);
    bench.by_hand(&person(), 1);
    bench.act(agent("bot"), "desk.refresh", "refresh");
    bench.by_hand(&person(), 2);
    assert_eq!(found(&detector), None);
    // the same actor all along, but an agent's runs are not a person's
    let (mut bench, detector) = heard(4, 2);
    bench.by_hand(&person(), 1);
    bench.by_hand(&agent("bot"), 2);
    assert_eq!(found(&detector), None);
}

#[test]
fn another_order_is_not_a_repetition() {
    let (mut bench, detector) = heard(4, 2);
    bench.act(person(), "desk.doc[1]", "review");
    bench.act(person(), "desk.doc[1]", "approve");
    bench.act(person(), "desk.doc[2]", "approve");
    bench.act(person(), "desk.doc[2]", "review");
    assert_eq!(found(&detector), None);
}

#[test]
fn a_refused_step_is_not_a_repetition() {
    let (mut bench, detector) = heard(4, 2);
    bench.by_hand(&person(), 1);
    bench.act(person(), "desk.doc[2]", "review");
    // published before it was approved: refused, the action is off
    let early = bench.act(person(), "desk.doc[2]", "publish");
    assert!(matches!(bench.stage(early), Stage::Rejected(_)));
    bench.act(person(), "desk.doc[2]", "approve");
    bench.act(person(), "desk.doc[2]", "publish");
    assert_eq!(found(&detector), None, "the refusal cuts the second run");
    // the same run again, whole, and it is found from there on
    bench.by_hand(&person(), 3);
    assert_eq!(found(&detector), None, "one whole run since the cut");
    bench.by_hand(&person(), 4);
    assert_eq!(found(&detector).map(|f| f.shape), Some(thesis()));
}

#[test]
fn two_items_in_one_run_are_not_one_item() {
    let (mut bench, detector) = heard(4, 2);
    for (a, b) in [(1, 2), (3, 4)] {
        bench.act(person(), &format!("desk.doc[{a}]"), "review");
        bench.act(person(), &format!("desk.doc[{b}]"), "approve");
    }
    assert_eq!(found(&detector), None);
}

#[test]
fn one_action_repeated_is_no_sequence() {
    let (mut bench, detector) = heard(4, 2);
    for doc in 1..=4 {
        bench.act(person(), &format!("desk.doc[{doc}]"), "review");
    }
    assert_eq!(found(&detector), None);
}

#[test]
fn a_step_that_ends_later_enters_in_the_order_it_was_asked() {
    let (mut bench, detector) = heard(3, 2);
    for doc in [1, 2] {
        let target = format!("desk.doc[{doc}]");
        let fetch = bench.act(person(), &target, "fetch");
        assert_eq!(bench.stage(fetch), Stage::Executing);
        // approved before the fetch ends: still fetch, then approve
        bench.act(person(), &target, "approve");
        assert_eq!(found(&detector), None, "the fetch holds what came after");
        bench.fetched();
    }
    let got = found(&detector).expect("found");
    assert_eq!(got.shape.steps, vec![item("fetch"), item("approve")]);
}

#[test]
fn a_step_that_waited_for_a_person_is_not_a_step() {
    let (mut bench, detector) = heard(5, 2);
    for doc in [1, 2] {
        let target = format!("desk.doc[{doc}]");
        bench.act(person(), &target, "review");
        let shred = bench.act(person(), &target, "shred");
        assert_eq!(bench.stage(shred), Stage::AwaitingConfirmation);
        bench.now += 10;
        let (desk, d) = (&mut bench.desk, &mut bench.dispatcher);
        d.confirm(desk, shred, person(), bench.now).unwrap();
        assert!(matches!(bench.stage(shred), Stage::Completed(_)));
    }
    assert_eq!(found(&detector), None);
}

/// the privacy promise: a step with a parameter is never a step, and the
/// detector keeps nothing of what was typed, secret or not.
#[test]
fn a_parameter_is_never_kept_nor_a_step() {
    let (mut bench, detector) = heard(2, 2);
    for _ in 0..3 {
        bench.ask(
            person(),
            "desk.code",
            "set_value",
            Some(Value::Text(SECRET.to_owned())),
        );
        bench.act(person(), "desk.submit", "submit");
    }
    assert_eq!(found(&detector), None);
    let kept = format!("{:?}", detector.lock().unwrap());
    assert!(!kept.contains(SECRET), "{kept}");
    assert!(!kept.contains("hunter2"), "{kept}");
}
