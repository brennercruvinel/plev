//! the thesis scenario (generative-runtime-thesis.md, sections 8 and 9).
//!
//! an agent observes a git client: files A, B and C staged, the message
//! "fix login". it decides to commit. while it decides, the person at the
//! screen unstages B. the agent's commit carries what it observed, so it
//! comes back rejected with the delta (removed: B) and nothing is
//! committed. the agent observes again and commits A and C.
//!
//! `cargo run -p sem --example commit` prints the three receipts.

use std::collections::BTreeSet;
use std::error::Error;

use indexmap::IndexMap;
use sem::{
    Actor, Change, Dispatcher, Execution, Operable, Outcome, ParamSpec, Receipt, Rejection,
    Request, RequestId, Role, Semantics, Snapshot, SnapshotBuilder, Stage, Value, ValueKind,
};

/// the app: what is staged, the message, the commits made.
struct Git {
    staged: BTreeSet<String>,
    message: String,
    commits: Vec<(String, Vec<String>)>,
}

impl Semantics for Git {
    fn project(&self, b: &mut SnapshotBuilder) {
        b.node("changes", Role::List, "changes")
            .state("staged", self.staged.clone())
            .action("unstage")
            .reads("changes:staged")
            .param(ParamSpec {
                kind: ValueKind::Text,
                constrained: false,
            });
        b.node("commit.message", Role::TextInput, "commit message")
            .state("text", self.message.as_str());
        b.node("commit.submit", Role::Button, "commit")
            .enabled(!self.staged.is_empty())
            .action("submit")
            .reads("changes:staged")
            .reads("commit.message:text");
    }
}

impl Operable for Git {
    fn execute(&mut self, req: &Request) -> Execution {
        match (req.action.as_str(), &req.param) {
            ("unstage", Some(Value::Text(file))) => {
                self.staged.remove(file);
                Execution::Done(Outcome::default())
            }
            ("submit", None) => {
                let files: Vec<String> = self.staged.iter().cloned().collect();
                let id = format!("c{}", self.commits.len() + 1);
                self.commits.push((id.clone(), files.clone()));
                self.staged.clear();
                Execution::Done(Outcome {
                    result: IndexMap::from([
                        ("commit".to_owned(), Value::Text(id)),
                        ("files".to_owned(), Value::List(files)),
                    ]),
                })
            }
            (action, _) => Execution::Failed(format!("the example has no {action}")),
        }
    }
}

fn look(git: &Git) -> Result<Snapshot, Box<dyn Error>> {
    let mut b = SnapshotBuilder::new();
    git.project(&mut b);
    Ok(b.finish()?)
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut git = Git {
        staged: ["A", "B", "C"].map(String::from).into(),
        message: "fix login".to_owned(),
        commits: Vec::new(),
    };
    let mut dispatcher = Dispatcher::new();
    let agent = Actor::Agent {
        name: "reviewer".to_owned(),
    };

    let seen_by_agent = look(&git)?;
    let commit = Request::observe(
        &seen_by_agent,
        agent.clone(),
        "commit.submit",
        "submit",
        None,
    )?;

    let seen_by_human = look(&git)?;
    let unstage_b = Request::observe(
        &seen_by_human,
        Actor::Human,
        "changes",
        "unstage",
        Some("B".into()),
    )?;
    let first = dispatcher.dispatch(&mut git, unstage_b, 1_000);
    let second = dispatcher.dispatch(&mut git, commit, 1_200);

    let seen_again = look(&git)?;
    let again = Request::observe(&seen_again, agent, "commit.submit", "submit", None)?;
    let third = dispatcher.dispatch(&mut git, again, 1_500);

    for id in [first, second, third] {
        show(&dispatcher, id);
    }
    println!("commits: {:?}", git.commits);
    Ok(())
}

fn show(dispatcher: &Dispatcher, id: RequestId) {
    let Some(receipt) = dispatcher.receipt(id) else {
        println!("receipt {id}: not kept");
        return;
    };
    let Receipt {
        request,
        validated_against,
        stages,
        ..
    } = receipt;
    println!(
        "receipt {id}\n  target:   {}\n  action:   {}\n  actor:    {}",
        request.target, request.action, request.actor
    );
    println!("  observed: (revision {})", request.observed.revision);
    for (at, value) in &request.observed.values {
        println!("    {at} = {}", value.as_ref().map_or("none".into(), text));
    }
    if let Some(revision) = validated_against {
        println!("  validated against revision {revision}");
    }
    for (stage, t) in stages {
        println!("  t={t:<6} {}", describe(stage));
    }
    println!();
}

fn describe(stage: &Stage) -> String {
    match stage {
        Stage::Confirmed { by } => format!("confirmed by {by}"),
        Stage::Rejected(Rejection::Stale { changes }) => {
            let mut out = "rejected: the state it observed changed".to_owned();
            for change in changes {
                if let Change::StateChanged {
                    at, added, removed, ..
                } = change
                {
                    out.push_str(&format!(
                        "\n{:13}{at}: added {added:?}, removed {removed:?}",
                        ""
                    ));
                }
            }
            out
        }
        Stage::Rejected(why) => format!("rejected: {why:?}"),
        Stage::Completed(outcome) if outcome.result.is_empty() => "completed".to_owned(),
        Stage::Completed(outcome) => {
            let result: Vec<String> = outcome
                .result
                .iter()
                .map(|(k, v)| format!("{k} = {}", text(v)))
                .collect();
            format!("completed: {}", result.join(", "))
        }
        Stage::Failed { error } => format!("failed: {error}"),
        other => other.name().to_owned(),
    }
}

fn text(value: &Value) -> String {
    match value {
        Value::Bool(v) => v.to_string(),
        Value::Int(v) => v.to_string(),
        Value::Num(v) => v.to_string(),
        Value::Text(v) => format!("{v:?}"),
        Value::List(items) => format!("[{}]", items.join(", ")),
        Value::Set(items) => {
            let items: Vec<&str> = items.iter().map(String::as_str).collect();
            format!("{{{}}}", items.join(", "))
        }
    }
}
