//! `Chain`: a composite running, one request per step, through the app's
//! own dispatcher.
//!
//! the composite's own request goes through `dispatch` as every request
//! does and its `execute` answers `Started`; the app keeps a `Chain` and
//! advances it right after, in the same lending of the dispatcher, and
//! again whenever a step that started ends (from the app's wake). each
//! step is a `Request` of the composite's actor, checked as any:
//!
//! - the first is observed on the app right after the composite's
//!   dispatch, so it rests on what the composite's actor saw (the
//!   composite reads what its steps read, and was checked against it);
//! - every next one is observed on the state the step before left, right
//!   after that step's dispatch returns, and dispatched when that step
//!   ends: at once when it completed, or when the app ends it later. what
//!   moved in between (another actor, while a frame decodes) makes it
//!   stale, with the delta and who acted, as any stale request.
//!
//! the first step refused, failed or left waiting for a person stops the
//! chain, and the composite's receipt ends `failed` saying which step and
//! why; the steps that ran stay run: no capability here offers to undo
//! what they did. when every step completed, the composite completes with
//! how many steps ran and their requests.

use sem::{
    ActionName, Actor, Change, Dispatcher, Millis, Observation, Operable, Outcome, Receipt,
    Rejection, Request, RequestId, SemId, Snapshot, SnapshotBuilder, Stage, Value,
};

/// a composite running.
#[derive(Clone, Debug)]
pub struct Chain {
    /// the composite's own request: it ends when the chain does.
    pub id: RequestId,
    actor: Actor,
    calls: Vec<(SemId, ActionName)>,
    /// the step to dispatch next.
    next: usize,
    /// the step dispatched last, while it has not ended.
    waiting: Option<RequestId>,
    /// the next step, observed on what the step before it left.
    prepared: Option<Request>,
    /// every step dispatched, in order.
    asked: Vec<RequestId>,
}

impl Chain {
    /// the chain of the composite request `id`, asked by `actor`, that
    /// runs `calls` in order (`Composite::calls`).
    pub fn new(id: RequestId, actor: Actor, calls: Vec<(SemId, ActionName)>) -> Chain {
        Chain {
            id,
            actor,
            calls,
            next: 0,
            waiting: None,
            prepared: None,
            asked: Vec::new(),
        }
    }

    /// the steps dispatched so far, in order.
    pub fn asked(&self) -> &[RequestId] {
        &self.asked
    }

    /// true while a step that started has not ended.
    pub fn waits(&self) -> bool {
        self.waiting.is_some()
    }

    /// runs what can run now over `app`: the steps one after the other,
    /// until one starts and has not ended, or the chain ends. true when
    /// it ended, and the composite's receipt with it.
    pub fn advance<A: Operable>(&mut self, d: &mut Dispatcher, app: &mut A, now: Millis) -> bool {
        loop {
            if let Some(step) = self.waiting {
                match d.receipt(step).and_then(Receipt::stage) {
                    Some(Stage::Executing) => return false,
                    Some(Stage::Completed(_)) => self.waiting = None,
                    other => {
                        let why = said(other);
                        return self.stop(d, now, step, &why);
                    }
                }
            }
            if self.next == self.calls.len() {
                return self.complete(d, now);
            }
            let req = match self.prepared.take() {
                Some(req) => req,
                None => match observe(app, &self.actor, &self.calls[self.next]) {
                    Ok(req) => req,
                    Err(why) => return self.fail(d, now, &why),
                },
            };
            let step = d.dispatch(app, req, now);
            self.asked.push(step);
            self.next += 1;
            if let Some(call) = self.calls.get(self.next) {
                match observe(app, &self.actor, call) {
                    Ok(req) => self.prepared = Some(req),
                    Err(why) => return self.fail(d, now, &why),
                }
            }
            match d.receipt(step).and_then(Receipt::stage) {
                Some(Stage::Completed(_)) => {}
                Some(Stage::Executing) => {
                    self.waiting = Some(step);
                    return false;
                }
                other => {
                    let why = said(other);
                    return self.stop(d, now, step, &why);
                }
            }
        }
    }

    /// every step ran: the composite completes with how many and their
    /// requests.
    fn complete(&mut self, d: &mut Dispatcher, now: Millis) -> bool {
        let mut outcome = Outcome::default();
        let steps = i64::try_from(self.asked.len()).unwrap_or(i64::MAX);
        outcome.result.insert("steps".to_owned(), Value::Int(steps));
        let requests = self.asked.iter().map(ToString::to_string).collect();
        outcome
            .result
            .insert("requests".to_owned(), Value::List(requests));
        // the composite's request executes while its chain runs; nothing
        // else ends it, so there is no error to tell here
        let _ = d.complete(self.id, outcome, now);
        true
    }

    /// the step `step` did not complete, for `why`: the composite fails,
    /// saying which step, why, and what ran before it.
    fn stop(&mut self, d: &mut Dispatcher, now: Millis, step: RequestId, why: &str) -> bool {
        let at = self.asked.len();
        let (target, action) = self
            .calls
            .get(at.saturating_sub(1))
            .map_or(("?".to_owned(), "?"), |(t, a)| (t.to_string(), a.as_str()));
        let ran = &self.asked[..at.saturating_sub(1)];
        let mut text = format!(
            "stopped at step {at} of {}, {action} on {target} (request {step}): {why}",
            self.calls.len()
        );
        match ran {
            [] => text.push_str("; no step ran before it"),
            [one] => text.push_str(&format!("; step 1 ran and stays (request {one})")),
            many => {
                let ids: Vec<String> = many.iter().map(ToString::to_string).collect();
                text.push_str(&format!(
                    "; steps 1 to {} ran and stay (requests {})",
                    many.len(),
                    ids.join(", ")
                ));
            }
        }
        self.fail(d, now, &text)
    }

    fn fail(&mut self, d: &mut Dispatcher, now: Millis, why: &str) -> bool {
        let _ = d.fail(self.id, why.to_owned(), now);
        true
    }
}

/// the request `actor` makes for `call` having seen `app` now; one that
/// saw nothing when the app no longer declares the call (the dispatcher
/// refuses it, and says why). an error when the app does not project.
fn observe<A: Operable>(
    app: &A,
    actor: &Actor,
    (target, action): &(SemId, ActionName),
) -> Result<Request, String> {
    let mut b = SnapshotBuilder::new();
    app.project(&mut b);
    let seen: Snapshot = b
        .finish()
        .map_err(|e| format!("the app does not project: {e}"))?;
    let asked = Request::observe(&seen, actor.clone(), target, action, None);
    Ok(asked.unwrap_or_else(|_| Request {
        actor: actor.clone(),
        target: target.clone(),
        action: action.clone(),
        param: None,
        observed: Observation {
            revision: seen.revision(),
            values: Vec::new(),
        },
        ask: None,
    }))
}

/// why a step did not complete, in words.
fn said(stage: Option<&Stage>) -> String {
    match stage {
        Some(Stage::Rejected(why)) => refused(why),
        Some(Stage::Failed { error }) => format!("failed: {error}"),
        Some(Stage::AwaitingConfirmation) => {
            "it waits for a person to confirm it, and stays waiting".to_owned()
        }
        Some(other) => format!("it is {}", other.name()),
        None => "the dispatcher no longer holds it".to_owned(),
    }
}

fn refused(why: &Rejection) -> String {
    match why {
        Rejection::NotFound { target } => format!("refused: {target} is not there"),
        Rejection::UnknownAction { target, action } => {
            format!("refused: {target} does not offer {action}")
        }
        Rejection::NotAuthorized { actor } => format!("refused: {actor} may not ask it"),
        Rejection::Stale { changes, acted } => {
            let moved: Vec<String> = changes.iter().map(change).collect();
            let mut text = format!("stale: {}", moved.join(", "));
            if !acted.is_empty() {
                let who: Vec<String> = acted
                    .iter()
                    .map(|a| format!("{} ran {} on {}", a.actor, a.action, a.target))
                    .collect();
                text.push_str(&format!(" ({})", who.join(", ")));
            }
            text
        }
        Rejection::Inert { target } => format!("refused: {target} is inert"),
        Rejection::Disabled { target, action } => format!("refused: {action} on {target} is off"),
        Rejection::InvalidParam { reason } => format!("refused: {reason}"),
        Rejection::InvalidProjection { .. } => {
            "refused: the app's projection does not validate".to_owned()
        }
        Rejection::Declined { by } => format!("declined by {by}"),
    }
}

fn change(c: &Change) -> String {
    match c {
        Change::StateChanged {
            at, before, after, ..
        } => format!(
            "{at} was {}, is {}",
            value(before.as_ref()),
            value(after.as_ref())
        ),
        other => format!("{other:?}"),
    }
}

fn value(v: Option<&Value>) -> String {
    match v {
        None => "nothing".to_owned(),
        Some(Value::Text(t)) => format!("{t:?}"),
        Some(Value::Bool(b)) => b.to_string(),
        Some(Value::Int(n)) => n.to_string(),
        Some(Value::Num(n)) => n.to_string(),
        Some(Value::List(items)) => format!("[{}]", items.join(", ")),
        Some(Value::Set(items)) => {
            let items: Vec<&str> = items.iter().map(String::as_str).collect();
            format!("{{{}}}", items.join(", "))
        }
    }
}
