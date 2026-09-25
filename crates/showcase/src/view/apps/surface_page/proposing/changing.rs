//! A change of the surface shown, asked through the proposing row
//! (task-70 phase 2), shared by the Surface page and the Silences.
//!
//! With a surface open, `propose` reads the intent as a change of it
//! first (`change`). By the rules, `propose::transform_verified(Rules)`
//! answers at once: the operations are applied through the host
//! (`SurfaceHost::apply_all`, all or none, verified again under the
//! surface's own policy, reconciled by `SemId`), as the actor of the
//! request, and "undo" takes the last change back whole
//! (`SurfaceHost::undo_asked`). By the Claude, when the intent reads as a
//! change (`propose::reads_as_change`), the worker asks for operations off
//! the app's thread (`asking::Job::Change`), the request starts, and the
//! page's `settle` applies them when they arrive (`arrived`), as whoever
//! asked. An intent that is no change goes on to the proposal of a
//! surface, as in task-69. Nothing here is a new capability: an agent
//! asks for a change through the port the way it proposes, `set_value`
//! on the intent and `propose`.

use std::time::Duration;

use propose::{
    Adapt, Adaptation, ProposeError, Rules, Transformed, Transformer, reads_as_change,
    transform_verified,
};
use sem::{Actor, Execution, NodeBuilder, Outcome, RequestId, SemId, Snapshot, Value};
use surface::{Change, SurfaceHost};
use uir::{Capabilities, Op};

use super::super::asking::{Job, Pending};
use super::{Proposer, Proposing, Said, int, usd};

fn steps(n: usize) -> String {
    if n == 1 {
        "1 step".to_owned()
    } else {
        format!("{n} steps")
    }
}

/// The status line of a change, of a change by the Claude, of an undo.
pub(crate) fn text(said: &Said) -> String {
    match said {
        Said::Changed { intent, steps: n } => {
            format!("changed by the rules for \"{intent}\", {}", steps(*n))
        }
        Said::ChangedBy {
            intent,
            steps: n,
            attempts,
            took,
            cost_usd,
        } => format!(
            "changed by the Claude for \"{intent}\", {}, in {:.1} s, {attempts} {}, {}",
            steps(*n),
            took.as_secs_f64(),
            if *attempts == 1 {
                "attempt"
            } else {
                "attempts"
            },
            usd(*cost_usd)
        ),
        Said::Undone(n) => format!("undone: the last change, {}", steps(*n)),
        other => format!("{other:?}"),
    }
}

/// The status's state of a change: how many steps it made, or undid.
pub(crate) fn project(said: Option<&Said>, status: &mut NodeBuilder<'_>) {
    match said {
        Some(Said::Changed { steps, .. } | Said::ChangedBy { steps, .. }) => {
            status.state("steps", int(*steps));
        }
        Some(Said::Undone(n)) => {
            status.state("undone", int(*n));
        }
        _ => {}
    }
}

/// What a `propose` with a surface open came to.
pub(crate) enum Changing {
    /// The intent asks for no change: a surface is proposed, as before.
    NotAChange,
    /// The request ends with this, or has started (the Claude).
    Ran(Execution),
}

fn refused(proposing: &mut Proposing, why: String) -> Execution {
    proposing.said = Some(Said::Refused(why.clone()));
    Execution::Failed(why)
}

/// The outcome of a change: the surface, how many steps, each in a line.
fn outcome(surface: &SemId, changes: &[Change], key: &str) -> Outcome {
    let mut outcome = Outcome::default();
    outcome
        .result
        .insert("surface".to_owned(), Value::Text(surface.to_string()));
    outcome
        .result
        .insert(key.to_owned(), Value::Int(int(changes.len())));
    let lines = changes.iter().map(|c| c.op.to_string()).collect();
    outcome.result.insert("ops".to_owned(), Value::List(lines));
    outcome
}

/// `ops` applied on the open surface `open` through the host, asked by
/// `by`, over the app as it is: the outcome, or why the surface stays.
fn apply(
    host: &mut SurfaceHost,
    open: &SemId,
    ops: Vec<Op>,
    by: &Actor,
    app: &Snapshot,
) -> Result<Outcome, String> {
    host.apply_all(open, ops, by.clone(), app)
        .map_err(|e| e.to_string())?;
    Ok(outcome(open, host.last_ask(open), "steps"))
}

/// The last change asked on `open` taken back: the outcome.
fn undo(
    host: &mut SurfaceHost,
    open: &SemId,
    proposing: &mut Proposing,
    app: &Snapshot,
) -> Execution {
    match host.undo_asked(open, app) {
        Ok(undone) => {
            proposing.said = Some(Said::Undone(undone.len()));
            Execution::Done(outcome(open, &undone, "undone"))
        }
        Err(e) => refused(proposing, e.to_string()),
    }
}

/// `propose` asked by `by` for the request `request`, with the surface
/// `open` shown, over the app as it is (`app`, the app's own projection,
/// never the row's): the intent read as a change of that surface. By the
/// rules it is applied or undone now, and the request ends; by the
/// Claude the worker starts and the request stays open until `arrived`.
/// `NotAChange` when the intent asks for a surface.
pub(crate) fn change(
    host: &mut SurfaceHost,
    open: &SemId,
    proposing: &mut Proposing,
    request: RequestId,
    by: &Actor,
    app: Snapshot,
) -> Changing {
    let intent = proposing.intent.text().trim().to_owned();
    let Some(current) = host.current(open) else {
        return Changing::NotAChange;
    };
    let caps = Capabilities::from_snapshot(&app);
    let policy = host.policy(open).cloned().unwrap_or_default();
    let ask = Adapt {
        intent: &intent,
        caps: &caps,
        app: &app,
        current: &current,
        policy: &policy,
        by,
    };
    let chosen = proposing.chosen();
    if chosen == Proposer::Claude && reads_as_change(&intent) {
        if Rules.transform(&ask) == Ok(Adaptation::Undo) {
            return Changing::Ran(undo(host, open, proposing, &app));
        }
        if proposing.waiting() {
            let why = "a proposal is already being asked of the Claude: wait for it, or cancel it";
            return Changing::Ran(refused(proposing, why.to_owned()));
        }
        let job = Job::Change(Box::new(current.clone()));
        let started = proposing.claude.start(
            request,
            intent.clone(),
            app.clone(),
            caps.clone(),
            job,
            policy.clone(),
            by.clone(),
        );
        return Changing::Ran(match started {
            Ok(pending) => {
                proposing.pending = Some(pending);
                Execution::Started
            }
            Err(why) => refused(proposing, why),
        });
    }
    if chosen == Proposer::Claude {
        return Changing::NotAChange;
    }
    match transform_verified(&mut Rules, &ask) {
        Err(ProposeError::NotAChange { .. }) => Changing::NotAChange,
        Err(e) => Changing::Ran(refused(proposing, e.to_string())),
        Ok(Transformed::Undo) => Changing::Ran(undo(host, open, proposing, &app)),
        Ok(Transformed::Changed { ops, .. }) => {
            let steps = ops.len();
            Changing::Ran(match apply(host, open, ops, by, &app) {
                Ok(outcome) => {
                    proposing.said = Some(Said::Changed { intent, steps });
                    Execution::Done(outcome)
                }
                Err(why) => refused(proposing, why),
            })
        }
    }
}

/// The Claude answered `pending` with `ops`, in `attempts` calls that
/// took `took` and cost `cost_usd`: applied on the open surface through
/// the host, as whoever asked, over the app as it is now; the outcome
/// with the attempts, the seconds and the cost, or why the surface stays.
pub(crate) fn arrived(
    host: &mut SurfaceHost,
    open: Option<&SemId>,
    proposing: &mut Proposing,
    pending: Pending,
    ops: Vec<Op>,
    (attempts, took, cost_usd): (usize, Duration, Option<f64>),
    app: &Snapshot,
) -> Result<Outcome, String> {
    let open = open.ok_or("no surface is open any more")?;
    let steps = ops.len();
    let mut outcome = apply(host, open, ops, &pending.by, app)?;
    outcome
        .result
        .insert("attempts".to_owned(), Value::Int(int(attempts)));
    outcome
        .result
        .insert("seconds".to_owned(), Value::Num(took.as_secs_f64()));
    if let Some(cost) = cost_usd {
        outcome
            .result
            .insert("cost_usd".to_owned(), Value::Num(cost));
    }
    proposing.said = Some(Said::ChangedBy {
        intent: pending.intent,
        steps,
        attempts,
        took,
        cost_usd,
    });
    Ok(outcome)
}
