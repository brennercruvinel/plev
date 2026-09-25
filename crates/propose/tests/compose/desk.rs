//! A small app for the composite tests: a desk of documents a person
//! reviews, approves and publishes, one by one (the thesis' "review,
//! approve, publish"). Every document is `desk.doc[<id>]`, an item of the
//! collection `desk.doc`; `fetch` starts and ends later, when the test
//! says; `sign` is kept from agents; `shred` asks for a confirmation; the
//! code is a secret field typed with `set_value`.

use sem::{
    Actor, Actors, Confirm, Constraints, Dispatcher, Execution, Millis, Operable, Outcome,
    ParamSpec, Policy, Request, RequestId, Role, Semantics, Snapshot, SnapshotBuilder, Value,
    ValueKind,
};

/// Who may sign: a person, at the screen or through assistive
/// technology, and a test; never an agent (the showcase's `PERSON_ONLY`).
pub const PERSON_ONLY: Actors = Actors {
    human: true,
    agent: false,
    assistive: true,
    test: true,
};

/// The secret code a person types in the tests.
pub const SECRET: &str = "hunter2-open-sesame";

#[derive(Clone, Debug, PartialEq)]
pub struct Doc {
    pub id: u64,
    pub status: &'static str,
    pub seen: u32,
}

#[derive(Debug)]
pub struct Desk {
    pub docs: Vec<Doc>,
    pub code: String,
    /// The `fetch` requests that started and wait for the test.
    pub fetching: Vec<RequestId>,
    /// Every request `execute` ran, in order: (target, action).
    pub ran: Vec<(String, String)>,
}

impl Desk {
    pub fn new(n: u64) -> Desk {
        Desk {
            docs: (1..=n)
                .map(|id| Doc {
                    id,
                    status: "draft",
                    seen: 0,
                })
                .collect(),
            code: String::new(),
            fetching: Vec::new(),
            ran: Vec::new(),
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        let mut b = SnapshotBuilder::new();
        self.project(&mut b);
        b.finish().expect("the desk projects")
    }

    fn doc_mut(&mut self, target: &str) -> Option<&mut Doc> {
        let id: u64 = target
            .strip_prefix("desk.doc[")?
            .strip_suffix(']')?
            .parse()
            .ok()?;
        self.docs.iter_mut().find(|d| d.id == id)
    }

    pub fn status(&self, id: u64) -> &'static str {
        self.docs
            .iter()
            .find(|d| d.id == id)
            .map_or("gone", |d| d.status)
    }
}

impl Semantics for Desk {
    fn project(&self, b: &mut SnapshotBuilder) {
        let ids: Vec<String> = self.docs.iter().map(|d| d.id.to_string()).collect();
        b.node("desk", Role::Region, "desk")
            .state("docs", Value::List(ids));
        let mut code = b.node("desk.code", Role::TextInput, "code");
        code.parent("desk")
            .state("text", self.code.as_str())
            .constraints(Constraints {
                editable: true,
                secret: true,
                ..Constraints::default()
            });
        code.action("set_value").param(ParamSpec {
            kind: ValueKind::Text,
            constrained: true,
        });
        b.node("desk.submit", Role::Button, "submit")
            .parent("desk")
            .action("submit")
            .reads("desk.code:text");
        b.node("desk.refresh", Role::Button, "refresh")
            .parent("desk")
            .action("refresh");
        for doc in &self.docs {
            let id = format!("desk.doc[{}]", doc.id);
            let status = format!("{id}:status");
            let mut node = b.node(id.as_str(), Role::ListItem, format!("doc {}", doc.id));
            node.parent("desk")
                .state("status", doc.status)
                .state("seen", i64::from(doc.seen));
            node.action("review");
            node.action("approve")
                .enabled(doc.status == "draft")
                .reads(status.as_str());
            node.action("publish")
                .enabled(doc.status == "approved")
                .reads(status.as_str());
            node.action("fetch");
            node.action("sign").policy(Policy {
                actors: PERSON_ONLY,
                ..Policy::default()
            });
            node.action("shred").policy(Policy {
                confirm: Confirm::Always,
                destructive: true,
                ..Policy::default()
            });
        }
    }
}

impl Operable for Desk {
    fn execute(&mut self, id: RequestId, req: &Request) -> Execution {
        let target = req.target.as_str().to_owned();
        self.ran
            .push((target.clone(), req.action.as_str().to_owned()));
        match req.action.as_str() {
            "set_value" => {
                if let Some(Value::Text(text)) = &req.param {
                    self.code = text.clone();
                }
            }
            "submit" => self.code.clear(),
            "refresh" | "sign" => {}
            "fetch" => {
                self.fetching.push(id);
                return Execution::Started;
            }
            "shred" => self
                .docs
                .retain(|d| format!("desk.doc[{}]", d.id) != target),
            action => {
                let Some(doc) = self.doc_mut(&target) else {
                    return Execution::Failed(format!("no {target}"));
                };
                match action {
                    "review" => doc.seen += 1,
                    "approve" => doc.status = "approved",
                    "publish" => doc.status = "published",
                    _ => return Execution::Failed(format!("no {action}")),
                }
            }
        }
        Execution::Done(Outcome::default())
    }
}

pub fn person() -> Actor {
    Actor::Human
}

pub fn agent(name: &str) -> Actor {
    Actor::Agent {
        name: name.to_owned(),
    }
}

/// The desk and its one dispatcher, with a clock.
pub struct Bench {
    pub desk: Desk,
    pub dispatcher: Dispatcher,
    pub now: Millis,
}

impl Bench {
    pub fn new(docs: u64) -> Bench {
        Bench {
            desk: Desk::new(docs),
            dispatcher: Dispatcher::new(),
            now: 1_000,
        }
    }

    /// `actor` asks `action` of `target`, having seen the desk as it is
    /// now; the request's id.
    pub fn act(&mut self, actor: Actor, target: &str, action: &str) -> RequestId {
        self.ask(actor, target, action, None)
    }

    pub fn ask(
        &mut self,
        actor: Actor,
        target: &str,
        action: &str,
        param: Option<Value>,
    ) -> RequestId {
        let seen = self.desk.snapshot();
        let req = Request::observe(&seen, actor, target, action, param).expect("observed");
        self.now += 10;
        self.dispatcher.dispatch(&mut self.desk, req, self.now)
    }

    /// The `fetch` that started first ends completed.
    pub fn fetched(&mut self) -> RequestId {
        let id = self.desk.fetching.remove(0);
        self.now += 10;
        self.dispatcher
            .complete(id, Outcome::default(), self.now)
            .expect("the fetch was running");
        id
    }

    /// A person reviews, approves and publishes `doc`.
    pub fn by_hand(&mut self, actor: &Actor, doc: u64) {
        let target = format!("desk.doc[{doc}]");
        for action in ["review", "approve", "publish"] {
            self.act(actor.clone(), &target, action);
        }
    }

    pub fn stage(&self, id: RequestId) -> sem::Stage {
        self.dispatcher
            .receipt(id)
            .and_then(sem::Receipt::stage)
            .cloned()
            .expect("a receipt")
    }
}
