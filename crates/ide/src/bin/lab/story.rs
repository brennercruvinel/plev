//! the story of the run as the report tells it: every act of an agent
//! against the observations that were served, which world each act
//! rested on, whether the stale check was exercised at all, and whether
//! the agent recovered.

use sem::{Actor, Millis, Receipt, Rejection, Revision, Stage};
use serde_json::{Value as Json, json};

use crate::host::{ActSeen, Intervention, Served};

/// one act of an agent, told against the observations.
#[derive(Clone, Debug)]
pub struct ActStory {
    pub t: Millis,
    pub actor: Actor,
    pub target: String,
    pub action: String,
    pub stages: Vec<&'static str>,
    pub terminal: &'static str,
    /// the revision the act rested on (`observed.revision`).
    pub revision: Revision,
    /// the observation that served that revision, when one did: its index
    /// among the served observations and its time.
    pub served_index: Option<usize>,
    pub served_at: Option<Millis>,
    /// observations of the same actor served after that one and before the
    /// act: 0 means the act rests on the freshest observation it had.
    pub observations_between: usize,
    /// the observation right before the act was scoped to the act's target
    /// and served the revision the act rests on: the shape of `plev act`
    /// without `--from-revision`, which observes the target first and can
    /// never be stale.
    pub fresh_before: bool,
    pub stale: bool,
}

/// the acts of agents in `receipts`, in the order they started.
pub fn acts(receipts: &[Receipt], seen: &[ActSeen], served: &[Served]) -> Vec<ActStory> {
    let mut out = Vec::new();
    for r in receipts {
        if !matches!(r.request.actor, Actor::Agent { .. }) {
            continue;
        }
        let Some(at) = seen.iter().find(|a| a.request == r.id) else {
            continue;
        };
        let revision = r.request.observed.revision;
        let before = &served[..at.served_before.min(served.len())];
        let served_index = before.iter().position(|s| s.revision == revision);
        let observations_between = served_index.map_or(0, |i| {
            before[i + 1..]
                .iter()
                .filter(|s| s.actor == r.request.actor)
                .count()
        });
        let fresh_before = before
            .iter()
            .rev()
            .find(|s| s.actor == r.request.actor)
            .is_some_and(|last| {
                last.revision == revision
                    && last.scope.as_ref().is_some_and(|s| *s == r.request.target)
            });
        let stages: Vec<&'static str> = r.stages.iter().map(|(s, _)| s.name()).collect();
        out.push(ActStory {
            t: r.stages.first().map_or(0, |(_, t)| *t),
            actor: r.request.actor.clone(),
            target: r.request.target.to_string(),
            action: r.request.action.to_string(),
            terminal: r.stage().map_or("none", Stage::name),
            stale: matches!(r.stage(), Some(Stage::Rejected(Rejection::Stale { .. }))),
            stages,
            revision,
            served_index,
            served_at: served_index.map(|i| before[i].t),
            observations_between,
            fresh_before,
        });
    }
    out
}

/// whether the run measured the stale check at all: an intervention
/// happened, and afterwards a `submit` rested on a revision served before
/// it (the agent acted on the old world). the text says why.
pub fn stale_measurement(acts: &[ActStory], intervention: Option<&Intervention>) -> (bool, String) {
    let Some(intervention) = intervention else {
        return (
            false,
            "no intervention in this run: the stale check was not exercised".to_owned(),
        );
    };
    let old_world = acts.iter().find(|a| {
        a.action == "submit"
            && a.t >= intervention.t
            && a.served_index
                .is_some_and(|i| i <= intervention.served_index)
    });
    match old_world {
        Some(a) => (
            true,
            format!(
                "valid: the submit at {} ms rested on revision {} served before the intervention \
                 and ended {}",
                a.t, a.revision, a.terminal
            ),
        ),
        None => {
            let fresh = acts
                .iter()
                .filter(|a| a.action == "submit" && a.t >= intervention.t)
                .count();
            (
                false,
                format!(
                    "invalid: none of the {fresh} submit(s) after the intervention rested on a \
                     revision served before it (each observed the new world first, the shape of \
                     plev act without --from-revision); the stale check was not exercised"
                ),
            )
        }
    }
}

/// a `submit` completed after a `submit` was rejected stale.
pub fn recovered(acts: &[ActStory]) -> bool {
    let mut stale_at = None;
    for a in acts.iter().filter(|a| a.action == "submit") {
        if a.stale {
            stale_at.get_or_insert(a.t);
        } else if a.terminal == "completed" && stale_at.is_some_and(|s| a.t >= s) {
            return true;
        }
    }
    false
}

pub fn served_json(served: &[Served]) -> Json {
    Json::Array(
        served
            .iter()
            .enumerate()
            .map(|(i, s)| {
                json!({
                    "index": i,
                    "t": s.t,
                    "actor": s.actor.to_string(),
                    "scope": s.scope.as_ref().map(ToString::to_string),
                    "revision": s.revision.to_string(),
                })
            })
            .collect(),
    )
}

pub fn acts_json(acts: &[ActStory]) -> Json {
    Json::Array(
        acts.iter()
            .map(|a| {
                json!({
                    "t": a.t,
                    "actor": a.actor.to_string(),
                    "target": a.target,
                    "action": a.action,
                    "stages": a.stages,
                    "terminal": a.terminal,
                    "revision": a.revision.to_string(),
                    "revision_served_index": a.served_index,
                    "revision_served_at": a.served_at,
                    "observations_between": a.observations_between,
                    "fresh_before": a.fresh_before,
                    "form": if a.served_index.is_some() { "from_revision" } else { "observed" },
                })
            })
            .collect(),
    )
}
