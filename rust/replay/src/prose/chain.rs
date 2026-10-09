//! the causal chain of one request as text: one labelled line per
//! question of the thesis (which control, its state, who initiated it,
//! which action, what it asked a person and who answered, what was
//! observed, the precondition, what came before, the stages, the verdict,
//! what changed, the result).

use std::fmt::Write as _;

use accord::{Actor, Rejection, Stage};

use super::{
    cont, control_line, deeper, field, input_name, item, request, scope_text, stamp, state_line,
};
use crate::describe;
use crate::why::{Basis, By, Cause, Ending, Unrecorded, Why};

/// a request's causal chain, one labelled line per question of the thesis.
pub fn why(w: &Why) -> String {
    let mut out = String::new();
    let asked = &w.request;
    let _ = writeln!(
        out,
        "{} (origin {}): {} on {} by {}, at {}",
        request(0, w.id),
        w.origin,
        asked.action,
        asked.target,
        asked.actor,
        stamp(asked.at)
    );
    if !w.whole {
        let _ = writeln!(
            out,
            "  (the window starts after the request: its first stages were cut; the request is whole)"
        );
    }
    field(
        &mut out,
        "control",
        &control_line(&asked.target, asked.control.as_ref()),
    );
    field(&mut out, "state", &state_line(asked.control.as_ref()));
    field(&mut out, "initiated by", &asked.actor.to_string());
    let action = match &asked.param {
        Some(param) => format!("{}, with {}", asked.action, describe::value(param)),
        None => format!("{}, no parameter", asked.action),
    };
    field(&mut out, "action", &action);
    if let Some(question) = &asked.ask {
        field(
            &mut out,
            "asked",
            &format!("{} asked a person first: \"{question}\"", asked.actor),
        );
        field(&mut out, "answered", &answered(w));
    }
    basis(&mut out, w);
    field(
        &mut out,
        "precondition",
        &format!("what the action reads, as {} saw it:", asked.actor),
    );
    for read in &asked.reads {
        let scope = match read.in_scope {
            Some(false) => ", outside the scope observed: that observation did not serve it",
            _ => "",
        };
        item(
            &mut out,
            &format!(
                "{} = {}{scope}",
                read.at,
                describe::maybe(read.seen.as_ref())
            ),
        );
    }
    before(&mut out, w);
    stages(&mut out, w);
    verdict(&mut out, w);
    result(&mut out, w);
    if w.between.reads.iter().any(|r| !r.moves.is_empty()) {
        field(
            &mut out,
            "note",
            "\"by\" is a correlation in time, not a declared write: the receipts that ran",
        );
        cont(
            &mut out,
            "(executing, completed or failed) between the recorded state before a change and",
        );
        cont(
            &mut out,
            "the change. the contract declares what an action reads, never what it writes.",
        );
    }
    out
}

/// who answered the question a request asked a person (task-92), and
/// how: from its stages.
fn answered(w: &Why) -> String {
    for s in &w.stages {
        match &s.stage {
            Stage::Confirmed { by } => return format!("{by} confirmed it, at {}", stamp(s.at)),
            Stage::Rejected(Rejection::Declined { by }) => {
                return format!("{by} declined it, at {}", stamp(s.at));
            }
            _ => {}
        }
    }
    match w.stages.last().map(|s| &s.stage) {
        Some(Stage::AwaitingConfirmation) => {
            "nobody yet: it still waits for a person when the window ends".to_owned()
        }
        _ if w
            .stages
            .iter()
            .any(|s| s.stage == Stage::AwaitingConfirmation) =>
        {
            "not recorded in this window".to_owned()
        }
        _ => "nobody: it was refused before a person was asked".to_owned(),
    }
}

fn basis(out: &mut String, w: &Why) {
    let actor = &w.request.actor;
    match &w.basis {
        Basis::Observed {
            at,
            scope,
            revision,
            ..
        } => {
            field(
                out,
                "observed",
                &format!(
                    "{}, {}, at revision {revision}",
                    stamp(*at),
                    scope_text(scope.as_ref())
                ),
            );
            cont(
                out,
                &format!("(the last observation by {actor} at the revision the request carries)"),
            );
        }
        Basis::NotRecorded { reason, state } => {
            let revision = w.request.revision;
            match reason {
                Unrecorded::NoObservation => {
                    field(
                        out,
                        "observed",
                        &format!("not recorded: no observation by {actor} before the request"),
                    );
                    cont(
                        out,
                        match actor {
                            Actor::Human | Actor::Assistive => {
                                "(a person acts on the frame on screen; the recorder keeps only \
                                 the observations the app reports)"
                            }
                            _ => "(the app did not report one, or the window starts after it)",
                        },
                    );
                }
                Unrecorded::OtherRevision {
                    at,
                    revision: other,
                    ..
                } => {
                    field(
                        out,
                        "observed",
                        &format!(
                            "not recorded at revision {revision}: the observations by {actor} \
                             are at other revisions"
                        ),
                    );
                    cont(
                        out,
                        &format!(
                            "(the last at {}, revision {other}); the revision it acted on came \
                             another way, such as the last stage of an earlier act",
                            stamp(*at)
                        ),
                    );
                }
            }
            let held = match state {
                Some(s) => format!(
                    "the recorded state was at revision {} last at {}",
                    w.request.revision,
                    stamp(s.at)
                ),
                None => format!(
                    "the recorded state never had revision {} in this window",
                    w.request.revision
                ),
            };
            cont(out, &held);
        }
    }
}

fn before(out: &mut String, w: &Why) {
    let between = &w.between;
    let Some(from) = between.from else {
        field(
            out,
            "before",
            "the recording does not say when the state was at that revision, so not what moved",
        );
        for read in &between.reads {
            item(
                out,
                &format!(
                    "{}: seen {}, right before the request {}",
                    read.at,
                    describe::maybe(read.seen.as_ref()),
                    describe::maybe(read.before_request.as_ref())
                ),
            );
        }
        return;
    };
    let origin = match w.basis {
        Basis::Observed { .. } => "the observation",
        Basis::NotRecorded { .. } => "the last state at that revision",
    };
    field(
        out,
        "before",
        &format!(
            "from {origin} ({}) to the request ({}):",
            stamp(from.at),
            stamp(between.to.at)
        ),
    );
    for read in &between.reads {
        if read.moves.is_empty() {
            item(out, &format!("{} did not move", read.at));
        }
        for moved in &read.moves {
            item(
                out,
                &format!(
                    "{} {}: {}",
                    stamp(moved.at),
                    read.at,
                    describe::moved(
                        moved.before.as_ref(),
                        moved.after.as_ref(),
                        &moved.added,
                        &moved.removed
                    )
                ),
            );
            deeper(out, &by(&moved.by));
        }
    }
    for input in &between.inputs {
        let on = input
            .target
            .as_ref()
            .map_or(String::new(), |t| format!(" on {t}"));
        item(
            out,
            &format!("{} input: {}{on}", stamp(input.at), input_name(input.input)),
        );
    }
}

fn stages(out: &mut String, w: &Why) {
    for (i, s) in w.stages.iter().enumerate() {
        let detail = describe::stage_detail(&s.stage).map_or(String::new(), |d| format!(": {d}"));
        let line = format!("{} {}{detail}", stamp(s.at), s.stage.name());
        if i == 0 {
            field(out, "stages", &line);
        } else {
            cont(out, &line);
        }
    }
}

fn verdict(out: &mut String, w: &Why) {
    let decided = w
        .stages
        .iter()
        .find(|s| !matches!(s.stage, Stage::Requested));
    let line = match decided.map(|s| (&s.stage, s.at)) {
        Some((Stage::Accepted, at)) => format!("accepted at {}", stamp(at)),
        Some((Stage::AwaitingConfirmation, at)) => {
            format!("waited for a person to confirm, from {}", stamp(at))
        }
        Some((Stage::Rejected(r), at)) => format!(
            "rejected at {}: {}, {}",
            stamp(at),
            describe::rejection(r),
            refusal(r)
        ),
        Some((other, at)) => format!("{} at {}", other.name(), stamp(at)),
        None => "not decided when the window ends".to_owned(),
    };
    field(out, "verdict", &line);
    if let Ending::Rejected {
        rejection: Rejection::Stale { changes, acted },
        at_truth,
        ..
    } = &w.ending
    {
        if *at_truth {
            cont(
                out,
                "refused at the point of truth, after it started: stale",
            );
        }
        for (i, change) in changes.iter().enumerate() {
            let text = describe::change(change);
            if i == 0 {
                field(out, "what changed", &text);
            } else {
                cont(out, &text);
            }
        }
        if acted.is_empty() {
            field(out, "acted", "the refusal names nobody");
            for line in [
                "(acted names the requests taken after the look the act rested on, the",
                "author's own included, when the port knew the look; without one, other",
                "actors' receipts from the first one, of any actor, that ran on the exact",
                "revision the author saw. nobody ran since the look, or, without one, none",
                "ran on that revision. a change no receipt made names nobody, and a refusal",
                "recorded before acted existed names nobody. before says what the recording",
                "shows)",
            ] {
                cont(out, line);
            }
        } else {
            let acted: Vec<String> = acted
                .iter()
                .map(|a| {
                    format!(
                        "{}, request {}: {} on {}",
                        a.actor, a.id, a.action, a.target
                    )
                })
                .collect();
            field(out, "acted", &acted.join("; "));
        }
    } else if let Ending::Rejected {
        rejection,
        at_truth: true,
        ..
    } = &w.ending
    {
        cont(
            out,
            &format!(
                "refused at the point of truth, after it started: {}",
                describe::rejection(rejection)
            ),
        );
    }
}

fn result(out: &mut String, w: &Why) {
    let line = match &w.ending {
        Ending::Completed { at, outcome, .. } => {
            if outcome.result.is_empty() {
                format!("completed at {}", stamp(*at))
            } else {
                format!(
                    "completed at {}: {}",
                    stamp(*at),
                    describe::outcome(outcome)
                )
            }
        }
        Ending::Rejected { at_truth, .. } => {
            if *at_truth {
                "nothing ran: the point of truth refused it before the effect".to_owned()
            } else {
                "nothing ran: the refusal came before any effect".to_owned()
            }
        }
        Ending::Failed { at, error, .. } => format!("failed at {}: {error}", stamp(*at)),
        Ending::Open { at, stage, .. } => {
            format!("not finished: still {stage} (since {})", stamp(*at))
        }
    };
    field(out, "result", &line);
}

/// why a request was refused, in words.
pub(super) fn refusal(r: &Rejection) -> String {
    match r {
        Rejection::NotFound { target } => format!("{target} is not in the app's state"),
        Rejection::UnknownAction { target, action } => {
            format!("{target} does not declare {action}")
        }
        Rejection::NotAuthorized { actor } => {
            format!("the action's policy does not let {actor} ask")
        }
        Rejection::Stale { .. } => {
            "a state the action reads changed since the observation".to_owned()
        }
        Rejection::Inert { target } => format!("{target} is inert"),
        Rejection::Disabled { target, action } => format!("{action} on {target} is disabled"),
        Rejection::InvalidParam { reason } => format!("the parameter does not fit: {reason}"),
        Rejection::InvalidProjection { errors } => {
            format!("the app's projection did not validate: {errors}")
        }
        Rejection::Declined { by } => format!("{by} declined to confirm"),
    }
}

pub(super) fn by(b: &By) -> String {
    match b {
        By::Outside => {
            "by no request: no receipt ran between the state before and this change, so \
                        it came from outside the app"
                .to_owned()
        }
        By::Receipt { cause } => format!("by {}", cause_line(cause)),
        By::Ambiguous { causes } => {
            let all: Vec<String> = causes.iter().map(cause_line).collect();
            format!(
                "by one of {} receipts that ran then (ambiguous): {}",
                causes.len(),
                all.join("; ")
            )
        }
    }
}

fn cause_line(c: &Cause) -> String {
    let param = c
        .param
        .as_ref()
        .map_or(String::new(), |p| format!(" with {}", describe::value(p)));
    let same = if c.same_actor { ", the same actor" } else { "" };
    format!(
        "{}{same}, {}: {} on {}{param} ({} at {})",
        c.actor,
        request(c.origin, c.id),
        c.action,
        c.target,
        c.stage,
        stamp(c.at)
    )
}
