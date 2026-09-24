//! the laboratory of the experiment, run as the orchestrator runs it
//! (task-62 phase c): the `ide-lab` binary against a real repository, a
//! real agent port and the real git worker, driven by its own scripted
//! agent, with and without the person of the script. the agent comes to
//! its submit in two ways (it observes right before it, or it chains it on
//! the revision the last stage carried, `--scripted chain`), and the person
//! acts right after an observation or right before a submit reaches the
//! dispatcher: every combination commits the task, and the stale
//! measurement says, by the value the submit carried and the value the app
//! held, whether the person made the check work.
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
                panic!(
                    "the report has no {path} (stopped at {key}): {}",
                    self.report
                )
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
    lab(app, human, &[])
}

/// the same, with the scripted agent that chains its first submit.
fn chained(app: &str, human: &str) -> Run {
    lab(app, human, &["--scripted", "chain"])
}

fn lab(app: &str, human: &str, more: &[&str]) -> Run {
    let out = tempfile::tempdir().expect("a temp dir for the run");
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ide-lab"));
    cmd.args(["--arm", "scripted", "--human", human]);
    cmd.args(more);
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

/// every act of the agent rests on a revision this port served: one an
/// observation served, or one the last stage of an earlier act carried
/// (task-87; the script writes it after each act, "the app at <rev>").
/// the report's `form` tells only the first kind, since the laboratory
/// hears observations and not stages. returns how many acts rest on a
/// revision no observation served.
fn every_act_rests_on_a_served_revision(run: &Run) -> usize {
    let observed: BTreeSet<&str> = run
        .list("agent.observations")
        .iter()
        .filter_map(|o| o["revision"].as_str())
        .collect();
    let carried: BTreeSet<&str> = run
        .strings("agent.script")
        .into_iter()
        .filter_map(|step| step.rsplit_once(", the app at ").map(|(_, r)| r))
        .collect();
    let mut chained = 0;
    for act in run.list("agent.acts") {
        let revision = act["revision"].as_str().unwrap_or_default();
        let seen = observed.contains(revision);
        assert!(
            seen || carried.contains(revision),
            "an act rests on a revision the port served: {act}"
        );
        chained += usize::from(!seen);
    }
    chained
}

/// the scripted agent confirmed its commit on the observation right after
/// the receipt: the node of the sha git says holds the task's files.
fn confirmed_by_the_node(run: &Run) {
    let node = format!("stacks.commit[{}]", run.str("truth.head_after"));
    let files = format!("{:?}", task_files());
    let script = run.strings("agent.script");
    assert!(
        script
            .iter()
            .any(|step| step
                .starts_with(&format!("stacks: {node} holds the receipt's files {files}"))),
        "the agent confirmed its commit by the node's files: {script:?}"
    );
    let notes = run.strings("notes");
    assert!(
        !notes.iter().any(|n| n.starts_with("the scripted agent")),
        "the scripted agent ended clean: {notes:?}"
    );
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
    // and it names who moved it: the person's unstage of that file, the
    // request the intervention made, and nobody else.
    let acted = stale[0]["acted"].as_array().expect("who acted");
    assert_eq!(acted.len(), 1, "one actor named: {acted:?}");
    assert_eq!(acted[0]["actor"].as_str(), Some("human"));
    assert_eq!(acted[0]["action"].as_str(), Some("unstage"));
    assert_eq!(
        acted[0]["target"].as_str(),
        Some("changes.file[src/session.rs]")
    );
    assert_eq!(
        acted[0]["request"].as_str(),
        Some(run.str("intervention.request"))
    );

    // the refusal committed nothing: HEAD is the one the run started
    // with, at the intervention and at the refusal alike.
    let head_before = run.str("truth.head_before").to_owned();
    assert_eq!(run.str("intervention.head"), head_before);
    assert_eq!(stale[0]["head"].as_str(), Some(head_before.as_str()));

    // the agent observed again, put the file back and committed the task.
    assert!(run.bool("agent.recovered"), "the agent recovered");
    assert!(every_act_rests_on_a_served_revision(&run) > 0);
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
    confirmed_by_the_node(&run);

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
    confirmed_by_the_node(&run);

    // every act of the agent rests on a revision this port served, and
    // the ones with nothing to decide before them rest on the revision the
    // act before carried, with no observation only to learn it.
    assert!(
        every_act_rests_on_a_served_revision(&run) > 0,
        "the agent chained acts on the revision a last stage carried: {:?}",
        run.strings("agent.script")
    );
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

/// every run ends the same way: exit 0, the commit is the task's, the
/// session says what git says, the agent confirmed it by the node, and no
/// act was refused before the dispatcher.
fn commits_the_task(run: &Run) {
    assert_eq!(run.code, 0, "the report: {}", run.report);
    assert_eq!(run.str("truth.verdict"), "MATCH");
    assert_eq!(run.strings("truth.head_files"), task_files());
    assert!(run.bool("truth.files_are_the_task"));
    assert_eq!(run.str("truth.session_commit"), run.str("truth.head_after"));
    confirmed_by_the_node(run);
    assert_eq!(run.u64("agent.protocol_refusals.before_dispatch"), 0);
    assert_eq!(run.u64("agent.protocol_refusals.not_observed"), 0);
    assert_eq!(run.u64("agent.protocol_refusals.unknown_revision"), 0);
}

/// the states each stale refusal names, in order.
fn stale_on(run: &Run, index: usize) -> Vec<String> {
    run.list("stale")[index]["changes"]
        .as_array()
        .expect("the changes")
        .iter()
        .map(|c| {
            c["state_changed"]["at"]
                .as_str()
                .unwrap_or_default()
                .to_owned()
        })
        .collect()
}

/// the actors a stale refusal names, as `actor action target`.
fn acted_on(run: &Run, index: usize) -> Vec<String> {
    run.list("stale")[index]["acted"]
        .as_array()
        .expect("who acted")
        .iter()
        .map(|a| {
            format!(
                "{} {} {}",
                a["actor"].as_str().unwrap_or_default(),
                a["action"].as_str().unwrap_or_default(),
                a["target"].as_str().unwrap_or_default()
            )
        })
        .collect()
}

/// the request ids of the agent's submits, in the order they reached the
/// dispatcher.
fn submits(run: &Run) -> Vec<u64> {
    run.list("agent.acts")
        .iter()
        .filter(|a| a["action"] == "submit")
        .map(|a| {
            a["request"]
                .as_str()
                .and_then(|r| r.parse().ok())
                .expect("a request id")
        })
        .collect()
}

/// the stale check was measured by value: the submit that came after the
/// person carried the three files while the app held two.
fn measured_on_the_old_world(run: &Run) {
    assert!(
        run.bool("agent.stale_measurement_valid"),
        "{}",
        run.str("agent.stale_measurement")
    );
    assert_eq!(run.strings("agent.stale_measured.seen"), task_files());
    assert_eq!(
        run.strings("agent.stale_measured.live"),
        ["src/login.rs", "tests/login_test.rs"]
    );
    assert_eq!(
        run.str("agent.stale_measured.request"),
        run.list("stale")[0]["request"].as_str().unwrap_or_default(),
        "the measured submit is the refused one"
    );
}

#[test]
fn the_person_before_the_submit_makes_the_observed_submit_stale() {
    let run = scripted("ide-lab-before", "unstage-before-submit:src/session.rs");
    commits_the_task(&run);

    assert_eq!(run.str("intervention.trigger"), "submit arriving");
    assert_eq!(run.str("intervention.path"), "src/session.rs");
    assert_eq!(run.strings("intervention.staged_before"), task_files());
    assert_eq!(
        run.strings("intervention.staged_after"),
        ["src/login.rs", "tests/login_test.rs"]
    );
    // the submit that brought the person is the next request, and it is
    // refused on the staged set she moved, naming her.
    let person: u64 = run.str("intervention.request").parse().expect("an id");
    assert_eq!(submits(&run).first(), Some(&(person + 1)));
    assert_eq!(run.list("stale").len(), 1);
    assert_eq!(stale_on(&run, 0), ["changes:staged"]);
    assert_eq!(
        acted_on(&run, 0),
        ["human unstage changes.file[src/session.rs]"]
    );
    assert_eq!(
        run.list("stale")[0]["head"].as_str(),
        Some(run.str("truth.head_before"))
    );
    measured_on_the_old_world(&run);
    assert!(run.bool("agent.recovered"));
}

#[test]
fn the_chained_submit_is_refused_by_the_agents_own_write() {
    // a terminal stage shows no state (task-89): the first submit rests on
    // the message the agent was last shown, empty, and the agent wrote it
    // since. nobody else acted, so the refusal names nobody.
    let run = chained("ide-lab-chain", "none");
    commits_the_task(&run);

    assert!(run.at("intervention").is_null());
    assert_eq!(run.list("stale").len(), 1);
    assert_eq!(stale_on(&run, 0), ["commit.message:text"]);
    assert!(acted_on(&run, 0).is_empty(), "{:?}", acted_on(&run, 0));
    assert!(run.bool("agent.recovered"));
    assert!(!run.bool("agent.stale_measurement_valid"));
    assert!(
        run.str("agent.stale_measurement")
            .starts_with("no intervention")
    );
    assert_eq!(submits(&run).len(), 2);
}

#[test]
fn after_the_look_the_person_comes_too_late_for_the_chained_submit() {
    // the expected result, not a failure: the agent never looks between
    // writing the message and its first submit, so the person, who acts
    // right after a look, acts on the look of its recovery. the agent then
    // sees the file out, puts it back and commits: the check was not
    // exercised by the person, and the measurement says why.
    let run = chained("ide-lab-chain-late", "unstage-after-observe:src/session.rs");
    commits_the_task(&run);

    assert_eq!(run.str("intervention.trigger"), "observation served");
    let person: u64 = run.str("intervention.request").parse().expect("an id");
    let submits = submits(&run);
    assert!(
        submits[0] < person && submits[1..].iter().all(|s| *s > person),
        "the person came between the first submit and the next: {submits:?}, person {person}"
    );
    assert_eq!(run.list("stale").len(), 1);
    assert_eq!(stale_on(&run, 0), ["commit.message:text"]);
    assert!(acted_on(&run, 0).is_empty());

    assert!(!run.bool("agent.stale_measurement_valid"));
    let text = run.str("agent.stale_measurement");
    assert!(
        text.contains("the world was put back") && text.contains("too late"),
        "{text}"
    );
    // what the submit after her carried is what the app held again.
    assert_eq!(run.strings("agent.stale_measured.seen"), task_files());
    assert_eq!(run.strings("agent.stale_measured.live"), task_files());
}

#[test]
fn before_the_submit_the_person_makes_the_chained_submit_stale() {
    let run = chained(
        "ide-lab-chain-before",
        "unstage-before-submit:src/session.rs",
    );
    commits_the_task(&run);

    assert_eq!(run.str("intervention.trigger"), "submit arriving");
    assert_eq!(run.list("stale").len(), 1);
    // the person's unstage and the agent's own write, in one refusal that
    // names the person.
    assert_eq!(stale_on(&run, 0), ["changes:staged", "commit.message:text"]);
    assert_eq!(
        acted_on(&run, 0),
        ["human unstage changes.file[src/session.rs]"]
    );
    measured_on_the_old_world(&run);
    // measured whatever revision it named: the refused submit rested on a
    // revision a terminal stage carried, which no observation served.
    let refused = run.list("stale")[0]["request"].as_str().unwrap_or_default();
    let act = run
        .list("agent.acts")
        .iter()
        .find(|a| a["request"].as_str() == Some(refused))
        .expect("the refused submit");
    assert!(act["revision_served_index"].is_null(), "{act}");
    assert!(run.bool("agent.recovered"));
}
