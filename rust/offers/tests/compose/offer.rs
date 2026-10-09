//! What the composer offers the person and what it keeps of the answers
//! (task-70 phase 3): only what the app admits, nothing installed without
//! `accept`, a declined sequence never again in the session, a composite
//! named and labelled from the app's own declarations.

use offers::compose::{Composer, Composite, Shape, Target};
use accord::ActionName;

use super::desk::{Bench, person};

/// The desk heard by a composer that offers after two runs, and two
/// documents reviewed, approved and published by a person.
fn repeated() -> (Bench, Composer) {
    let mut bench = Bench::new(4);
    let composer = Composer::new(2);
    bench.dispatcher.add_sink(Box::new(composer.listener()));
    bench.by_hand(&person(), 1);
    bench.by_hand(&person(), 2);
    (bench, composer)
}

fn name(text: &str) -> ActionName {
    ActionName::parse(text).unwrap()
}

#[test]
fn the_app_offers_only_what_it_admits() {
    let (_, mut composer) = repeated();
    assert!(composer.offer(|_| false).is_none());
    let found = composer
        .offer(|shape| shape.collection().is_some())
        .cloned();
    assert_eq!(found.map(|f| f.repeats), Some(2));
    assert!(composer.installed().is_empty(), "offered, not installed");
}

#[test]
fn declined_it_is_not_offered_again() {
    let (mut bench, mut composer) = repeated();
    assert!(composer.offer(|_| true).is_some());
    let shape = composer.decline().expect("declined");
    assert_eq!(shape.steps.len(), 3);
    assert!(composer.offered().is_none());
    bench.by_hand(&person(), 3);
    assert!(
        composer.offer(|_| true).is_none(),
        "the same sequence, again"
    );
    assert!(composer.decline().is_none(), "nothing to decline");
}

#[test]
fn accepted_it_is_named_from_the_app_and_not_offered_again() {
    let (mut bench, mut composer) = repeated();
    composer.offer(|_| true);
    let app = bench.desk.snapshot();
    let composite = composer.accept(&app).expect("installed").clone();
    assert_eq!(composite.name, name("review_then_approve_then_publish"));
    assert_eq!(composite.label, "review, then approve, then publish");
    assert!(matches!(
        &composite.shape.steps[0].target,
        Target::Item(base) if base.as_str() == "desk.doc"
    ));
    assert!(composer.offered().is_none());
    bench.by_hand(&person(), 3);
    assert!(composer.offer(|_| true).is_none(), "installed already");
    assert!(composer.accept(&app).is_err(), "nothing is offered");
}

#[test]
fn a_name_taken_gets_a_number() {
    let (bench, mut composer) = repeated();
    composer.offer(|_| true);
    let shape: Shape = composer.offered().expect("offered").shape.clone();
    let app = bench.desk.snapshot();
    let taken = [name("review_then_approve_then_publish")];
    let again = Composite::new(shape, &app, &taken).expect("named");
    assert_eq!(again.name, name("review_then_approve_then_publish_2"));
}
