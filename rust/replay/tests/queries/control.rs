//! `control`: the node at an instant and every request that touched it up
//! to then, each with its causal chain as the window up to then tells it.

use accord::{Outcome, Value};
use indexmap::IndexMap;
use replay::{Ending, Inspector};

use crate::thesis::record;
use crate::toys::{Lab, agent, id};

type Fallible = Result<(), Box<dyn std::error::Error>>;

#[test]
fn the_button_before_its_first_request_during_and_after() -> Fallible {
    let thesis = record();
    let inspector = Inspector::new(&thesis.session);
    let submit = id("commit.submit");

    let before = inspector.control(&submit, 1_299)?;
    assert_eq!(before.at, 299);
    let node = before.node.as_ref().ok_or("the button is projected")?;
    assert_eq!((node.label.as_str(), node.enabled), ("Commit", true));
    assert!(
        node.source.is_some(),
        "the projection site, for the code link"
    );
    assert!(before.receipts.is_empty());
    assert_eq!(before.build, thesis.session.build().git);

    let refused = inspector.control(&submit, 1_300)?;
    let [stale] = refused.receipts.as_slice() else {
        panic!("one receipt, got {:?}", refused.receipts);
    };
    assert_eq!(stale.id, thesis.stale);
    assert!(matches!(stale.ending, Ending::Rejected { .. }));

    let after = inspector.control(&submit, 1_500)?;
    let ids: Vec<_> = after.receipts.iter().map(|w| w.id).collect();
    assert_eq!(ids, [thesis.stale, thesis.done]);
    assert!(matches!(after.receipts[1].ending, Ending::Completed { .. }));
    let node = after.node.as_ref().ok_or("the button is projected")?;
    assert!(!node.enabled, "nothing is staged after the commit");
    Ok(())
}

#[test]
fn the_list_is_touched_by_every_request_that_reads_it() -> Fallible {
    let thesis = record();
    let inspector = Inspector::new(&thesis.session);
    let changes = inspector.control(&id("changes"), 1_500)?;
    let ids: Vec<_> = changes.receipts.iter().map(|w| w.id).collect();
    assert_eq!(ids, [thesis.unstage, thesis.stale, thesis.done]);
    let node = changes.node.as_ref().ok_or("the list is projected")?;
    assert_eq!(
        node.state.get("staged"),
        Some(&Value::Set(Default::default()))
    );
    Ok(())
}

#[test]
fn a_control_the_state_does_not_hold() -> Fallible {
    let thesis = record();
    let inspector = Inspector::new(&thesis.session);
    let gone = inspector.control(&id("stacks"), 1_500)?;
    assert!(gone.node.is_none() && gone.receipts.is_empty() && !gone.focused);
    let early = inspector.control(&id("commit.submit"), 10)?;
    assert!(early.node.is_none(), "before the first snapshot");
    Ok(())
}

/// a submit that runs on a worker: at an instant between its start and its
/// end, the control shows it still executing; after, completed.
#[test]
fn a_request_still_running_at_the_instant_is_open() -> Fallible {
    let mut lab = Lab::new(&["A", "B"], "fix login", 0);
    lab.git.slow = true;
    let seen = lab.observe(10, agent());
    let submit = lab.act(20, &seen, agent(), "commit.submit", "submit", None);
    lab.git.staged.clear();
    let outcome = Outcome {
        result: IndexMap::from([("commit".to_owned(), Value::Text("abc1234".into()))]),
    };
    lab.dispatcher.complete(submit, outcome, 50)?;
    lab.rec.snapshot(50, &lab.look());
    let session = lab.session();
    let inspector = Inspector::new(&session);
    let button = id("commit.submit");

    let running = inspector.control(&button, 30)?;
    let [why] = running.receipts.as_slice() else {
        panic!("one receipt, got {:?}", running.receipts);
    };
    assert!(matches!(
        why.ending,
        Ending::Open {
            stage: "executing",
            at: 20,
            ..
        }
    ));
    assert_eq!(why.stages.len(), 3);

    let done = inspector.control(&button, 50)?;
    assert!(matches!(
        done.receipts[0].ending,
        Ending::Completed { at: 50, .. }
    ));
    Ok(())
}
