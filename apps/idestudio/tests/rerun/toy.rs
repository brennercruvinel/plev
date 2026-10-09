//! the harness against a toy fixture: a git of the thesis in a few lines,
//! recorded the way an app records (a real dispatcher whose stages reach
//! the recorder, the projection recorded after each act), written as an
//! excerpt, read back, and rerun against a fresh one. it tells whether the
//! harness keeps the recorded order, settles where it must and nowhere
//! else, reads the marks at their point, and says what diverged.

use std::collections::BTreeSet;

use record::{Limits, Recorder, Session};
use accord::{
    Actor, Confirm, Dispatcher, Execution, Millis, Operable, Outcome, ParamSpec, Policy, Receipt,
    Request, RequestId, Role, Semantics, Snapshot, SnapshotBuilder, Stage, Value, ValueKind,
};

use crate::harness::{Fixture, rerun};
use crate::marks;
use crate::report::Report;

/// what is staged, the message and the commits. `reads_staged` off is the
/// submit's precondition broken on purpose; a `slow` submit only starts,
/// and ends when the fixture settles, the way a git worker answers later.
struct Git {
    staged: BTreeSet<String>,
    message: String,
    commits: Vec<Vec<String>>,
    reads_staged: bool,
    slow: bool,
}

impl Git {
    fn new(files: &[&str], reads_staged: bool, slow: bool) -> Git {
        Git {
            staged: files.iter().map(|f| (*f).to_owned()).collect(),
            message: "fix login".to_owned(),
            commits: Vec::new(),
            reads_staged,
            slow,
        }
    }

    /// commits what is staged.
    fn commit(&mut self) -> Outcome {
        let files: Vec<String> = std::mem::take(&mut self.staged).into_iter().collect();
        self.commits.push(files.clone());
        let mut outcome = Outcome::default();
        outcome
            .result
            .insert("files".to_owned(), Value::List(files));
        outcome
    }
}

impl Semantics for Git {
    fn project(&self, b: &mut SnapshotBuilder) {
        let text = ParamSpec {
            kind: ValueKind::Text,
            constrained: false,
        };
        let mut changes = b.node("changes", Role::List, "changes");
        changes.state("staged", self.staged.clone());
        changes
            .action("unstage")
            .reads("changes:staged")
            .param(text);
        changes
            .action("discard")
            .reads("changes:staged")
            .param(text)
            .policy(Policy {
                confirm: Confirm::ForAgents,
                destructive: true,
                ..Policy::default()
            });
        b.node("commit.message", Role::TextInput, "commit message")
            .state("text", self.message.as_str())
            .action("set_value")
            .param(text);
        let mut submit = b.node("commit.submit", Role::Button, "Commit");
        submit.enabled(!self.staged.is_empty());
        let mut action = submit.action("submit");
        if self.reads_staged {
            action.reads("changes:staged");
        }
        action.reads("commit.message:text");
    }
}

impl Operable for Git {
    fn execute(&mut self, _id: RequestId, req: &Request) -> Execution {
        let done = Execution::Done(Outcome::default());
        match (req.action.as_str(), &req.param) {
            ("unstage" | "discard", Some(Value::Text(file))) => {
                self.staged.remove(file);
                done
            }
            ("set_value", Some(Value::Text(text))) => {
                self.message = text.clone();
                done
            }
            ("submit", None) if self.slow => Execution::Started,
            ("submit", None) => Execution::Done(self.commit()),
            (action, _) => Execution::Failed(format!("no {action} here")),
        }
    }
}

fn look(git: &Git) -> Snapshot {
    let mut b = SnapshotBuilder::new();
    git.project(&mut b);
    match b.finish() {
        Ok(snapshot) => snapshot,
        Err(errors) => panic!("the toy projection is invalid: {errors}"),
    }
}

fn agent() -> Actor {
    Actor::Agent {
        name: "toy".to_owned(),
    }
}

/// the toy, its dispatcher and a recorder, driven with explicit times:
/// how a session of it is recorded.
struct Lab {
    rec: Recorder,
    dispatcher: Dispatcher,
    git: Git,
    started: Vec<RequestId>,
}

impl Lab {
    fn new(git: Git) -> Lab {
        let rec = Recorder::new(Limits::default());
        let mut dispatcher = Dispatcher::new();
        dispatcher.add_sink(Box::new(rec.sink()));
        rec.snapshot(1_000, &look(&git));
        Lab {
            rec,
            dispatcher,
            git,
            started: Vec::new(),
        }
    }

    fn observe(&self, t: Millis) -> Snapshot {
        let seen = look(&self.git);
        self.rec.snapshot(t, &seen);
        self.rec.observed(t, agent(), None, seen.revision());
        seen
    }

    fn act(
        &mut self,
        t: Millis,
        seen: &Snapshot,
        actor: Actor,
        target: &str,
        action: &str,
        param: Option<&str>,
    ) -> RequestId {
        let param = param.map(Value::from);
        let req = match Request::observe(seen, actor, target, action, param) {
            Ok(req) => req,
            Err(e) => panic!("{target} {action}: {e}"),
        };
        let id = self.dispatcher.dispatch(&mut self.git, req, t);
        if matches!(
            self.dispatcher.receipt(id).and_then(Receipt::stage),
            Some(Stage::Executing)
        ) {
            self.started.push(id);
        }
        self.rec.snapshot(t, &look(&self.git));
        id
    }

    /// a person unstages `file` at `t`, on the state as it is now.
    fn person_unstages(&mut self, t: Millis, file: &str) -> RequestId {
        let seen = look(&self.git);
        self.act(t, &seen, Actor::Human, "changes", "unstage", Some(file))
    }

    /// the slow submits end at `t`.
    fn land(&mut self, t: Millis) {
        for id in std::mem::take(&mut self.started) {
            let outcome = self.git.commit();
            if let Err(e) = self.dispatcher.complete(id, outcome, t) {
                panic!("request {id} cannot end: {e}");
            }
        }
        self.rec.snapshot(t, &look(&self.git));
    }

    /// the whole window as a session file would carry it, read back.
    fn excerpt(&self, t0: Millis, t1: Millis) -> Session {
        let bytes = match self.rec.snapshot_window().excerpt(t0, t1).to_bytes() {
            Ok((bytes, _)) => bytes,
            Err(e) => panic!("the excerpt does not write: {e}"),
        };
        match Session::from_bytes(&bytes) {
            Ok(session) => session,
            Err(e) => panic!("the excerpt does not read: {e}"),
        }
    }
}

/// the thesis, recorded: the agent observes A, B and C staged, a person
/// unstages B, the agent's submit is refused stale, the agent observes
/// again and commits A and C.
fn thesis(slow: bool) -> Lab {
    let mut lab = Lab::new(Git::new(&["A", "B", "C"], true, slow));
    let seen = lab.observe(1_000);
    lab.person_unstages(1_100, "B");
    lab.act(1_300, &seen, agent(), "commit.submit", "submit", None);
    let again = lab.observe(1_400);
    lab.act(1_500, &again, agent(), "commit.submit", "submit", None);
    lab.land(1_600);
    lab
}

/// the toy rerun: a fresh app and dispatcher on the clock the harness
/// gives, every call it took in order.
struct Toy {
    git: Git,
    dispatcher: Dispatcher,
    started: Vec<RequestId>,
    now: Millis,
    calls: Vec<String>,
}

impl Toy {
    fn new(files: &[&str], reads_staged: bool, slow: bool) -> Toy {
        Toy {
            git: Git::new(files, reads_staged, slow),
            dispatcher: Dispatcher::new(),
            started: Vec::new(),
            now: 0,
            calls: Vec::new(),
        }
    }

    /// the fixture the thesis starts from.
    fn start(reads_staged: bool) -> impl FnOnce() -> Result<Toy, String> {
        move || Ok(Toy::new(&["A", "B", "C"], reads_staged, false))
    }
}

impl Fixture for Toy {
    fn dispatch(&mut self, req: Request, t: Millis) -> RequestId {
        self.now = t;
        self.calls.push(format!("dispatch {} at {t}", req.action));
        let id = self.dispatcher.dispatch(&mut self.git, req, t);
        if matches!(
            self.dispatcher.receipt(id).and_then(Receipt::stage),
            Some(Stage::Executing)
        ) {
            self.started.push(id);
        }
        id
    }

    fn confirm(&mut self, id: RequestId, by: Actor, t: Millis) -> Result<(), String> {
        self.calls.push(format!("confirm {id}"));
        self.dispatcher
            .confirm(&mut self.git, id, by, t)
            .map_err(|e| e.to_string())
    }

    fn decline(&mut self, id: RequestId, by: Actor, t: Millis) -> Result<(), String> {
        self.calls.push(format!("decline {id}"));
        self.dispatcher
            .decline(id, by, t)
            .map_err(|e| e.to_string())
    }

    fn settle(&mut self) -> Result<(), String> {
        self.calls.push("settle".to_owned());
        for id in std::mem::take(&mut self.started) {
            let outcome = self.git.commit();
            self.dispatcher
                .complete(id, outcome, self.now)
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn project(&self) -> Result<Snapshot, String> {
        let mut b = SnapshotBuilder::new();
        self.git.project(&mut b);
        b.finish().map_err(|e| e.to_string())
    }

    fn receipt(&self, id: RequestId) -> Option<Receipt> {
        self.dispatcher.receipt(id).cloned()
    }
}

fn marks_of(text: &str) -> Vec<marks::Mark> {
    match marks::parse(text) {
        Ok(marks) => marks,
        Err(e) => panic!("{e}"),
    }
}

fn run<F: Fixture>(
    excerpt: &Session,
    marks: &str,
    start: impl FnOnce() -> Result<F, String>,
) -> Report {
    match rerun(excerpt, &marks_of(marks), start) {
        Ok(report) => report,
        Err(e) => panic!("the rerun could not be made: {e}"),
    }
}

/// the marks of the thesis: B out before the stale submit, the refusal on
/// what is staged, the commit of A and C, the state at a time.
const THESIS_MARKS: &str = r#"
# the person's unstage lands first
after request 1  changes:staged = {"A", "C"}
after request 2  stale = {"changes:staged"}
after request 3  outcome.files = ["A", "C"]
after request 3  changes:staged = {}
at 0     changes:staged = {"A", "B", "C"}
at +0.2s commit.message:text = "fix login"
"#;

#[test]
fn a_recorded_session_reruns_with_the_same_verdicts_and_its_marks_hold() {
    let lab = thesis(false);
    let excerpt = lab.excerpt(0, 2_000);
    let report = run(&excerpt, THESIS_MARKS, Toy::start(true));
    assert!(!report.diverged(), "{report}");
    let terminal: Vec<&str> = report
        .requests
        .iter()
        .map(|r| r.terminal.rerun.as_str())
        .collect();
    assert_eq!(
        terminal,
        [
            "completed",
            "rejected: stale, changes:staged (removed B)",
            "completed"
        ]
    );
    assert_eq!(report.marks.len(), 6);
    let text = report.to_string();
    assert!(text.ends_with("nothing diverged"), "{text}");
    assert!(
        text.contains(
            "same      request 2 (origin 0, rerun as 2): submit on commit.submit by agent:toy"
        ),
        "{text}"
    );
}

#[test]
fn a_broken_precondition_diverges_and_the_report_says_where() {
    let lab = thesis(false);
    let excerpt = lab.excerpt(0, 2_000);
    // the same session, against the toy whose submit no longer reads
    // what is staged: the stale submit now goes through.
    let report = run(&excerpt, THESIS_MARKS, Toy::start(false));
    assert!(report.diverged());
    let text = report.to_string();
    println!("{text}");
    for says in [
        "DIVERGED  request 2 (origin 0, rerun as 2): submit on commit.submit by agent:toy",
        "verdict   recorded  rejected: stale, changes:staged (removed B)",
        "rerun     accepted  <- differs",
        "terminal  recorded  rejected: stale, changes:staged (removed B)",
        "rerun     completed  <- differs",
        "DIVERGED  line 4: after request 2  stale = {\"changes:staged\"}",
        "found     absent",
        "DIVERGED  line 5: after request 3  outcome.files = [\"A\", \"C\"]",
        "found     the request did not complete in the rerun: rejected: disabled",
        "the rerun diverged: 2 request(s), 2 mark(s)",
    ] {
        assert!(text.contains(says), "{says:?} in\n{text}");
    }
    let first = &report.requests[0];
    assert!(first.same(), "the person's unstage is the same: {first:?}");
}

#[test]
fn a_mark_that_does_not_hold_says_what_was_found() {
    let lab = thesis(false);
    let excerpt = lab.excerpt(0, 2_000);
    let report = run(
        &excerpt,
        "after request 1  changes:staged = {\"A\", \"B\", \"C\"}\nat 0  commit.message:text = absent\n",
        Toy::start(true),
    );
    assert!(report.requests.iter().all(|r| r.same()));
    let text = report.to_string();
    assert!(text.contains("DIVERGED  line 1: after request 1  changes:staged = {\"A\", \"B\", \"C\"}\n            found     {\"A\", \"C\"}"), "{text}");
    assert!(text.contains("found     \"fix login\""), "{text}");
}

#[test]
fn an_execution_running_in_the_recording_is_in_flight_in_the_rerun_too() {
    // the slow submit started at 1_500 and ended at 1_600; a person
    // unstaged C at 1_550 while it ran, so the commit took A alone.
    let mut lab = Lab::new(Git::new(&["A", "C"], true, true));
    let seen = lab.observe(1_000);
    lab.act(1_500, &seen, agent(), "commit.submit", "submit", None);
    lab.person_unstages(1_550, "C");
    lab.land(1_600);
    let excerpt = lab.excerpt(0, 2_000);

    let mut toy = Toy::new(&["A", "C"], true, true);
    let report = run(
        &excerpt,
        "after request 1  outcome.files = [\"A\"]\n",
        || Ok(&mut toy),
    );
    assert!(!report.diverged(), "{report}");
    assert_eq!(
        toy.calls,
        [
            "settle",
            "dispatch submit at 1500",
            "dispatch unstage at 1550",
            "settle"
        ],
        "no settle while the submit was executing"
    );
}

#[test]
fn a_persons_confirmation_and_decline_are_rerun_as_answers() {
    let mut lab = Lab::new(Git::new(&["A", "B", "C"], true, false));
    let seen = lab.observe(1_000);
    let confirmed = lab.act(1_100, &seen, agent(), "changes", "discard", Some("B"));
    if let Err(e) = lab
        .dispatcher
        .confirm(&mut lab.git, confirmed, Actor::Human, 1_200)
    {
        panic!("{e}");
    }
    lab.rec.snapshot(1_200, &look(&lab.git));
    let again = lab.observe(1_300);
    let declined = lab.act(1_400, &again, agent(), "changes", "discard", Some("C"));
    if let Err(e) = lab.dispatcher.decline(declined, Actor::Human, 1_500) {
        panic!("{e}");
    }
    let excerpt = lab.excerpt(0, 2_000);
    let report = run(
        &excerpt,
        "at +0.6s changes:staged = {\"A\", \"C\"}\n",
        Toy::start(true),
    );
    assert!(!report.diverged(), "{report}");
    let terminal: Vec<&str> = report
        .requests
        .iter()
        .map(|r| r.terminal.rerun.as_str())
        .collect();
    assert_eq!(terminal, ["completed", "rejected: declined"]);
    assert_eq!(report.requests[0].verdict.rerun, "awaiting_confirmation");
}

#[test]
fn what_cannot_be_rerun_is_an_error_before_anything_runs() {
    let mut lab = Lab::new(Git::new(&["A", "C"], true, true));
    let seen = lab.observe(1_000);
    lab.act(1_500, &seen, agent(), "commit.submit", "submit", None);
    // a keyframe while the submit runs: an excerpt from here holds its
    // end and not its start.
    lab.rec.snapshot_full(1_550, &look(&lab.git));
    lab.land(1_600);
    let mut started = false;
    let cut = lab.excerpt(1_550, 2_000);
    let err = rerun(&cut, &[], || {
        started = true;
        Ok(Toy::new(&["A", "C"], true, true))
    });
    assert_eq!(
        err.err().as_deref(),
        Some(
            "request 1 (origin 0) started before the excerpt: its first stage here is \
             completed; export from a keyframe before it"
        )
    );
    let whole = lab.excerpt(0, 2_000);
    let err = rerun(
        &whole,
        &marks_of("\nafter request 9  changes:staged = {}"),
        || {
            started = true;
            Ok(Toy::new(&["A", "C"], true, true))
        },
    );
    assert_eq!(
        err.err().as_deref(),
        Some("mark on line 2: the excerpt holds no request 9")
    );
    assert!(!started, "the fixture was never built");
    let failing = rerun(&whole, &[], || Err::<Toy, _>("no fixture here".to_owned()));
    assert_eq!(failing.err().as_deref(), Some("no fixture here"));
}
