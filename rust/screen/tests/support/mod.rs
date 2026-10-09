//! What the surface tests share: the fixtures of task-68 (the IR and the
//! projections they are written against, read from `rust/plevir/tests/fixtures`),
//! a page that projects an app and its surface as one snapshot, and a
//! small app with a real `sem::Dispatcher` whose state is the fixture's.

use std::path::PathBuf;

use accord::{
    Constraints, Dispatcher, Execution, Node, Operable, Outcome, ParamSpec, Receipt, Request,
    RequestId, Role, SemId, Semantics, Snapshot, SnapshotBuilder, StateKey, StateRef, Value,
    ValueKind,
};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::Rect;
use indexmap::IndexMap;
use plevir::{Capabilities, GenerationPolicy, Ir, VerifiedIr, parse, verify};
use screen::{Dispatch, Surface};

pub const COMMIT_FORM: &str = "idestudio-commit-form.ir.json";
pub const REVIEW: &str = "idestudio-review-commit.ir.json";
pub const TODO: &str = "uigallery-todo.ir.json";
pub const IDE: &str = "idestudio-workbench.snapshot.json";
pub const SHOWCASE: &str = "uigallery-todo.snapshot.json";
/// The todo surface in two columns (a local fixture): the same controls
/// as `TODO`, in a grid of two columns where the width allows (the
/// filter and the counts, then the entry) and one where it does not.
pub const COLUMNS: &str = "uigallery-todo-columns.ir.json";
/// The mixer surface (local fixtures, over `mixer()`): the volume slider
/// in the left panel, then in the right one.
pub const MIXER_LEFT: &str = "mixer-left.ir.json";
pub const MIXER_RIGHT: &str = "mixer-right.ir.json";
/// The silences app of the showcase (task-69 phase 4a), as
/// `rust/plevir/tests/fixtures` pins its projection, and the two forms of its
/// generated surface (local fixtures, phase 4b): the review, with the
/// video's frame and one per cut, and the comparison of the original
/// with the result, side by side, over the same cuts.
pub const SILENCES: &str = "uigallery-silences.snapshot.json";
pub const REVIEW_SILENCES: &str = "uigallery-silences-review.ir.json";
pub const COMPARE_SILENCES: &str = "uigallery-silences-compare.ir.json";

/// A small app with a slider, which no fixture of task-68 has: a volume
/// from 0 to 1 that `set_value` sets and `reset` puts back to 0, and a
/// mute that `toggle` flips.
pub fn mixer() -> Snapshot {
    let mut b = SnapshotBuilder::new();
    b.node("mixer", Role::Region, "mixer");
    let mut volume = b.node("mixer.volume", Role::Slider, "volume");
    volume
        .parent("mixer")
        .state("value", 0.5)
        .constraints(Constraints {
            editable: true,
            range: Some((0.0, 1.0)),
            ..Constraints::default()
        });
    volume
        .action("set_value")
        .param(ParamSpec {
            kind: ValueKind::Num,
            constrained: true,
        })
        .reads("mixer.volume:value");
    volume.action("reset").reads("mixer.volume:value");
    let mut mute = b.node("mixer.mute", Role::Toggle, "mute");
    mute.parent("mixer").state("on", false);
    mute.action("toggle").reads("mixer.mute:on");
    b.finish().expect("the mixer projects")
}

/// Each IR fixture and the projection it is written against.
pub const CASES: [(&str, &str); 3] = [(COMMIT_FORM, IDE), (REVIEW, IDE), (TODO, SHOWCASE)];

/// One turn of a 60 Hz display.
pub const TURN: f32 = 1.0 / 60.0;

/// Draws `surface` in `area` and then, a turn apart, every frame it asks
/// for while its transitions run, as a shell under render on demand
/// would: the last frame, and how many were drawn after the first.
pub fn settle(surface: &mut Surface, area: Rect, theme: &Theme) -> (Compositor, usize) {
    let mut c = Compositor::new();
    surface.render(&mut c, area, theme);
    let mut frames = 0;
    loop {
        let wake = surface.wake(TURN);
        if wake.now {
            c = Compositor::new();
            surface.render(&mut c, area, theme);
            frames += 1;
        }
        if !wake.is_continuous() {
            return (c, frames);
        }
        assert!(frames < 600, "the transitions never settle");
    }
}

pub fn fixture(name: &str) -> String {
    let at = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../plevir/tests/fixtures")
        .join(name);
    std::fs::read_to_string(&at).unwrap_or_else(|e| panic!("{}: {e}", at.display()))
}

/// A fixture of this crate's own tests (`tests/fixtures`): the other
/// forms a surface transforms into.
pub fn local(name: &str) -> String {
    let at = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    std::fs::read_to_string(&at).unwrap_or_else(|e| panic!("{}: {e}", at.display()))
}

/// The local fixture `ir_name` verified against the registry of `app`.
pub fn verified_local(ir_name: &str, app: &Snapshot) -> VerifiedIr {
    let ir = parse(&local(ir_name)).unwrap_or_else(|errors| panic!("{ir_name}: {errors:#?}"));
    verify(
        &ir,
        &Capabilities::from_snapshot(app),
        &GenerationPolicy::default(),
    )
    .unwrap_or_else(|errors| panic!("{ir_name}: {errors:#?}"))
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
            ("set_value", Some(Value::Num(n))) => {
                self.state(&req.target, "value", Value::Num(*n));
            }
            ("reset", None) => self.state(&req.target, "value", Value::Num(0.0)),
            ("toggle", None) => {
                let on = self
                    .nodes
                    .get(&req.target)
                    .and_then(|n| n.state.get(&StateKey::parse("on").unwrap()));
                let on = matches!(on, Some(Value::Bool(true)));
                self.state(&req.target, "on", Value::Bool(!on));
            }
            ("stage" | "unstage", None) => {
                let on = req.action.as_str() == "stage";
                self.state(&req.target, "staged", Value::Bool(on));
                self.state(&req.target, "unstaged", Value::Bool(!on));
            }
            ("apply_cut" | "reject_cut", None) => {
                let decision = if req.action.as_str() == "apply_cut" {
                    "apply"
                } else {
                    "reject"
                };
                self.state(&req.target, "decision", Value::Text(decision.to_owned()));
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
