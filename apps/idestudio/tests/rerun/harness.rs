//! the harness: an excerpt of a recorded session rerun, act by act, against
//! a fixture of the app, and compared with what was recorded.
//!
//! this is re-execution, under control, and it lives only here: the
//! fixture runs the app somewhere isolated (a temporary directory, the
//! clock the test gives where the app takes one, no network), and nothing
//! in the harness reaches past the `Fixture`. replay (the recorder, the
//! inspector) still runs nothing.
//!
//! what it does, in the order the excerpt recorded it:
//!
//! - every act is rerun: a request's first stage (`requested`) becomes a
//!   dispatch of the request as recorded (actor, target, action,
//!   parameter, and the values the actor observed); a person's
//!   confirmation and decline become the same answer. every other stage
//!   is the app's answer, which the rerun produces on its own.
//! - before each act the app settles (everything asynchronous finishes),
//!   unless the recording had an execution still running at that point:
//!   then the act goes in with that execution in flight, as it did.
//! - the marks due at that point are read (see `marks`): right before the
//!   first act recorded after their anchor, or at the end.
//! - at the end, every request's verdict (the stage after `requested`)
//!   and terminal stage are compared with the recorded ones: the same
//!   stage, and for a refusal the same kind; a stale refusal must name
//!   the same reads. times, revisions, `acted` and outcome values are not
//!   compared (a mark checks an outcome value when one matters).

use std::collections::HashMap;

use rec::{Kind, Session};
use sem::{Actor, Millis, Receipt, Rejection, Request, RequestId, Snapshot, Stage, Value};

use crate::marks::{Anchor, Mark, What, render};
use crate::report::{
    Compared, MarkLine, Report, RequestLine, alike, describe, stale_reads, verdict,
};

/// what an app gives the harness to rerun its sessions.
///
/// the fixture is built by the closure `rerun` takes: the app in the state
/// the excerpt starts from, in an isolated environment. everything else
/// the harness does goes through these methods.
pub trait Fixture {
    /// takes `req` through the app's one dispatcher, as a live request
    /// would come. `t` is when the recording took it, on the recorder's
    /// clock: the time to stamp with, for an app that takes its time from
    /// the caller.
    fn dispatch(&mut self, req: Request, t: Millis) -> RequestId;

    /// a person (`by`) confirms request `id`, which waits for it.
    fn confirm(&mut self, id: RequestId, by: Actor, t: Millis) -> Result<(), String>;

    /// a person (`by`) declines request `id`, which waits for it.
    fn decline(&mut self, id: RequestId, by: Actor, t: Millis) -> Result<(), String>;

    /// lets everything asynchronous the app started finish: every
    /// execution that left ends, every read it asked for lands. an error
    /// when it does not, in the time the fixture allows.
    fn settle(&mut self) -> Result<(), String>;

    /// the app projected now.
    fn project(&self) -> Result<Snapshot, String>;

    /// the receipt of `id`, as the app's dispatcher holds it.
    fn receipt(&self, id: RequestId) -> Option<Receipt>;
}

/// a fixture lent to the harness: the test keeps it, and reads it after the
/// rerun.
impl<T: Fixture + ?Sized> Fixture for &mut T {
    fn dispatch(&mut self, req: Request, t: Millis) -> RequestId {
        (**self).dispatch(req, t)
    }

    fn confirm(&mut self, id: RequestId, by: Actor, t: Millis) -> Result<(), String> {
        (**self).confirm(id, by, t)
    }

    fn decline(&mut self, id: RequestId, by: Actor, t: Millis) -> Result<(), String> {
        (**self).decline(id, by, t)
    }

    fn settle(&mut self) -> Result<(), String> {
        (**self).settle()
    }

    fn project(&self) -> Result<Snapshot, String> {
        (**self).project()
    }

    fn receipt(&self, id: RequestId) -> Option<Receipt> {
        (**self).receipt(id)
    }
}

/// one request of the excerpt, as recorded: every stage with the index of
/// the event it came in.
struct Recorded {
    origin: u16,
    id: RequestId,
    request: Request,
    stages: Vec<(Stage, usize)>,
}

impl Recorded {
    /// the recorded stage it was at right before event `i`.
    fn at(&self, i: usize) -> Option<&Stage> {
        self.stages
            .iter()
            .take_while(|(_, at)| *at < i)
            .last()
            .map(|(s, _)| s)
    }

    /// the event after which it is over in the recording: its terminal
    /// stage's; `None` while it is still open at the end of the excerpt.
    fn over(&self) -> Option<usize> {
        self.stages
            .last()
            .filter(|(s, _)| s.is_terminal())
            .map(|(_, at)| *at)
    }

    fn what(&self) -> String {
        let r = &self.request;
        format!("{} on {} by {}", r.action, r.target, r.actor)
    }
}

/// an act of the recording, rerun in its order.
enum Act {
    Dispatch,
    Confirm(Actor),
    Decline(Actor),
}

/// one act, with the request it belongs to (an index into the recorded
/// requests), the index of its event and its time.
struct Step {
    act: Act,
    request: usize,
    index: usize,
    t: Millis,
}

/// the requests and the acts of an excerpt. a request whose first stage
/// the excerpt does not hold (it started before the cut) cannot be rerun:
/// its request would be dispatched without what came before it.
fn script(excerpt: &Session) -> Result<(Vec<Recorded>, Vec<Step>), String> {
    let mut recorded: Vec<Recorded> = Vec::new();
    let mut by_key: HashMap<(u16, RequestId), usize> = HashMap::new();
    let mut steps = Vec::new();
    for (index, event) in excerpt.events().iter().enumerate() {
        let Kind::Stage {
            origin,
            id,
            request,
            stage,
            ..
        } = &event.kind
        else {
            continue;
        };
        let at = *by_key.entry((*origin, *id)).or_insert_with(|| {
            recorded.push(Recorded {
                origin: *origin,
                id: *id,
                request: (**request).clone(),
                stages: Vec::new(),
            });
            recorded.len() - 1
        });
        let Some(this) = recorded.get_mut(at) else {
            continue;
        };
        if this.stages.is_empty() && *stage != Stage::Requested {
            return Err(format!(
                "request {id} (origin {origin}) started before the excerpt: its first stage \
                 here is {}; export from a keyframe before it",
                stage.name()
            ));
        }
        this.stages.push((stage.clone(), index));
        let act = match stage {
            Stage::Requested => Act::Dispatch,
            Stage::Confirmed { by } => Act::Confirm(by.clone()),
            Stage::Rejected(Rejection::Declined { by }) => Act::Decline(by.clone()),
            _ => continue,
        };
        steps.push(Step {
            act,
            request: at,
            index,
            t: event.t,
        });
    }
    Ok((recorded, steps))
}

/// the recorded request a mark's anchor names, by index.
fn anchored(recorded: &[Recorded], mark: &Mark) -> Result<Option<usize>, String> {
    let Anchor::After { id, origin } = &mark.anchor else {
        return Ok(None);
    };
    let found: Vec<usize> = recorded
        .iter()
        .enumerate()
        .filter(|(_, r)| r.id == *id && origin.is_none_or(|o| r.origin == o))
        .map(|(n, _)| n)
        .collect();
    match found.as_slice() {
        [one] => Ok(Some(*one)),
        [] => Err(format!(
            "mark on line {}: the excerpt holds no request {id}{}",
            mark.line,
            origin.map_or(String::new(), |o| format!(" of origin {o}"))
        )),
        _ => Err(format!(
            "mark on line {}: request {id} is in several dispatchers; add `origin <n>`",
            mark.line
        )),
    }
}

/// the rerun, in progress.
struct Run<'a, F> {
    fixture: F,
    recorded: &'a [Recorded],
    /// the rerun's id of each recorded request, once dispatched.
    ids: Vec<Option<RequestId>>,
    /// the last receipt read of each, kept: an app may drop old ones.
    seen: Vec<Option<Receipt>>,
    refused: Vec<Option<String>>,
    marks: Vec<(&'a Mark, Option<usize>)>,
    read: Vec<Option<MarkLine>>,
    start: Millis,
}

impl<F: Fixture> Run<'_, F> {
    /// true when the recording had an execution still running right
    /// before event `i`.
    fn busy(&self, i: usize) -> bool {
        self.recorded
            .iter()
            .any(|r| matches!(r.at(i), Some(Stage::Executing)))
    }

    /// keeps the receipt of every request the rerun dispatched.
    fn keep_receipts(&mut self) {
        for (n, id) in self.ids.iter().enumerate() {
            if let Some(receipt) = id.and_then(|id| self.fixture.receipt(id)) {
                if let Some(slot) = self.seen.get_mut(n) {
                    *slot = Some(receipt);
                }
            }
        }
    }

    /// reads every mark due right before the act at event `i` and time `t`
    /// (`next`), or at the end (`None`).
    fn read_marks(&mut self, next: Option<(usize, Millis)>) -> Result<(), String> {
        for n in 0..self.marks.len() {
            let Some(&(mark, request)) = self.marks.get(n) else {
                continue;
            };
            if self.read.get(n).is_some_and(Option::is_some) {
                continue;
            }
            let due = match (next, &mark.anchor, request) {
                (None, _, _) => true,
                (Some((i, _)), Anchor::After { .. }, Some(r)) => self
                    .recorded
                    .get(r)
                    .and_then(Recorded::over)
                    .is_some_and(|over| over < i),
                (Some((_, t)), Anchor::At(ms), _) => t > self.start.saturating_add(*ms),
                (Some(_), Anchor::After { .. }, None) => false,
            };
            if !due {
                continue;
            }
            let found = self.found(mark, request)?;
            let line = MarkLine {
                line: mark.line,
                text: mark.text.clone(),
                same: found.as_ref().is_ok_and(|v| *v == mark.value),
                found: match found {
                    Ok(v) => render(v.as_ref()),
                    Err(why) => why,
                },
            };
            if let Some(slot) = self.read.get_mut(n) {
                *slot = Some(line);
            }
        }
        Ok(())
    }

    /// the value a mark reads in the rerun now; `Err` when it cannot be
    /// read (the anchored request did not complete), said in words.
    fn found(
        &self,
        mark: &Mark,
        request: Option<usize>,
    ) -> Result<Result<Option<Value>, String>, String> {
        let receipt = request
            .and_then(|r| self.seen.get(r))
            .and_then(Option::as_ref);
        Ok(match &mark.what {
            What::State(at) => Ok(self.fixture.project()?.value(at).cloned()),
            What::Outcome(key) => match receipt.and_then(Receipt::stage) {
                Some(Stage::Completed(outcome)) => Ok(outcome.result.get(key).cloned()),
                other => Err(format!(
                    "the request did not complete in the rerun: {}",
                    describe(other)
                )),
            },
            What::Stale => Ok(receipt.and_then(|r| {
                r.stages.iter().find_map(|(s, _)| match s {
                    Stage::Rejected(Rejection::Stale { changes, .. }) => {
                        Some(Value::Set(stale_reads(changes).into_iter().collect()))
                    }
                    _ => None,
                })
            })),
        })
    }

    /// a rest point, or a busy one, right before `step`, or the end.
    fn before(&mut self, next: Option<&Step>) -> Result<(), String> {
        let busy = next.is_some_and(|s| self.busy(s.index));
        if !busy {
            self.fixture.settle()?;
        }
        self.keep_receipts();
        self.read_marks(next.map(|s| (s.index, s.t)))
    }

    fn act(&mut self, step: &Step) {
        let Some(recorded) = self.recorded.get(step.request) else {
            return;
        };
        let id = self.ids.get(step.request).copied().flatten();
        let refused = match (&step.act, id) {
            (Act::Dispatch, _) => {
                let id = self.fixture.dispatch(recorded.request.clone(), step.t);
                if let Some(slot) = self.ids.get_mut(step.request) {
                    *slot = Some(id);
                }
                None
            }
            (Act::Confirm(by), Some(id)) => self.fixture.confirm(id, by.clone(), step.t).err(),
            (Act::Decline(by), Some(id)) => self.fixture.decline(id, by.clone(), step.t).err(),
            (_, None) => Some("the request was never dispatched".to_owned()),
        };
        if let (Some(why), Some(slot)) = (refused, self.refused.get_mut(step.request)) {
            let verb = match step.act {
                Act::Confirm(_) => "confirmed",
                _ => "declined",
            };
            *slot = Some(format!(
                "the recording {verb} it; the rerun could not: {why}"
            ));
        }
    }

    fn report(self, recorded_by: String) -> Report {
        let requests = self
            .recorded
            .iter()
            .zip(&self.ids)
            .zip(self.seen.iter().zip(&self.refused))
            .map(|((r, id), (seen, refused))| {
                let stages = seen.as_ref().map(|s| &s.stages[..]).unwrap_or_default();
                let want = verdict(r.stages.iter().map(|(s, _)| s));
                let got = verdict(stages.iter().map(|(s, _)| s));
                let want_end = r.stages.last().map(|(s, _)| s).filter(|s| s.is_terminal());
                let got_end = stages.last().map(|(s, _)| s).filter(|s| s.is_terminal());
                // a request still open when the excerpt ends has no recorded
                // terminal stage to hold the rerun to.
                let open = want_end.is_none();
                RequestLine {
                    origin: r.origin,
                    id: r.id,
                    rerun: *id,
                    what: r.what(),
                    verdict: Compared {
                        recorded: describe(want),
                        rerun: describe(got),
                        same: alike(want, got),
                    },
                    terminal: Compared {
                        recorded: if open {
                            "open when the excerpt ends".to_owned()
                        } else {
                            describe(want_end)
                        },
                        rerun: describe(got_end),
                        same: open || alike(want_end, got_end),
                    },
                    refused_act: refused.clone(),
                }
            })
            .collect();
        Report {
            recorded_by,
            requests,
            marks: self.read.into_iter().flatten().collect(),
        }
    }
}

/// reruns `excerpt` against the fixture `start` builds and compares it
/// with the recording and with `marks`. `Err` when the rerun cannot be
/// made (a request cut by the excerpt, a mark naming a request the
/// excerpt does not hold, a fixture that cannot start or settle); a
/// divergence is in the report (`Report::diverged`).
pub fn rerun<F: Fixture>(
    excerpt: &Session,
    marks: &[Mark],
    start: impl FnOnce() -> Result<F, String>,
) -> Result<Report, String> {
    let (recorded, steps) = script(excerpt)?;
    let marks = marks
        .iter()
        .map(|m| anchored(&recorded, m).map(|r| (m, r)))
        .collect::<Result<Vec<_>, _>>()?;
    let n = recorded.len();
    let mut run = Run {
        fixture: start()?,
        recorded: &recorded,
        ids: vec![None; n],
        seen: vec![None; n],
        refused: vec![None; n],
        read: vec![None; marks.len()],
        marks,
        start: excerpt.start(),
    };
    for step in &steps {
        run.before(Some(step))?;
        run.act(step);
    }
    run.before(None)?;
    let build = excerpt.build();
    Ok(run.report(format!("rec {}, built from {}", build.version, build.git)))
}
