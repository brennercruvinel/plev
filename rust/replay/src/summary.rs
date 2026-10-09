//! `summary`: a session at a glance: the build that recorded it, its span,
//! the actors, every request with how it ended, and every refusal with its
//! causal chain.

use record::ReadError;
use accord::{ActionName, Actor, Millis, RequestId, SemId, Stage};

use crate::describe;
use crate::index::{Inspector, Stamp};
use crate::timeline::Tally;
use crate::why::Why;

/// a session at a glance.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct Summary {
    /// the build of the recorder that wrote the session: the `rec` crate
    /// version and the git commit it was built from.
    pub version: String,
    pub git: String,
    pub start: Millis,
    pub end: Millis,
    pub duration: Millis,
    pub events: usize,
    pub frames: usize,
    pub actors: Vec<Tally>,
    pub requests: Vec<Line>,
    /// every request that ended refused, with its causal chain.
    pub refusals: Vec<Why>,
}

/// one request and how it ended.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct Line {
    pub origin: u16,
    pub id: RequestId,
    /// its first recorded stage.
    pub t: Millis,
    pub at: Millis,
    pub actor: Actor,
    pub target: SemId,
    pub action: ActionName,
    /// `completed`, `rejected`, `failed`, or the last stage of one still
    /// open when the window ends.
    pub ending: &'static str,
    /// the rejection, the outcome or the error.
    pub detail: Option<String>,
}

impl Inspector<'_> {
    /// the session at a glance.
    pub fn summary(&self) -> Result<Summary, ReadError> {
        let session = self.session();
        let mut requests = Vec::new();
        let mut refusals = Vec::new();
        for tracked in self.tracked() {
            let receipt = &tracked.at.receipt;
            let Some((_, first)) = receipt.stages.first() else {
                continue;
            };
            let Stamp { t, at } = self.stamp(*first);
            let last = receipt.stage();
            requests.push(Line {
                origin: tracked.at.origin,
                id: receipt.id,
                t,
                at,
                actor: receipt.request.actor.clone(),
                target: receipt.request.target.clone(),
                action: receipt.request.action.clone(),
                ending: last.map_or("none", Stage::name),
                detail: last.and_then(describe::stage_detail),
            });
            if tracked.refused() {
                refusals.extend(self.why(tracked.at.origin, receipt.id)?);
            }
        }
        let build = session.build();
        Ok(Summary {
            version: build.version.clone(),
            git: build.git.clone(),
            start: self.start(),
            end: self.end(),
            duration: self.end().saturating_sub(self.start()),
            events: session.events().len(),
            frames: session.frame_times().len(),
            actors: self.tallies(),
            requests,
            refusals,
        })
    }
}
