//! the scripted agent (arm A0): the laboratory's own agent over the wire,
//! doing the canonical cycle with no model behind it. it observes, puts
//! the task's files in the index and takes the others out, writes the
//! message, observes again, submits on the revision it observed; when the
//! submit comes back stale it observes the changes since, repairs the
//! index and submits again on the new revision; then it looks at the
//! stacks. every act rests on the observation it was decided on
//! (`from_revision`), so a stale world is refused, never acted on.

use std::collections::BTreeSet;
use std::path::Path;

use sem::{ActionName, Change, Node, Outcome, Rejection, Revision, SemId, Stage, StateKey, Value};
use wire::{Act, Client, Observe, Reply, SnapshotReply, StageReply};

use crate::repo::{TASK_FILES, TASK_MESSAGE};

/// how long the agent waits for one reply.
const TIMEOUT_MS: u64 = 30_000;

/// how many submits the agent tries before giving up: one for the
/// decision, one for the recovery after a stale world, one spare.
const ATTEMPTS: usize = 3;

/// what the scripted agent went through.
#[derive(Debug, Default)]
pub struct Scripted {
    /// one line per step, as the agent saw it.
    pub steps: Vec<String>,
    /// the delta of each stale rejection the agent met.
    pub stale: Vec<Vec<Change>>,
    /// the outcome of the completed submit.
    pub completed: Option<Outcome>,
    /// what stopped the script early.
    pub error: Option<String>,
}

pub fn run(path: &Path, name: &str) -> Result<Scripted, String> {
    let mut out = Scripted::default();
    let mut agent = Agent::connect(path, name, &mut out)?;
    match agent.cycle(&mut out) {
        Ok(()) => {}
        Err(e) => out.error = Some(e),
    }
    Ok(out)
}

struct Agent {
    client: Client,
}

impl Agent {
    fn connect(path: &Path, name: &str, out: &mut Scripted) -> Result<Agent, String> {
        let mut client = Client::connect(path).map_err(|e| e.to_string())?;
        client
            .set_timeout(Some(TIMEOUT_MS))
            .map_err(|e| e.to_string())?;
        let actor = client.hello(name).map_err(|e| e.to_string())?;
        out.steps.push(format!("hello: {actor}"));
        Ok(Agent { client })
    }

    fn cycle(&mut self, out: &mut Scripted) -> Result<(), String> {
        let seen = self.observe(out)?;
        let staged = staged(&seen);
        let files = files(&seen);
        let mut revision = seen.revision;

        // the index as the task wants it: its files in, the rest out.
        for path in &files {
            let is_task = TASK_FILES.contains(&path.as_str());
            let is_staged = staged.contains(path);
            let action = match (is_task, is_staged) {
                (true, false) => "stage",
                (false, true) => "unstage",
                _ => continue,
            };
            self.act(
                out,
                &format!("changes.file[{path}]"),
                action,
                None,
                revision,
            )?;
            // the next act reads its own file, and the world just moved:
            // the revision an act rests on is the one it was decided on.
            revision = self.observe(out)?.revision;
        }

        // the view shows each act as it completes, and a status read
        // before an act never lands over it (task-84), so the agent goes
        // on from the next observation as it is. the message is still
        // empty here, so the person of the script, who waits for a
        // message too, cannot fire yet.
        let now = self.observe(out)?;
        if value(&now, "commit.form", "open") != Some(Value::Bool(true)) {
            self.act(out, "commit.form", "open", None, now.revision)?;
        }
        let seen = self.observe(out)?;
        self.act(
            out,
            "commit.message",
            "set_value",
            Some(Value::Text(TASK_MESSAGE.to_owned())),
            seen.revision,
        )?;

        // the decision: one observation, then the submit on exactly that
        // observation. anything that moves the staged set between the two
        // (the person of the script does, right after this observation is
        // served) makes the submit stale, with the delta.
        for attempt in 1..=ATTEMPTS {
            let decided = self.observe(out)?;
            let stages = self.act(out, "commit.submit", "submit", None, decided.revision)?;
            match terminal(&stages) {
                Some(Stage::Completed(outcome)) => {
                    out.completed = Some(outcome.clone());
                    let stacks = self.observe_scope(out, "stacks")?;
                    out.steps.push(format!(
                        "stacks: {} commit(s) projected",
                        stacks.nodes.len().saturating_sub(1)
                    ));
                    return Ok(());
                }
                Some(Stage::Rejected(Rejection::Stale { changes, acted })) => {
                    out.stale.push(changes.clone());
                    out.steps.push(format!(
                        "submit {attempt} refused as stale: {changes:?}; acted: {}",
                        crate::story::acted_text(acted)
                    ));
                    self.repair(out, decided.revision)?;
                }
                other => return Err(format!("submit {attempt} ended {other:?}")),
            }
        }
        Err(format!("{ATTEMPTS} submits were refused as stale"))
    }

    /// after a stale refusal: what changed since the decision, then the
    /// index put back where the task wants it.
    fn repair(&mut self, out: &mut Scripted, since: Revision) -> Result<(), String> {
        let reply = self
            .client
            .observe(Observe {
                since: Some(since),
                ..Observe::default()
            })
            .map_err(|e| e.to_string())?;
        match reply {
            Reply::Delta(d) => out.steps.push(format!(
                "observed since {since}: {} change(s)",
                d.changes.len()
            )),
            Reply::Snapshot(_) => out
                .steps
                .push(format!("observed since {since}: the whole snapshot")),
            other => return Err(format!("an observation: {other:?}")),
        }
        let mut seen = self.observe(out)?;
        for path in TASK_FILES {
            if !staged(&seen).contains(path) {
                let revision = seen.revision;
                self.act(
                    out,
                    &format!("changes.file[{path}]"),
                    "stage",
                    None,
                    revision,
                )?;
            }
            seen = self.observe(out)?;
        }
        Ok(())
    }

    fn observe(&mut self, out: &mut Scripted) -> Result<SnapshotReply, String> {
        let reply = self
            .client
            .observe(Observe::default())
            .map_err(|e| e.to_string())?;
        match reply {
            Reply::Snapshot(s) => {
                out.steps.push(format!(
                    "observed revision {} ({} nodes, staged {:?})",
                    s.revision,
                    s.nodes.len(),
                    staged(&s)
                ));
                Ok(s)
            }
            other => Err(format!("a snapshot: {other:?}")),
        }
    }

    fn observe_scope(&mut self, out: &mut Scripted, scope: &str) -> Result<SnapshotReply, String> {
        let scope = SemId::parse(scope).map_err(|e| e.to_string())?;
        let reply = self
            .client
            .observe(Observe {
                scope: Some(scope.clone()),
                ..Observe::default()
            })
            .map_err(|e| e.to_string())?;
        match reply {
            Reply::Snapshot(s) => {
                out.steps
                    .push(format!("observed {scope}: {} nodes", s.nodes.len()));
                Ok(s)
            }
            other => Err(format!("a snapshot: {other:?}")),
        }
    }

    /// one act resting on `revision`; its stages until the terminal one.
    fn act(
        &mut self,
        out: &mut Scripted,
        target: &str,
        action: &str,
        param: Option<Value>,
        revision: Revision,
    ) -> Result<Vec<StageReply>, String> {
        let target = SemId::parse(target).map_err(|e| e.to_string())?;
        let action = ActionName::parse(action).map_err(|e| e.to_string())?;
        let mut act = Act::from_revision(target.clone(), action.clone(), revision);
        if let Some(param) = param {
            act = act.with_param(param);
        }
        let stages = self.client.act(act, |_| {}).map_err(|e| e.to_string())?;
        let names: Vec<&str> = stages.iter().map(|s| s.stage.name()).collect();
        out.steps.push(format!(
            "act {target} {action} from {revision}: {}",
            names.join(" -> ")
        ));
        match terminal(&stages) {
            Some(Stage::Completed(_)) => Ok(stages),
            Some(Stage::Rejected(Rejection::Stale { .. })) if action.as_str() == "submit" => {
                Ok(stages)
            }
            other => Err(format!("{target} {action} ended {other:?}")),
        }
    }
}

fn terminal(stages: &[StageReply]) -> Option<&Stage> {
    stages.last().map(|s| &s.stage).filter(|s| s.is_terminal())
}

fn node<'a>(seen: &'a SnapshotReply, id: &str) -> Option<&'a Node> {
    seen.nodes.iter().find(|n| n.id.as_str() == id)
}

fn value(seen: &SnapshotReply, id: &str, key: &str) -> Option<Value> {
    let key = StateKey::parse(key).ok()?;
    node(seen, id)?.state.get(&key).cloned()
}

fn staged(seen: &SnapshotReply) -> BTreeSet<String> {
    match value(seen, "changes", "staged") {
        Some(Value::Set(staged)) => staged,
        _ => BTreeSet::new(),
    }
}

fn files(seen: &SnapshotReply) -> Vec<String> {
    match value(seen, "changes", "files") {
        Some(Value::List(files)) => files,
        _ => Vec::new(),
    }
}
