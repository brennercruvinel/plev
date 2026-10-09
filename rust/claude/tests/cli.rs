//! `ClaudeCli` against a fake claude (`tests/fake-claude`, a shell script),
//! never the real one: no test here reaches the network. over the todo's
//! golden projection, through `offers::Llm` and `propose_verified`: a
//! good answer is a verified surface, a refused one is repaired by the
//! second call, a child past the timeout is killed, a call cancelled from
//! another thread kills the child too, a grandchild that holds the pipes
//! does not hold the call, a program that fails or runs out of budget is
//! the error at once, and a program that is not there is an error too.

#![cfg(unix)]

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use accord::Snapshot;
use claude::{ClaudeCli, SESSION_VARS};
use offers::{Ask, Llm, ProposeError, prompt, propose_verified};
use plevir::{Capabilities, GenerationPolicy, VerifiedIr, verify_json};
use serde_json::json;
use tempfile::TempDir;

const INTENT: &str = "only what is active and the add field";

fn manifest(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn read(rel: &str) -> String {
    let path = manifest(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// the compact surface of the Surface page (the task-68 fixture).
fn compact() -> String {
    read("../plevir/tests/fixtures/uigallery-todo.ir.json")
}

/// what the proposer is asked about the todo projected, with the compact
/// surface shown.
struct Todo {
    app: Snapshot,
    caps: Capabilities,
    policy: GenerationPolicy,
    current: VerifiedIr,
}

impl Todo {
    fn new() -> Todo {
        let app: Snapshot = serde_json::from_str(&read(
            "../plevir/tests/fixtures/uigallery-todo.snapshot.json",
        ))
        .unwrap();
        let caps = Capabilities::from_snapshot(&app);
        let policy = GenerationPolicy::default();
        let current = verify_json(&compact(), &caps, &policy).unwrap();
        Todo {
            app,
            caps,
            policy,
            current,
        }
    }

    fn ask(&self) -> Ask<'_> {
        Ask {
            intent: INTENT,
            caps: &self.caps,
            app: &self.app,
            current: Some(&self.current),
            policy: &self.policy,
        }
    }
}

/// `reply` as `claude -p --output-format json` writes it.
fn result(reply: &str) -> String {
    json!({
        "type": "result", "subtype": "success", "is_error": false,
        "num_turns": 1, "total_cost_usd": 0.05, "result": reply
    })
    .to_string()
}

/// a directory where the fake answers in `mode`, and a `ClaudeCli` that
/// runs the fake there.
fn fake(mode: &str) -> (TempDir, ClaudeCli) {
    let dir = tempfile::tempdir().unwrap();
    let wrong = compact().replace("todo.counter:active", "todo.counter:gone");
    let over = json!({
        "type": "result", "subtype": "error_max_budget_usd", "is_error": true,
        "num_turns": 1, "total_cost_usd": 0.00133,
        "errors": ["Reached maximum budget ($0.0001)"]
    });
    for (name, text) in [
        ("mode", mode.to_owned()),
        ("good", result(&compact())),
        ("bad", result(&wrong)),
        ("over", over.to_string()),
    ] {
        std::fs::write(dir.path().join(name), text).unwrap();
    }
    let mut cli = ClaudeCli::new();
    cli.program = manifest("tests/fakes/claude");
    cli.dir = dir.path().to_owned();
    (dir, cli)
}

fn left(dir: &TempDir, name: &str) -> String {
    std::fs::read_to_string(dir.path().join(name)).unwrap_or_default()
}

fn alive(pid: &str) -> bool {
    Command::new("kill")
        .args(["-0", pid])
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

fn model_error(e: ProposeError) -> String {
    match e {
        ProposeError::Model(text) => text,
        other => panic!("a model error: {other:?}"),
    }
}

/// the whole prompt goes by stdin (17 KB, more than a pipe holds), the
/// child gets exactly `args`, none of the session's variables and the
/// rest of the environment, and its answer comes out verified.
#[test]
fn a_good_answer_is_a_verified_surface() {
    let todo = Todo::new();
    let (dir, mut cli) = fake("ok");
    let ir = propose_verified(&mut Llm::new(&mut cli), &todo.ask()).expect("verifies");
    assert_eq!(ir.ir().surface.id.as_str(), "todo_panel");

    let sent = prompt(&todo.ask());
    assert!(sent.len() > 16 * 1024);
    assert_eq!(left(&dir, "prompt.1"), sent);
    assert!(!dir.path().join("prompt.2").exists());
    let args: Vec<String> = left(&dir, "args").lines().map(str::to_owned).collect();
    assert_eq!(args, cli.args());
    let env = left(&dir, "env");
    for var in SESSION_VARS {
        assert!(
            !env.lines().any(|l| l.starts_with(&format!("{var}="))),
            "{var}"
        );
    }
    assert!(env.lines().any(|l| l.starts_with("PATH=")));

    let calls = cli.calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].reply, Ok(compact().len()));
    assert_eq!(calls[0].turns, Some(1));
    assert_eq!(calls[0].cost_usd, Some(0.05));
    assert!((cli.spent_usd() - 0.05).abs() < 1e-12);
}

/// the first answer binds a state the todo does not have; the second
/// call carries it and the verifier's errors, and its answer verifies.
#[test]
fn a_refused_answer_is_repaired_by_the_second_call() {
    let todo = Todo::new();
    let (dir, mut cli) = fake("repair");
    let ir = propose_verified(&mut Llm::new(&mut cli), &todo.ask()).expect("verifies");
    assert_eq!(ir.ir().surface.id.as_str(), "todo_panel");
    assert_eq!(cli.calls().len(), 2);
    let (first, second) = (left(&dir, "prompt.1"), left(&dir, "prompt.2"));
    assert!(!first.contains("was refused"));
    assert!(second.starts_with(&first));
    assert!(second.contains("## attempt 1 was refused"));
    assert!(second.contains("todo.counter:gone"));
    assert!(cli.calls().iter().all(|c| c.reply.is_ok()));
}

/// a child that sleeps past the timeout is killed by its pid, and the
/// proposal stops with that error, long before the child would have
/// answered.
#[test]
fn a_child_past_the_timeout_is_killed() {
    let todo = Todo::new();
    let (dir, mut cli) = fake("slow");
    cli.timeout = Duration::from_secs(1);
    let started = Instant::now();
    let e = propose_verified(&mut Llm::new(&mut cli), &todo.ask()).unwrap_err();
    let took = started.elapsed();
    let text = model_error(e);
    let pid = left(&dir, "pid").trim().to_owned();
    assert!(!pid.is_empty(), "the child wrote its pid");
    assert!(
        text.ends_with(&format!("did not answer in 1 s: killed (pid {pid})")),
        "{text}"
    );
    assert!(!alive(&pid), "the child {pid} is dead");
    assert!(
        took >= Duration::from_secs(1) && took < Duration::from_secs(10),
        "{took:?}"
    );
    assert_eq!(
        cli.calls().len(),
        1,
        "a model that fails is not asked again"
    );
    assert!(cli.calls()[0].reply.is_err());
}

/// a call cancelled from another thread while the child sleeps (the
/// page's cancel button): the child is killed by its pid, the proposal
/// stops with that error long before the deadline, the signal stays
/// raised so the next call does not even start, and a fresh signal
/// lets a new call run.
#[test]
fn a_call_cancelled_from_another_thread_kills_the_child() {
    let todo = Todo::new();
    let (dir, mut cli) = fake("slow");
    cli.timeout = Duration::from_secs(30);
    let cancel = cli.cancel.clone();
    let started = Instant::now();
    let worker = thread::spawn(move || {
        let todo = Todo::new();
        let e = propose_verified(&mut Llm::new(&mut cli), &todo.ask()).unwrap_err();
        (e, cli)
    });
    let pid = loop {
        let pid = left(&dir, "pid").trim().to_owned();
        if !pid.is_empty() {
            break pid;
        }
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "the child started"
        );
        thread::sleep(Duration::from_millis(10));
    };
    assert!(alive(&pid), "the child {pid} sleeps");
    cancel.cancel();
    let (e, mut cli) = worker.join().unwrap();
    let took = started.elapsed();
    let text = model_error(e);
    assert!(
        text.ends_with(&format!("was cancelled: killed (pid {pid})")),
        "{text}"
    );
    assert!(!alive(&pid), "the child {pid} is dead");
    assert!(took < Duration::from_secs(10), "{took:?}");
    assert_eq!(cli.calls().len(), 1);
    assert!(cli.calls()[0].reply.is_err());

    // raised, the signal stops the next call before it starts
    let e = propose_verified(&mut Llm::new(&mut cli), &todo.ask()).unwrap_err();
    assert!(
        model_error(e).ends_with("was cancelled before it started"),
        "not started"
    );
    assert!(!dir.path().join("prompt.2").exists());
    assert_eq!(cli.calls().len(), 2);

    // a new signal, a new call
    cli.cancel = claude::Cancel::new();
    std::fs::write(dir.path().join("mode"), "ok").unwrap();
    let ir = propose_verified(&mut Llm::new(&mut cli), &todo.ask()).expect("verifies");
    assert_eq!(ir.ir().surface.id.as_str(), "todo_panel");
    assert!(dir.path().join("prompt.2").exists());
}

/// a child that answers and exits while a grandchild keeps its stdout and
/// stderr open: the answer is kept and the call ends a grace second after
/// the exit, not when the grandchild does.
#[test]
fn a_grandchild_holding_the_pipes_does_not_hold_the_call() {
    let todo = Todo::new();
    let (dir, mut cli) = fake("orphan");
    let started = Instant::now();
    let ir = propose_verified(&mut Llm::new(&mut cli), &todo.ask());
    let took = started.elapsed();
    let pid = left(&dir, "pid").trim().to_owned();
    if !pid.is_empty() {
        let _ = Command::new("kill").arg(&pid).status();
    }
    assert_eq!(ir.expect("verifies").ir().surface.id.as_str(), "todo_panel");
    assert!(took < Duration::from_secs(4), "{took:?}");
}

/// a program that exits with a failure: its stderr is the error, at once.
#[test]
fn a_failing_program_is_the_error_at_once() {
    let todo = Todo::new();
    let (_dir, mut cli) = fake("fail");
    let e = propose_verified(&mut Llm::new(&mut cli), &todo.ask()).unwrap_err();
    let text = model_error(e);
    assert!(
        text.ends_with("failed (exit status: 1): Not logged in. Please run /login"),
        "{text}"
    );
    assert_eq!(cli.calls().len(), 1);
}

/// an exceeded budget, as the probe saw it: exit 1, the reason in the
/// json result on stdout, and what the call cost.
#[test]
fn an_exceeded_budget_is_the_error_with_its_cost() {
    let todo = Todo::new();
    let (_dir, mut cli) = fake("budget");
    let e = propose_verified(&mut Llm::new(&mut cli), &todo.ask()).unwrap_err();
    let text = model_error(e);
    assert!(text.contains("error_max_budget_usd"), "{text}");
    assert!(text.ends_with("Reached maximum budget ($0.0001)"), "{text}");
    assert_eq!(cli.calls()[0].cost_usd, Some(0.00133));
}

#[test]
fn a_program_that_is_not_there_is_the_error() {
    let todo = Todo::new();
    let (_dir, mut cli) = fake("ok");
    cli.program = manifest("tests/no-such-claude");
    let e = propose_verified(&mut Llm::new(&mut cli), &todo.ask()).unwrap_err();
    let text = model_error(e);
    assert!(text.starts_with("could not start "), "{text}");
    assert!(text.contains("no-such-claude"), "{text}");
}
