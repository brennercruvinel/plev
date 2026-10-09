//! The agent's side of the demo's script, over `wire::Client`: every act
//! from a revision it was shown, the waits for the page's worker (the
//! frames decoded), the person asked to click once, and the buttons of
//! the generated surface found by what the proposer named them.

use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

use accord::{ActionName, RequestId, SemId, Stage, StateKey, Value};
use socket::{Act, Client, Observe, Reply, SnapshotReply, StageReply};

use crate::run::{COMPARE, IDLE_WAIT, Request, TIMEOUT_MS};
use crate::shell::{Poke, Pokes, poke};

/// What the agent asked, in order, for the summary.
pub struct Story {
    pub observations: usize,
    pub detect: RequestId,
    pub cuts: Vec<String>,
    pub review: RequestId,
    pub surface: String,
    pub applied: (RequestId, SemId),
    pub rejected_by_person: SemId,
    pub compare: RequestId,
    pub close: RequestId,
}

fn look(c: &mut Client, observe: Observe) -> Result<SnapshotReply, String> {
    match c.observe(observe).map_err(|e| format!("observe: {e}"))? {
        Reply::Snapshot(s) => Ok(s),
        other => Err(format!("a snapshot was expected, got {other:?}")),
    }
}

/// The whole page as it is now, a new revision to rest the next act on.
fn now(c: &mut Client) -> Result<SnapshotReply, String> {
    look(c, Observe::default())
}

/// The whole page with where each control shows.
fn bounds(c: &mut Client) -> Result<SnapshotReply, String> {
    look(
        c,
        Observe {
            bounds: true,
            ..Observe::default()
        },
    )
}

fn act(
    c: &mut Client,
    target: &str,
    action: &str,
    param: Option<Value>,
    seen: &SnapshotReply,
) -> Result<Vec<StageReply>, String> {
    let target = SemId::parse(target).map_err(|e| e.to_string())?;
    let action = ActionName::parse(action).map_err(|e| e.to_string())?;
    let mut act = Act::from_revision(target.clone(), action.clone(), seen.revision);
    if let Some(param) = param {
        act = act.with_param(param);
    }
    c.act(act, |_| {})
        .map_err(|e| format!("{action} on {target}: {e}"))
}

/// The request's id when its last stage is `completed`; the stage
/// otherwise, in words.
fn done(stages: &[StageReply], what: &str) -> Result<RequestId, String> {
    match stages.last() {
        Some(last) if matches!(last.stage, Stage::Completed(_)) => Ok(last.request),
        Some(last) => Err(format!("{what} ended {:?}", last.stage)),
        None => Err(format!("{what}: no stage came back")),
    }
}

/// A text of the outcome of a completed request.
fn outcome(stages: &[StageReply], key: &str) -> Option<String> {
    match &stages.last()?.stage {
        Stage::Completed(outcome) => match outcome.result.get(key)? {
            Value::Text(text) => Some(text.clone()),
            other => Some(format!("{other:?}")),
        },
        _ => None,
    }
}

/// The keys of the cuts, in the order the page shows them.
fn cuts(seen: &SnapshotReply) -> Vec<String> {
    seen.nodes
        .iter()
        .find(|n| n.id.as_str() == "silences")
        .and_then(|n| n.state.get("cuts"))
        .and_then(|v| match v {
            Value::List(keys) => Some(keys.clone()),
            _ => None,
        })
        .unwrap_or_default()
}

/// The node of the generated surface that runs `action` on the cut
/// `key`: named by the rules `<surface>.silences.cut[<key>].<action>`;
/// from the Claude, any node under the surface (or under the app's own
/// id, when its template took it) that carries the cut and `activate`s
/// something called after the action.
fn button(seen: &SnapshotReply, surface: &str, key: &str, action: &str) -> Option<SemId> {
    let own = format!("silences.cut[{key}]");
    let word = action.split('_').next().unwrap_or(action);
    seen.nodes
        .iter()
        .filter(|n| n.id.as_str() != own && n.id.as_str().contains(&format!("cut[{key}]")))
        .filter(|n| n.id.as_str().starts_with(surface) || n.id.as_str().starts_with(&own))
        .filter(|n| n.actions.iter().any(|a| a.name.as_str() == "activate"))
        .find(|n| n.id.as_str().ends_with(action) || n.label.to_lowercase().contains(word))
        .map(|n| n.id.clone())
}

/// Waits for the page's worker to be idle twice in a row (the frames the
/// screen and the surface show are decoded), observing the status alone.
fn idle(c: &mut Client, observations: &mut usize) -> Result<(), String> {
    let start = Instant::now();
    let mut idle = 0;
    thread::sleep(Duration::from_millis(300));
    while idle < 2 {
        if start.elapsed() > IDLE_WAIT {
            return Err("the page's worker never went idle".to_owned());
        }
        let status = look(
            c,
            Observe {
                scope: Some(SemId::parse("silences.status").map_err(|e| e.to_string())?),
                keys: Some(vec![StateKey::parse("busy").map_err(|e| e.to_string())?]),
                ..Observe::default()
            },
        )?;
        *observations += 1;
        let busy = status
            .nodes
            .iter()
            .find_map(|n| n.state.get("busy"))
            .is_some_and(|v| *v == Value::Bool(true));
        idle = if busy { 0 } else { idle + 1 };
        thread::sleep(Duration::from_millis(200));
    }
    Ok(())
}

/// The proposer chosen and the intent written, then `propose`: the
/// request's id and the surface it opened or transformed.
fn propose(
    c: &mut Client,
    observations: &mut usize,
    proposer: &str,
    intent: &str,
) -> Result<(RequestId, String), String> {
    let seen = now(c)?;
    *observations += 1;
    let chosen = Some(Value::Text(proposer.to_owned()));
    let selected = act(c, "silences_surface.proposer", "select", chosen, &seen)?;
    done(&selected, "select the proposer")?;
    let seen = now(c)?;
    *observations += 1;
    let intent = Some(Value::Text(intent.to_owned()));
    let written = act(c, "silences_surface.intent", "set_value", intent, &seen)?;
    done(&written, "write the intent")?;
    let seen = now(c)?;
    *observations += 1;
    let proposed = act(c, "silences_surface.propose", "propose", None, &seen)?;
    let id = done(&proposed, "propose")?;
    let surface = outcome(&proposed, "surface").unwrap_or_default();
    idle(c, observations)?;
    Ok((id, surface))
}

/// The agent's side of the script.
pub fn agent(path: &Path, pokes: Pokes, req: &Request) -> Result<Story, String> {
    let mut c = Client::connect(path).map_err(|e| format!("connect: {e}"))?;
    c.set_timeout(Some(TIMEOUT_MS))
        .map_err(|e| format!("timeout: {e}"))?;
    c.hello(&req.name).map_err(|e| format!("hello: {e}"))?;
    let mut observations = 0;
    let seen = bounds(&mut c)?;
    observations += 1;

    // 2. the silences detected by ffmpeg; the frames follow
    let detected = act(&mut c, "silences.detect", "detect_silence", None, &seen)?;
    let detect = done(&detected, "detect_silence")?;
    idle(&mut c, &mut observations)?;
    let seen = now(&mut c)?;
    observations += 1;
    let cuts = cuts(&seen);
    let [first, second, ..] = cuts.as_slice() else {
        return Err(format!(
            "the detection found {} cuts; two are needed",
            cuts.len()
        ));
    };

    // 3. the review, by the proposer chosen, for its sentence
    let review_by = req.review.name();
    let (review, surface) = propose(&mut c, &mut observations, review_by, req.review.review())?;

    // 4. the second cut applied through the generated surface: the agent
    // acts by id, and its card may sit below the fold
    let seen = bounds(&mut c)?;
    observations += 1;
    let apply = button(&seen, &surface, second, "apply_cut")
        .ok_or_else(|| format!("no apply button of cut {second} on the surface {surface}"))?;
    let applied = act(&mut c, apply.as_str(), "activate", None, &seen)?;
    let applied = (done(&applied, "apply the second cut")?, apply);

    // 5. the person rejects the first cut on the generated surface: its
    // card shows, or the person scrolls to it
    let seen = now(&mut c)?;
    observations += 1;
    let reject = button(&seen, &surface, first, "reject_cut")
        .ok_or_else(|| format!("no reject button of cut {first} on the surface {surface}"))?;
    poke(&pokes, Poke::Click(reject.clone()))?;

    // 6. the comparison, in place
    let (compare, _) = propose(&mut c, &mut observations, req.compare.name(), COMPARE)?;

    // 7. close
    let seen = bounds(&mut c)?;
    observations += 1;
    let closed = act(&mut c, "silences_surface.close", "close", None, &seen)?;
    let close = done(&closed, "close")?;
    now(&mut c)?;
    observations += 1;
    Ok(Story {
        observations,
        detect,
        cuts,
        review,
        surface,
        applied,
        rejected_by_person: reject,
        compare,
        close,
    })
}
