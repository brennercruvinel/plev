//! Native alerts and confirmations answering the same contract (task-76
//! item 8), proven without a platform: an app whose button shows an alert
//! for a person alone (an agent is refused before anything opens, and
//! reads the text the person saw as state), and an app whose destructive
//! action waits for a person and is answered into the dispatcher from the
//! platform's confirmation as `Actor::Human`, confirmed or declined, with
//! the receipt the drawn modal would leave.

use std::sync::{Arc, Mutex};

use accord::{
    Actor, Actors, Confirm as Policy, Dispatcher, Execution, Operable, Outcome, Policy as Care,
    Rejection, Request, RequestId, Role, SemId, Semantics, Snapshot, SnapshotBuilder, Stage, Value,
};
use engine::runtime::platform::alerts::{
    Alert, AlertKind, Alerts, Answered, Confirm, Dialog, FakeAlerts, Level, NoAlerts,
};

fn sid(text: &str) -> SemId {
    SemId::parse(text).expect("a valid id")
}

/// Who may ask for a native panel: a person, at the screen or through
/// assistive technology, and a test over the fake; never an agent.
const PERSON_ONLY: Actors = Actors {
    human: true,
    agent: false,
    assistive: true,
    test: true,
};

/// An export with two capabilities: `warn` shows the platform's alert
/// (a person's), `clear` throws the export away after a confirmation
/// (`Confirm::Always`, destructive). The last alert shown is state.
struct Export {
    items: u32,
    alerted: String,
    alerts: Box<dyn Alerts>,
}

impl Export {
    fn new(alerts: Box<dyn Alerts>) -> Self {
        Self {
            items: 3,
            alerted: String::new(),
            alerts,
        }
    }

    fn snapshot(&self) -> Snapshot {
        let mut b = SnapshotBuilder::new();
        self.project(&mut b);
        b.finish().expect("the projection validates")
    }

    fn warning(&self) -> Alert {
        Alert::warning(
            "Export",
            format!("{} items kept in memory; nothing was written.", self.items),
        )
    }
}

impl Semantics for Export {
    fn project(&self, b: &mut SnapshotBuilder) {
        b.node(sid("export"), Role::Region, "export")
            .state("items", Value::Int(i64::from(self.items)))
            .state("alert", Value::Text(self.alerted.clone()));
        let mut warn = b.node(sid("export.warn"), Role::Button, "warn");
        warn.parent(sid("export"));
        warn.action("warn").reads("export:items").policy(Care {
            actors: PERSON_ONLY,
            ..Care::default()
        });
        let mut clear = b.node(sid("export.clear"), Role::Button, "clear");
        clear.parent(sid("export"));
        clear.action("clear").reads("export:items").policy(Care {
            confirm: Policy::Always,
            destructive: true,
            ..Care::default()
        });
    }
}

impl Operable for Export {
    fn execute(&mut self, _id: RequestId, req: &Request) -> Execution {
        match req.action.as_str() {
            "warn" => {
                let alert = self.warning();
                match self.alerts.alert(&alert, None) {
                    Ok(()) => {
                        self.alerted = alert.body;
                        Execution::Done(Outcome::default())
                    }
                    Err(e) => Execution::Failed(e.to_string()),
                }
            }
            "clear" => {
                self.items = 0;
                Execution::Done(Outcome::default())
            }
            other => Execution::Failed(format!("no {other}")),
        }
    }
}

type Shared = Arc<Mutex<FakeAlerts>>;

fn setup(fake: FakeAlerts) -> (Export, Dispatcher, Shared) {
    let shared = Arc::new(Mutex::new(fake));
    (
        Export::new(Box::new(shared.clone())),
        Dispatcher::new(),
        shared,
    )
}

fn ask(
    app: &mut Export,
    d: &mut Dispatcher,
    actor: Actor,
    target: &str,
    action: &str,
    now: u64,
) -> RequestId {
    let seen = app.snapshot();
    let req = Request::observe(&seen, actor, target, action, None).expect("observe");
    d.dispatch(app, req, now)
}

fn stage(d: &Dispatcher, id: RequestId) -> &Stage {
    d.receipt(id).and_then(|r| r.stage()).expect("a stage")
}

fn names(d: &Dispatcher, id: RequestId) -> Vec<&'static str> {
    d.receipt(id)
        .expect("kept")
        .stages
        .iter()
        .map(|(s, _)| s.name())
        .collect()
}

#[test]
fn a_persons_warn_shows_the_alert_and_the_text_shown_is_state_an_agent_reads() {
    let (mut app, mut d, fake) = setup(FakeAlerts::new());
    let id = ask(&mut app, &mut d, Actor::Human, "export.warn", "warn", 1);
    assert!(
        matches!(stage(&d, id), Stage::Completed(_)),
        "{:?}",
        stage(&d, id)
    );

    let shown = fake.lock().unwrap().take_asked();
    assert_eq!(shown.len(), 1);
    assert_eq!(shown[0].kind(), AlertKind::Alert);
    assert!(!shown[0].with_parent, "no window in a test");
    let Dialog::Alert(alert) = &shown[0].dialog else {
        panic!("an alert");
    };
    assert_eq!(alert.level, Level::Warning);
    assert_eq!(alert.title, "Export");
    assert_eq!(alert.body, "3 items kept in memory; nothing was written.");

    // the agent reads what the person saw, as state
    let snap = app.snapshot();
    assert_eq!(
        snap.value(&"export:alert".parse().unwrap()),
        Some(&Value::Text(
            "3 items kept in memory; nothing was written.".to_owned()
        ))
    );
}

#[test]
fn an_agent_is_refused_the_alert_before_anything_opens_and_a_test_is_not() {
    let (mut app, mut d, fake) = setup(FakeAlerts::new());
    let agent = Actor::Agent {
        name: "claude".into(),
    };
    let id = ask(&mut app, &mut d, agent.clone(), "export.warn", "warn", 1);
    assert!(
        matches!(stage(&d, id), Stage::Rejected(Rejection::NotAuthorized { actor }) if *actor == agent),
        "{:?}",
        stage(&d, id)
    );
    assert_eq!(names(&d, id), ["requested", "rejected"]);
    assert!(fake.lock().unwrap().asked().is_empty(), "never asked");

    let tester = Actor::Test {
        name: "alerts".into(),
    };
    let id = ask(&mut app, &mut d, tester, "export.warn", "warn", 2);
    assert!(matches!(stage(&d, id), Stage::Completed(_)));
    assert_eq!(fake.lock().unwrap().asked().len(), 1);

    let id = ask(&mut app, &mut d, Actor::Assistive, "export.warn", "warn", 3);
    assert!(matches!(stage(&d, id), Stage::Completed(_)));
}

#[test]
fn a_target_without_panels_fails_the_request_and_says_why() {
    let mut app = Export::new(Box::new(NoAlerts));
    let mut d = Dispatcher::new();
    let id = ask(&mut app, &mut d, Actor::Human, "export.warn", "warn", 1);
    assert!(
        matches!(stage(&d, id), Stage::Failed { error } if error == "alert: no native dialog on this target"),
        "{:?}",
        stage(&d, id)
    );
    assert_eq!(
        app.snapshot().value(&"export:alert".parse().unwrap()),
        Some(&Value::Text(String::new())),
        "nothing was shown"
    );
}

/// The platform's confirmation answers the request that waits, as the
/// drawn modal does: the shell asks the panel with the request's words and
/// calls `confirm` or `decline` as the person. Confirmed runs (after the
/// second check), declined runs nothing, and the receipts say who answered.
#[test]
fn the_platforms_confirmation_answers_into_the_dispatcher_as_the_person() {
    let (mut app, mut d, fake) = setup(FakeAlerts::new().will_confirm().will_decline());
    let mut alerts: Box<dyn Alerts> = Box::new(fake.clone());

    // a person's clear waits for a person: `Confirm::Always`
    let first = ask(&mut app, &mut d, Actor::Human, "export.clear", "clear", 1);
    assert_eq!(stage(&d, first), &Stage::AwaitingConfirmation);
    let ask_clear = Confirm::new("Clear the export?", "clear 3 items. This cannot be undone.")
        .labels("Clear", "Cancel")
        .destructive(true);
    match alerts.confirm(&ask_clear, None).expect("a panel") {
        Answered::Confirmed => d
            .confirm(&mut app, first, Actor::Human, 2)
            .expect("confirms"),
        Answered::Declined => d.decline(first, Actor::Human, 2).expect("declines"),
    }
    assert_eq!(
        names(&d, first),
        [
            "requested",
            "awaiting_confirmation",
            "confirmed",
            "executing",
            "completed"
        ]
    );
    assert!(matches!(
        d.receipt(first).unwrap().stages[2].0,
        Stage::Confirmed { by: Actor::Human }
    ));
    assert_eq!(app.items, 0, "confirmed: it ran");

    // an agent's clear waits too; the person declines it in the panel
    let agent = Actor::Agent {
        name: "claude".into(),
    };
    app.items = 2;
    let second = ask(&mut app, &mut d, agent.clone(), "export.clear", "clear", 3);
    assert_eq!(stage(&d, second), &Stage::AwaitingConfirmation);
    assert!(
        matches!(
            d.confirm(&mut app, second, agent, 4),
            Err(accord::SemError::ConfirmByMachine { .. })
        ),
        "an agent never answers a panel"
    );
    match alerts.confirm(&ask_clear, None).expect("a panel") {
        Answered::Confirmed => d
            .confirm(&mut app, second, Actor::Human, 4)
            .expect("confirms"),
        Answered::Declined => d.decline(second, Actor::Human, 4).expect("declines"),
    }
    assert_eq!(
        stage(&d, second),
        &Stage::Rejected(Rejection::Declined { by: Actor::Human })
    );
    assert_eq!(app.items, 2, "declined: nothing ran");

    let shown = fake.lock().unwrap().take_asked();
    assert_eq!(shown.len(), 2);
    assert!(shown.iter().all(|s| s.kind() == AlertKind::Confirm));
    assert_eq!(shown[0].dialog, Dialog::Confirm(ask_clear));
}
