//! A `promote` aimed at the template of an item (task-70 phase 3): a
//! composite the Silences declares on every cut is promoted into the
//! cut's template of the review, one button per cut, each asking the
//! composite of its own cut; undone, the template is as it was; and when
//! one cut lacks the action, the verifier refuses the template, as it
//! refuses any template the items do not declare alike.

// shared with the other surface tests: this binary uses the fixtures and
// the page.
#[allow(dead_code)]
mod support;

use accord::{ActionName, Actor, Request, Snapshot, SnapshotBuilder};
use support::{REVIEW_SILENCES, SILENCES, copy, id, page, snapshot, verified_local};
use screen::{SurfaceError, SurfaceHost};
use plevir::Op;

const COMPOSITE: &str = "preview_segment_then_apply_cut";

/// The Silences as the fixture projects it, with the composite declared
/// on every cut but `lacking`.
fn with_composite(lacking: Option<&str>) -> Snapshot {
    let app = snapshot(SILENCES);
    let mut b = SnapshotBuilder::new();
    for node in app.nodes().values() {
        copy(&mut b, node);
    }
    for cut in ["silences.cut[1]", "silences.cut[2]"] {
        if Some(cut) == lacking {
            continue;
        }
        let mut node = b.reopen(&id(cut)).expect("the cut is declared");
        node.action(COMPOSITE)
            .label("preview segment, then apply cut")
            .reads(format!("{cut}:decision").as_str());
    }
    b.finish().expect("the app projects")
}

fn promote() -> Op {
    Op::Promote {
        target: id("silences.cut[{item}]"),
        action: ActionName::parse(COMPOSITE).unwrap(),
        into: id("review.cut[{item}]"),
    }
}

#[test]
fn a_promote_on_the_template_of_an_item_puts_one_button_in_each() {
    let app = with_composite(None);
    let mut host = SurfaceHost::new();
    let ir = verified_local(REVIEW_SILENCES, &app);
    let surface = ir.ir().surface.id.clone();
    let before = ir.clone();
    host.open(ir, &app).expect("opens");
    let promoted = host
        .apply(&surface, promote(), Actor::Human, &app)
        .expect("promoted into the template");
    let ids: Vec<String> = promoted.nodes().map(ToString::to_string).collect();
    for cut in [1, 2] {
        let button = format!("review.cut[{cut}].{COMPOSITE}");
        assert!(ids.contains(&button), "{button} in {ids:?}");
    }

    // each button asks the composite of its own cut
    let shown = host.get(&surface).expect("open");
    let seen = page(&app, shown);
    let at = format!("review.cut[2].{COMPOSITE}");
    let req = Request::observe(&seen, Actor::Human, at.as_str(), "activate", None).unwrap();
    let routed = host.route(req);
    assert_eq!(routed.target, id("silences.cut[2]"));
    assert_eq!(routed.action.as_str(), COMPOSITE);

    // the change is kept with its inverse, and undone the template is back
    assert_eq!(host.changes(&surface).len(), 1);
    host.undo(&surface, &app).expect("undone");
    assert_eq!(host.get(&surface).map(|s| s.verified()), Some(&before));
}

#[test]
fn a_template_whose_items_do_not_all_declare_it_is_refused() {
    let app = with_composite(Some("silences.cut[2]"));
    let mut host = SurfaceHost::new();
    let ir = verified_local(REVIEW_SILENCES, &app);
    let surface = ir.ir().surface.id.clone();
    host.open(ir, &app).expect("opens");
    let refused = host.apply(&surface, promote(), Actor::Human, &app);
    let Err(SurfaceError::Transform(why)) = refused else {
        panic!("refused: {refused:?}");
    };
    assert!(why.to_string().contains("silences.cut[2]"), "{why}");
    assert!(host.changes(&surface).is_empty());
}
