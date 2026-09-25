//! The review of items by the rules (task-96): "review these three
//! commits" over a workbench with four commits, the same in portuguese,
//! the files of the ide's own golden, and every refusal with why.

// shared with the other test binaries: this one reads less of it.
#[allow(dead_code)]
mod support;

use propose::{ProposeError, Rules, propose_verified};
use sem::{Confirm, Policy, Role, Snapshot, SnapshotBuilder};
use support::{Asking, bindings, ids, nodes};
use uir::{Component, IrNode, Layout, SizeToken, VerifiedIr};

/// The commits of the stacks, newest first: sha, subject, files.
const COMMITS: [(&str, &str, &[&str]); 4] = [
    (
        "c3",
        "readme: how to run the demo",
        &["Cargo.toml", "README.md"],
    ),
    ("c2", "login remembers the user", &["src/login.rs"]),
    ("c1", "session timeout from the server", &["src/session.rs"]),
    ("c0", "login form and session store", &["README.md"]),
];

/// A workbench as the ide projects it, with four commits and one change:
/// each commit with its summary, author, files and `show_diff`, which
/// reads the diff's target; the change with its status, its own actions
/// and a destructive `discard`.
fn workbench() -> Snapshot {
    let mut b = SnapshotBuilder::new();
    b.node("changes", Role::List, "changes")
        .state("files", vec!["notes.txt".to_owned()]);
    let mut file = b.node("changes.file[notes.txt]", Role::ListItem, "notes.txt");
    file.parent("changes").state("status", "untracked");
    file.action("stage").reads("changes.file[notes.txt]:status");
    file.action("discard")
        .reads("changes.file[notes.txt]:status")
        .policy(Policy {
            confirm: Confirm::ForAgents,
            destructive: true,
            ..Policy::default()
        });
    file.action("show_diff").reads("diff:target");
    b.node("diff", Role::Region, "diff")
        .state("target", "")
        .state("lines", Vec::<String>::new())
        .state("count", 0i64);
    b.node("stacks", Role::List, "stacks")
        .state("branch", "main");
    for (sha, subject, files) in COMMITS {
        let files: Vec<String> = files.iter().map(|f| (*f).to_owned()).collect();
        let mut commit = b.node(
            format!("stacks.commit[{sha}]").as_str(),
            Role::ListItem,
            subject,
        );
        commit
            .parent("stacks")
            .state("summary", subject)
            .state("author", "Lab Author")
            .state("files", files);
        commit.action("show_diff").reads("diff:target");
    }
    b.finish().expect("the workbench projects")
}

fn review(asking: &Asking, intent: &str) -> Result<VerifiedIr, ProposeError> {
    propose_verified(&mut Rules, &asking.ask(intent))
}

fn node<'a>(root: &'a IrNode, id: &str) -> &'a IrNode {
    nodes(root)
        .into_iter()
        .find(|n| n.sem.as_str() == id)
        .unwrap_or_else(|| panic!("{id} in {:?}", ids(root)))
}

/// What a node binds, as `bindings` writes it.
fn bound(root: &IrNode, id: &str) -> Vec<String> {
    bindings(root)
        .into_iter()
        .find(|(at, _)| at == id)
        .map(|(_, b)| b)
        .unwrap_or_default()
}

#[test]
fn review_these_three_commits_shows_the_three_with_their_files_and_the_diff() {
    let asking = Asking::new(workbench());
    let ir = review(&asking, "review these three commits").expect("verified");
    let surface = &ir.ir().surface;
    assert_eq!(surface.id.as_str(), "proposed");
    assert_eq!(surface.title, "review these three commits");
    let root = &surface.root;
    assert_eq!(root.component, Some(Component::Card));
    assert!(
        matches!(root.layout, Some(Layout::Split { .. })),
        "{:?}",
        root.layout
    );
    let [list, region] = root.children.as_slice() else {
        panic!("the items and the region: {:?}", ids(root));
    };
    assert_eq!(list.sem.as_str(), "proposed.stacks");
    let cards: Vec<&str> = list.children.iter().map(|c| c.sem.as_str()).collect();
    assert_eq!(
        cards,
        [
            "proposed.stacks.commit[c3]",
            "proposed.stacks.commit[c2]",
            "proposed.stacks.commit[c1]"
        ],
        "the three from the top, in the stacks' order"
    );
    for (card, (sha, subject, _)) in list.children.iter().zip(COMMITS) {
        assert_eq!(card.component, Some(Component::Card));
        assert_eq!(card.label.as_deref(), Some(subject));
        let at = |key: &str| format!("{}.{key}", card.sem);
        let app = format!("stacks.commit[{sha}]");
        assert_eq!(bound(root, &at("summary")), [format!("text={app}:summary")]);
        assert_eq!(bound(root, &at("author")), [format!("text={app}:author")]);
        assert_eq!(bound(root, &at("files")), [format!("rows={app}:files")]);
        assert_eq!(node(root, &at("files")).component, Some(Component::Table));
        let show = node(root, &at("show_diff"));
        assert_eq!(show.component, Some(Component::Button));
        assert_eq!(show.label.as_deref(), Some("show diff"));
        assert_eq!(
            bound(root, &at("show_diff")),
            [format!("activate={app} show_diff")]
        );
    }
    let head = node(root, "proposed.stacks.commit[c3].head");
    let head: Vec<&str> = head.children.iter().map(|c| c.sem.as_str()).collect();
    assert_eq!(
        head,
        [
            "proposed.stacks.commit[c3].summary",
            "proposed.stacks.commit[c3].show_diff"
        ],
        "what names the commit and its action on one line"
    );
    assert_eq!(region.sem.as_str(), "proposed.diff");
    assert_eq!(region.component, Some(Component::Card));
    let last = region.children.last().map(|c| c.sem.as_str());
    assert_eq!(
        last,
        Some("proposed.diff.lines"),
        "the lines last, to scroll"
    );
    assert_eq!(bound(root, "proposed.diff.target"), ["text=diff:target"]);
    assert_eq!(bound(root, "proposed.diff.lines"), ["rows=diff:lines"]);
    assert_eq!(bound(root, "proposed.diff.count"), ["value=diff:count"]);
    let lines = node(root, "proposed.diff.lines");
    assert_eq!(
        lines.size.max,
        Some(SizeToken::DropdownMaxH),
        "bounded in height"
    );
    let all = bindings(root);
    assert!(
        !all.iter()
            .flat_map(|(_, b)| b)
            .any(|b| b.contains("[c0]") || b.contains("changes")),
        "nothing of the fourth commit or of the working tree: {all:?}"
    );
}

#[test]
fn the_same_review_in_portuguese_is_the_same_surface() {
    let asking = Asking::new(workbench());
    let en = review(&asking, "review these three commits").expect("verified");
    for intent in ["revise estes três commits", "revisar os 3 últimos commits"] {
        let pt = review(&asking, intent).expect("verified");
        assert_eq!(pt.ir().surface.root, en.ir().surface.root, "{intent}");
        assert_eq!(pt.ir().surface.title, intent);
    }
    let all = review(&asking, "review the commits").expect("verified");
    let list = &all.ir().surface.root.children[0];
    assert_eq!(list.children.len(), 4, "without a number, every one");
}

#[test]
fn a_review_of_the_files_shows_what_a_file_takes_and_never_a_destructive_action() {
    let asking = Asking::new(workbench());
    let ir = review(&asking, "review the files").expect("verified");
    let root = &ir.ir().surface.root;
    let card = "proposed.changes.file[notes.txt]";
    assert_eq!(
        bound(root, &format!("{card}.status")),
        ["text=changes.file[notes.txt]:status"]
    );
    let actions: Vec<String> = bindings(root)
        .into_iter()
        .flat_map(|(_, b)| b)
        .filter(|b| b.starts_with("activate="))
        .collect();
    assert_eq!(
        actions,
        [
            "activate=changes.file[notes.txt] stage",
            "activate=changes.file[notes.txt] show_diff"
        ]
    );
    // the ide's own golden reviews the same way
    let golden = Asking::new(support::ide());
    let files = review(&golden, "review the three files").expect("verified");
    assert_eq!(files.ir().surface.root.children[0].children.len(), 3);
}

#[test]
fn a_review_over_the_one_shown_keeps_the_ids_of_what_stays() {
    let asking = Asking::new(workbench());
    let three = review(&asking, "review these three commits").expect("verified");
    let over = Asking::new(workbench()).over(three.clone());
    let two = review(&over, "review these two commits").expect("verified");
    assert_eq!(
        two.ir().surface.id,
        three.ir().surface.id,
        "the same surface"
    );
    assert_eq!(two.ir().surface.title, three.ir().surface.title);
    let kept = ids(&two.ir().surface.root);
    for id in [
        "proposed.stacks.commit[c3]",
        "proposed.stacks.commit[c3].show_diff",
        "proposed.stacks.commit[c2].files",
        "proposed.diff.lines",
    ] {
        assert!(kept.iter().any(|k| k == id), "{id} in {kept:?}");
    }
    assert!(!kept.iter().any(|k| k.contains("[c1]")));
}

#[test]
fn what_the_rules_cannot_review_is_refused_with_why() {
    let asking = Asking::new(workbench());
    match review(&asking, "review these three potatoes") {
        Err(ProposeError::NoMatch { words, app }) => {
            assert_eq!(words, ["potatoes"]);
            assert!(app.contains(&"commit".to_owned()), "{app:?}");
            assert!(app.contains(&"file".to_owned()), "{app:?}");
        }
        other => panic!("{other:?}"),
    }
    let five = review(&asking, "review these five commits").unwrap_err();
    assert_eq!(five.to_string(), "5 commit asked for, and stacks has 4");
    let which = review(&asking, "review these three").unwrap_err();
    assert_eq!(which.to_string(), "say which items to review: file, commit");
    let none = review(&Asking::new(Snapshot::default()), "review everything").unwrap_err();
    assert!(none.to_string().contains("nothing to review"), "{none}");
    // an intent with no word of review reads as it always did
    assert!(matches!(
        review(&asking, "these three commits"),
        Err(ProposeError::NoMatch { .. })
    ));
}
