//! The Files page as a capability: what it means (`accord::Semantics`), what
//! it runs (`accord::Operable`), and the one path every actor takes to run
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
//! the agent before `execute` runs), `files.warn` (button: `warn`, the
//! platform's warning panel saying how many files were received and
//! that nothing was written; the same person-only policy, reading
//! `files:received` since the text counts it; the body shown is the
//! region's `alert`, empty until one was shown, so an agent reads what
//! the person saw) and `files.status` (`message`). The first two actions
//! read the list because both change it. The region's `notice` is the
//! body of the last notification the page asked for (files that landed
//! while no window had the focus), empty until one was.

use std::path::PathBuf;

use accord::{
    Actor, Actors, Execution, Operable, Outcome, ParamSpec, Policy, Receipt, Request, RequestId,
    Role, SemError, SemErrors, SemId, Semantics, Snapshot, SnapshotBuilder, Value, ValueKind,
};
use engine::runtime::perf::stages::{self, Stage};
use engine::runtime::platform::alerts::Alert;
use engine::runtime::platform::files::DialogRequest;
use engine::runtime::platform::notify::{Action, Notification};
use engine::widgets::prelude::{EventResult, Rect};

use super::{FilesPage, Said};
use crate::views::apps::todo::TodoPage;

/// Who may ask for a native panel (the file dialog, the alert): a person
/// at the screen or through assistive technology, and a test (over the
/// fake); never an agent.
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
    pub warn: SemId,
    pub status: SemId,
}

impl Ids {
    pub(super) fn new() -> Result<Self, SemError> {
        let region = SemId::name("files")?;
        Ok(Self {
            drop: region.child("drop")?,
            open: region.child("open")?,
            warn: region.child("warn")?,
            status: region.child("status")?,
            region,
        })
    }

    /// True for every id the page projects.
    pub(super) fn holds(&self, id: &SemId) -> bool {
        *id == self.region
            || *id == self.drop
            || *id == self.open
            || *id == self.warn
            || *id == self.status
    }

    fn target(&self, target: Target) -> SemId {
        match target {
            Target::Open => self.open.clone(),
            Target::Drop => self.drop.clone(),
            Target::Warn => self.warn.clone(),
        }
    }

    /// The button `id` names, if it is one of the page's two.
    fn button(&self, id: &SemId) -> Option<Target> {
        if *id == self.open {
            Some(Target::Open)
        } else if *id == self.warn {
            Some(Target::Warn)
        } else {
            None
        }
    }

    /// What a request accepted against the page asks it to do.
    fn op(&self, req: &Request) -> Option<Op> {
        match (req.action.as_str(), &req.param) {
            ("open_file", _) if req.target == self.open => Some(Op::Open),
            ("warn", _) if req.target == self.warn => Some(Op::Warn),
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
    Warn,
}

/// The one handler of each capability.
enum Op {
    Open,
    Drop(Vec<PathBuf>),
    Warn,
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
            .state("count", count(self.received.len()))
            .state("alert", self.alerted.as_str())
            .state("notice", self.notice.as_str());
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
        let mut warn = b.node(&ids.warn, Role::Button, "warn");
        warn.parent(&ids.region).focusable(true);
        warn.action("warn").reads(reads.as_str()).policy(Policy {
            actors: OPEN_FILE_ACTORS,
            ..Policy::default()
        });
        let message = self.status_text();
        b.node(&ids.status, Role::Status, message.as_str())
            .parent(&ids.region)
            .state("message", message.as_str());
        if let Some(focused) = self.focused {
            b.focus(ids.target(focused));
        }
    }
}

impl FilesPage {
    /// The warning the page shows: how many files it holds, and that
    /// nothing was written anywhere.
    fn warning(&self) -> Alert {
        let files = match self.received.len() {
            1 => "1 file".to_owned(),
            n => format!("{n} files"),
        };
        Alert::warning(
            "Files",
            format!("{files} received; nothing was written to disk."),
        )
    }

    /// What the person is told when `now` files land while no window has
    /// the focus: how many came, with the open button as the
    /// notification's own button (as the projection declares it: a
    /// person's, enabled), so a click on it runs `open_file` by the person.
    fn received_notice(&self, now: usize) -> Notification {
        let files = match now {
            1 => "1 file".to_owned(),
            n => format!("{n} files"),
        };
        let mut notice = Notification::info("Files", format!("{files} received"));
        if let Some(ids) = &self.ids
            && let Ok(seen) = self.snapshot()
            && let Some(open) = Action::of(&seen, &ids.open, "open_file")
        {
            notice = notice.action(open);
        }
        notice
    }
}

impl Operable for FilesPage {
    /// Everything here ends now: the dialog and the panel block until the
    /// person answers, so the request id is not kept.
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
                let now = paths.len();
                outcome.result.insert("received".to_owned(), count(now));
                self.said = Said::Received(now);
                self.received.extend(paths);
                // nobody was looking: the person is told, and the text
                // is state whether the platform shows it or not
                if !self.window_focused {
                    let notice = self.received_notice(now);
                    outcome
                        .result
                        .insert("notice".to_owned(), Value::Text(notice.body.clone()));
                    self.notice = notice.body.clone();
                    self.notices.push(notice);
                }
            }
            Op::Warn => {
                let alert = self.warning();
                // The page knows no window: the panel opens on its own,
                // not as a sheet (the shell owns the windows).
                if let Err(e) = self.alerts.alert(&alert, None) {
                    self.said = Said::NoAlert;
                    return Execution::Failed(e.to_string());
                }
                outcome
                    .result
                    .insert("alert".to_owned(), Value::Text(alert.body.clone()));
                self.alerted = alert.body;
                self.said = Said::Warned;
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

    /// Keyboard focus asked from outside the page's pointer handling: a
    /// button takes it; another node of the page takes it from the
    /// buttons. `false` for an id the page does not project, and while
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
        let on = ids.button(id);
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
