//! The Files page as a capability: what it means (`sem::Semantics`), what
//! it runs (`sem::Operable`), and the one path every actor takes to run
//! it, the gallery's dispatcher (the todo page's, lent by
//! `TodoPage::with_dispatcher` as to the Surface and Silences pages: the
//! agent port adds its sink to that one dispatcher and matches acts by
//! `RequestId`).
//!
//! The semantic model: `files` (region; `received`, the paths received
//! in the order they came, and `count`), `files.drop` (region: the zone;
//! `drop_files` with a list of paths as its param, reading
//! `files:received`, for every actor: a drop by the person and a drop by
//! an agent through the port are the same capability), `files.open`
//! (button: `open_file`, reading `files:received`; a person at the
//! screen, assistive technology and a test may ask, an agent may not: a
//! native dialog is opened only by a person, and the dispatcher refuses
//! the agent before `execute` runs) and `files.status` (`message`).
//! Both actions read the list because both change it.

use std::path::PathBuf;

use comps::prelude::{EventResult, Rect};
use engine::perf::stages::{self, Stage};
use engine::platform::files::DialogRequest;
use sem::{
    Actor, Actors, Execution, Operable, Outcome, ParamSpec, Policy, Receipt, Request, RequestId,
    Role, SemError, SemErrors, SemId, Semantics, Snapshot, SnapshotBuilder, Value, ValueKind,
};

use super::{FilesPage, Said};
use crate::view::apps::app::TodoPage;

/// Who may ask for the native dialog: a person at the screen or through
/// assistive technology, and a test (over the fake); never an agent.
pub(super) const OPEN_FILE_ACTORS: Actors = Actors {
    human: true,
    agent: false,
    assistive: true,
    test: true,
};

/// The ids the page projects and binds, parsed once.
pub(super) struct Ids {
    pub region: SemId,
    pub drop: SemId,
    pub open: SemId,
    pub status: SemId,
}

impl Ids {
    pub(super) fn new() -> Result<Self, SemError> {
        let region = SemId::name("files")?;
        Ok(Self {
            drop: region.child("drop")?,
            open: region.child("open")?,
            status: region.child("status")?,
            region,
        })
    }

    /// True for every id the page projects.
    pub(super) fn holds(&self, id: &SemId) -> bool {
        *id == self.region || *id == self.drop || *id == self.open || *id == self.status
    }

    fn target(&self, target: Target) -> SemId {
        match target {
            Target::Open => self.open.clone(),
            Target::Drop => self.drop.clone(),
        }
    }

    /// What a request accepted against the page asks it to do.
    fn op(&self, req: &Request) -> Option<Op> {
        match (req.action.as_str(), &req.param) {
            ("open_file", _) if req.target == self.open => Some(Op::Open),
            ("drop_files", Some(Value::List(paths))) if req.target == self.drop => {
                Some(Op::Drop(paths.iter().map(PathBuf::from).collect()))
            }
            _ => None,
        }
    }
}

/// What the page's own input asks for, named without the ids.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Target {
    Open,
    Drop,
}

/// The one handler of each capability.
enum Op {
    Open,
    Drop(Vec<PathBuf>),
}

/// `(x, y, w, h)`, the form `Compositor::bind` takes.
pub(super) fn bounds(r: Rect) -> [f32; 4] {
    [r.x, r.y, r.w, r.h]
}

fn count(n: usize) -> Value {
    Value::Int(i64::try_from(n).unwrap_or(i64::MAX))
}

impl Semantics for FilesPage {
    fn project(&self, b: &mut SnapshotBuilder) {
        let Some(ids) = &self.ids else {
            return;
        };
        let received: Vec<String> = self
            .received
            .iter()
            .map(|p| p.display().to_string())
            .collect();
        b.node(&ids.region, Role::Region, "files")
            .inert(self.inert)
            .state("received", Value::List(received))
            .state("count", count(self.received.len()));
        let reads = format!("{}:received", ids.region);
        let mut zone = b.node(&ids.drop, Role::Region, "drop files here");
        zone.parent(&ids.region);
        zone.action("drop_files")
            .param(ParamSpec {
                kind: ValueKind::List,
                constrained: false,
            })
            .reads(reads.as_str());
        let mut open = b.node(&ids.open, Role::Button, "open file");
        open.parent(&ids.region).focusable(true);
        open.action("open_file")
            .reads(reads.as_str())
            .policy(Policy {
                actors: OPEN_FILE_ACTORS,
                ..Policy::default()
            });
        let message = self.status_text();
        b.node(&ids.status, Role::Status, message.as_str())
            .parent(&ids.region)
            .state("message", message.as_str());
        if self.focused {
            b.focus(&ids.open);
        }
    }
}

impl Operable for FilesPage {
    /// Everything here ends now: the dialog blocks until the person
    /// answers, so the request id is not kept.
    fn execute(&mut self, _id: RequestId, req: &Request) -> Execution {
        let Some(op) = self.ids.as_ref().and_then(|ids| ids.op(req)) else {
            return Execution::Failed(format!(
                "the files page has no {} on {}",
                req.action, req.target
            ));
        };
        let mut outcome = Outcome::default();
        match op {
            Op::Open => {
                let ask = DialogRequest::new().title("Open a file");
                match self.dialogs.pick_file(&ask) {
                    Ok(Some(path)) => {
                        outcome
                            .result
                            .insert("path".to_owned(), Value::Text(path.display().to_string()));
                        self.received.push(path);
                        self.said = Said::Received(1);
                    }
                    Ok(None) => {
                        outcome
                            .result
                            .insert("cancelled".to_owned(), Value::Bool(true));
                        self.said = Said::Cancelled;
                    }
                    Err(e) => {
                        self.said = Said::NoDialog;
                        return Execution::Failed(e.to_string());
                    }
                }
            }
            Op::Drop(paths) => {
                if paths.is_empty() {
                    return Execution::Failed("the drop carries no file".to_owned());
                }
                outcome
                    .result
                    .insert("received".to_owned(), count(paths.len()));
                self.said = Said::Received(paths.len());
                self.received.extend(paths);
            }
        }
        outcome
            .result
            .insert("count".to_owned(), count(self.received.len()));
        Execution::Done(outcome)
    }
}

impl FilesPage {
    /// The page projected now.
    pub(crate) fn snapshot(&self) -> Result<Snapshot, SemErrors> {
        let _probe = stages::span(Stage::Projection);
        let mut b = SnapshotBuilder::new();
        self.project(&mut b);
        b.finish()
    }

    /// Runs `req` through the gallery's dispatcher (the todo page lends
    /// it, with its clock's now), over this page.
    pub(crate) fn dispatch(&mut self, req: Request, todo: &mut TodoPage) -> RequestId {
        todo.with_dispatcher(|d, _, now| d.dispatch(self, req, now))
    }

    /// A request from an actor outside the page's own input (an agent
    /// through the port, a screen reader), through the gallery's one
    /// dispatcher; the receipt as it stands after the dispatch.
    pub(crate) fn act(
        &mut self,
        req: Request,
        todo: &mut TodoPage,
        inert: bool,
    ) -> Option<Receipt> {
        self.set_inert(inert);
        let id = self.dispatch(req, todo);
        todo.receipt(id).cloned()
    }

    /// Keyboard focus asked from outside the page's pointer handling: the
    /// button takes it; another node of the page takes it from the
    /// button. `false` for an id the page does not project, and while
    /// inert.
    pub(crate) fn focus(&mut self, id: &SemId) -> bool {
        if self.inert {
            return false;
        }
        let Some(ids) = &self.ids else {
            return false;
        };
        if !ids.holds(id) {
            return false;
        }
        let on = *id == ids.open;
        self.set_focus(on);
        true
    }

    /// A person's click, key or drop as a request: observe the page as
    /// it is now, ask through the dispatcher, and ask for a frame when
    /// the projected state moved. A refused request is still handled:
    /// the person aimed at the page.
    pub(super) fn ask(
        &mut self,
        todo: &mut TodoPage,
        target: Target,
        action: &str,
        param: Option<Value>,
    ) -> EventResult {
        let Some(target) = self.ids.as_ref().map(|ids| ids.target(target)) else {
            return EventResult::IGNORED;
        };
        let seen = match self.snapshot() {
            Ok(seen) => seen,
            Err(errors) => {
                log::warn!("files: the projection does not validate: {errors}");
                return EventResult::IGNORED;
            }
        };
        let req = match Request::observe(&seen, Actor::Human, &target, action, param) {
            Ok(req) => req,
            Err(e) => {
                log::warn!("files: {e}");
                return EventResult::IGNORED;
            }
        };
        let id = self.dispatch(req, todo);
        log::debug!("files: {target} {action}: {:?}", todo.receipt(id));
        let moved = self
            .snapshot()
            .map_or(true, |now| now.revision() != seen.revision());
        if moved {
            EventResult::clicked()
        } else {
            EventResult {
                handled: true,
                ..EventResult::IGNORED
            }
        }
    }
}
