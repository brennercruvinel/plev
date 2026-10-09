//! the one path from a request to its effect: `Dispatcher`. a person, an
//! agent, assistive technology and a test all go through it; there is no
//! second way to run an action.

use std::collections::{BTreeMap, VecDeque};
use std::fmt;

use crate::assembly::{Semantics, SnapshotBuilder};
use crate::decision::{Acted, Rejection, Verdict, check};
use crate::failures::SemError;
use crate::receipts::{Mark, Millis, Outcome, Receipt, RequestId, Stage};
use crate::requests::{Actor, Request};

/// an app whose actions run through the dispatcher.
///
/// the app owns its state and the execution. the dispatcher calls
/// `execute` only for a request that was accepted, or confirmed by a
/// person, against a snapshot projected from the live state at that
/// instant.
pub trait Operable: Semantics {
    /// runs the request `id` names. when the truth lives outside the app
    /// (the git index, a server), the request goes there with its observed
    /// values and the point of truth checks them again before the effect,
    /// with `check` against its own state; if they disagree it answers
    /// `Execution::Rejected` with the delta it saw. an execution that ends
    /// later keeps `id` to report how it ended (`Execution::Started`).
    fn execute(&mut self, id: RequestId, req: &Request) -> Execution;
}

/// what `execute` did.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum Execution {
    /// finished now, with this outcome.
    Done(Outcome),
    /// started. the app holds the id `execute` received and, when the
    /// work ends, calls `Dispatcher::complete`, `fail` or `reject_at_truth`
    /// with it; no other channel is needed to know which request ended.
    Started,
    /// could not run.
    Failed(String),
    /// the point of truth disagreed with the request (a stale index).
    Rejected(Rejection),
}

/// sees every stage of every receipt, in order, as it happens: `receipt`
/// already holds `stage` as its last entry. a recorder, a log, the agent
/// protocol.
///
/// a sink that serves observations (the agent port) also knows when an
/// actor looked, and tells the dispatcher through `looked`; the other two
/// methods do nothing by default.
pub trait ReceiptSink {
    fn on_stage(&mut self, receipt: &Receipt, stage: &Stage);

    /// the dispatcher took this sink at `mark` (`Dispatcher::add_sink`):
    /// every request it takes from now on comes after it, and reaches
    /// `on_stage` first as `Stage::Requested`.
    fn attached(&mut self, mark: Mark) {
        let _ = mark;
    }

    /// when the actor of `receipt` looked at what its request rests on,
    /// if this sink knows: the dispatcher's mark at that instant (the agent
    /// port knows it for the requests it dispatched, since it served the
    /// observations). the dispatcher reads it only to name who acted
    /// (`Acted`), for a stale refusal of that receipt, and never to decide.
    fn looked(&self, receipt: &Receipt) -> Option<Mark> {
        let _ = receipt;
        None
    }
}

/// how many finished receipts `Dispatcher::new` keeps.
pub const DEFAULT_RETENTION: usize = 256;

/// takes requests from every actor, validates each one against the live
/// state and keeps its receipt.
///
/// open receipts (awaiting confirmation, executing) stay until they end.
/// a finished receipt goes to the sinks stage by stage as it happens, then
/// stays readable through `receipt` among the last `retention` finished
/// ones, and is dropped after that: nothing grows without bound but the
/// requests still open.
///
/// every stale refusal it records (at dispatch, at a confirmation's second
/// check, from the point of truth through `execute` or `reject_at_truth`)
/// names in `Rejection::Stale::acted` the actions that ran since the
/// refused request's actor looked, when a sink knows the look
/// (`ReceiptSink::looked`), or those of other actors since the state it
/// observed, read from those same receipts (see `Acted`); whatever the
/// refusal held there is replaced.
pub struct Dispatcher {
    last: u64,
    open: BTreeMap<RequestId, Receipt>,
    finished: VecDeque<Receipt>,
    retention: usize,
    sinks: Vec<Box<dyn ReceiptSink>>,
}

impl Default for Dispatcher {
    fn default() -> Self {
        Dispatcher::new()
    }
}

impl fmt::Debug for Dispatcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Dispatcher")
            .field("last", &self.last)
            .field("open", &self.open)
            .field("finished", &self.finished)
            .field("retention", &self.retention)
            .field("sinks", &self.sinks.len())
            .finish()
    }
}

impl Dispatcher {
    /// a dispatcher that keeps the last `DEFAULT_RETENTION` finished
    /// receipts.
    pub fn new() -> Self {
        Dispatcher::with_retention(DEFAULT_RETENTION)
    }

    /// a dispatcher that keeps the last `retention` finished receipts;
    /// with zero, a receipt is dropped as soon as the sinks saw its last
    /// stage.
    pub fn with_retention(retention: usize) -> Self {
        Dispatcher {
            last: 0,
            open: BTreeMap::new(),
            finished: VecDeque::new(),
            retention,
            sinks: Vec::new(),
        }
    }

    /// adds a sink; it sees the stages that happen from now on, and is
    /// told the mark it came at (`ReceiptSink::attached`).
    pub fn add_sink(&mut self, mut sink: Box<dyn ReceiptSink>) {
        sink.attached(self.mark());
        self.sinks.push(sink);
    }

    /// the mark of now: every request taken from here on comes after it.
    pub fn mark(&self) -> Mark {
        Mark::new(self.last)
    }

    /// takes a request at `now`. it projects the app's live state, checks
    /// the request against that snapshot (never against a frame drawn
    /// earlier) and, when it is accepted, executes it. a request that
    /// needs confirmation (its policy asks for one, or its agent asks a
    /// person, `Request::ask`) waits for a person's `confirm` or
    /// `decline`, for as long as nobody answers. the verdict and the
    /// result are in the receipt; the id names it.
    ///
    /// when the app's projection does not validate, nothing can be
    /// checked: the request is rejected with `Rejection::InvalidProjection`
    /// and the receipt has no `validated_against`.
    pub fn dispatch<A: Operable>(&mut self, app: &mut A, req: Request, now: Millis) -> RequestId {
        self.last = self.last.wrapping_add(1);
        let id = RequestId::new(self.last);
        let mut receipt = Receipt::new(id, req);
        self.push(&mut receipt, Stage::Requested, now);
        match validate(app, &mut receipt) {
            Verdict::Accepted => {
                self.push(&mut receipt, Stage::Accepted, now);
                self.run(app, &mut receipt, now);
            }
            Verdict::NeedsConfirmation => {
                self.push(&mut receipt, Stage::AwaitingConfirmation, now);
            }
            Verdict::Rejected(why) => self.push(&mut receipt, Stage::Rejected(why), now),
        }
        self.store(receipt);
        id
    }

    /// a person confirms a request that awaits confirmation: at the
    /// screen (`Actor::Human`) or through assistive technology
    /// (`Actor::Assistive`), since a person using a screen reader is a
    /// person. an agent or a test never confirms. the request is checked
    /// again against the live state first: if it went stale (or anything
    /// else now refuses it) while it waited, the receipt ends rejected with
    /// that reason and nothing runs; otherwise it is confirmed and
    /// executed.
    ///
    /// errors, with the receipt unchanged: `UnknownRequest` for an id the
    /// dispatcher does not hold, `IllegalTransition` when the request is
    /// not awaiting confirmation, `ConfirmByMachine` when `by` is an agent
    /// or a test.
    pub fn confirm<A: Operable>(
        &mut self,
        app: &mut A,
        id: RequestId,
        by: Actor,
        now: Millis,
    ) -> Result<(), SemError> {
        self.require_person(id, &by, "confirmed")?;
        let mut receipt = self.open.remove(&id).ok_or(SemError::UnknownRequest(id))?;
        match validate(app, &mut receipt) {
            Verdict::Accepted | Verdict::NeedsConfirmation => {
                self.push(&mut receipt, Stage::Confirmed { by }, now);
                self.run(app, &mut receipt, now);
            }
            Verdict::Rejected(why) => self.push(&mut receipt, Stage::Rejected(why), now),
        }
        self.store(receipt);
        Ok(())
    }

    /// a person declines a request that awaits confirmation, with the
    /// same rules as `confirm`: at the screen (`Actor::Human`) or through
    /// assistive technology (`Actor::Assistive`), never an agent or a
    /// test. the receipt ends rejected with `Rejection::Declined { by }`
    /// and nothing runs, so the live state is not checked again and
    /// `validated_against` stays the one of the dispatch.
    ///
    /// errors, with the receipt unchanged: `UnknownRequest` for an id the
    /// dispatcher does not hold, `IllegalTransition` when the request is
    /// not awaiting confirmation, `ConfirmByMachine` when `by` is an agent
    /// or a test.
    pub fn decline(&mut self, id: RequestId, by: Actor, now: Millis) -> Result<(), SemError> {
        self.require_person(id, &by, "rejected")?;
        let mut receipt = self.open.remove(&id).ok_or(SemError::UnknownRequest(id))?;
        self.push(
            &mut receipt,
            Stage::Rejected(Rejection::Declined { by }),
            now,
        );
        self.store(receipt);
        Ok(())
    }

    /// an execution that started ends with `outcome`.
    ///
    /// errors, with the receipt unchanged: `UnknownRequest`, or
    /// `IllegalTransition` when the request is not executing.
    pub fn complete(
        &mut self,
        id: RequestId,
        outcome: Outcome,
        now: Millis,
    ) -> Result<(), SemError> {
        self.end(id, Stage::Completed(outcome), now)
    }

    /// an execution that started failed with `error`.
    ///
    /// errors, with the receipt unchanged: `UnknownRequest`, or
    /// `IllegalTransition` when the request is not executing.
    pub fn fail(&mut self, id: RequestId, error: String, now: Millis) -> Result<(), SemError> {
        self.end(id, Stage::Failed { error }, now)
    }

    /// the point of truth refused an execution that started: the receipt
    /// goes from executing to rejected with `rejection` (a stale index and
    /// the delta seen there).
    ///
    /// errors, with the receipt unchanged: `UnknownRequest`, or
    /// `IllegalTransition` when the request is not executing (a request
    /// awaiting confirmation never reached a point of truth).
    pub fn reject_at_truth(
        &mut self,
        id: RequestId,
        rejection: Rejection,
        now: Millis,
    ) -> Result<(), SemError> {
        self.end(id, Stage::Rejected(rejection), now)
    }

    /// the receipt of `id`: open, or among the finished ones retention
    /// keeps.
    pub fn receipt(&self, id: RequestId) -> Option<&Receipt> {
        self.open
            .get(&id)
            .or_else(|| self.finished.iter().find(|r| r.id == id))
    }

    /// ends an executing request with a terminal stage.
    fn end(&mut self, id: RequestId, stage: Stage, now: Millis) -> Result<(), SemError> {
        self.require(id, &Stage::Executing, stage.name())?;
        let mut receipt = self.open.remove(&id).ok_or(SemError::UnknownRequest(id))?;
        self.push(&mut receipt, stage, now);
        self.store(receipt);
        Ok(())
    }

    /// `Ok` when `id` is open and at stage `at`; otherwise why it cannot
    /// go `to` the next stage: it is not held (`UnknownRequest`), or it is
    /// at another stage (`IllegalTransition`).
    fn require(&self, id: RequestId, at: &Stage, to: &str) -> Result<(), SemError> {
        if self.open.get(&id).and_then(Receipt::stage) == Some(at) {
            return Ok(());
        }
        Err(match self.receipt(id) {
            None => SemError::UnknownRequest(id),
            Some(r) => SemError::IllegalTransition {
                id,
                from: r.stage().map_or("none", Stage::name).to_owned(),
                to: to.to_owned(),
            },
        })
    }

    /// the guards a person's answer to a confirmation passes, in this
    /// order: `id` awaits confirmation (see `require`, going `to` the
    /// answer's stage), and `by` is a person (`ConfirmByMachine`
    /// otherwise).
    fn require_person(&self, id: RequestId, by: &Actor, to: &str) -> Result<(), SemError> {
        self.require(id, &Stage::AwaitingConfirmation, to)?;
        match by {
            Actor::Human | Actor::Assistive => Ok(()),
            Actor::Agent { .. } | Actor::Test { .. } => {
                Err(SemError::ConfirmByMachine { id, by: by.clone() })
            }
        }
    }

    /// executes an accepted or confirmed request and records what came
    /// back.
    fn run<A: Operable>(&mut self, app: &mut A, receipt: &mut Receipt, now: Millis) {
        self.push(receipt, Stage::Executing, now);
        match app.execute(receipt.id, &receipt.request) {
            Execution::Done(outcome) => self.push(receipt, Stage::Completed(outcome), now),
            Execution::Started => {}
            Execution::Failed(error) => self.push(receipt, Stage::Failed { error }, now),
            Execution::Rejected(why) => self.push(receipt, Stage::Rejected(why), now),
        }
    }

    /// appends a stage the caller already knows is legal and tells every
    /// sink. a stale refusal gets its `acted` here, the one place every
    /// stage goes through.
    fn push(&mut self, receipt: &mut Receipt, mut stage: Stage, now: Millis) {
        if let Stage::Rejected(Rejection::Stale { acted, .. }) = &mut stage {
            *acted = self.acted(receipt);
        }
        receipt.push(stage, now);
        if let Some((stage, _)) = receipt.stages.last() {
            for sink in &mut self.sinks {
                sink.on_stage(receipt, stage);
            }
        }
    }

    /// who acted since the refused `receipt`'s actor looked, in id order
    /// (see `Acted`). with the look a sink knows (`ReceiptSink::looked`):
    /// every request that ran and was taken after it, the author's own
    /// included. without one: the actions of other actors from the first
    /// request, of any actor, that ran after being checked against the
    /// state the author observed; ids grow in dispatch order, so that one
    /// and every later one ran after the actor looked. only the receipts
    /// held here count: the open ones and the finished ones retention
    /// keeps.
    fn acted(&self, receipt: &Receipt) -> Vec<Acted> {
        let req = &receipt.request;
        let held = || {
            self.open
                .values()
                .chain(&self.finished)
                .filter(|r| r.id != receipt.id && ran(r))
        };
        let named: Vec<&Receipt> = match self.sinks.iter().find_map(|s| s.looked(receipt)) {
            Some(look) => held().filter(|r| look.precedes(r.id)).collect(),
            None => {
                let seen = Some(req.observed.revision);
                let Some(start) = held()
                    .filter(|r| r.validated_against == seen)
                    .map(|r| r.id)
                    .min()
                else {
                    return Vec::new();
                };
                held()
                    .filter(|r| r.id >= start && r.request.actor != req.actor)
                    .collect()
            }
        };
        let mut out: Vec<Acted> = named
            .into_iter()
            .map(|r| Acted {
                id: r.id,
                actor: r.request.actor.clone(),
                target: r.request.target.clone(),
                action: r.request.action.clone(),
            })
            .collect();
        out.sort_by_key(|a| a.id);
        out
    }

    /// files a receipt: open ones by id, finished ones in the bounded
    /// history.
    fn store(&mut self, receipt: Receipt) {
        if receipt.is_finished() {
            self.finished.push_back(receipt);
            while self.finished.len() > self.retention {
                self.finished.pop_front();
            }
        } else {
            self.open.insert(receipt.id, receipt);
        }
    }
}

/// true when the request reached execution: it is executing, or it
/// completed or failed after. a refusal (at the check or at the point of
/// truth), a decline and a wait for confirmation never ran.
fn ran(r: &Receipt) -> bool {
    matches!(
        r.stage(),
        Some(Stage::Executing | Stage::Completed(_) | Stage::Failed { .. })
    )
}

/// projects the app now and checks the receipt's request against it,
/// recording the revision it was validated against.
fn validate<A: Semantics>(app: &A, receipt: &mut Receipt) -> Verdict {
    let mut b = SnapshotBuilder::new();
    app.project(&mut b);
    match b.finish() {
        Ok(now) => {
            receipt.validated_against = Some(now.revision());
            check(&receipt.request, &now)
        }
        Err(errors) => {
            receipt.validated_against = None;
            Verdict::Rejected(Rejection::InvalidProjection { errors })
        }
    }
}
