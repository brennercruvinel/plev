//! A composite runs as chained requests (task-70 phase 3): every step a
//! request of the composite's actor through the desk's own dispatcher,
//! each checked against the state the step before left. All of them
//! complete; or one the step before turned off stops the chain; or one
//! whose state another actor moved while the step before ran stops it,
//! stale, naming who acted; the composite's receipt says which step and
//! why, and what ran stays run. An agent never runs a composite one of
//! whose steps is kept from it.

use accord::{Actor, Rejection, Request, RequestId, Stage, Value};
use offers::compose::Composer;

use super::desk::{Bench, agent, person};

/// A desk of `docs` documents where `actor` did `actions` on docs 1 and
/// 2, and a person made them one action.
fn composed(docs: u64, actor: &Actor, actions: &[&str]) -> (Bench, String) {
    let mut bench = Bench::new(docs);
    let mut composer = Composer::new(2);
    bench.dispatcher.add_sink(Box::new(composer.listener()));
    for doc in [1, 2] {
        for action in actions {
            bench.act(actor.clone(), &format!("desk.doc[{doc}]"), action);
            if *action == "fetch" {
                bench.fetched();
            }
        }
    }
    assert!(composer.offer(|_| true).is_some(), "offered");
    let app = bench.desk.snapshot();
    let name = composer.accept(&app).expect("installed").name.to_string();
    bench.desk.composer = Some(composer);
    (bench, name)
}

fn stages(bench: &Bench, id: RequestId) -> Vec<&'static str> {
    let receipt = bench.dispatcher.receipt(id).expect("a receipt");
    receipt.stages.iter().map(|(s, _)| s.name()).collect()
}

fn failed(bench: &Bench, id: RequestId) -> String {
    match bench.stage(id) {
        Stage::Failed { error } => error,
        other => panic!("failed: {other:?}"),
    }
}

fn steps_of(bench: &Bench, id: RequestId) -> Vec<RequestId> {
    let Stage::Completed(outcome) = bench.stage(id) else {
        panic!("completed: {:?}", bench.stage(id));
    };
    assert_eq!(outcome.result.get("steps"), Some(&Value::Int(3)));
    match outcome.result.get("requests") {
        Some(Value::List(ids)) => ids.iter().map(|t| RequestId::parse(t).unwrap()).collect(),
        other => panic!("the requests: {other:?}"),
    }
}

#[test]
fn every_step_runs_as_a_request_of_its_actor_and_the_composite_completes() {
    let (mut bench, name) = composed(4, &person(), &["review", "approve", "publish"]);
    bench.desk.ran.clear();
    let composite = bench.act(person(), "desk.doc[3]", &name);
    assert_eq!(
        stages(&bench, composite),
        ["requested", "accepted", "executing", "completed"]
    );
    let steps = steps_of(&bench, composite);
    for (step, action) in steps.iter().zip(["review", "approve", "publish"]) {
        let receipt = bench.dispatcher.receipt(*step).expect("the step's receipt");
        assert_eq!(receipt.request.actor, person());
        assert_eq!(receipt.request.target.as_str(), "desk.doc[3]");
        assert_eq!(receipt.request.action.as_str(), action);
        assert!(matches!(receipt.stage(), Some(Stage::Completed(_))));
    }
    let ran: Vec<&str> = bench.desk.ran.iter().map(|(_, a)| a.as_str()).collect();
    assert_eq!(ran, ["review", "approve", "publish"]);
    assert_eq!(bench.desk.status(3), "published");
    // run again on the next document: still no offer, the steps are the
    // composite's and not the person's own
    bench.act(person(), "desk.doc[4]", &name);
    assert_eq!(bench.desk.status(4), "published");
    let composer = bench.desk.composer.as_mut().expect("the composer");
    assert!(composer.offer(|_| true).is_none());
}

#[test]
fn a_step_the_one_before_it_left_off_stops_the_chain_and_says_why() {
    let (mut bench, name) = composed(3, &person(), &["review", "approve", "publish"]);
    // document 3 was approved already: its review runs, its approve is off
    bench.act(agent("bot"), "desk.doc[3]", "approve");
    bench.desk.ran.clear();
    let composite = bench.act(person(), "desk.doc[3]", &name);
    let why = failed(&bench, composite);
    assert!(
        why.starts_with("stopped at step 2 of 3, approve on desk.doc[3]"),
        "{why}"
    );
    assert!(
        why.contains("refused: approve on desk.doc[3] is off"),
        "{why}"
    );
    assert!(why.contains("step 1 ran and stays"), "{why}");
    // what ran stays run, and nothing after the refusal ran
    let ran: Vec<&str> = bench.desk.ran.iter().map(|(_, a)| a.as_str()).collect();
    assert_eq!(ran, ["review"]);
    assert_eq!(bench.desk.docs[2].seen, 1);
    assert_eq!(bench.desk.status(3), "approved");
}

#[test]
fn a_step_whose_state_moved_while_the_one_before_ran_is_stale_and_names_who() {
    let (mut bench, name) = composed(3, &person(), &["fetch", "approve"]);
    bench.desk.ran.clear();
    let composite = bench.act(person(), "desk.doc[3]", &name);
    // the fetch started: the chain waits for it, and the composite with it
    assert_eq!(bench.stage(composite), Stage::Executing);
    assert_eq!(bench.desk.chains.len(), 1);
    // meanwhile an agent approves the document
    bench.act(agent("bot"), "desk.doc[3]", "approve");
    // the fetch ends: the approve comes, observed when the fetch began
    bench.fetched();
    assert!(bench.desk.chains.is_empty(), "the chain ended");
    let why = failed(&bench, composite);
    assert!(
        why.starts_with("stopped at step 2 of 2, approve on desk.doc[3]"),
        "{why}"
    );
    assert!(
        why.contains(r#"stale: desk.doc[3]:status was "draft", is "approved""#),
        "{why}"
    );
    assert!(
        why.contains("(agent:bot ran approve on desk.doc[3])"),
        "{why}"
    );
    assert!(why.contains("step 1 ran and stays"), "{why}");
    // the step's own receipt holds the refusal whole
    let step = RequestId::parse(&(composite.as_u64() + 3).to_string()).unwrap();
    let receipt = bench.dispatcher.receipt(step).expect("the approve step");
    assert_eq!(receipt.request.actor, person());
    assert!(matches!(
        receipt.stage(),
        Some(Stage::Rejected(Rejection::Stale { acted, .. })) if acted.len() == 1
    ));
    // the agent's approve stands; the person's composite ran the fetch only
    let ran: Vec<&str> = bench.desk.ran.iter().map(|(_, a)| a.as_str()).collect();
    assert_eq!(
        ran,
        ["fetch", "approve"],
        "the fetch, then the agent's approve"
    );
    assert_eq!(bench.desk.status(3), "approved");
}

#[test]
fn an_agent_never_runs_a_composite_with_a_step_kept_from_it() {
    let (mut bench, name) = composed(3, &person(), &["review", "sign"]);
    bench.desk.ran.clear();
    let composite = bench.act(agent("bot"), "desk.doc[3]", &name);
    assert_eq!(
        bench.stage(composite),
        Stage::Rejected(Rejection::NotAuthorized {
            actor: agent("bot")
        })
    );
    assert!(bench.desk.ran.is_empty(), "no step ran");
    assert!(bench.desk.chains.is_empty());
    // the person runs it
    let composite = bench.act(person(), "desk.doc[3]", &name);
    assert!(matches!(bench.stage(composite), Stage::Completed(_)));
    let ran: Vec<&str> = bench.desk.ran.iter().map(|(_, a)| a.as_str()).collect();
    assert_eq!(ran, ["review", "sign"]);
}

#[test]
fn a_composite_asked_of_a_node_it_is_not_declared_on_is_unknown() {
    let (bench, name) = composed(3, &person(), &["review", "approve", "publish"]);
    let seen = bench.desk.snapshot();
    let asked = Request::observe(&seen, person(), "desk.refresh", name.as_str(), None);
    assert!(asked.is_err(), "not declared on the refresh");
    assert!(bench.desk.chains.is_empty());
}

/// A chain on its own, over a request that stands for the composite (a
/// fetch, which starts and stays open): a step that asks a person to
/// confirm it stops the chain, stays waiting for the person, and the
/// composite says so; nobody confirms for the person in the middle.
#[test]
fn a_step_that_waits_for_a_person_stops_the_chain_and_stays_waiting() {
    use accord::{ActionName, SemId};
    use offers::compose::Chain;

    let mut bench = Bench::new(3);
    let stand_in = bench.act(person(), "desk.doc[1]", "fetch");
    assert_eq!(bench.stage(stand_in), Stage::Executing);
    let doc = SemId::parse("desk.doc[2]").unwrap();
    let calls = ["review", "shred"]
        .map(|a| (doc.clone(), ActionName::parse(a).unwrap()))
        .to_vec();
    let mut chain = Chain::new(stand_in, person(), calls);
    bench.now += 10;
    let ended = chain.advance(&mut bench.dispatcher, &mut bench.desk, bench.now);
    assert!(ended);
    let why = failed(&bench, stand_in);
    assert!(
        why.starts_with("stopped at step 2 of 2, shred on desk.doc[2]"),
        "{why}"
    );
    assert!(why.contains("waits for a person to confirm it"), "{why}");
    let shred = chain.asked()[1];
    assert_eq!(bench.stage(shred), Stage::AwaitingConfirmation);
    assert_eq!(bench.desk.status(2), "draft", "not shredded");
}
