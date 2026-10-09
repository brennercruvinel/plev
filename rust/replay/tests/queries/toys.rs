//! the toy app the inspector's tests record, and a small lab that drives
//! it the way an app with the recorder does: every act goes through the
//! dispatcher, whose stages reach the recorder's sink, and the app's
//! projection is recorded after it (the pattern of
//! rust/record/tests/session/).

use std::collections::BTreeSet;

use accord::{
    Actor, Dispatcher, Execution, Millis, Operable, Outcome, ParamSpec, Request, RequestId, Role,
    SemId, Semantics, Snapshot, SnapshotBuilder, Value, ValueKind,
};
use indexmap::IndexMap;
use record::{InputKind, Limits, Recorder, Session};

/// the git client of the thesis: what is staged, the message, the commits.
/// a `slow` one only starts a submit (`Execution::Started`), and the test
/// ends it later through the dispatcher, the way the ide's git worker does.
pub struct Git {
    pub staged: BTreeSet<String>,
    pub message: String,
    pub commits: Vec<(String, Vec<String>)>,
    pub executed: usize,
    pub slow: bool,
}

impl Git {
    pub fn new(files: &[&str], message: &str) -> Self {
        Git {
            staged: files.iter().map(|f| (*f).to_owned()).collect(),
            message: message.to_owned(),
            commits: Vec::new(),
            executed: 0,
            slow: false,
        }
    }
}

impl Semantics for Git {
    fn project(&self, b: &mut SnapshotBuilder) {
        let text = ParamSpec {
            kind: ValueKind::Text,
            constrained: false,
        };
        b.node("changes", Role::List, "changes")
            .state("staged", self.staged.clone())
            .action("unstage")
            .reads("changes:staged")
            .param(text);
        b.node("commit.message", Role::TextInput, "commit message")
            .state("text", self.message.as_str())
            .action("set_value")
            .param(text);
        b.node("commit.submit", Role::Button, "Commit")
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
            ("set_value", Some(Value::Text(text))) => {
                self.message = text.clone();
                Execution::Done(Outcome::default())
            }
            ("submit", None) if self.slow => Execution::Started,
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

pub fn rid(n: u64) -> RequestId {
    match RequestId::parse(&n.to_string()) {
        Ok(id) => id,
        Err(e) => panic!("{n}: {e}"),
    }
}

/// the app, its dispatcher and the recorder, driven with explicit times.
pub struct Lab {
    pub rec: Recorder,
    pub dispatcher: Dispatcher,
    pub git: Git,
}

impl Lab {
    /// the app with `files` staged and `message` written, its first
    /// projection recorded at `t`.
    pub fn new(files: &[&str], message: &str, t: Millis) -> Lab {
        let rec = Recorder::new(Limits::default());
        let mut dispatcher = Dispatcher::new();
        dispatcher.add_sink(Box::new(rec.sink()));
        let git = Git::new(files, message);
        rec.snapshot(t, &look(&git));
        Lab {
            rec,
            dispatcher,
            git,
        }
    }

    /// what the app projects now.
    pub fn look(&self) -> Snapshot {
        look(&self.git)
    }

    /// `actor` observes the whole app at `t`, and the recorder is told.
    pub fn observe(&self, t: Millis, actor: Actor) -> Snapshot {
        let seen = self.look();
        self.rec.snapshot(t, &seen);
        self.rec.observed(t, actor, None, seen.revision());
        seen
    }

    /// `actor` observes only `scope` at `t`: the revision is still the
    /// whole snapshot's, as the agent port serves it.
    pub fn observe_scope(&self, t: Millis, actor: Actor, scope: &str) -> Snapshot {
        let seen = self.look();
        self.rec.snapshot(t, &seen);
        self.rec
            .observed(t, actor, Some(id(scope)), seen.revision());
        seen
    }

    /// `actor` acts at `t` on what it saw in `seen`, through the
    /// dispatcher; the projection after it is recorded at `t` too.
    pub fn act(
        &mut self,
        t: Millis,
        seen: &Snapshot,
        actor: Actor,
        target: &str,
        action: &str,
        param: Option<Value>,
    ) -> RequestId {
        let req = match Request::observe(seen, actor, target, action, param) {
            Ok(req) => req,
            Err(e) => panic!("{target} {action}: {e}"),
        };
        let done = self.dispatcher.dispatch(&mut self.git, req, t);
        self.rec.snapshot(t, &self.look());
        done
    }

    /// a person clicks on `target` at `t` and unstages `file`, on the
    /// state as it is now.
    pub fn person_unstages(&mut self, t: Millis, file: &str) -> RequestId {
        self.rec.input(
            t,
            InputKind::PointerDown,
            Some([10.0, 20.0]),
            Some(id("changes")),
        );
        let seen = self.look();
        self.act(
            t,
            &seen,
            Actor::Human,
            "changes",
            "unstage",
            Some(file.into()),
        )
    }

    /// the index moves at `t` from outside the app (a `git` in a
    /// terminal): no request, only the next projection shows it.
    pub fn outside_unstages(&mut self, t: Millis, file: &str) {
        self.git.staged.remove(file);
        self.rec.snapshot(t, &self.look());
    }

    pub fn session(&self) -> Session {
        self.rec.snapshot_window()
    }
}
