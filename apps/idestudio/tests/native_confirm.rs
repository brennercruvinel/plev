//! The platform's confirmation in the modal's place (task-76 item 8),
//! against a real repository and the real git worker, with the fake
//! dialogs of `engine::platform::alerts` (no window, no native panel):
//! an agent's discard waits for a person; with the option on, the
//! workbench asks the fake with the modal's own words and no modal opens;
//! confirmed, the discard runs after the second check and its receipt
//! says a person confirmed it; declined, nothing runs and the receipt says
//! a person declined; a target without a native dialog falls back to the
//! drawn modal. The session recorded meanwhile holds the person's answer
//! as the receipt's stage, with no input (no click reached the window),
//! and the inspector reads who answered from it. With the option off the
//! modal opens, as `human_agent.rs` proves click by click.

// shared with the other ide tests: this binary uses the rig and a few
// helpers, not every one.
#[allow(dead_code)]
mod support;

use std::sync::{Arc, Mutex};

use accord::{Actor, Rejection, RequestId, SemError, Stage, Value};
use engine::runtime::platform::alerts::{Alerts, Dialog, FakeAlerts};
use idestudio::capability::Workbench;
use idestudio::interfaces::window::record::Recording;
use record::{Kind, Limits, Session};
use replay::{Ending, Inspector};
use support::{Rig, agent};

type Shared = Arc<Mutex<FakeAlerts>>;

/// The rig's workbench asking the platform (the shared fake here) instead
/// of drawing the modal; the test keeps the other handle.
fn natively(rig: &mut Rig, fake: FakeAlerts) -> Shared {
    let shared = Arc::new(Mutex::new(fake));
    let alerts: Box<dyn Alerts> = Box::new(shared.clone());
    rig.bench.confirm_natively(alerts, None);
    assert!(rig.bench.confirms_natively());
    shared
}

fn confirmed_by(rig: &Rig, id: RequestId) -> Option<Actor> {
    rig.receipt(id).stages.iter().find_map(|(s, _)| match s {
        Stage::Confirmed { by } => Some(by.clone()),
        _ => None,
    })
}

fn declined_by(rig: &Rig, id: RequestId) -> Option<Actor> {
    match rig.receipt(id).stage() {
        Some(Stage::Rejected(Rejection::Declined { by })) => Some(by.clone()),
        _ => None,
    }
}

#[test]
fn a_confirmed_discard_runs_and_a_declined_one_does_not_with_the_modals_words_and_no_modal() {
    let mut rig = Rig::new(&["README", "new.txt"], &[]);
    let readme = rig.dir.path().join("README");
    let fake = natively(&mut rig, FakeAlerts::new().will_confirm().will_decline());

    // The agent asks to discard README; the person confirms in the dialog.
    let first = rig.ask(agent(), "changes.file[README]", "discard", None);
    assert!(!rig.bench.workspace().modal_open(), "no modal was drawn");
    assert!(
        rig.receipt(first).stage() != Some(&Stage::AwaitingConfirmation),
        "answered before the dispatch returned: {:?}",
        rig.stages(first)
    );
    rig.settle(first);
    assert_eq!(
        rig.stages(first),
        [
            "requested",
            "awaiting_confirmation",
            "confirmed",
            "executing",
            "completed"
        ]
    );
    assert_eq!(confirmed_by(&rig, first), Some(Actor::Human));
    assert_eq!(std::fs::read_to_string(&readme).unwrap(), "readme\n");
    assert_eq!(
        rig.value("changes:files"),
        Value::List(vec!["new.txt".into()])
    );

    // The agent asks to discard new.txt; the person declines in the dialog.
    let second = rig.ask(agent(), "changes.file[new.txt]", "discard", None);
    assert!(!rig.bench.workspace().modal_open());
    assert_eq!(
        rig.stages(second),
        ["requested", "awaiting_confirmation", "rejected"]
    );
    assert_eq!(declined_by(&rig, second), Some(Actor::Human));
    assert_eq!(
        std::fs::read_to_string(rig.dir.path().join("new.txt")).unwrap(),
        "new.txt\n",
        "nothing ran"
    );
    assert_eq!(
        rig.notices(),
        ["Declined: discard of new.txt by test-agent did not run"]
    );
    assert!(
        matches!(
            rig.bench.confirm(second, Actor::Human),
            Err(SemError::IllegalTransition { .. })
        ),
        "answered once"
    );

    // What the platform was asked: the modal's title, body and button,
    // destructive as the discard's policy says, no parent in a test.
    let asked = fake.lock().unwrap().take_asked();
    assert_eq!(asked.len(), 2);
    let words = |dialog: &Dialog| match dialog {
        Dialog::Confirm(c) => (
            c.title.clone(),
            c.body.clone(),
            c.confirm_label.clone(),
            c.cancel_label.clone(),
            c.destructive,
        ),
        Dialog::Alert(a) => panic!("a confirmation, not {a:?}"),
    };
    assert_eq!(
        words(&asked[0].dialog),
        (
            "Discard changes?".to_owned(),
            "test-agent asks to discard all changes to README. This cannot be undone.".to_owned(),
            "Discard".to_owned(),
            "Cancel".to_owned(),
            true,
        )
    );
    assert_eq!(
        words(&asked[1].dialog).1,
        "test-agent asks to discard all changes to new.txt. This cannot be undone."
    );
    assert!(asked.iter().all(|a| !a.with_parent), "no window here");
    assert_eq!(fake.lock().unwrap().pending(), 0);
}

/// An agent's question (`--ask`, task-92) reaches the person by the same
/// dialog, quoted in the third person, and a non-destructive action asks
/// without the warning; the agent itself never answers.
#[test]
fn an_agents_question_is_asked_in_the_dialog_and_the_agent_never_answers_it() {
    let mut rig = Rig::new(&["README", "new.txt"], &[]);
    let fake = natively(&mut rig, FakeAlerts::new().will_confirm());
    let question = "new.txt is part of the task: stage it?";
    let req = rig
        .observe(agent(), "changes.file[new.txt]", "stage", None)
        .asking(question);
    let id = rig.bench.dispatch(req);
    rig.settle(id);
    assert_eq!(confirmed_by(&rig, id), Some(Actor::Human));
    assert_eq!(rig.value("changes:staged"), support::set(&["new.txt"]));
    assert!(
        matches!(
            rig.bench.confirm(id, agent()),
            Err(SemError::IllegalTransition { .. })
        ),
        "a finished request; and an agent never confirms"
    );

    let asked = fake.lock().unwrap().take_asked();
    let Dialog::Confirm(c) = &asked[0].dialog else {
        panic!("a confirmation");
    };
    assert_eq!(c.title, "Question from test-agent");
    assert_eq!(
        c.body,
        format!("test-agent asks the person: \"{question}\" If allowed, it will stage new.txt.")
    );
    assert_eq!(c.confirm_label, "Allow");
    assert!(!c.destructive, "a stage is not destructive");
    let projected = format!("{:?}", rig.look());
    assert!(!projected.contains("stage it?"), "not in the projection");
}

/// On a target without a native dialog (`Unsupported`) the option falls
/// back to the drawn modal, which asks the same thing and answers the
/// same way.
#[test]
fn without_a_native_dialog_the_modal_is_drawn_instead() {
    let mut rig = Rig::new(&["README"], &[]);
    let fake = natively(&mut rig, FakeAlerts::new().will_refuse());
    let id = rig.ask(agent(), "changes.file[README]", "discard", None);
    assert_eq!(rig.stages(id), ["requested", "awaiting_confirmation"]);
    assert!(rig.bench.workspace().modal_open(), "the modal asks instead");
    assert_eq!(
        fake.lock().unwrap().asked().len(),
        1,
        "the platform was tried"
    );
    assert!(
        matches!(
            rig.bench.confirm(id, agent()),
            Err(SemError::ConfirmByMachine { .. })
        ),
        "an agent never confirms"
    );
    rig.bench
        .decline(id, Actor::Human)
        .expect("the person declines in the modal");
    assert!(!rig.bench.workspace().modal_open());
    assert_eq!(declined_by(&rig, id), Some(Actor::Human));
}

/// The option off (the default): the modal, as today, and no dialog.
#[test]
fn with_the_option_off_the_modal_opens_as_today() {
    let mut rig = Rig::new(&["README"], &[]);
    assert!(!rig.bench.confirms_natively());
    assert!(!Workbench::new(1280.0, 800.0).confirms_natively());
    let id = rig.ask(agent(), "changes.file[README]", "discard", None);
    assert_eq!(rig.stages(id), ["requested", "awaiting_confirmation"]);
    assert!(rig.bench.workspace().modal_open());
}

/// The session recorded with the option on: the receipt holds the
/// person's answer as its stage (`confirmed { by: human }`, or the
/// decline), no input is recorded (no click reached the window), and the
/// inspector reads who answered from the stages.
#[test]
fn the_recorded_session_says_a_person_answered_and_holds_no_click() {
    let mut rig = Rig::new(&["README", "new.txt"], &[]);
    let dir = tempfile::TempDir::new().expect("a temp dir");
    let mut recording = Recording::new(dir.path().join("sessions"), Limits::default());
    recording.attach(&mut rig.bench);
    let _fake = natively(&mut rig, FakeAlerts::new().will_confirm().will_decline());

    let confirmed = rig.ask(agent(), "changes.file[README]", "discard", None);
    rig.settle(confirmed);
    recording.state(&rig.bench);
    let declined = rig.ask(agent(), "changes.file[new.txt]", "discard", None);
    recording.state(&rig.bench);

    let (path, _) = recording.save().expect("the window is saved");
    let session = Session::open(&path).expect("the session opens");
    let (_, end) = session.span().expect("events");
    assert!(
        !session
            .events()
            .iter()
            .any(|e| matches!(e.kind, Kind::Input { .. })),
        "no input: the dialog is the platform's, no click reached the window"
    );
    let receipts = session.receipts(end);
    let origin = |id: RequestId| {
        receipts
            .iter()
            .find(|r| r.receipt.id == id)
            .map(|r| r.origin)
            .expect("recorded")
    };

    let inspector = Inspector::new(&session);
    let why = inspector
        .why(origin(confirmed), confirmed)
        .expect("it reads")
        .expect("in the session");
    assert_eq!(why.request.actor, agent());
    assert!(
        why.stages
            .iter()
            .any(|s| s.stage == Stage::Confirmed { by: Actor::Human }),
        "who answered: {:?}",
        why.stages
    );
    assert!(matches!(why.ending, Ending::Completed { .. }));
    assert!(why.between.inputs.is_empty());

    let why = inspector
        .why(origin(declined), declined)
        .expect("it reads")
        .expect("in the session");
    let Ending::Rejected { rejection, .. } = &why.ending else {
        panic!("declined: {:?}", why.ending);
    };
    assert_eq!(*rejection, Rejection::Declined { by: Actor::Human });
}
