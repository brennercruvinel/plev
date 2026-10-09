//! the toy apps the recorder tests drive: the git client of the thesis
//! (the pattern of demo/commit/src/main.rs, with a sha for the
//! commit) and a login form with a secret field.

use std::collections::BTreeSet;

use accord::{
    Actor, Constraints, Execution, Operable, Outcome, ParamSpec, Request, RequestId, Role, SemId,
    Semantics, Snapshot, SnapshotBuilder, Value, ValueKind,
};
use indexmap::IndexMap;

/// the git client: what is staged, the message, the commits made, and how
/// many times anything was executed.
pub struct Git {
    pub staged: BTreeSet<String>,
    pub message: String,
    pub commits: Vec<(String, Vec<String>)>,
    pub executed: usize,
}

impl Git {
    pub fn new(files: &[&str], message: &str) -> Self {
        Git {
            staged: files.iter().map(|f| (*f).to_owned()).collect(),
            message: message.to_owned(),
            commits: Vec::new(),
            executed: 0,
        }
    }
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

/// a short hex digest standing in for the commit sha.
pub fn sha(files: &[String], message: &str) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in files.iter().flat_map(|f| f.bytes()).chain(message.bytes()) {
        h ^= u64::from(byte);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{:07x}", h >> 36)
}

impl Operable for Git {
    fn execute(&mut self, _id: RequestId, req: &Request) -> Execution {
        self.executed += 1;
        match (req.action.as_str(), &req.param) {
            ("unstage", Some(Value::Text(file))) => {
                self.staged.remove(file);
                Execution::Done(Outcome::default())
            }
            ("submit", None) => {
                let files: Vec<String> = self.staged.iter().cloned().collect();
                let sha = sha(&files, &self.message);
                self.commits.push((sha.clone(), files.clone()));
                self.staged.clear();
                Execution::Done(Outcome {
                    result: IndexMap::from([
                        ("commit".to_owned(), Value::Text(sha)),
                        ("files".to_owned(), Value::List(files)),
                    ]),
                })
            }
            (action, _) => Execution::Failed(format!("no {action} here")),
        }
    }
}

/// a login form whose password is a secret field. its `set` echoes the
/// secret back in the outcome and its failure quotes it, as a careless app
/// would: the recorder must keep neither.
pub struct Login {
    pub user: String,
    pub password: String,
}

impl Semantics for Login {
    fn project(&self, b: &mut SnapshotBuilder) {
        b.node("login.user", Role::TextInput, "user")
            .state("text", self.user.as_str());
        b.node("login.password", Role::TextInput, "password")
            .state("text", self.password.as_str())
            .constraints(Constraints {
                secret: true,
                editable: true,
                ..Constraints::default()
            })
            .action("set")
            .param(ParamSpec {
                kind: ValueKind::Text,
                constrained: false,
            });
        b.node("login.submit", Role::Button, "sign in")
            .action("submit")
            .reads("login.user:text")
            .reads("login.password:text");
    }
}

impl Operable for Login {
    fn execute(&mut self, _id: RequestId, req: &Request) -> Execution {
        match (req.action.as_str(), &req.param) {
            ("set", Some(Value::Text(secret))) => {
                self.password = secret.clone();
                Execution::Done(Outcome {
                    result: IndexMap::from([("echo".to_owned(), Value::Text(secret.clone()))]),
                })
            }
            ("set", _) => Execution::Failed(format!("cannot set {:?}", req.param)),
            _ => Execution::Done(Outcome::default()),
        }
    }
}

pub fn look(app: &impl Semantics) -> Snapshot {
    let mut b = SnapshotBuilder::new();
    app.project(&mut b);
    match b.finish() {
        Ok(snapshot) => snapshot,
        Err(errors) => panic!("the test projection is invalid: {errors}"),
    }
}

pub fn agent() -> Actor {
    Actor::Agent {
        name: "reviewer".to_owned(),
    }
}

pub fn id(text: &str) -> SemId {
    match SemId::parse(text) {
        Ok(id) => id,
        Err(e) => panic!("{text}: {e}"),
    }
}
