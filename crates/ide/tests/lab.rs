//! the laboratory of the experiment, run as the orchestrator runs it
//! (task-62 phase c): the `ide-lab` binary against a real repository, a
//! real agent port and the real git worker, driven by its own scripted
//! agent, with and without the person of the script.
//!
//! what it holds the laboratory to: the refusal names exactly the file
//! the person took out of the index, HEAD does not move while it is
//! refused, the agent recovers, the commit is the task's, the session
//! file opens and its receipt matches git, and the recorded state shows
//! the file leaving `changes:staged`. the visual track needs a GPU, so
//! the assertions about frames are skipped with a message on a machine
//! without one, as the engine's headless tests do, and never fail.

#![cfg(unix)]

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

use rec::Session;
use sem::{SemId, StateRef, Value};
use serde_json::Value as Json;

/// how long one run may take before the test fails it.
const TIMEOUT_S: &str = "120";

/// what a run left behind: its exit code and its report.
struct Run {
    code: i32,
    report: Json,
    out: tempfile::TempDir,
}

impl Run {
    /// the value at a dotted path of the report.
    fn at(&self, path: &str) -> &Json {
        let mut node = &self.report;
        for key in path.split('.') {
            node = node.get(key).unwrap_or_else(|| {
                panic!("the report has no {path} (stopped at {key}): {}", self.report)
            });
        }
        node
    }

    fn u64(&self, path: &str) -> u64 {
        self.at(path)
            .as_u64()
            .unwrap_or_else(|| panic!("{path} is not a number: {}", self.at(path)))
    }

    fn bool(&self, path: &str) -> bool {
        self.at(path)
            .as_bool()
            .unwrap_or_else(|| panic!("{path} is not a bool: {}", self.at(path)))
    }

    fn str(&self, path: &str) -> &str {
        self.at(path)
            .as_str()
            .unwrap_or_else(|| panic!("{path} is not a string: {}", self.at(path)))
    }

    fn list(&self, path: &str) -> &Vec<Json> {
        self.at(path)
            .as_array()
            .unwrap_or_else(|| panic!("{path} is not a list: {}", self.at(path)))
    }

    /// the strings of a list of strings.
    fn strings(&self, path: &str) -> Vec<&str> {
        self.list(path)
            .iter()
            .map(|v| v.as_str().unwrap_or_default())
            .collect()
    }

    fn session(&self) -> Session {
        let file = self.out.path().join("session.monster");
        Session::open(&file).unwrap_or_else(|e| panic!("{}: {e}", file.display()))
    }
}

/// runs `ide-lab --arm scripted` with `human`, in its own output
/// directory and under its own app name.
fn scripted(app: &str, human: &str) -> Run {
    let out = tempfile::tempdir().expect("a temp dir for the run");
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ide-lab"));
    cmd.args(["--arm", "scripted", "--human", human]);
    cmd.args(["--app", app, "--timeout", TIMEOUT_S]);
    cmd.arg("--out").arg(out.path());
    let output = cmd.output().expect("ide-lab runs");
    let code = output.status.code().unwrap_or(-1);
    let report = read_report(out.path(), &output.stdout, &output.stderr);
    Run { code, report, out }
}

fn read_report(out: &Path, stdout: &[u8], stderr: &[u8]) -> Json {
    let path = out.join("report.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "{}: {e}\nstdout:\n{}\nstderr:\n{}",
            path.display(),
            String::from_utf8_lossy(stdout),
            String::from_utf8_lossy(stderr)
        )
    });
    serde_json::from_str(&text).expect("the report is json")
}

/// the staged set a recorded state holds.
fn staged_at(session: &Session, t: u64) -> BTreeSet<String> {
    let at = StateRef::parse("changes:staged").expect("a state ref");
    let state = session
        .state_at(t)
        .expect("the recorded state reads back")
        .unwrap_or_else(|| panic!("no state recorded at {t} ms"));
    match state.value(&at) {
        Some(Value::Set(staged)) => staged.clone(),
        other => panic!("changes:staged at {t} ms is {other:?}"),
    }
}

/// the task's three files, as the report and git spell them.
fn task_files() -> Vec<&'static str> {
    vec!["src/login.rs", "src/session.rs", "tests/login_test.rs"]
}

#[test]
fn the_person_makes_the_agents_submit_stale_and_the_agent_recovers() {
    let run = scripted("ide-lab-stale", "unstage-after-observe:src/session.rs");
    assert_eq!(run.code, 0, "the report: {}", run.report);

    // the person acted once, on the file the scenario names, and the
    // observation the agent had just been served was the one it acted on.
    assert_eq!(run.str("intervention.path"), "src/session.rs");
    assert!(run.bool("intervention.revision_matched"));
    let before: Vec<&str> = run.strings("intervention.staged_before");
    let after: Vec<&str> = run.strings("intervention.staged_after");
    assert_eq!(before, task_files());
    assert_eq!(after, ["src/login.rs", "tests/login_test.rs"]);

    // one refusal, and it names exactly that file leaving changes:staged.
    let stale = run.list("stale");
    assert_eq!(stale.len(), 1, "one stale rejection: {stale:?}");
    let change = &stale[0]["changes"][0]["state_changed"];
    assert_eq!(
        change["at"].as_str(),
        Some("changes:staged"),
        "the delta is on the staged set: {change}"
    );
    assert_eq!(
        change["removed"].as_array().map(Vec::as_slice),
        Some([Json::from("src/session.rs")].as_slice()),
        "the delta names the file: {change}"
    );
    assert!(
        change["added"].as_array().is_some_and(Vec::is_empty),
        "nothing was added: {change}"
    );

    // the refusal committed nothing: HEAD is the one the run started
    // with, at the intervention and at the refusal alike.
    let head_before = run.str("truth.head_before").to_owned();
    assert_eq!(run.str("intervention.head"), head_before);
    assert_eq!(stale[0]["head"].as_str(), Some(head_before.as_str()));

    // the agent observed again, put the file back and committed the task.
    assert!(run.bool("agent.recovered"), "the agent recovered");
    assert!(
        run.bool("agent.stale_measurement_valid"),
        "the submit rested on a revision served before the intervention: {}",
        run.str("agent.stale_measurement")
    );
    assert_eq!(run.str("truth.verdict"), "MATCH");
    assert_eq!(run.strings("truth.head_files"), task_files());
    assert!(run.bool("truth.files_are_the_task"));
    assert_ne!(run.str("truth.head_after"), head_before, "a commit exists");
    assert_eq!(run.str("truth.head_subject"), "fix login session timeout");
    // the session says the same thing git says.
    assert_eq!(run.str("truth.session_commit"), run.str("truth.head_after"));
    assert_eq!(run.strings("truth.session_files"), task_files());

    // the port carried it: more lines out than in, and bytes both ways.
    let (msgs_in, msgs_out) = (run.u64("traffic.msgs_in"), run.u64("traffic.msgs_out"));
    assert!(msgs_out >= msgs_in, "{msgs_out} out, {msgs_in} in");
    assert!(run.u64("traffic.bytes_in") > 0 && run.u64("traffic.bytes_out") > 0);
    assert_eq!(run.u64("traffic.conns"), 1);
    assert_eq!(run.u64("traffic.rejected_stale"), 1);
    assert!(run.u64("traffic.acts") >= 6, "every step is an act");
    assert!(run.u64("traffic.observes_snapshot") > 0);
    assert_eq!(run.u64("traffic.errors"), 0);

    // the session file the laboratory wrote opens again here, and its
    // state at each instant tells the same story.
    let session = run.session();
    let submit = SemId::parse("commit.submit").expect("an id");
    let end = session.span().expect("a span").1;
    let receipts = session.receipts_for(&submit, end);
    assert!(
        receipts.len() >= 2,
        "the refused submit and the one that committed: {}",
        receipts.len()
    );
    let t = run.u64("intervention.t");
    let recorded = run.u64("intervention_recorded_at");
    assert!(staged_at(&session, t - 1).contains("src/session.rs"));
    assert!(!staged_at(&session, recorded).contains("src/session.rs"));

    if run.bool("cost.recording.gpu") {
        assert!(
            run.u64("session.frames") > 0,
            "the visual track holds frames: {}",
            run.at("session")
        );
        assert!(run.u64("session.chunks_bytes") > 0);
    } else {
        eprintln!("no GPU adapter; skipping the visual track of the session");
    }
}

#[test]
fn without_the_person_the_agent_commits_the_task_with_no_refusal() {
    let run = scripted("ide-lab-control", "none");
    assert_eq!(run.code, 0, "the report: {}", run.report);

    assert!(run.at("intervention").is_null(), "nobody intervened");
    assert!(run.list("stale").is_empty(), "no refusal");
    assert_eq!(run.u64("traffic.rejected_stale"), 0);
    assert!(!run.bool("agent.recovered"));
    assert!(
        !run.bool("agent.stale_measurement_valid"),
        "with no intervention the stale check was not exercised, and the report says so"
    );
    assert_eq!(run.str("truth.verdict"), "MATCH");
    assert_eq!(run.strings("truth.head_files"), task_files());
    assert!(run.bool("truth.files_are_the_task"));

    // every act of the agent rests on a revision this port served.
    for act in run.list("agent.acts") {
        assert_eq!(
            act["form"].as_str(),
            Some("from_revision"),
            "an act rests on what it observed: {act}"
        );
    }
    let session = run.session();
    let submit = SemId::parse("commit.submit").expect("an id");
    let end = session.span().expect("a span").1;
    let done = session.receipts_for(&submit, end);
    assert_eq!(done.len(), 1, "one submit, completed");
    assert_eq!(
        done[0].receipt.stage().map(sem::Stage::name),
        Some("completed")
    );
}
