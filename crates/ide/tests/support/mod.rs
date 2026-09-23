//! A real repository in a temp dir and a workbench on a real git worker.
//! Every wait is on a worker event with a timeout, never a sleep.

use std::path::Path;
use std::process::Command;
use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

use git::{GitClient, GitEvent};
use ide::workbench::Workbench;
use sem::{Actor, Outcome, Receipt, Request, RequestId, Snapshot, Stage, Value};
use tempfile::TempDir;

/// How long a git answer may take before a test fails.
pub const WAIT: Duration = Duration::from_secs(10);

/// The in-process agent of the tests.
pub fn agent() -> Actor {
    Actor::Agent {
        name: "test-agent".into(),
    }
}

pub struct Rig {
    pub dir: TempDir,
    pub bench: Workbench,
    events: Receiver<GitEvent>,
}

impl Rig {
    /// A repository with one commit (`README`), `files` written in the
    /// worktree (each holding its own name) and `staged` added to the
    /// index, and a workbench connected to it with its first status, log
    /// and branches applied.
    pub fn new(files: &[&str], staged: &[&str]) -> Rig {
        let dir = TempDir::new().expect("temp dir");
        let at = dir.path();
        git(at, &["init", "--initial-branch=main"]);
        git(at, &["config", "user.name", "Test Author"]);
        git(at, &["config", "user.email", "test@example.com"]);
        git(at, &["config", "commit.gpgsign", "false"]);
        write(at, "README", "readme\n");
        git(at, &["add", "README"]);
        git(at, &["commit", "--message", "initial"]);
        for file in files {
            write(at, file, &format!("{file}\n"));
        }
        if !staged.is_empty() {
            let mut add = vec!["add", "--"];
            add.extend_from_slice(staged);
            git(at, &add);
        }
        let (tx, events) = channel();
        let client = GitClient::spawn(at, move |event| {
            let _ = tx.send(event);
        })
        .expect("a git worker on the temp repository");
        let mut bench = Workbench::new(1280.0, 800.0);
        bench.connect(client);
        let mut rig = Rig { dir, bench, events };
        rig.until(|e| matches!(e, GitEvent::Branches(_)));
        rig
    }

    /// Applies worker events, in order, until one matches `last`.
    pub fn until(&mut self, last: impl Fn(&GitEvent) -> bool) {
        loop {
            let event = self.events.recv_timeout(WAIT).expect("a git event in time");
            let stop = last(&event);
            self.bench.apply(event);
            if stop {
                return;
            }
        }
    }

    /// Waits for the worker's answer to request `id`, which left for it,
    /// then for the status refresh that answer triggers.
    pub fn settle(&mut self, id: RequestId) {
        while !self.receipt(id).is_finished() {
            let event = self
                .events
                .recv_timeout(WAIT)
                .expect("a git answer in time");
            self.bench.apply(event);
        }
        self.until(|e| matches!(e, GitEvent::Branches(_)));
    }

    /// The workbench projected now.
    pub fn look(&self) -> Snapshot {
        self.bench.snapshot().expect("the projection validates")
    }

    /// `actor` observes the workbench now and asks for `action`.
    pub fn observe(
        &self,
        actor: Actor,
        target: &str,
        action: &str,
        param: Option<Value>,
    ) -> Request {
        Request::observe(&self.look(), actor, target, action, param).expect("observe")
    }

    /// Observes and dispatches at once.
    pub fn ask(
        &mut self,
        actor: Actor,
        target: &str,
        action: &str,
        param: Option<Value>,
    ) -> RequestId {
        let req = self.observe(actor, target, action, param);
        self.bench.dispatch(req)
    }

    pub fn receipt(&self, id: RequestId) -> Receipt {
        self.bench
            .receipt(id)
            .cloned()
            .expect("the receipt is kept")
    }

    /// The names of the stages receipt `id` went through.
    pub fn stages(&self, id: RequestId) -> Vec<&'static str> {
        self.receipt(id)
            .stages
            .iter()
            .map(|(s, _)| s.name())
            .collect()
    }

    /// Runs `git` in the repository; its stdout.
    pub fn git(&self, args: &[&str]) -> String {
        git(self.dir.path(), args)
    }

    pub fn head(&self) -> String {
        self.git(&["rev-parse", "HEAD"]).trim().to_owned()
    }

    /// How many commits HEAD has.
    pub fn commits(&self) -> usize {
        self.git(&["rev-list", "--count", "HEAD"])
            .trim()
            .parse()
            .expect("a count")
    }

    /// The paths the last commit recorded, as `git show --name-only`
    /// lists them.
    pub fn head_files(&self) -> Vec<String> {
        self.git(&["show", "--name-only", "--format=", "HEAD"])
            .lines()
            .filter(|l| !l.is_empty())
            .map(str::to_owned)
            .collect()
    }

    /// A value of the workbench's projection now.
    pub fn value(&self, at: &str) -> Value {
        self.look()
            .value(&at.parse().expect("a state ref"))
            .cloned()
            .expect("projected")
    }

    /// The agent opens the commit form and writes `message`, each step
    /// ending now.
    pub fn agent_writes(&mut self, message: &str) {
        let open = self.ask(agent(), "commit.form", "open", None);
        let text = Some(message.into());
        let set = self.ask(agent(), "commit.message", "set_value", text);
        for id in [open, set] {
            assert!(
                completed(&self.receipt(id)).is_some(),
                "{:?}",
                self.receipt(id)
            );
        }
    }
}

/// The outcome of a receipt that completed.
pub fn completed(receipt: &Receipt) -> Option<&Outcome> {
    match receipt.stage() {
        Some(Stage::Completed(outcome)) => Some(outcome),
        _ => None,
    }
}

/// The set value `{a, b}` from a list of paths.
pub fn set(paths: &[&str]) -> Value {
    Value::Set(paths.iter().map(|p| p.to_string()).collect())
}

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .expect("git runs");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn write(dir: &Path, rel: &str, content: &str) {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("dirs");
    }
    std::fs::write(path, content).expect("write");
}
