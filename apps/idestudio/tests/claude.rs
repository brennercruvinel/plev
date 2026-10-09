//! The Claude as the proposer of the ide's line (task-96), by the path the
//! showcase takes (the `propose::Llm` over a `ClaudeCli`, off the
//! workbench's thread), tested only with the repository's fake
//! (`crates/claude-model/tests/fake-claude`, which answers from files in
//! its directory and keeps every prompt it was given): no real `claude`
//! runs here.
//!
//! The rules stay the first choice. The Claude is asked only for a review
//! the rules read, and its prompt holds only what that review needs of
//! the repository: the messages and the files of the commits asked, and a
//! diff only when it is one of theirs. Never another commit, the working
//! tree, an author, the repository's path, or the line itself.

#![cfg(unix)]

// shared with the other ide tests: this binary uses part of the rig.
#[allow(dead_code)]
mod support;

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use accord::{Actor, Receipt, RequestId, SemId, Stage, Value};
use claude::ClaudeCli;
use gitdrv::GitEvent;
use idestudio::repository::eval_repo::{FIRST, REVIEWED, UNCOMMITTED};
use support::{Rig, completed};

/// A turn of the loop, as the window's wake takes it.
const TURN: f32 = 1.0 / 60.0;

/// A commit message the person is writing, which no review needs.
const DRAFT: &str = "a draft nobody sent yet";

/// The fake, answering from `dir`: `mode` and, for `ok`, `good`.
fn fake(dir: &Path, mode: &str, good: Option<&str>) -> ClaudeCli {
    std::fs::write(dir.join("mode"), mode).expect("the mode");
    if let Some(ir) = good {
        let result = serde_json::json!({
            "type": "result", "subtype": "success", "is_error": false,
            "num_turns": 1, "total_cost_usd": 0.05, "result": ir
        });
        std::fs::write(dir.join("good"), result.to_string()).expect("the answer");
    }
    let mut cli = ClaudeCli::new();
    cli.program =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../rust/claude/tests/fakes/claude");
    cli.dir = dir.to_owned();
    cli.timeout = Duration::from_secs(20);
    cli
}

/// The commits of the stacks, newest first.
fn commits(rig: &Rig) -> Vec<SemId> {
    let app = rig.bench.app_snapshot().expect("validates");
    app.children(&SemId::parse("stacks").unwrap())
        .map(|n| n.id.clone())
        .collect()
}

/// A review as a model would write it: each of `commits` with its message,
/// its files and its diff button, and the diff's lines.
fn review_ir(commits: &[SemId]) -> String {
    let cards: Vec<serde_json::Value> = commits
        .iter()
        .enumerate()
        .map(|(n, c)| {
            serde_json::json!({
                "sem": format!("review.commit_{n}"),
                "component": "card",
                "children": [
                    { "sem": format!("review.commit_{n}.message"), "component": "text",
                      "bind": { "state": { "text": format!("{c}:summary") } } },
                    { "sem": format!("review.commit_{n}.files"), "component": "table",
                      "label": "files", "bind": { "state": { "rows": format!("{c}:files") } } },
                    { "sem": format!("review.commit_{n}.diff"), "component": "button",
                      "label": "diff", "bind": { "actions": {
                          "activate": { "target": c.to_string(), "action": "show_diff" } } } }
                ]
            })
        })
        .collect();
    serde_json::json!({
        "plev_ir": 1,
        "surface": { "id": "review", "title": "the three commits", "root": {
            "sem": "review.root",
            "children": [
                { "sem": "review.commits", "children": cards },
                { "sem": "review.lines", "component": "table", "label": "diff",
                  "bind": { "state": { "rows": "diff:lines" } } }
            ]
        } }
    })
    .to_string()
}

/// The person chooses who proposes, writes `intent` and proposes, each
/// through the dispatcher; the proposal's request.
fn propose(rig: &mut Rig, proposer: &str, intent: &str) -> RequestId {
    let chose = rig.ask(
        Actor::Human,
        "ide_surface.proposer",
        "select",
        Some(proposer.into()),
    );
    assert!(completed(&rig.receipt(chose)).is_some(), "{proposer}");
    let wrote = rig.ask(
        Actor::Human,
        "ide_surface.intent",
        "set_value",
        Some(intent.into()),
    );
    assert!(completed(&rig.receipt(wrote)).is_some());
    rig.ask(Actor::Human, "ide_surface.propose", "propose", None)
}

/// Turns the workbench's wake until request `id` ends; its receipt.
fn settled(rig: &mut Rig, id: RequestId) -> Receipt {
    let started = Instant::now();
    while !rig.receipt(id).is_finished() {
        assert!(
            started.elapsed() < Duration::from_secs(20),
            "the answer came"
        );
        rig.bench.wake(TURN);
        std::thread::sleep(Duration::from_millis(10));
    }
    rig.receipt(id)
}

/// The prompts the fake was given, in order.
fn prompts(dir: &Path) -> Vec<String> {
    (1..)
        .map(|n| dir.join(format!("prompt.{n}")))
        .take_while(|p| p.exists())
        .map(|p| std::fs::read_to_string(p).expect("a prompt"))
        .collect()
}

#[test]
fn the_rules_come_first_and_the_claude_is_offered_only_when_there_is_one() {
    let dir = tempfile::tempdir().expect("a temp dir");
    let mut rig = Rig::review();
    rig.bench
        .host_surfaces_with(Some(fake(dir.path(), "ok", None)));
    let snap = rig.look();
    let proposer = SemId::parse("ide_surface.proposer").unwrap();
    let node = snap.get(&proposer).expect("the proposer");
    assert_eq!(
        node.constraints.options.as_deref(),
        Some(&["rules".to_owned(), "claude".to_owned()][..])
    );
    assert_eq!(
        rig.value("ide_surface.proposer:selected"),
        Value::Text("rules".into())
    );
    let mut alone = Rig::review();
    alone.bench.host_surfaces_with(None);
    let snap = alone.look();
    let node = snap.get(&proposer).expect("the proposer");
    assert_eq!(
        node.constraints.options.as_deref(),
        Some(&["rules".to_owned()][..])
    );
}

#[test]
fn the_claude_proposes_off_the_thread_over_only_the_commits_asked() {
    let dir = tempfile::tempdir().expect("a temp dir");
    let mut rig = Rig::review();
    let all = commits(&rig);
    let three = &all[..3];
    rig.bench
        .host_surfaces_with(Some(fake(dir.path(), "ok", Some(&review_ir(three)))));
    // the diff shows a file of the working tree when the person asks
    let note = format!("changes.file[{}]", UNCOMMITTED.0);
    rig.ask(Actor::Human, &note, "show_diff", None);
    rig.until(|e| matches!(e, GitEvent::DiffWorkdir { .. }));
    // and a commit message is being written
    rig.ask(Actor::Human, "commit.form", "open", None);
    let draft = Some(DRAFT.into());
    rig.ask(Actor::Human, "commit.message", "set_value", draft);

    let asked = propose(&mut rig, "claude", "review these three commits");
    assert_eq!(
        rig.receipt(asked).stage(),
        Some(&Stage::Executing),
        "off the thread"
    );
    assert_eq!(rig.value("ide_surface.status:busy"), Value::Bool(true));
    let receipt = settled(&mut rig, asked);
    let outcome = completed(&receipt).unwrap_or_else(|| panic!("{receipt:?}"));
    assert_eq!(receipt.request.actor, Actor::Human);
    assert_eq!(
        outcome.result.get("surface"),
        Some(&Value::Text("review".into()))
    );
    assert_eq!(outcome.result.get("attempts"), Some(&Value::Int(1)));
    assert_eq!(outcome.result.get("cost_usd"), Some(&Value::Num(0.05)));
    assert_eq!(
        rig.bench.surface_open().map(|s| s.to_string()),
        Some("review".into())
    );
    assert!(
        rig.look()
            .get(&SemId::parse("review.commit_2.files").unwrap())
            .is_some(),
        "the surface the Claude proposed is open"
    );

    // the prompt: the commits asked, and nothing else of the repository
    let seen = prompts(dir.path());
    assert_eq!(seen.len(), 1, "one call");
    let prompt = &seen[0];
    for (subject, files) in REVIEWED {
        assert!(prompt.contains(subject), "{subject}");
        for (path, _) in files {
            assert!(prompt.contains(path), "{path}");
        }
    }
    let repo = rig.dir.path().display().to_string();
    for never in [
        FIRST,
        all[3].as_str(),
        UNCOMMITTED.0,
        UNCOMMITTED.1.lines().next().unwrap(),
        "Lab Author",
        "lab@example.com",
        repo.as_str(),
        // the changed files and the index, not the word "changes", which
        // the schema's own example and the prompt's guidance use
        "changes:files",
        "changes:staged",
        DRAFT,
        "ide_surface",
    ] {
        assert!(
            !prompt.contains(never),
            "{never:?} is not the review's to see"
        );
    }

    // a diff of one of the commits asked is the review's: it goes along
    let second = format!("{}", three[1]);
    rig.ask(Actor::Human, &second, "show_diff", None);
    rig.until(|e| matches!(e, GitEvent::DiffCommit { .. }));
    let again = rig.ask(Actor::Human, "ide_surface.propose", "propose", None);
    assert!(completed(&settled(&mut rig, again)).is_some());
    // (the prompt shows each value's first 60 characters, as it does for
    // every app)
    let seen = prompts(dir.path());
    let lines = |prompt: &str| {
        prompt
            .lines()
            .find(|l| l.contains("state diff:lines"))
            .map(str::to_owned)
    };
    assert_eq!(
        lines(&seen[0]).as_deref(),
        Some("  state diff:lines: list = []"),
        "the working tree's diff was not the review's"
    );
    let asked = lines(&seen[1]).expect("the diff's lines");
    assert!(
        asked.contains("\"@@ -1,5 +1,6 @@\", \" pub struct Login {\""),
        "the diff of a commit asked goes along: {asked}"
    );
    // and the kind of each of its lines, under the same condition
    let kinds = |prompt: &str| {
        prompt
            .lines()
            .find(|l| l.contains("state diff:kinds"))
            .map(str::to_owned)
    };
    assert_eq!(
        kinds(&seen[0]).as_deref(),
        Some("  state diff:kinds: list = []")
    );
    let asked = kinds(&seen[1]).expect("the kinds");
    assert!(
        asked.contains("[\"header\", \"context\""),
        "the kinds of a commit asked go along: {asked}"
    );
    assert!(!seen[1].contains(UNCOMMITTED.0));
}

#[test]
fn an_intent_that_names_no_commit_is_never_sent_to_the_claude() {
    let dir = tempfile::tempdir().expect("a temp dir");
    let mut rig = Rig::review();
    rig.bench
        .host_surfaces_with(Some(fake(dir.path(), "ok", None)));
    for intent in ["the items", "zzz qqq", "review the files"] {
        let asked = propose(&mut rig, "claude", intent);
        let receipt = rig.receipt(asked);
        let Some(Stage::Failed { error }) = receipt.stage() else {
            panic!("{intent}: {receipt:?}");
        };
        assert!(
            error.starts_with("the Claude is shown only the commits a review names"),
            "{error}"
        );
        assert_eq!(
            rig.value("ide_surface.status:message"),
            Value::Text(error.clone())
        );
    }
    assert!(prompts(dir.path()).is_empty(), "no call at all");
    assert_eq!(rig.bench.surface_open(), None);
}

#[test]
fn a_cancel_ends_the_claudes_proposal_as_cancelled_by_the_person() {
    let dir = tempfile::tempdir().expect("a temp dir");
    let mut rig = Rig::review();
    rig.bench
        .host_surfaces_with(Some(fake(dir.path(), "slow", None)));
    let asked = propose(&mut rig, "claude", "review these three commits");
    assert_eq!(rig.receipt(asked).stage(), Some(&Stage::Executing));
    let cancel = rig.ask(Actor::Human, "ide_surface.cancel", "activate", None);
    assert!(completed(&rig.receipt(cancel)).is_some());
    let receipt = rig.receipt(asked);
    assert_eq!(
        receipt.stage(),
        Some(&Stage::Failed {
            error: "cancelled by human".into()
        })
    );
    assert_eq!(rig.value("ide_surface.status:busy"), Value::Bool(false));
    assert_eq!(rig.bench.surface_open(), None);
}
