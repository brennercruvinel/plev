//! a toy app the port serves in these tests: a small todo list with a
//! field, a secret pin, a destructive `remove` that asks agents for a
//! person's confirmation, and a `sync` that ends later. it implements
//! `Semantics` and `Operable`, and `ToyHost` holds it with its one
//! dispatcher, as a shell would.

use std::collections::BTreeSet;

use sem::{
    Confirm, Constraints, Dispatcher, Execution, Operable, Outcome, ParamSpec, Policy, ReceiptSink,
    Request, RequestId, Role, SemId, Semantics, Snapshot, SnapshotBuilder, Value, ValueKind,
};
use wire::{Bound, Host};

pub struct Toy {
    /// id, text, done.
    pub items: Vec<(u64, String, bool)>,
    pub text: String,
    pub pin: String,
    pub next: u64,
    /// `sync` requests started and not ended.
    pub syncing: Vec<RequestId>,
}

impl Toy {
    pub fn new() -> Self {
        Toy {
            items: vec![
                (1, "buy milk".into(), false),
                (2, "call mom".into(), false),
                (3, "ship it".into(), true),
            ],
            text: String::new(),
            pin: "1234".into(),
            next: 4,
            syncing: Vec::new(),
        }
    }

    pub fn done(&self, id: u64) -> Option<bool> {
        self.items
            .iter()
            .find(|(i, _, _)| *i == id)
            .map(|(_, _, d)| *d)
    }

    fn item(id: &SemId) -> Option<u64> {
        id.as_str()
            .strip_prefix("todo.item[")?
            .strip_suffix(']')?
            .parse()
            .ok()
    }
}

impl Semantics for Toy {
    fn project(&self, b: &mut SnapshotBuilder) {
        b.node("todo", Role::Region, "todo");
        let mut input = b.node("todo.input", Role::TextInput, "new todo");
        input.parent("todo").state("text", self.text.as_str());
        input.constraints(Constraints {
            editable: true,
            ..Constraints::default()
        });
        input
            .action("submit")
            .enabled(!self.text.trim().is_empty())
            .reads("todo.input:text");
        input.action("set_value").param(ParamSpec {
            kind: ValueKind::Text,
            constrained: true,
        });
        let mut pin = b.node("todo.pin", Role::TextInput, "pin");
        pin.parent("todo").state("text", self.pin.as_str());
        pin.constraints(Constraints {
            editable: true,
            secret: true,
            ..Constraints::default()
        });
        pin.action("set_value")
            .param(ParamSpec {
                kind: ValueKind::Text,
                constrained: false,
            })
            .reads("todo.pin:text");
        for (id, text, done) in &self.items {
            let sid = format!("todo.item[{id}]");
            let mut node = b.node(sid.as_str(), Role::ListItem, text.as_str());
            node.parent("todo").state("completed", *done);
            node.action("toggle").reads(format!("{sid}:completed"));
            node.action("remove").policy(Policy {
                confirm: Confirm::ForAgents,
                destructive: true,
                ..Policy::default()
            });
        }
        b.node("todo.sync", Role::Button, "sync")
            .parent("todo")
            .action("sync");
        let done: BTreeSet<String> = self
            .items
            .iter()
            .filter(|(_, _, d)| *d)
            .map(|(i, _, _)| i.to_string())
            .collect();
        b.node("todo.counter", Role::Status, "counter")
            .parent("todo")
            .state("done", Value::Set(done));
    }
}

impl Operable for Toy {
    fn execute(&mut self, id: RequestId, req: &Request) -> Execution {
        let mut outcome = Outcome::default();
        match (req.target.as_str(), req.action.as_str()) {
            ("todo.input", "submit") => {
                let item = self.next;
                self.next += 1;
                self.items
                    .push((item, std::mem::take(&mut self.text), false));
                outcome
                    .result
                    .insert("item".into(), Value::Text(format!("todo.item[{item}]")));
            }
            ("todo.input", "set_value") => match &req.param {
                Some(Value::Text(t)) => self.text = t.clone(),
                _ => return Execution::Failed("a text".into()),
            },
            ("todo.pin", "set_value") => match &req.param {
                Some(Value::Text(t)) if t.len() == 4 => {
                    self.pin = t.clone();
                    outcome.result.insert("pin".into(), Value::Text(t.clone()));
                }
                Some(Value::Text(t)) => return Execution::Failed(format!("{t} is not 4 digits")),
                _ => return Execution::Failed("a text".into()),
            },
            ("todo.sync", "sync") => {
                self.syncing.push(id);
                return Execution::Started;
            }
            (_, "toggle") => match Toy::item(&req.target) {
                Some(i) => {
                    for item in &mut self.items {
                        if item.0 == i {
                            item.2 = !item.2;
                        }
                    }
                }
                None => return Execution::Failed("no item".into()),
            },
            (_, "remove") => match Toy::item(&req.target) {
                Some(i) => self.items.retain(|item| item.0 != i),
                None => return Execution::Failed("no item".into()),
            },
            _ => return Execution::Failed("unknown".into()),
        }
        Execution::Done(outcome)
    }
}

/// the toy as a shell holds it: the app, its one dispatcher, the clock
/// the dispatcher stamps with.
pub struct ToyHost {
    pub app: Toy,
    pub dispatcher: Dispatcher,
    pub clock: u64,
    /// false makes `add_sink` drop the sink, like a shell that forgot it.
    pub wires_sinks: bool,
}

impl ToyHost {
    pub fn new() -> Self {
        ToyHost {
            app: Toy::new(),
            dispatcher: Dispatcher::new(),
            clock: 1000,
            wires_sinks: true,
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        let mut b = SnapshotBuilder::new();
        self.app.project(&mut b);
        b.finish().unwrap()
    }

    /// a person's click, through the same dispatcher.
    pub fn click(&mut self, target: &str, action: &str) {
        let seen = self.snapshot();
        let req = Request::observe(&seen, sem::Actor::Human, target, action, None).unwrap();
        self.clock += 1;
        self.dispatcher.dispatch(&mut self.app, req, self.clock);
    }

    /// ends the oldest `sync` still running.
    pub fn finish_sync(&mut self) {
        if self.app.syncing.is_empty() {
            panic!("nothing is syncing");
        }
        let id = self.app.syncing.remove(0);
        self.clock += 5;
        let outcome = Outcome {
            result: [("synced".to_owned(), Value::Int(3))].into_iter().collect(),
        };
        self.dispatcher.complete(id, outcome, self.clock).unwrap();
    }
}

impl Host for ToyHost {
    fn project(&self) -> Option<Snapshot> {
        Some(self.snapshot())
    }

    fn bounds(&self) -> Vec<Bound> {
        self.app
            .items
            .iter()
            .enumerate()
            .map(|(row, (id, _, _))| Bound {
                id: SemId::parse(&format!("todo.item[{id}]")).unwrap(),
                rect: [0.0, 40.0 * row as f32, 300.0, 40.0],
            })
            .collect()
    }

    fn dispatch(&mut self, req: Request) -> Option<RequestId> {
        self.clock += 1;
        Some(self.dispatcher.dispatch(&mut self.app, req, self.clock))
    }

    fn add_sink(&mut self, sink: Box<dyn ReceiptSink>) {
        if self.wires_sinks {
            self.dispatcher.add_sink(sink);
        }
    }
}
