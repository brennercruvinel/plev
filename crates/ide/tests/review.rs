//! The review of three commits in the ide (task-96, the demo 5 of
//! task-80), on the laboratory's repository of the review (three commits
//! over the first, a note no commit holds) and the real git worker.
//!
//! The registry a proposal binds is the workspace's own projection
//! (`Workbench::app_snapshot`), never the intent line's: each commit with
//! its message, its files and its `show_diff`, and the one diff region
//! that shows the diff of whichever commit asked last. Nothing of the
//! projection changed for it: a review shows each commit's diff by
//! opening it, one at a time, as the fixed panel does.

// shared with the other ide tests: this binary uses part of the rig.
#[allow(dead_code)]
mod support;

use git::GitEvent;
use ide::lab_repo::REVIEWED;
use sem::{Actor, Node, SemId, StateRef, Value, ValueKind};
use support::{Rig, completed};
use uir::Capabilities;

fn id(text: &str) -> SemId {
    SemId::parse(text).expect("an id")
}

fn at(node: &SemId, key: &str) -> StateRef {
    StateRef::new(node.clone(), key.parse().expect("a key"))
}

/// The commits of the stacks as the workspace projects them, newest first.
fn commits(rig: &Rig) -> Vec<Node> {
    let app = rig.bench.app_snapshot().expect("the projection validates");
    app.children(&id("stacks")).cloned().collect()
}

/// The diff lines the workspace projects now.
fn lines(rig: &Rig) -> Vec<String> {
    match rig.value("diff:lines") {
        Value::List(lines) => lines,
        other => panic!("{other:?}"),
    }
}

#[test]
fn the_registry_is_the_workspace_with_each_commits_files_and_the_diff() {
    let mut rig = Rig::review();
    rig.bench.host_surfaces();
    let app = rig.bench.app_snapshot().expect("validates");
    let caps = Capabilities::from_snapshot(&app);
    assert!(
        caps.nodes().all(|n| !n.as_str().starts_with("ide_surface")),
        "the line is not the app's"
    );
    let commits = commits(&rig);
    assert_eq!(commits.len(), REVIEWED.len() + 1, "the three and the first");
    for (node, (subject, files)) in commits.iter().zip(REVIEWED.iter().rev()) {
        assert_eq!(node.label, *subject);
        let kind = |key: &str| caps.datum(&at(&node.id, key)).map(|d| d.kind);
        assert_eq!(kind("summary"), Some(ValueKind::Text));
        assert_eq!(kind("files"), Some(ValueKind::List));
        let want: Vec<String> = files.iter().map(|(p, _)| (*p).to_owned()).collect();
        assert_eq!(app.value(&at(&node.id, "files")), Some(&Value::List(want)));
        let show = caps
            .action(&node.id, &"show_diff".parse().unwrap())
            .expect("each commit shows its diff");
        assert_eq!(show.param, None);
        let decl = node.actions.iter().find(|a| a.name.as_str() == "show_diff");
        let reads: Vec<String> = decl
            .unwrap()
            .reads
            .iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(reads, ["diff:target"]);
    }
    let diff = id("diff");
    assert_eq!(
        caps.datum(&at(&diff, "target")).map(|d| d.kind),
        Some(ValueKind::Text)
    );
    assert_eq!(
        caps.datum(&at(&diff, "lines")).map(|d| d.kind),
        Some(ValueKind::List)
    );
    assert_eq!(
        caps.datum(&at(&diff, "count")).map(|d| d.kind),
        Some(ValueKind::Int)
    );

    // each commit's diff, asked by the person through the dispatcher: the
    // region names the commit, and holds its lines once the worker answers
    for (node, (_, files)) in commits.iter().zip(REVIEWED.iter().rev()) {
        let asked = rig.ask(Actor::Human, node.id.as_str(), "show_diff", None);
        let receipt = rig.receipt(asked);
        assert!(completed(&receipt).is_some(), "{receipt:?}");
        assert_eq!(rig.value("diff:target"), Value::Text(node.id.to_string()));
        rig.until(|e| matches!(e, GitEvent::DiffCommit { .. }));
        let shown = lines(&rig);
        assert!(shown.iter().any(|l| l.starts_with('+')), "{shown:?}");
        if files.len() > 1 {
            for (path, _) in *files {
                let header = format!("{path}: @@");
                assert!(
                    shown.iter().any(|l| l.starts_with(&header)),
                    "{header} in {shown:?}"
                );
            }
        }
    }
}
