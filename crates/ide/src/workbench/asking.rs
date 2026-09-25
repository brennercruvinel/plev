//! The Claude as the proposer of the line (task-96), the workbench's side
//! of `crate::generated::claude`, as the Silences page asks it
//! (`crates/showcase/src/view/apps/silences/generated/page.rs`): `propose`
//! answers `Execution::Started` and a worker asks the Claude off the
//! workbench's thread; the workbench's wake looks at it, and when the
//! answer arrives the surface opens (or takes its form in place) and the
//! request ends through the same dispatcher, `completed` with the
//! surface, the attempts, the seconds and the cost, or `failed` with why
//! there is none. A cancel ends it as cancelled by whoever asked.
//!
//! The Claude is asked only for a review the rules read, and is shown
//! only what that review needs (`crate::generated::scope`): the commits
//! the rules' own proposal names, with their messages and files, and the
//! diff region only while it shows one of them. Never the workspace whole
//! and never the surface shown.

use sem::{Execution, Outcome, RequestId, Snapshot, Value};
use uir::{Capabilities, GenerationPolicy, VerifiedIr};

use super::Workbench;
use super::generated::shown;
use crate::generated::{Said, scope};

fn int(n: usize) -> Value {
    Value::Int(i64::try_from(n).unwrap_or(i64::MAX))
}

impl Workbench {
    /// Asks the Claude for `intent`, for the request `id`, over the
    /// review the rules read in it (`reviewed`, their own proposal, which
    /// names the commits): a worker starts with only what that review
    /// needs, and the request stays open. `Failed` with why, when there is
    /// no such review, a proposal is already asked, or no worker starts.
    pub(super) fn ask_claude(
        &mut self,
        id: RequestId,
        intent: String,
        app: &Snapshot,
        reviewed: Result<VerifiedIr, String>,
        policy: GenerationPolicy,
    ) -> Execution {
        let asking = self.generated.as_ref().is_some_and(|g| g.waiting());
        if asking {
            let why = "a proposal is already being asked of the Claude: wait for it, or cancel it";
            return self.refuse(why.to_owned());
        }
        let Some(ids) = self.workspace.ids.as_ref() else {
            return self.refuse("the workspace has no ids".to_owned());
        };
        let commits = match &reviewed {
            Ok(ir) => scope::commits(ir.ir(), ids),
            Err(why) => {
                return self.refuse(format!(
                    "the Claude is shown only the commits a review names, and the rules read \
                     none in this intent: {why}"
                ));
            }
        };
        if commits.is_empty() {
            return self.refuse(
                "the Claude is shown only the commits a review names, and this intent names none"
                    .to_owned(),
            );
        }
        let scoped = match scope::scoped(app, &commits, ids) {
            Ok(scoped) => scoped,
            Err(e) => return self.refuse(e.to_string()),
        };
        let caps = Capabilities::from_snapshot(&scoped);
        let Some(g) = self.generated.as_mut() else {
            return Execution::Failed("no surface is hosted here".to_owned());
        };
        match g.claude.start(id, intent, scoped, caps, policy) {
            Ok(pending) => {
                g.pending = Some(pending);
                Execution::Started
            }
            Err(why) => self.refuse(why),
        }
    }

    /// No proposal: the line says why, and so does the request.
    fn refuse(&mut self, why: String) -> Execution {
        self.say(Said::Refused(why.clone()));
        Execution::Failed(why)
    }

    /// What happened to the proposal the Claude is answering since the
    /// workbench last looked: a cancel ends it now; the attempt or the
    /// seconds moved; or the answer arrived, the surface opens or takes
    /// its form in place, and the request ends. True when what the line
    /// shows moved.
    pub(super) fn settle_claude(&mut self) -> bool {
        let Some(g) = self.generated.as_mut() else {
            return false;
        };
        let Some(pending) = g.pending.as_mut() else {
            return false;
        };
        if let Some(by) = pending.cancelled.clone() {
            let id = pending.id;
            g.pending = None;
            let why = format!("cancelled by {by}");
            g.said = Some(Said::Refused(why.clone()));
            self.end(id, Err(why));
            return true;
        }
        let (moved, answer) = pending.poll();
        let Some(answer) = answer else {
            return moved;
        };
        let Some(pending) = g.pending.take() else {
            return true;
        };
        let shown_now = answer.proposed.map_err(|e| e.to_string()).and_then(|ir| {
            let app = self.app_snapshot().map_err(|e| e.to_string())?;
            self.show(ir, pending.policy.clone(), &app)
        });
        match shown_now {
            Ok((surface, nodes)) => {
                let mut outcome = shown(&surface, nodes);
                let result = &mut outcome.result;
                result.insert("attempts".to_owned(), int(answer.attempts));
                result.insert("seconds".to_owned(), Value::Num(answer.took.as_secs_f64()));
                if let Some(cost) = answer.cost_usd {
                    result.insert("cost_usd".to_owned(), Value::Num(cost));
                }
                self.say(Said::Answered {
                    intent: pending.intent,
                    attempts: answer.attempts,
                    took: answer.took,
                    cost_usd: answer.cost_usd,
                });
                self.end(pending.id, Ok(outcome));
            }
            Err(why) => {
                self.say(Said::Refused(why.clone()));
                self.end(pending.id, Err(why));
            }
        }
        true
    }

    /// Ends request `id` through the dispatcher, completed or failed, at
    /// the workbench's now, and says a failure as any other.
    fn end(&mut self, id: RequestId, how: Result<Outcome, String>) {
        let now = self.now();
        let ended = match how {
            Ok(outcome) => self.dispatcher.complete(id, outcome, now),
            Err(why) => self.dispatcher.fail(id, why, now),
        };
        match ended {
            Ok(()) => self.report(id),
            Err(e) => log::warn!("ide: the proposal's request {id} did not end: {e}"),
        }
    }
}
