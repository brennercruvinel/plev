//! The media part of the rules over the silences app (task-69 phase 4b):
//! found by structure, a media node whose text states name a media node
//! and an instant (a frame ref), and a list whose items carry one. "the
//! cuts as a list with the video and the counts" gives the review, an
//! image of the video over the list with an image per cut; "compare the
//! original with the cuts" gives the comparison, every view of the video
//! side by side over the same list; a proposal over the review keeps the
//! ids of what continues; and the todo and the ide, with no media, give
//! what they gave (their own tests) and refuse the media words.

// shared with the other tests of the crate: this binary uses a part of it.
#[allow(dead_code)]
mod support;

use offers::{Part, ProposeError, Proposer, Rules, propose_verified};
use accord::Snapshot;
use support::{Asking, bindings, ide, ids, nodes, todo, verified};
use plevir::{Component, Layout, VerifiedIr};

fn silences() -> Snapshot {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../uir/fixtures/showcase-silences.snapshot.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn propose(asking: &Asking, intent: &str) -> VerifiedIr {
    propose_verified(&mut Rules, &asking.ask(intent))
        .unwrap_or_else(|e| panic!("\"{intent}\": {e}"))
}

/// `(id, what it binds)` of every image of `ir`, in tree order.
fn images(ir: &VerifiedIr) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut stack = vec![&ir.ir().surface.root];
    while let Some(n) = stack.pop() {
        if n.component == Some(Component::Image) {
            let source = n.bind.state.get("source").map(ToString::to_string);
            out.push((n.sem.to_string(), source.unwrap_or_default()));
        }
        if let Some(r) = &n.repeat {
            stack.push(&r.item);
        }
        stack.extend(n.children.iter().rev());
    }
    out
}

const VIDEO: &str = "source=silences.video:original";
const RESULT: &str = "source=silences.video:result";
const CUT_FRAME: &str = "source=silences.cut[{item}]:frame";
const APPLY: &str = "activate=silences.cut[{item}] apply_cut";
const CUT_SECS: &str = "value=silences.counts:cut_secs";

/// `bindings`, the templates walked too.
fn all_bindings(ir: &VerifiedIr) -> Vec<(String, Vec<String>)> {
    let mut out = bindings(&ir.ir().surface.root);
    let mut stack = vec![&ir.ir().surface.root];
    while let Some(n) = stack.pop() {
        if let Some(r) = &n.repeat {
            out.extend(bindings(&r.item));
        }
        stack.extend(n.children.iter());
    }
    out
}

fn binds(ir: &VerifiedIr, binding: &str) -> bool {
    all_bindings(ir)
        .iter()
        .any(|(_, b)| b.iter().any(|x| x == binding))
}

/// The review: the video's first view on top as an image, the counts,
/// and the cuts as a list whose every item shows its frame as an image
/// (never as a text), its range, its decision and its two buttons.
#[test]
fn the_review_has_the_video_the_counts_and_a_frame_per_cut() {
    let fresh = Asking::new(silences());
    let ir = propose(&fresh, "the cuts as a list with the video and the counts");
    let root = &ir.ir().surface.root;
    assert_eq!(ir.ir().surface.id.as_str(), "proposed");
    // the video first
    let first = root.children.first().expect("a child");
    assert_eq!(first.component, Some(Component::Image));
    assert_eq!(first.label.as_deref(), Some("silences-test.mp4"));
    assert_eq!(
        images(&ir),
        [
            (
                "proposed.silences.video.original".to_owned(),
                "silences.video:original".to_owned()
            ),
            (
                "proposed.silences.cut[{item}].frame".to_owned(),
                "silences.cut[{item}]:frame".to_owned()
            ),
        ]
    );
    assert!(binds(&ir, VIDEO));
    assert!(!binds(&ir, RESULT), "one view: the first");
    assert!(binds(&ir, CUT_FRAME));
    assert!(binds(&ir, APPLY));
    assert!(binds(&ir, "activate=silences.cut[{item}] reject_cut"));
    assert!(binds(&ir, "text=silences.cut[{item}]:range"));
    assert!(binds(&ir, "text=silences.cut[{item}]:decision"));
    assert!(
        !binds(&ir, "text=silences.cut[{item}]:frame"),
        "a frame ref is never a text"
    );
    // the row by sense (the checkpoint b of task-69): what the cut is,
    // its numbers, its actions, each a group of the template
    let list = nodes(root)
        .into_iter()
        .find(|n| n.repeat.is_some())
        .expect("the list");
    let row = &list.repeat.as_ref().unwrap().item;
    let senses: Vec<&str> = row.children.iter().map(|n| n.sem.as_str()).collect();
    assert_eq!(
        senses,
        [
            "proposed.silences.cut[{item}].about",
            "proposed.silences.cut[{item}].numbers",
            "proposed.silences.cut[{item}].actions",
        ]
    );
    // what the cut is: its frame first, as a thumbnail, its range, and
    // its decision as a badge (two actions set it, and nothing else)
    let about = &row.children[0];
    let parts: Vec<Option<Component>> = about.children.iter().map(|n| n.component).collect();
    assert_eq!(
        parts,
        [
            Some(Component::Image),
            Some(Component::Text),
            Some(Component::Badge)
        ]
    );
    assert_eq!(
        about.children[0].props.control_size,
        Some(plevir::ControlSize::Xl)
    );
    // its numbers, each a small stat named by its key
    let numbers: Vec<(Option<Component>, Option<&str>, Option<plevir::TypographyRole>)> = row.children
        [1]
    .children
    .iter()
    .map(|n| (n.component, n.label.as_deref(), n.props.typography))
    .collect();
    let small = Some(plevir::TypographyRole::BaseM);
    assert_eq!(
        numbers,
        [
            (Some(Component::Stat), Some("start"), small),
            (Some(Component::Stat), Some("end"), small),
            (Some(Component::Stat), Some("duration"), small),
        ]
    );
    let actions: Vec<Option<&str>> = row.children[2]
        .children
        .iter()
        .map(|n| n.label.as_deref())
        .collect();
    assert_eq!(
        actions,
        [
            Some("preview segment"),
            Some("apply cut"),
            Some("reject cut")
        ]
    );
    // "cuts" names the list, and the count it also names; a cut's own
    // numbers are the list's, never counts
    assert!(binds(&ir, CUT_SECS));
    assert!(!binds(&ir, "value=silences.cut[1]:start"));
    // no entry, no choice: the silences have none
    assert!(!binds(&ir, "set_value=silences.threshold set_value"));
}

/// The comparison: every view of the video side by side (the original
/// and the result), and the list of the cuts with their decisions.
#[test]
fn the_comparison_shows_every_view_side_by_side_over_the_same_list() {
    let fresh = Asking::new(silences());
    let ir = propose(&fresh, "compare the original with the cuts");
    let root = &ir.ir().surface.root;
    let views = root.children.first().expect("the views");
    assert!(
        matches!(views.layout, Some(Layout::Grid { .. })),
        "{:?}",
        views.layout
    );
    assert_eq!(views.children.len(), 2);
    assert_eq!(views.children[0].label.as_deref(), Some("original"));
    assert_eq!(views.children[1].label.as_deref(), Some("result"));
    assert!(binds(&ir, VIDEO));
    assert!(binds(&ir, RESULT));
    assert!(binds(&ir, CUT_FRAME));
    assert!(binds(&ir, "text=silences.cut[{item}]:decision"));
    // "the original" alone names that view
    let one = propose(&fresh, "the original");
    assert!(binds(&one, VIDEO));
    assert!(!binds(&one, RESULT));
    assert!(!binds(&one, CUT_FRAME));
    // "the result" the other
    let other = propose(&fresh, "the result");
    assert!(binds(&other, RESULT));
    assert!(!binds(&other, VIDEO));
}

/// Continuity: the comparison proposed over the review keeps the ids of
/// the video's view, the list and its template, so the instances go on;
/// the result's view is new; and the surface keeps its id.
#[test]
fn a_comparison_over_the_review_keeps_what_continues() {
    let fresh = Asking::new(silences());
    let review = propose(&fresh, "the cuts as a list with the video and the counts");
    let then = Asking::new(silences()).over(review.clone());
    let compare = propose(&then, "compare the original with the cuts");
    assert_eq!(compare.ir().surface.id, review.ir().surface.id);
    let before = ids(&review.ir().surface.root);
    let after = ids(&compare.ir().surface.root);
    for id in [
        "proposed.card",
        "proposed.silences.video.original",
        "proposed.silences.cuts",
        "proposed.silences.counts.cut_secs",
    ] {
        assert!(before.contains(&id.to_owned()), "{id} before");
        assert!(after.contains(&id.to_owned()), "{id} after");
    }
    let template = |ir: &VerifiedIr| {
        nodes(&ir.ir().surface.root)
            .into_iter()
            .find_map(|n| n.repeat.as_ref().map(|r| ids(&r.item)))
            .expect("a template")
    };
    assert_eq!(template(&review), template(&compare), "the instances go on");
    assert!(after.contains(&"proposed.silences.video.result".to_owned()));
    assert!(!before.contains(&"proposed.silences.video.result".to_owned()));
    // and back to the review: the result's view leaves, the rest stays
    let back = Asking::new(silences()).over(compare);
    let review_again = propose(&back, "the cuts as a list with the video and the counts");
    assert_eq!(ids(&review_again.ir().surface.root), before);
}

/// The silences with no surface asked for by a form word alone shows
/// everything: the video, the counts and the list.
#[test]
fn everything_over_the_silences_verifies_with_the_media_on_top() {
    let fresh = Asking::new(silences());
    for intent in ["everything", "tudo", "compact", "the video and the list"] {
        let ir = propose(&fresh, intent);
        assert!(binds(&ir, VIDEO), "{intent}");
        assert!(binds(&ir, CUT_FRAME), "{intent}");
        let first = ir.ir().surface.root.children.first().unwrap();
        assert_eq!(first.component, Some(Component::Image), "{intent}: on top");
    }
    // the counts of everything: the video's and the summary's numbers,
    // never a cut's (those are the list's)
    let all = propose(&fresh, "everything");
    assert!(binds(&all, CUT_SECS));
    assert!(binds(&all, "value=silences.video:duration"));
    assert!(!binds(&all, "value=silences.cut[1]:start"));
    assert!(!binds(&all, "value=silences.cut[2]:duration"));
}

/// The todo and the ide have no media: the media words are a clear
/// refusal there, and their lists carry no image.
#[test]
fn an_app_without_media_refuses_the_media_words_and_lists_no_image() {
    for (name, app) in [("todo", todo()), ("ide", ide())] {
        let asking = Asking::new(app);
        let refused = Rules.propose(&asking.ask("the video")).err();
        assert!(
            matches!(
                refused,
                Some(ProposeError::Nothing {
                    part: Part::Media,
                    ..
                })
            ),
            "{name}: {refused:?}"
        );
        let ir = propose(&asking, "the items");
        assert!(images(&ir).is_empty(), "{name}");
    }
    // and over the todo's own surface the media words are still refused
    let over = Asking::new(todo()).over(verified(&support::compact(), &todo()));
    assert!(Rules.propose(&over.ask("compare")).is_err());
}
