//! System notifications as capabilities out of the window (task-76 item
//! 5), proven without a platform: an app notifies through the fake with
//! a button that names one of its capabilities, a click on it becomes the
//! request the button on screen makes and gets the same receipt by
//! `Actor::Human`, a click that saw an older state is stale, a click on a
//! capability that left the projection makes no request at all, an
//! agent's capability is never a notification's button, and the target
//! without notifications says so.

use accord::{
    Actor, Dispatcher, Execution, Operable, Outcome, Policy, Request, RequestId, Role, SemId,
    Semantics, Snapshot, SnapshotBuilder, Stage, Value,
};
use engine::runtime::platform::notify::{
    Action, FakeNotifier, Level, NoNotifier, Notification, Notifier, NotifyError,
};

fn sid(text: &str) -> SemId {
    SemId::parse(text).expect("a valid id")
}

/// An inbox that fills in the background: `open` (a person's) marks it
/// read, `archive` (for agents alone) puts it away, and the open button
/// leaves the projection with it. What the app tells the person is
/// projected as `inbox:notice`, so an agent reads it too.
struct Inbox {
    unread: i64,
    notice: String,
    opened: usize,
    archived: bool,
}

impl Semantics for Inbox {
    fn project(&self, b: &mut SnapshotBuilder) {
        b.node("inbox", Role::Region, "inbox")
            .state("unread", self.unread)
            .state("notice", self.notice.as_str());
        if !self.archived {
            b.node("inbox.open", Role::Button, "open the inbox")
                .parent("inbox")
                .action("open")
                .reads("inbox:unread")
                .policy(Policy {
                    actors: accord::Actors {
                        agent: false,
                        ..accord::Actors::default()
                    },
                    ..Policy::default()
                });
        }
        b.node("inbox.archive", Role::Button, "archive")
            .parent("inbox")
            .action("archive")
            .policy(Policy {
                actors: accord::Actors {
                    human: false,
                    ..accord::Actors::default()
                },
                ..Policy::default()
            });
    }
}

impl Operable for Inbox {
    fn execute(&mut self, _id: RequestId, req: &Request) -> Execution {
        let mut outcome = Outcome::default();
        match req.action.as_str() {
            "open" => {
                self.opened += 1;
                outcome
                    .result
                    .insert("read".to_owned(), Value::Int(self.unread));
                self.unread = 0;
            }
            "archive" => self.archived = true,
            other => return Execution::Failed(format!("no {other}")),
        }
        Execution::Done(outcome)
    }
}

impl Inbox {
    /// Mail arrived while nobody looked: the app notifies, with the open
    /// button as the notification's, and projects what it said.
    fn arrived(&mut self, n: i64, notifier: &mut dyn Notifier) -> Result<(), NotifyError> {
        self.unread += n;
        let seen = project(self);
        let body = format!("{} new messages", self.unread);
        let mut notification = Notification::info("Inbox", body.clone());
        if let Some(open) = Action::of(&seen, &sid("inbox.open"), "open") {
            notification = notification.action(open.label("open"));
        }
        self.notice = body;
        notifier.notify(&notification).map(|_| ())
    }
}

fn project(app: &Inbox) -> Snapshot {
    let mut b = SnapshotBuilder::new();
    app.project(&mut b);
    b.finish().expect("the inbox projects")
}

fn inbox() -> Inbox {
    Inbox {
        unread: 0,
        notice: String::new(),
        opened: 0,
        archived: false,
    }
}

#[test]
fn a_click_on_the_notification_is_the_request_the_button_makes_and_gets_its_receipt() {
    let mut app = inbox();
    let mut dispatcher = Dispatcher::new();
    let mut notifier = FakeNotifier::new();

    // the button on screen: a person opens the inbox
    app.unread = 1;
    let seen = project(&app);
    let by_button = Request::observe(&seen, Actor::Human, "inbox.open", "open", None).unwrap();
    let button_id = dispatcher.dispatch(&mut app, by_button, 10);
    let button_receipt = dispatcher.receipt(button_id).unwrap().clone();
    assert_eq!(app.unread, 0);

    // mail arrives in the background: the app notifies, the text is state
    app.arrived(2, &mut notifier).unwrap();
    let sent = notifier.sent();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].notification.level, Level::Info);
    assert_eq!(sent[0].notification.title, "Inbox");
    assert_eq!(sent[0].notification.body, "2 new messages");
    let action = sent[0].notification.action.as_ref().expect("a button");
    assert_eq!(action.label, "open");
    assert_eq!(action.sem, sid("inbox.open"));
    assert_eq!(
        project(&app).value(&"inbox:notice".parse().unwrap()),
        Some(&Value::from("2 new messages")),
        "what the app told the person, for an agent to read"
    );

    // the person clicks it: polled on the loop, observed on the state as
    // it is now, the same request and receipt as the button's
    assert!(notifier.click(sent[0].id));
    let clicks = notifier.poll();
    assert_eq!(clicks.len(), 1);
    let seen = project(&app);
    let by_click = clicks[0].request(&seen).unwrap();
    assert_eq!(by_click.actor, Actor::Human);
    assert_eq!(by_click.target, sid("inbox.open"));
    assert_eq!(by_click.action.as_str(), "open");
    let click_id = dispatcher.dispatch(&mut app, by_click, 20);
    let click_receipt = dispatcher.receipt(click_id).unwrap();
    let stages = |r: &accord::Receipt| r.stages.iter().map(|(s, _)| s.name()).collect::<Vec<_>>();
    assert_eq!(stages(click_receipt), stages(&button_receipt));
    let Some(Stage::Completed(outcome)) = click_receipt.stage() else {
        panic!("completed: {:?}", click_receipt.stage());
    };
    assert_eq!(outcome.result["read"], Value::Int(2));
    assert_eq!(app.opened, 2);
    assert_eq!(app.unread, 0);

    // a click that saw the inbox before it was read is stale, as the
    // button's would be
    let stale = clicks[0].request(&seen).unwrap();
    let stale_id = dispatcher.dispatch(&mut app, stale, 30);
    assert!(matches!(
        dispatcher.receipt(stale_id).unwrap().stage(),
        Some(Stage::Rejected(accord::Rejection::Stale { .. }))
    ));
    assert_eq!(app.opened, 2, "nothing ran");
}

#[test]
fn a_click_on_a_capability_that_left_the_projection_makes_no_request() {
    let mut app = inbox();
    let mut notifier = FakeNotifier::new();
    app.arrived(1, &mut notifier).unwrap();
    let id = notifier.sent()[0].id;

    // an agent archives the inbox meanwhile: the open button leaves the
    // projection
    let mut dispatcher = Dispatcher::new();
    let seen = project(&app);
    let agent = Actor::Agent {
        name: "claude".into(),
    };
    let req = Request::observe(&seen, agent, "inbox.archive", "archive", None).unwrap();
    let id_archive = dispatcher.dispatch(&mut app, req, 10);
    assert!(matches!(
        dispatcher.receipt(id_archive).unwrap().stage(),
        Some(Stage::Completed(_))
    ));
    let now = project(&app);
    assert!(now.get(&sid("inbox.open")).is_none());

    // the person clicks the notification late: the capability is gone,
    // so there is nothing to ask and no receipt, as with a menu item of
    // a page that closed
    assert!(notifier.click(id), "the notification is still on screen");
    let clicks = notifier.poll();
    assert_eq!(clicks.len(), 1);
    let refused = clicks[0].request(&now);
    assert!(refused.is_err(), "{refused:?}");
    assert_eq!(app.opened, 0, "nothing ran");
    assert_eq!(app.unread, 1);

    // a notification without a button hands back no click at all
    app.notice.clear();
    let no_button = Notification::warning("Inbox", "the server is slow");
    let plain = notifier.notify(&no_button).unwrap();
    assert!(!notifier.click(plain));
    assert!(notifier.poll().is_empty());
}

#[test]
fn an_agents_capability_is_never_the_notifications_button() {
    let mut app = inbox();
    app.unread = 3;
    let seen = project(&app);
    assert!(
        Action::of(&seen, &sid("inbox.archive"), "archive").is_none(),
        "archive is for agents alone"
    );
    let open = Action::of(&seen, &sid("inbox.open"), "open").unwrap();
    assert_eq!(open.label, "open the inbox");

    // and an agent asking the button's capability by hand is refused
    // where the person is not
    let mut dispatcher = Dispatcher::new();
    let agent = Actor::Agent {
        name: "claude".into(),
    };
    let req = Request::observe(&seen, agent.clone(), "inbox.open", "open", None).unwrap();
    let id = dispatcher.dispatch(&mut app, req, 10);
    assert!(matches!(
        dispatcher.receipt(id).unwrap().stage(),
        Some(Stage::Rejected(accord::Rejection::NotAuthorized { actor })) if *actor == agent
    ));
    assert_eq!(app.unread, 3);
}

#[test]
fn without_a_platform_the_app_still_projects_what_it_would_have_said() {
    let mut app = inbox();
    let mut none: Box<dyn Notifier> = Box::new(NoNotifier);
    assert_eq!(app.arrived(4, &mut *none), Err(NotifyError::Unsupported));
    assert_eq!(
        project(&app).value(&"inbox:notice".parse().unwrap()),
        Some(&Value::from("4 new messages")),
        "the text is the truth, shown or not"
    );
    assert!(none.poll().is_empty());

    let mut refusing = FakeNotifier::new().will_want_a_bundle();
    assert_eq!(app.arrived(1, &mut refusing), Err(NotifyError::NeedsBundle));
    assert!(refusing.sent().is_empty());
}
