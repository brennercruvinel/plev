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

use accord::{Actor, Node, SemId, StateRef, Value, ValueKind};
use gitdrv::GitEvent;
use idestudio::repository::eval_repo::REVIEWED;
use plevir::Capabilities;
use support::{Rig, completed};

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

/// The kind of each line of a commit's diff, by name, beside the lines
/// (task-97): as many kinds as lines, each a name of `plevir::LineKind`, a
/// header where a hunk begins and the name of the mark on every other
/// line; nothing before a diff is shown.
#[test]
fn the_diff_says_the_kind_of_each_line_beside_it() {
    let mut rig = Rig::review();
    let app = rig.bench.app_snapshot().expect("validates");
    let caps = Capabilities::from_snapshot(&app);
    let kinds_at = at(&id("diff"), "kinds");
    assert_eq!(caps.datum(&kinds_at).map(|d| d.kind), Some(ValueKind::List));
    assert_eq!(rig.value("diff:kinds"), Value::List(Vec::new()));
    for node in commits(&rig).iter().take(REVIEWED.len()) {
        rig.ask(Actor::Human, node.id.as_str(), "show_diff", None);
        rig.until(|e| matches!(e, GitEvent::DiffCommit { .. }));
        let shown = lines(&rig);
        let Value::List(kinds) = rig.value("diff:kinds") else {
            panic!("the kinds are a list");
        };
        assert_eq!(kinds.len(), shown.len(), "{}", node.id);
        for (line, kind) in shown.iter().zip(&kinds) {
            let named = plevir::LineKind::parse(kind).unwrap_or_else(|| panic!("{kind}"));
            let want = match line.chars().next() {
                Some('+') => plevir::LineKind::Added,
                Some('-') => plevir::LineKind::Removed,
                Some(' ') => plevir::LineKind::Context,
                _ => plevir::LineKind::Header,
            };
            assert_eq!(named, want, "{line:?}");
            if named == plevir::LineKind::Header {
                assert!(line.contains("@@ -"), "a hunk's header: {line:?}");
            }
        }
        assert!(kinds.iter().any(|k| k == "header") && kinds.iter().any(|k| k == "added"));
    }
}

/// The person writes `intent` in the line and proposes, each through the
/// dispatcher; the receipt of the proposal.
fn propose(rig: &mut Rig, intent: &str) -> accord::Receipt {
    let write = rig.ask(
        Actor::Human,
        "ide_surface.intent",
        "set_value",
        Some(intent.into()),
    );
    assert!(completed(&rig.receipt(write)).is_some());
    let asked = rig.ask(Actor::Human, "ide_surface.propose", "propose", None);
    rig.receipt(asked)
}

/// The ids of the surface's nodes in the projection now.
fn surface_nodes(rig: &Rig) -> Vec<String> {
    rig.look()
        .nodes()
        .keys()
        .map(ToString::to_string)
        .filter(|id| id == "proposed" || id.starts_with("proposed."))
        .collect()
}

#[test]
fn review_these_three_commits_opens_a_verified_surface_with_their_files_and_the_diff() {
    let mut rig = Rig::review();
    rig.bench.host_surfaces();
    let receipt = propose(&mut rig, "review these three commits");
    let outcome = completed(&receipt).unwrap_or_else(|| panic!("{receipt:?}"));
    assert_eq!(
        outcome.result.get("surface"),
        Some(&Value::Text("proposed".into()))
    );
    assert_eq!(
        rig.value("ide_surface.status:message"),
        Value::Text("proposed by the rules for \"review these three commits\"".into())
    );
    let snap = rig.look();
    let commits = commits(&rig);
    for (node, (subject, files)) in commits.iter().zip(REVIEWED.iter().rev()) {
        let card = id(&format!("proposed.{}", node.id));
        let shown = snap.get(&card).unwrap_or_else(|| panic!("{card}"));
        assert_eq!(shown.label, *subject, "each card named by its commit");
        let want: Vec<String> = files.iter().map(|(p, _)| (*p).to_owned()).collect();
        let table = id(&format!("{card}.files"));
        assert_eq!(snap.value(&at(&table, "rows")), Some(&Value::List(want)));
        let button = snap
            .get(&id(&format!("{card}.show_diff")))
            .expect("its diff");
        assert_eq!(button.role, accord::Role::Button);
        assert!(button.enabled);
    }
    let first = id(&format!("proposed.{}", commits[REVIEWED.len()].id));
    assert!(
        snap.get(&first).is_none(),
        "the fourth commit is not reviewed"
    );
    for part in ["proposed.diff.heading", "proposed.diff.lines"] {
        assert!(snap.get(&id(part)).is_some(), "{part}");
    }
    assert!(
        !surface_nodes(&rig).iter().any(|n| n.contains("notes.txt")),
        "the note no commit holds is not in the review"
    );
}

/// The diff's card on the surface says what it shows, "diff", whichever
/// commit's diff it shows: never the commit's id, the dispatcher's word
/// for which one (task-96, phase b). The commit's message is its card's,
/// on the left.
#[test]
fn the_diff_card_says_diff_never_the_id_of_the_commit_it_shows() {
    let mut rig = Rig::review();
    rig.bench.host_surfaces();
    assert!(completed(&propose(&mut rig, "review these three commits")).is_some());
    let commits = commits(&rig);
    for node in commits.iter().take(REVIEWED.len()) {
        let button = format!("proposed.{}.show_diff", node.id);
        let asked = rig.ask(Actor::Human, &button, "activate", None);
        assert!(completed(&rig.receipt(asked)).is_some());
        rig.until(|e| matches!(e, GitEvent::DiffCommit { .. }));
        assert_eq!(rig.value("diff:target"), Value::Text(node.id.to_string()));
        let snap = rig.look();
        let card = snap.get(&id("proposed.diff")).expect("the diff's card");
        let first = snap.children(&card.id).next().expect("what opens it");
        assert_eq!(first.id.as_str(), "proposed.diff.heading");
        assert_eq!(first.label, "diff", "{}", node.id);
        for shown in surface_nodes(&rig) {
            let n = snap.get(&id(&shown)).expect("projected");
            assert!(
                !n.label.contains("stacks.commit["),
                "{shown} is named {:?}",
                n.label
            );
            assert!(
                n.state
                    .values()
                    .all(|v| *v != Value::Text(node.id.to_string())),
                "{shown} shows the commit's id: {:?}",
                n.state
            );
        }
    }
}

#[test]
fn the_same_review_in_portuguese_opens_the_same_surface() {
    let mut en = Rig::review();
    en.bench.host_surfaces();
    assert!(completed(&propose(&mut en, "review these three commits")).is_some());
    let mut pt = Rig::review();
    pt.bench.host_surfaces();
    assert!(completed(&propose(&mut pt, "revise estes três commits")).is_some());
    assert_eq!(surface_nodes(&pt), surface_nodes(&en));
}

#[test]
fn a_review_the_rules_do_not_read_is_refused_and_nothing_opens() {
    let mut rig = Rig::review();
    rig.bench.host_surfaces();
    let receipt = propose(&mut rig, "review these three potatoes");
    let Some(accord::Stage::Failed { error }) = receipt.stage() else {
        panic!("{receipt:?}");
    };
    assert!(
        error.contains("\"potatoes\" is not a word of the app"),
        "{error}"
    );
    assert_eq!(
        rig.value("ide_surface.status:message"),
        Value::Text(error.clone())
    );
    let receipt = propose(&mut rig, "review these nine commits");
    let Some(accord::Stage::Failed { error }) = receipt.stage() else {
        panic!("{receipt:?}");
    };
    assert_eq!(error, "9 commit asked for, and stacks has 4");
    assert!(surface_nodes(&rig).is_empty());
    assert_eq!(rig.bench.surface_open(), None);
}
