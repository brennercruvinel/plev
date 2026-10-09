//! Deep links (task-76 item 6), proven without registering anything and
//! without opening a link: texts opened on the fake source are read by
//! the strict grammar and asked of a test app's dispatcher as the agent
//! `link`, never the person. The policy holds as for any agent: a
//! capability for a person alone is refused, a confirmation the policy
//! asks of an agent waits for a person (the link cannot give it), a link
//! that saw an older state is stale, a parameter out of the options is
//! refused, and a capability that left the projection makes no request.
//! The app's open list comes before all of it.

use engine::platform::links::{self, FakeLinks, Link, LinkError, LinkSource};
use sem::{
    Actor, Actors, Confirm, Constraints, Dispatcher, Execution, Operable, Outcome, ParamSpec,
    Policy, Rejection, Request, RequestId, Role, SemError, Semantics, Snapshot, SnapshotBuilder,
    Stage, Value, ValueKind,
};

const SCHEME: &str = "plev-test";

/// What this app opens to links: the title and the theme. Publishing and
/// deleting are not in it; the tests that reach them pass a list of
/// their own, to show the policy holds under any list.
const OPEN: &[(&str, &str)] = &[("doc.title", "set_value"), ("doc.theme", "select")];

const TEXT: ParamSpec = ParamSpec {
    kind: ValueKind::Text,
    constrained: true,
};

/// A document: a title anyone may set, a theme picked among two, a
/// publish that waits for a person when an agent asks it, a delete for a
/// person alone, and a draft to discard while there is one.
struct Doc {
    title: String,
    theme: String,
    published: bool,
    deleted: bool,
    draft: bool,
}

impl Semantics for Doc {
    fn project(&self, b: &mut SnapshotBuilder) {
        b.node("doc", Role::Region, "doc")
            .state("published", self.published);
        b.node("doc.title", Role::TextInput, "title")
            .parent("doc")
            .state("value", self.title.as_str())
            .constraints(Constraints {
                editable: true,
                max_len: Some(40),
                ..Constraints::default()
            })
            .action("set_value")
            .param(TEXT);
        b.node("doc.theme", Role::Select, "theme")
            .parent("doc")
            .state("selected", self.theme.as_str())
            .constraints(Constraints {
                editable: true,
                options: Some(vec!["light".to_owned(), "dark".to_owned()]),
                ..Constraints::default()
            })
            .action("select")
            .param(TEXT)
            .reads("doc.theme:selected");
        b.node("doc.publish", Role::Button, "publish")
            .parent("doc")
            .action("publish")
            .reads("doc.title:value")
            .policy(Policy {
                confirm: Confirm::ForAgents,
                ..Policy::default()
            });
        b.node("doc.delete", Role::Button, "delete")
            .parent("doc")
            .action("delete")
            .policy(Policy {
                actors: Actors {
                    agent: false,
                    ..Actors::default()
                },
                destructive: true,
                ..Policy::default()
            });
        if self.draft {
            b.node("doc.draft", Role::Button, "discard the draft")
                .parent("doc")
                .action("discard");
        }
    }
}

impl Operable for Doc {
    fn execute(&mut self, _id: RequestId, req: &Request) -> Execution {
        let text = match &req.param {
            Some(Value::Text(text)) => text.clone(),
            _ => String::new(),
        };
        match req.action.as_str() {
            "set_value" => self.title = text,
            "select" => self.theme = text,
            "publish" => self.published = true,
            "delete" => self.deleted = true,
            "discard" => self.draft = false,
            other => return Execution::Failed(format!("no {other}")),
        }
        Execution::Done(Outcome::default())
    }
}

fn doc() -> Doc {
    Doc {
        title: "draft".to_owned(),
        theme: "light".to_owned(),
        published: false,
        deleted: false,
        draft: true,
    }
}

fn project(app: &Doc) -> Snapshot {
    let mut b = SnapshotBuilder::new();
    app.project(&mut b);
    b.finish().expect("the doc projects")
}

fn link_actor() -> Actor {
    Actor::Agent {
        name: "link".to_owned(),
    }
}

/// Every link opened on the source, read, and asked of the dispatcher
/// when `open` lists it: its stage, or why it was refused before any
/// request.
fn follow(
    source: &mut dyn LinkSource,
    app: &mut Doc,
    dispatcher: &mut Dispatcher,
    open: &[(&str, &str)],
) -> Vec<Result<Stage, String>> {
    let mut out = Vec::new();
    for delivered in source.poll() {
        let link = match delivered {
            Ok(link) => link,
            Err(refused) => {
                out.push(Err(refused.to_string()));
                continue;
            }
        };
        if !link.is_in(open) {
            out.push(Err(format!("{} is not open to links", link.target)));
            continue;
        }
        let req = match link.request(&project(app)) {
            Ok(req) => req,
            Err(e) => {
                out.push(Err(e.to_string()));
                continue;
            }
        };
        assert_eq!(req.actor, link_actor(), "a link is never a person");
        let id = dispatcher.dispatch(app, req, 10);
        let stage = dispatcher.receipt(id).and_then(|r| r.stage()).cloned();
        out.push(Ok(stage.expect("a stage")));
    }
    out
}

/// The one link opened on the source, asked of the dispatcher: its
/// request's id and stage.
fn dispatched(
    source: &mut dyn LinkSource,
    app: &mut Doc,
    dispatcher: &mut Dispatcher,
    open: &[(&str, &str)],
) -> (RequestId, Stage) {
    let mut links = source.poll();
    assert_eq!(links.len(), 1);
    let link = links.remove(0).expect("a link");
    assert!(link.is_in(open));
    let req = link.request(&project(app)).expect("observed");
    let id = dispatcher.dispatch(app, req, 10);
    let stage = dispatcher.receipt(id).and_then(|r| r.stage()).cloned();
    (id, stage.expect("a stage"))
}

#[test]
fn a_link_is_asked_by_the_agent_link_and_runs_where_the_app_opened_it() {
    let mut app = doc();
    let mut dispatcher = Dispatcher::new();
    let mut source = FakeLinks::new(SCHEME);
    source
        .open("plev-test://doc.title/set_value?value=Hello%2C%20world")
        .open("plev-test://doc.theme/select?value=dark")
        .open("plev-test://doc.delete/delete")
        .open("plev-test://doc.title/set_value?value=a%0Ab");
    let got = follow(&mut source, &mut app, &mut dispatcher, OPEN);
    assert!(matches!(got[0], Ok(Stage::Completed(_))), "{got:?}");
    assert!(matches!(got[1], Ok(Stage::Completed(_))), "{got:?}");
    assert_eq!(
        got[2],
        Err("doc.delete is not open to links".to_owned()),
        "the app's list comes first"
    );
    assert_eq!(
        got[3],
        Err(
            "plev-test://doc.title/set_value?value=a%0Ab: the link's text holds a control or a \
             direction character"
                .to_owned()
        )
    );
    assert_eq!(app.title, "Hello, world");
    assert_eq!(app.theme, "dark");
    assert!(!app.deleted);
}

#[test]
fn under_any_list_the_policy_holds_as_for_an_agent() {
    let everything = [
        ("doc.title", "set_value"),
        ("doc.theme", "select"),
        ("doc.publish", "publish"),
        ("doc.delete", "delete"),
    ];
    let mut app = doc();
    let mut dispatcher = Dispatcher::new();
    let mut source = FakeLinks::new(SCHEME);
    source
        .open("plev-test://doc.delete/delete")
        .open("plev-test://doc.theme/select?value=sepia")
        .open("plev-test://doc.title/set_value?value=".to_owned() + &"x".repeat(41));
    let got = follow(&mut source, &mut app, &mut dispatcher, &everything);
    assert_eq!(
        got[0],
        Ok(Stage::Rejected(Rejection::NotAuthorized {
            actor: link_actor()
        })),
        "a person's capability is refused to a link"
    );
    assert!(!app.deleted);
    assert!(
        matches!(
            &got[1],
            Ok(Stage::Rejected(Rejection::InvalidParam { reason })) if reason.contains("options")
        ),
        "{got:?}"
    );
    assert!(
        matches!(&got[2], Ok(Stage::Rejected(Rejection::InvalidParam { .. }))),
        "{got:?}"
    );
    assert_eq!(app.theme, "light");
    assert_eq!(app.title, "draft");
}

/// A policy that asks a person's confirmation of an agent asks it of a
/// link: the request waits, the link cannot answer it, a person can.
#[test]
fn a_confirmation_the_policy_asks_waits_for_a_person_the_link_cannot_give() {
    let open = [("doc.publish", "publish")];
    let mut app = doc();
    let mut dispatcher = Dispatcher::new();
    let mut source = FakeLinks::new(SCHEME);
    source.open("plev-test://doc.publish/publish");
    let (id, stage) = dispatched(&mut source, &mut app, &mut dispatcher, &open);
    assert_eq!(stage, Stage::AwaitingConfirmation);
    assert!(!app.published, "nothing ran yet");
    assert!(matches!(
        dispatcher.confirm(&mut app, id, links::actor(), 20),
        Err(SemError::ConfirmByMachine { .. })
    ));
    assert!(!app.published, "the link cannot confirm its own request");
    dispatcher
        .confirm(&mut app, id, Actor::Human, 30)
        .expect("a person confirms");
    assert!(app.published);
    let stages: Vec<Stage> = dispatcher
        .receipt(id)
        .unwrap()
        .stages
        .iter()
        .map(|(s, _)| s.clone())
        .collect();
    assert!(stages.contains(&Stage::Confirmed { by: Actor::Human }));
}

#[test]
fn a_link_that_saw_an_older_state_is_stale_and_one_whose_capability_left_asks_nothing() {
    let mut app = doc();
    let mut dispatcher = Dispatcher::new();
    // the link is read and observed now
    let seen = project(&app);
    let publish = Link::parse("plev-test://doc.publish/publish", SCHEME).expect("a link");
    let stale = publish.request(&seen).expect("observed");
    // a person retitles the doc before the link's request is taken
    let retitle = Request::observe(
        &seen,
        Actor::Human,
        "doc.title",
        "set_value",
        Some("final".into()),
    )
    .expect("observed");
    dispatcher.dispatch(&mut app, retitle, 10);
    let id = dispatcher.dispatch(&mut app, stale, 20);
    assert!(matches!(
        dispatcher.receipt(id).unwrap().stage(),
        Some(Stage::Rejected(Rejection::Stale { .. }))
    ));
    assert!(!app.published);

    // the draft is discarded: a link to it has nothing to ask
    app.draft = false;
    let gone = Link::parse("plev-test://doc.draft/discard", SCHEME).expect("a link");
    assert!(matches!(
        gone.request(&project(&app)),
        Err(SemError::UnknownAction { .. })
    ));
}

#[test]
fn what_is_not_a_link_of_this_app_makes_no_request() {
    let mut app = doc();
    let mut dispatcher = Dispatcher::new();
    let mut source = FakeLinks::new(SCHEME);
    source
        .open("https://doc.title/set_value?value=x")
        .open("plev-test://doc.title/set_value?value=x&also=y")
        .open("plev-test://doc.title/set_value/")
        .open("plev-test://user@doc.title/set_value");
    let got = follow(&mut source, &mut app, &mut dispatcher, OPEN);
    let why: Vec<LinkError> = [
        LinkError::OtherScheme {
            found: "https".to_owned(),
        },
        LinkError::ExtraKey,
        LinkError::ExtraPath,
        LinkError::UserInfo,
    ]
    .to_vec();
    assert_eq!(got.len(), why.len());
    for (got, why) in got.iter().zip(&why) {
        let line = got.as_ref().expect_err("refused");
        assert!(line.ends_with(&why.to_string()), "{line}");
    }
    assert_eq!(app.title, "draft");
    assert_eq!(dispatcher.mark(), Dispatcher::new().mark(), "no request");
}
