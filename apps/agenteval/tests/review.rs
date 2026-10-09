//! The demo 5 of task-80 (task-96) as the orchestrator runs it: the
//! `ide-lab --arm review` binary, with no window, on the laboratory's
//! repository of the review and the real git worker. A person reviews the
//! three newest commits with the fixed panel, then with the surface her
//! intent generates, and closes it; the run is recorded, and the session
//! file is read back here the way `plev-inspect` reads it.
//!
//! What it holds the demo to, the criterion of task-80: the acts of each
//! side; the files of the three at once on the surface, and each commit's
//! message whole on the fixed panel, which keeps the window's width with
//! no surface open (task-96, phase b); every act of the surface a request
//! of the person, completed, through the dispatcher, recorded as the
//! capability it stands for; the surface in the recorded state only
//! between its proposal and its close, every node of it from the
//! materializer; and nothing of it after the close: no node, no binding,
//! no surface in the host.

#![cfg(unix)]

use std::path::Path;
use std::process::Command;

use replay::Inspector;
use record::{InputKind, Kind, Session};
use accord::{Actor, SemId};
use serde_json::Value as Json;

/// What the person writes, as the laboratory writes it.
const INTENT: &str = "review these three commits";

fn run(out: &Path) -> Json {
    let output = Command::new(env!("CARGO_BIN_EXE_ide-lab"))
        .args(["--arm", "review", "--out"])
        .arg(out)
        .output()
        .expect("the laboratory runs");
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let text = std::fs::read_to_string(out.join("report.json")).expect("the report");
    serde_json::from_str(&text).expect("json")
}

fn n(v: &Json) -> u64 {
    v.as_u64().unwrap_or_else(|| panic!("a number: {v}"))
}

#[test]
fn the_demo_reviews_three_commits_with_the_fixed_panel_and_with_the_surface() {
    let out = tempfile::tempdir().expect("a temp dir");
    let report = run(out.path());

    // the acts of each side
    assert_eq!(n(&report["fixed"]["acts"]["clicks"]), 3);
    assert_eq!(n(&report["fixed"]["acts"]["keys"]), 0);
    assert_eq!(
        n(&report["surface"]["acts"]["clicks"]),
        5,
        "field, propose, three diffs"
    );
    assert_eq!(
        n(&report["surface"]["acts"]["keys"]),
        INTENT.chars().count() as u64
    );

    // what shows without scrolling
    assert_eq!(
        n(&report["surface"]["files_at_once"]),
        3,
        "the three commits' files at once"
    );
    assert_eq!(n(&report["surface"]["cards_whole"]), 3);
    assert!(
        n(&report["fixed"]["files_at_once"]) <= 1,
        "one diff at a time"
    );
    assert_eq!(
        n(&report["fixed"]["summaries_cut_at_the_width"]),
        0,
        "no surface open, the stacks keep the window's width: every message whole"
    );
    for side in ["fixed", "surface"] {
        let commits = report[side]["commits"].as_array().expect("commits");
        assert_eq!(commits.len(), 3, "{side}");
        for c in commits {
            assert!(n(&c["diff_lines_on_screen"]) > 0, "{side}: {c}");
            assert!(
                n(&c["diff_lines_on_screen"]) <= n(&c["diff_lines"]),
                "{side}: {c}"
            );
            assert!(
                n(&c["diff_lines_cut_at_the_width"]) <= n(&c["diff_lines_on_screen"]),
                "{side}: {c}"
            );
        }
    }
    // the surface's diff is a code with the kind of each line (task-97)
    let shown = report["surface"]["commits"].as_array().expect("commits");
    assert!(
        shown.iter().all(|c| c["kinds_beside_the_lines"] == true),
        "{shown:?}"
    );

    // every act, a request of the person through the dispatcher
    let requests = report["requests"].as_array().expect("requests");
    assert_eq!(requests.len(), 8, "{requests:?}");
    for r in requests {
        assert_eq!(r["actor"], "human", "{r}");
        assert_eq!(r["ended"], "completed", "{r}");
        if r["action"] == "show_diff" {
            let target = r["target"].as_str().unwrap();
            assert!(target.starts_with("stacks.commit["), "the capability: {r}");
        }
    }

    // nothing after the close
    for key in ["nodes_after", "bindings_after", "surfaces_in_host"] {
        assert_eq!(n(&report["closed"][key]), 0, "{key}");
    }

    // the session, read back as the inspector reads it
    let path = out.path().join("review.monster");
    let session = Session::open(&path).expect("the session opens");
    let summary = Inspector::new(&session).summary().expect("it reads");
    assert_eq!(summary.requests.len(), 8);
    assert!(summary.requests.iter().all(|r| r.actor == Actor::Human));
    assert!(summary.refusals.is_empty());
    let inputs: Vec<(InputKind, Option<SemId>)> = session
        .events()
        .iter()
        .filter_map(|e| match &e.kind {
            Kind::Input { kind, target, .. } => Some((*kind, target.clone())),
            _ => None,
        })
        .collect();
    let typed = inputs.iter().filter(|(k, _)| *k == InputKind::Text).count();
    assert_eq!(
        typed,
        INTENT.chars().count(),
        "each key as text, never the character"
    );
    assert!(
        inputs
            .iter()
            .filter(|(k, _)| *k == InputKind::Text)
            .all(|(_, t)| t.is_none())
    );
    let pressed: Vec<String> = inputs
        .iter()
        .filter_map(|(_, t)| t.as_ref().map(ToString::to_string))
        .collect();
    assert_eq!(
        pressed.len(),
        9,
        "3 cards, field, propose, 3 buttons, close: {pressed:?}"
    );
    assert!(
        pressed[5..8]
            .iter()
            .all(|p| p.starts_with("proposed.") && p.ends_with(".show_diff"))
    );

    // the surface lives in the recorded state only between its proposal
    // and its close, every node of it from the materializer
    let opened = n(&report["surface"]["opened_ms"]);
    let closed = n(&report["closed"]["closed_ms"]);
    let lived = &report["session"]["read_back"]["surface_in_state_ms"];
    let (from, to) = (n(&lived[0]), n(&lived[1]));
    assert!(
        from >= opened && to <= closed,
        "{from}..{to}, {opened}..{closed}"
    );
    let state = session
        .state_at(to)
        .expect("reads")
        .expect("a state while it is open");
    let mut surface = 0;
    for node in state.nodes() {
        let file = node
            .source
            .as_ref()
            .map(|s| s.file.to_string())
            .unwrap_or_default();
        let id = node.id.as_str();
        if id == "proposed" || id.starts_with("proposed.") {
            surface += 1;
            assert!(file.starts_with("crates/surface/src/"), "{id}: {file}");
        } else if id.starts_with("stacks") {
            assert!(file.starts_with("crates/ide/src/"), "{id}: {file}");
        }
    }
    assert!(surface > 10, "the surface is in the state: {surface}");
    let (_, end) = session.span().expect("a span");
    let after = session.state_at(end).expect("reads").expect("a state");
    assert!(
        !after
            .nodes()
            .iter()
            .any(|n| n.id.as_str().starts_with("proposed"))
    );
    assert_eq!(
        n(&report["session"]["read_back"]["last_frame_binds_of_it"]),
        0
    );
}
