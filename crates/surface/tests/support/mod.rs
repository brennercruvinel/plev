//! What the surface tests share: the fixtures of task-68 (the IR and the
//! projections they are written against, read from `crates/uir/fixtures`),
//! a page that projects an app and its surface as one snapshot, and a
//! small app with a real `sem::Dispatcher` whose state is the fixture's.

use std::path::PathBuf;

use indexmap::IndexMap;
use sem::{
    Dispatcher, Execution, Node, Operable, Outcome, Receipt, Request, RequestId, SemId, Semantics,
    Snapshot, SnapshotBuilder, StateKey, StateRef, Value,
};
use surface::{Dispatch, Surface};
use uir::{Capabilities, GenerationPolicy, Ir, VerifiedIr, parse, verify};

pub const COMMIT_FORM: &str = "ide-commit-form.ir.json";
pub const REVIEW: &str = "ide-review-commit.ir.json";
pub const TODO: &str = "showcase-todo.ir.json";
pub const IDE: &str = "ide-workbench.snapshot.json";
pub const SHOWCASE: &str = "showcase-todo.snapshot.json";

/// Each IR fixture and the projection it is written against.
pub const CASES: [(&str, &str); 3] = [(COMMIT_FORM, IDE), (REVIEW, IDE), (TODO, SHOWCASE)];

pub fn fixture(name: &str) -> String {
    let at = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../uir/fixtures")
        .join(name);
    std::fs::read_to_string(&at).unwrap_or_else(|e| panic!("{}: {e}", at.display()))
}

pub fn snapshot(name: &str) -> Snapshot {
    serde_json::from_str(&fixture(name)).expect("a sem snapshot")
}

pub fn ir(name: &str) -> Ir {
    parse(&fixture(name)).unwrap_or_else(|errors| panic!("{name}: {errors:#?}"))
}

/// The fixture `ir` verified against the registry of `app`.
pub fn verified(ir_name: &str, app: &Snapshot) -> VerifiedIr {
    verify(
        &ir(ir_name),
        &Capabilities::from_snapshot(app),
        &GenerationPolicy::default(),
    )
    .unwrap_or_else(|errors| panic!("{ir_name}: {errors:#?}"))
}

pub fn id(text: &str) -> SemId {
    SemId::parse(text).unwrap()
}

/// Declares `node` again in `b`, as the app projected it.
pub fn copy(b: &mut SnapshotBuilder, node: &Node) {
    let mut n = b.node(&node.id, node.role, node.label.as_str());
    if let Some(parent) = &node.parent {
        n.parent(parent);
    }
    if let Some(d) = &node.description {
        n.description(d.as_str());
    }
    for (key, value) in &node.state {
        n.state(key, value.clone());
    }
    n.constraints(node.constraints.clone())
        .enabled(node.enabled)
        .inert(node.inert)
        .focusable(node.focusable);
    for decl in &node.actions {
        let mut a = n.action(&decl.name);
        if let Some(label) = &decl.label {
            a.label(label.as_str());
        }
        a.enabled(decl.enabled).policy(decl.policy);
        if let Some(param) = decl.param {
            a.param(param);
        }
        for read in &decl.reads {
            a.reads(read);
        }
    }
}

/// The page: the app's projection and the surface's, one snapshot.
pub fn page(app: &Snapshot, surface: &Surface) -> Snapshot {
    let mut b = SnapshotBuilder::new();
    for node in app.nodes().values() {
        copy(&mut b, node);
    }
    if let Some(f) = app.focused() {
        b.focus(f);
    }
    surface.project(&mut b);
    b.finish()
        .unwrap_or_else(|e| panic!("the page projects: {e}"))
}

/// A small app over the state of a fixture projection: it projects that
/// state, runs every action through its own `Dispatcher`, and its
/// `execute` moves the state the way the ide and the todo would for the
/// actions the fixtures bind.
pub struct App {
    pub nodes: IndexMap<SemId, Node>,
    pub focused: Option<SemId>,
    pub dispatcher: Dispatcher,
    pub executed: Vec<Request>,
    now: u64,
}

impl App {
    pub fn over(snap: &Snapshot) -> App {
        App {
            nodes: snap.nodes().clone(),
            focused: snap.focused().cloned(),
            dispatcher: Dispatcher::new(),
            executed: Vec::new(),
            now: 0,
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        let mut b = SnapshotBuilder::new();
        Semantics::project(self, &mut b);
        b.finish()
            .unwrap_or_else(|e| panic!("the app projects: {e}"))
    }

    /// Sets a value of the app directly, as another actor's effect would.
    pub fn set(&mut self, at: &str, value: Value) {
        let at = StateRef::parse(at).unwrap();
        let node = self.nodes.get_mut(&at.node).expect("the node");
        node.state.insert(at.key, value);
    }

    fn state(&mut self, node: &SemId, key: &str, value: Value) {
        if let Some(n) = self.nodes.get_mut(node) {
            n.state.insert(StateKey::parse(key).unwrap(), value);
        }
    }

    pub fn receipt(&self, id: RequestId) -> Option<&Receipt> {
        self.dispatcher.receipt(id)
    }
}

impl Semantics for App {
    fn project(&self, b: &mut SnapshotBuilder) {
        for node in self.nodes.values() {
            copy(b, node);
        }
        if let Some(f) = &self.focused {
            b.focus(f);
        }
    }
}

impl Operable for App {
    fn execute(&mut self, _id: RequestId, req: &Request) -> Execution {
        self.executed.push(req.clone());
        match (req.action.as_str(), &req.param) {
            ("set_value", Some(Value::Text(text))) => {
                self.state(&req.target, "text", Value::Text(text.clone()));
            }
            ("stage" | "unstage", None) => {
                let on = req.action.as_str() == "stage";
                self.state(&req.target, "staged", Value::Bool(on));
                self.state(&req.target, "unstaged", Value::Bool(!on));
            }
            _ => {}
        }
        Execution::Done(Outcome::default())
    }
}

/// The app as a surface reaches it: its dispatcher, its projection.
impl Dispatch for App {
    fn dispatch(&mut self, req: Request) -> Option<Receipt> {
        self.now += 1;
        let mut dispatcher = std::mem::take(&mut self.dispatcher);
        let id = dispatcher.dispatch(self, req, self.now);
        self.dispatcher = dispatcher;
        self.dispatcher.receipt(id).cloned()
    }

    fn project(&self) -> Option<Snapshot> {
        Some(self.snapshot())
    }
}
