//! The recorded demo of the video editor (task-69 phase 4c), read back:
//! `fixtures/demo/silences-demo.monster` is the session `examples/demo.rs`
//! recorded with `PLEV_RECORD=1` and the rules as the proposer, through
//! the port, with no window; `silences-demo-claude.monster` the same
//! script with the Claude proposing the review (two real calls, 124 s,
//! 0.19 USD) and the rules the comparison, in place over the Claude's
//! surface. This test opens each the way `plev-inspect` does and holds
//! it to the script: the requests in the order the agent (and the
//! person, once) asked them, with their actors and how each ended, the
//! close last; the intents as written; the agent's observations; the
//! person's click with the generated surface's button under it; the
//! frames, with the generated surface's controls bound in them while it
//! was open and the video's frames in the file; and the inspector's
//! "why" of the cut applied through the generated surface. Nothing here
//! runs ffmpeg, the demo or the Claude: the files are data.
//!
//! To record them again (ffmpeg on the PATH; the rules need no model, the
//! Claude run needs `claude` and pays for it):
//! `PLEV_RECORD=1 cargo run -p showcase --example demo -- --out
//! crates/showcase/fixtures/demo/silences-demo.monster`, and with
//! `--review claude` for the other.
#![cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]

use inspect::{Basis, Ending, Inspector};
use rec::{InputKind, Kind, Session};
use sem::{Actor, SemId, Stage, Value};

/// The intent of the thesis, what the Claude was asked; the rules' own
/// sentence for the same screen; the comparison, the same for both.
const REVIEW: &str = "cut the silences of this video, then I want to review the cuts";
const REVIEW_BY_RULES: &str = "the list with the video and the counts";
const COMPARE: &str = "compare the original with the cuts";

/// One recorded demo and what differs in it: who proposed the review,
/// what the surface is called, and how the proposer named the buttons the
/// agent and the person pressed on it (the rules after the app's node and
/// the action; the Claude after its own words).
struct Demo {
    fixture: &'static str,
    review: &'static str,
    surface: &'static str,
    /// The surface's button the agent pressed: the apply of the second
    /// cut (its card sits below the fold; the agent acts by id).
    applied: &'static str,
    /// The surface's button the person clicked: the reject of the first.
    clicked: &'static str,
    /// The Claude's proposal took attempts, seconds and money; the
    /// rules' took nothing.
    paid: bool,
}

const RULES: Demo = Demo {
    fixture: concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/fixtures/demo/silences-demo.monster"
    ),
    review: REVIEW_BY_RULES,
    surface: "proposed",
    applied: "proposed.silences.cut[4].apply_cut",
    clicked: "proposed.silences.cut[3].reject_cut",
    paid: false,
};

const CLAUDE: Demo = Demo {
    fixture: concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/fixtures/demo/silences-demo-claude.monster"
    ),
    review: REVIEW,
    surface: "silence_review",
    applied: "silences.cut[4].apply_btn",
    clicked: "silences.cut[3].reject_btn",
    paid: true,
};

/// The script, as the session's summary lists it: actor, target, action.
/// The cuts are 3 and 4: a real detection replaces the seed's 1 and 2. An
/// act on the generated surface is recorded as the capability it was
/// routed to, on the app's node.
const SCRIPT: [(&str, &str, &str); 10] = [
    ("agent:demo", "silences.detect", "detect_silence"),
    ("agent:demo", "silences_surface.proposer", "select"),
    ("agent:demo", "silences_surface.intent", "set_value"),
    ("agent:demo", "silences_surface.propose", "propose"),
    ("agent:demo", "silences.cut[4]", "apply_cut"),
    ("human", "silences.cut[3]", "reject_cut"),
    ("agent:demo", "silences_surface.proposer", "select"),
    ("agent:demo", "silences_surface.intent", "set_value"),
    ("agent:demo", "silences_surface.propose", "propose"),
    ("agent:demo", "silences_surface.close", "close"),
];

/// A person's input as recorded: when, its kind, where, the control under
/// it.
type Input = (u64, InputKind, Option<[f32; 2]>, Option<SemId>);

fn sid(text: &str) -> SemId {
    SemId::parse(text).expect("an id")
}

fn text(s: &str) -> Option<Value> {
    Some(Value::Text(s.to_owned()))
}

fn holds(demo: &Demo) {
    let session = Session::open(demo.fixture).expect("the demo opens");
    let (start, end) = session.span().expect("events");
    assert!(start < 1_000, "{start} ms: the page's clock starts with it");
    assert!((2_000..600_000).contains(&end), "{end} ms");

    // the requests, in order, each completed whole, the close last
    let receipts = session.receipts(end);
    let script: Vec<(String, String, String)> = receipts
        .iter()
        .map(|r| {
            let req = &r.receipt.request;
            (
                req.actor.to_string(),
                req.target.to_string(),
                req.action.to_string(),
            )
        })
        .collect();
    let expected: Vec<(String, String, String)> = SCRIPT
        .iter()
        .map(|(a, t, n)| (a.to_string(), t.to_string(), n.to_string()))
        .collect();
    assert_eq!(script, expected);
    assert!(
        receipts
            .iter()
            .all(|r| r.whole && matches!(r.receipt.stage(), Some(Stage::Completed(_)))),
        "{receipts:#?}"
    );
    let outcome = |n: usize, key: &str| match receipts[n].receipt.stage() {
        Some(Stage::Completed(outcome)) => outcome.result.get(key).cloned(),
        _ => None,
    };
    assert_eq!(outcome(0, "cuts"), Some(Value::Int(2)));
    assert_eq!(outcome(3, "surface"), text(demo.surface));
    assert_eq!(outcome(4, "decision"), text("apply"));
    assert_eq!(outcome(5, "decision"), text("reject"));
    assert_eq!(outcome(8, "surface"), text(demo.surface), "in place");
    assert_eq!(outcome(9, "closed"), text(demo.surface));
    if demo.paid {
        assert_eq!(outcome(3, "attempts"), Some(Value::Int(2)));
        let Some(Value::Num(seconds)) = outcome(3, "seconds") else {
            panic!("the seconds: {:?}", outcome(3, "seconds"));
        };
        assert!(seconds > 60.0, "{seconds} s");
        let Some(Value::Num(cost)) = outcome(3, "cost_usd") else {
            panic!("the cost: {:?}", outcome(3, "cost_usd"));
        };
        assert!(cost > 0.0 && cost < 1.0, "{cost} USD");
    } else {
        assert_eq!(outcome(3, "attempts"), None);
        assert_eq!(outcome(3, "cost_usd"), None);
    }

    // the intents, as the agent wrote them
    let intents: Vec<Option<Value>> = receipts
        .iter()
        .filter(|r| r.receipt.request.action.as_str() == "set_value")
        .map(|r| r.receipt.request.param.clone())
        .collect();
    assert_eq!(intents, [text(demo.review), text(COMPARE)]);

    // who observed what: the agent alone, the whole page and, while it
    // waited for the frames, the status
    let observed: Vec<(Actor, Option<SemId>)> = session
        .events()
        .iter()
        .filter_map(|e| match &e.kind {
            Kind::Observed { actor, scope, .. } => Some((actor.clone(), scope.clone())),
            _ => None,
        })
        .collect();
    assert!(observed.len() >= 10, "{observed:?}");
    let agent = Actor::Agent {
        name: "demo".into(),
    };
    assert!(observed.iter().all(|(actor, _)| *actor == agent));
    assert!(observed.iter().any(|(_, scope)| scope.is_none()));
    assert!(
        observed
            .iter()
            .any(|(_, scope)| scope.as_ref() == Some(&sid("silences.status")))
    );

    // the person's click: the press and the release, with the generated
    // surface's button under them; the frame on screen then names it too
    let inputs: Vec<Input> = session
        .events()
        .iter()
        .filter_map(|e| match &e.kind {
            Kind::Input { kind, at, target } => Some((e.t, *kind, *at, target.clone())),
            _ => None,
        })
        .collect();
    let [(t_down, down, at, on_down), (_, up, _, on_up)] = inputs.as_slice() else {
        panic!("one click: {inputs:?}");
    };
    assert_eq!((*down, *up), (InputKind::PointerDown, InputKind::PointerUp));
    assert_eq!(on_down.as_ref(), Some(&sid(demo.clicked)));
    assert_eq!(on_up.as_ref(), Some(&sid(demo.clicked)));
    let [x, y] = at.expect("where the click landed");
    assert_eq!(session.hit_at(*t_down, x, y), Some(sid(demo.clicked)));

    // the frames: while the review was open, its controls were bound
    // beside the fixed screen's (the button the person clicked; the one
    // the agent pressed by id sits below the fold on the Claude's taller
    // surface, in the state and not in the frame); after the close the
    // surface is gone from the state
    let t_apply = receipts[4].receipt.stages[0].1;
    let bound: Vec<SemId> = session
        .bindings_at(t_apply)
        .expect("a frame on screen")
        .into_iter()
        .map(|b| b.id)
        .collect();
    for id in [demo.clicked, "silences.cut[4]", "silences_surface.close"] {
        assert!(bound.contains(&sid(id)), "{id} in {bound:?}");
    }
    let scene = session
        .scene_at(t_apply)
        .expect("a frame reads")
        .expect("a frame");
    assert!(!scene.is_empty());
    let open = session
        .state_at(t_apply)
        .expect("the state reads")
        .expect("a state");
    assert!(open.get(&sid(demo.surface)).is_some());
    assert!(open.get(&sid(demo.applied)).is_some());
    let closed = session
        .state_at(end)
        .expect("the state reads")
        .expect("a state");
    assert!(closed.get(&sid(demo.surface)).is_none(), "closed");
    assert!(closed.get(&sid("silences.cut[4]")).is_some());
    assert!(session.frame_times().len() >= 20);

    // the video's frames are in the file as images with their size and
    // no pixels (the recorder keeps the scene and the bindings, never the
    // atlas): the file written again counts them, the scene read back
    // draws none of them rather than another image's pixels
    let (_, written) = session.to_bytes().expect("the session writes again");
    assert!(
        written.images_without_pixels >= 3,
        "the frames of the video and of the two cuts: {written:?}"
    );

    // the inspector reads why the cut was applied: the act rested on the
    // agent's look at the whole page, the decision it read had not moved,
    // and it completed
    let inspector = Inspector::new(&session);
    let apply = &receipts[4];
    let why = inspector
        .why(apply.origin, apply.receipt.id)
        .expect("it reads")
        .expect("the request is in the session");
    let Basis::Observed {
        revision, scope, ..
    } = &why.basis
    else {
        panic!("the look is the basis: {:?}", why.basis);
    };
    assert_eq!(*revision, apply.receipt.request.observed.revision);
    assert_eq!(*scope, None);
    let decision = why
        .between
        .reads
        .iter()
        .find(|r| r.at.to_string() == "silences.cut[4]:decision")
        .expect("apply_cut reads the decision");
    assert_eq!(decision.seen, text("pending"));
    assert!(decision.moves.is_empty(), "{:?}", decision.moves);
    assert!(matches!(why.ending, Ending::Completed { .. }));
}

#[test]
fn the_demo_by_the_rules_holds_the_script_the_actors_the_frames_and_the_surface() {
    holds(&RULES);
}

#[test]
fn the_demo_by_the_claude_holds_the_same_script_over_its_own_surface() {
    holds(&CLAUDE);
}
