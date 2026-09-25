//! The composite capabilities on the Silences page (task-70 phase 3):
//! looking at a cut and applying it, twice, is offered to the person as
//! one action; only a person accepts it (an agent is refused, here and
//! through the real port); declined, it goes and is not asked again in
//! the session. Installed, it is an action of the app on every cut, and,
//! with the review open, a button in each cut of it.

use engine::gpu::image::load_image_rgba;
use sem::{Actor, Rejection, Request, RequestId, SemId, Stage, StateRef, Value};
use uir::{Capabilities, GenerationPolicy, verify_json};

use super::super::SilencesPage;
use super::super::generated::page::Page;
use super::super::semantics::ms;
use super::agent::{Dir, act, gallery, sid, snapshot};
use super::{TURN, center, click, dispatch_as_test, frame, setup, stage, tester, wide};
use crate::view::apps::app::TodoPage;
use engine::theme::Theme;

const BAND: &str = "silences_surface.composite";
const INSTALL: &str = "silences_surface.composite.install";
const DISMISS: &str = "silences_surface.composite.dismiss";

/// The page over the fake worker with a frame for every cut already
/// decoded: a preview ends at once.
fn staged() -> (TodoPage, SilencesPage, Theme) {
    let (todo, mut page, theme) = setup(false);
    let middles: Vec<u64> = page
        .model
        .cuts()
        .iter()
        .map(|c| ms(c.segment().middle()))
        .collect();
    for at in middles {
        let flat = load_image_rgba(4, 4, vec![128; 64]).map_err(|e| e.to_string());
        page.frames.insert(at, flat);
    }
    (todo, page, theme)
}

/// `actor` looks at the cut and applies it, as the page's own input does.
fn review(page: &mut SilencesPage, todo: &mut TodoPage, actor: &Actor, cut: u64) {
    for action in ["preview_segment", "apply_cut"] {
        let seen = page.snapshot().unwrap();
        let target = format!("silences.cut[{cut}]");
        let req = Request::observe(&seen, actor.clone(), target.as_str(), action, None).unwrap();
        let id = page.dispatch(req, todo);
        assert!(
            matches!(stage(todo, id), Stage::Completed(_)),
            "{target} {action}"
        );
    }
}

/// `actor` answers on the band's button `id`, over the whole page.
fn answer(page: &mut SilencesPage, todo: &mut TodoPage, actor: Actor, id: &str) -> Stage {
    let seen = page.projection().unwrap();
    let action = if id == INSTALL { "install" } else { "dismiss" };
    let req = Request::observe(&seen, actor, id, action, None).unwrap();
    let receipt = page.act(req, todo, false).expect("taken");
    receipt.stage().cloned().expect("a stage")
}

fn offered(page: &SilencesPage) -> Option<(Value, Value)> {
    let seen = page.projection().unwrap();
    let at = |key: &str| seen.value(&StateRef::new(sid(BAND), key.parse().unwrap()));
    Some((at("sequence")?.clone(), at("repeats")?.clone()))
}

fn composites(page: &SilencesPage) -> Vec<String> {
    let offer = &page.generated.offer;
    offer
        .composer
        .installed()
        .iter()
        .map(|c| c.name.to_string())
        .collect()
}

#[test]
fn looked_at_and_applied_twice_the_sequence_is_offered_and_nothing_installs() {
    let (mut todo, mut page, _) = staged();
    review(&mut page, &mut todo, &tester(), 1);
    assert_eq!(offered(&page), None, "once is no repetition");
    review(&mut page, &mut todo, &tester(), 2);
    let (sequence, repeats) = offered(&page).expect("offered");
    assert_eq!(sequence, Value::from("preview segment, then apply cut"));
    assert_eq!(repeats, Value::Int(2));
    // offered, not installed: no composite on the cuts, none kept
    assert!(composites(&page).is_empty());
    let app = page.snapshot().unwrap();
    let cut = app.get(&sid("silences.cut[1]")).unwrap();
    let names: Vec<&str> = cut.actions.iter().map(|a| a.name.as_str()).collect();
    assert_eq!(names, ["preview_segment", "apply_cut", "reject_cut"]);
    // the band is the row's, not the app's: a surface never binds it
    assert!(app.get(&sid(BAND)).is_none());
}

#[test]
fn the_person_makes_it_one_action_with_a_click() {
    let (mut todo, mut page, theme) = staged();
    review(&mut page, &mut todo, &Actor::Human, 1);
    review(&mut page, &mut todo, &Actor::Human, 2);
    let c = frame(&mut page, wide(), &theme);
    let at = center(&c, INSTALL);
    click(&mut page, &mut todo, wide(), &theme, at);
    assert_eq!(composites(&page), ["preview_segment_then_apply_cut"]);
    assert_eq!(offered(&page), None, "answered, the band goes");
    // the click was the person's own request
    let last = (1..=20)
        .rev()
        .filter_map(|n| todo.receipt(RequestId::parse(&n.to_string()).unwrap()))
        .find(|r| r.request.target == sid(INSTALL))
        .expect("the install's receipt");
    assert_eq!(last.request.actor, Actor::Human);
    assert!(matches!(last.stage(), Some(Stage::Completed(o))
        if o.result.get("installed") == Some(&Value::from("preview_segment_then_apply_cut"))));
}

#[test]
fn an_agent_is_refused_and_the_offer_stays() {
    let (mut todo, mut page, _) = staged();
    let bot = Actor::Agent {
        name: "bot".to_owned(),
    };
    review(&mut page, &mut todo, &bot, 1);
    review(&mut page, &mut todo, &bot, 2);
    assert!(
        offered(&page).is_some(),
        "an agent's repetition is offered to the person"
    );
    for id in [INSTALL, DISMISS] {
        let refused = answer(&mut page, &mut todo, bot.clone(), id);
        assert_eq!(
            refused,
            Stage::Rejected(Rejection::NotAuthorized { actor: bot.clone() })
        );
    }
    assert!(composites(&page).is_empty());
    assert!(offered(&page).is_some());
    // a person accepts the same offer
    assert!(matches!(
        answer(&mut page, &mut todo, Actor::Human, INSTALL),
        Stage::Completed(_)
    ));
    assert_eq!(composites(&page).len(), 1);
}

#[test]
fn declined_it_goes_and_the_same_sequence_is_not_offered_again() {
    let (mut todo, mut page, _) = staged();
    review(&mut page, &mut todo, &tester(), 1);
    review(&mut page, &mut todo, &tester(), 2);
    assert!(matches!(
        answer(&mut page, &mut todo, tester(), DISMISS),
        Stage::Completed(_)
    ));
    assert_eq!(offered(&page), None);
    // the cuts put back, then looked at and applied twice again
    for cut in [1, 2] {
        let target = format!("silences.cut[{cut}]");
        let id = dispatch_as_test(&mut page, &mut todo, &target, "reject_cut", None);
        assert!(matches!(stage(&todo, id), Stage::Completed(_)));
    }
    review(&mut page, &mut todo, &tester(), 1);
    review(&mut page, &mut todo, &tester(), 2);
    let found = page.generated.offer.composer.offer(|_| true).cloned();
    assert_eq!(found.map(|f| f.repeats), None, "declined in this session");
    assert_eq!(offered(&page), None);
    assert!(composites(&page).is_empty());
}

/// The acceptance through the port: an agent repeats the review of both
/// cuts, observes the offer the person is shown, and asks to install it;
/// the dispatcher refuses it (`not_authorized`), and nothing is
/// installed.
#[test]
fn through_the_port_an_agent_does_not_accept() {
    let dir = Dir(std::env::temp_dir().join(format!("plev-t70c-{}-offer", std::process::id())));
    let (seen, view) = gallery(&dir.0, |path| {
        let mut c = wire::Client::connect(&path).unwrap();
        c.set_timeout(Some(20_000)).unwrap();
        c.hello("claude").unwrap();
        let mut stages = Vec::new();
        for cut in [1, 2] {
            for action in ["preview_segment", "apply_cut"] {
                let now = snapshot(c.observe(wire::Observe::default()).unwrap());
                let target = format!("silences.cut[{cut}]");
                stages.push(act(&mut c, &target, action, now.revision));
            }
        }
        let now = snapshot(c.observe(wire::Observe::default()).unwrap());
        let offer = now.nodes.iter().any(|n| n.id == sid(BAND));
        let install = act(&mut c, INSTALL, "install", now.revision);
        (stages, offer, install)
    });
    let (stages, offer, install) = seen;
    for done in stages {
        assert_eq!(done, ["requested", "accepted", "executing", "completed"]);
    }
    assert!(offer, "the agent sees the offer");
    assert_eq!(install, ["requested", "rejected"]);
    let mut view = view;
    view.tick(TURN);
    assert!(composites(&view.panes.silences).is_empty());
    assert!(offered(&view.panes.silences).is_some());
}

/// The review of the checkpoint (task-69), verified against the app as it
/// is and shown as a proposal shows it.
fn review_open(page: &mut SilencesPage) -> SemId {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../surface/tests/fixtures/showcase-silences-review.ir.json");
    let json = std::fs::read_to_string(&path).expect("the review fixture");
    let app = page.snapshot().expect("the app projects");
    let caps = Capabilities::from_snapshot(&app);
    let ir = verify_json(&json, &caps, &GenerationPolicy::default()).expect("verified");
    let (id, _) = Page { app: page }
        .show(ir, GenerationPolicy::default())
        .expect("the surface shows");
    id
}

const COMPOSITE: &str = "preview_segment_then_apply_cut";

#[test]
fn installed_it_is_an_action_on_every_cut_and_a_button_in_the_open_review() {
    let (mut todo, mut page, _) = staged();
    let surface = review_open(&mut page);
    review(&mut page, &mut todo, &tester(), 1);
    review(&mut page, &mut todo, &tester(), 2);
    let installed = answer(&mut page, &mut todo, tester(), INSTALL);
    let Stage::Completed(outcome) = installed else {
        panic!("installed: {installed:?}");
    };
    assert_eq!(
        outcome.result.get("promoted"),
        Some(&Value::from(
            "promote preview_segment_then_apply_cut on silences.cut[{item}] into review.cut[{item}]"
        ))
    );

    // an action of the app, in its registry, on each cut
    let app = page.snapshot().unwrap();
    for cut in [1, 2] {
        let at = format!("silences.cut[{cut}]");
        let node = app.get(&sid(&at)).unwrap();
        let composite = node
            .actions
            .iter()
            .find(|a| a.name.as_str() == COMPOSITE)
            .unwrap_or_else(|| panic!("on {at}"));
        let reads: Vec<String> = composite.reads.iter().map(ToString::to_string).collect();
        assert_eq!(reads, ["silences.video:shown", &format!("{at}:decision")]);
        assert_eq!(
            composite.label.as_deref(),
            Some("preview segment, then apply cut")
        );
        assert_eq!(composite.policy, sem::Policy::default());
    }

    // promoted by the host into each cut of the review, as the tester
    // asked, with its inverse in the history
    let whole = page.projection().unwrap();
    for cut in [1, 2] {
        let button = format!("review.cut[{cut}].{COMPOSITE}");
        let node = whole
            .get(&sid(&button))
            .unwrap_or_else(|| panic!("{button}"));
        assert_eq!(node.label, "preview segment, then apply cut");
    }
    let changes = page.generated.host.changes(&surface);
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].by, tester());
}

#[test]
fn without_a_surface_it_is_installed_and_nothing_is_promoted() {
    let (mut todo, mut page, _) = staged();
    review(&mut page, &mut todo, &tester(), 1);
    review(&mut page, &mut todo, &tester(), 2);
    let Stage::Completed(outcome) = answer(&mut page, &mut todo, tester(), INSTALL) else {
        panic!("installed");
    };
    assert!(outcome.result.get("promoted").is_none());
    assert!(outcome.result.get("not_promoted").is_none());
    assert_eq!(composites(&page), [COMPOSITE]);
}
