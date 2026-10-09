//! The person's preferences of accessibility on the Silences page
//! (task-70 phase 4). Only a person opens the band ("accessibility"
//! through the row; an agent is refused) and only a person presses its
//! buttons (an agent is refused `not_authorized`, here and through the
//! real port); a preference turned on changes the review open, in the
//! person's name, and turned off gives it back. What leaves: the page's
//! projection is the same with and without preferences, and so is the
//! prompt a model gets. And the demo of the regress: the review by the
//! rules with larger text and more contrast, in the default window. The
//! contrast of the review, measured pair by pair, is `contrast.rs`. Since
//! task-75 phase 2a a person with a screen reader hears which preference
//! is on, in the reader's snapshot alone.

use engine::control::accessibility::accesskit::{Action, ActionRequest, Toggled, TreeId, TreeUpdate};
use engine::control::accessibility::{self, Assist, Chrome, Shown};
use engine::drawing::compositor::{Compositor, LayerId, SceneNode};
use engine::styling::theme::Theme;
use accord::{Actor, Rejection, Request, Stage, Value};
use plevir::{Preference, Preferences};

use super::super::super::SilencesPage;
use super::super::agent::{Dir, act as port_act, gallery_with, snapshot};
use super::super::{TURN, setup, sid, stage, tester};
use crate::views::ShowcaseView;
use crate::views::apps::todo::TodoPage;
use crate::views::dispatcher::PERSON_ONLY;

const RULES: &str = "the list with the video and the counts";
const BAND: &str = "silences_surface.access";
const LARGER: &str = "silences_surface.access.larger_text";
const CONTRAST: &str = "silences_surface.access.more_contrast";
const DONE: &str = "silences_surface.access.done";

fn bot() -> Actor {
    Actor::Agent {
        name: "bot".to_owned(),
    }
}

fn text(s: &str) -> Option<Value> {
    Some(Value::Text(s.to_owned()))
}

/// The review by the rules, open, asked by a person through the row.
pub(super) fn review() -> (TodoPage, SilencesPage) {
    let (mut todo, mut page, _) = setup(false);
    page.generated.proposing.intent.set_text(RULES);
    let asked = page.propose(&mut todo).expect("dispatched");
    assert!(matches!(stage(&todo, asked), Stage::Completed(_)));
    (todo, page)
}

/// `action` on `target` by `actor`, observed on the whole page.
fn act(
    page: &mut SilencesPage,
    todo: &mut TodoPage,
    actor: &Actor,
    target: &str,
    action: &str,
    param: Option<Value>,
) -> Stage {
    let seen = page.projection().expect("the page projects");
    let req = Request::observe(&seen, actor.clone(), target, action, param).unwrap();
    let receipt = page.act(req, todo, false).expect("a receipt");
    receipt.stage().expect("a stage").clone()
}

/// `intent` asked through the row by `actor`.
fn ask(page: &mut SilencesPage, todo: &mut TodoPage, actor: &Actor, intent: &str) -> Stage {
    let wrote = act(
        page,
        todo,
        actor,
        "silences_surface.intent",
        "set_value",
        text(intent),
    );
    assert!(matches!(wrote, Stage::Completed(_)), "{wrote:?}");
    act(
        page,
        todo,
        actor,
        "silences_surface.propose",
        "propose",
        None,
    )
}

fn has(page: &SilencesPage, id: &str) -> bool {
    page.projection().unwrap().get(&sid(id)).is_some()
}

#[test]
fn only_a_person_opens_the_band_and_presses_its_buttons() {
    let (mut todo, mut page) = review();
    assert!(!has(&page, BAND), "closed: nothing projected");

    // an agent asks for it through the row, as it asks for a change
    let Stage::Failed { error } = ask(&mut page, &mut todo, &bot(), "accessibility") else {
        panic!("refused");
    };
    assert!(
        error.contains("the accessibility preferences are the person's"),
        "{error}"
    );
    assert!(!has(&page, BAND));

    // a person opens it: five buttons, kept for a person, no state
    let opened = ask(&mut page, &mut todo, &tester(), "Acessibilidade");
    assert!(matches!(opened, Stage::Completed(_)), "{opened:?}");
    let snap = page.projection().unwrap();
    let band = snap.get(&sid(BAND)).expect("the band");
    assert!(band.state.is_empty() && band.actions.is_empty());
    let buttons: Vec<_> = snap
        .nodes()
        .values()
        .filter(|n| n.parent.as_ref() == Some(&sid(BAND)))
        .collect();
    let labels: Vec<&str> = buttons.iter().map(|n| n.label.as_str()).collect();
    assert_eq!(
        labels,
        ["larger text", "more contrast", "denser", "sparser", "done"]
    );
    for b in &buttons {
        assert!(b.state.is_empty(), "{}: no state", b.id);
        assert_eq!(b.actions.len(), 1);
        assert_eq!(b.actions[0].name.as_str(), "activate");
        assert_eq!(b.actions[0].policy.actors, PERSON_ONLY);
    }

    // an agent pressing is refused before anything runs
    let refused = act(&mut page, &mut todo, &bot(), LARGER, "activate", None);
    assert_eq!(
        refused,
        Stage::Rejected(Rejection::NotAuthorized { actor: bot() })
    );
    assert_eq!(page.generated.host.preferences(), Preferences::default());

    // the person turns larger text on: the review takes it, in their name
    let review = sid("proposed");
    let before = page.generated.host.get(&review).unwrap().verified().clone();
    let Stage::Completed(outcome) = act(&mut page, &mut todo, &tester(), LARGER, "activate", None)
    else {
        panic!("pressed");
    };
    assert!(outcome.result.is_empty(), "the outcome says nothing of it");
    assert!(page.generated.host.preferences().larger_text);
    let changes = page.generated.host.changes(&review);
    assert!(!changes.is_empty());
    assert!(
        changes
            .iter()
            .all(|c| { c.by == tester() && c.preference == Some(Preference::LargerText) })
    );
    // again: off, the review as it was
    act(&mut page, &mut todo, &tester(), LARGER, "activate", None);
    assert_eq!(page.generated.host.preferences(), Preferences::default());
    let back = page.generated.host.get(&review).unwrap().verified();
    assert_eq!(back, &before);
    // done: the band goes
    act(&mut page, &mut todo, &tester(), DONE, "activate", None);
    assert!(!has(&page, BAND));
}

/// Through the real port: the person left the band open; the agent sees
/// it, and pressing it is refused; asking for it through the row fails;
/// nothing is turned on.
#[test]
fn through_the_port_an_agent_turns_nothing_on() {
    let dir = Dir(std::env::temp_dir().join(format!("plev-t70d-{}-access", std::process::id())));
    let open_band = |view: &mut ShowcaseView| view.panes.silences.generated.access.open();
    let (seen, view) = gallery_with(&dir.0, open_band, |path| {
        let mut c = socket::Client::connect(&path).unwrap();
        c.set_timeout(Some(20_000)).unwrap();
        c.hello("claude").unwrap();
        let now = snapshot(c.observe(socket::Observe::default()).unwrap());
        let band = now
            .nodes
            .iter()
            .filter(|n| n.id == sid(BAND) || n.parent.as_ref() == Some(&sid(BAND)))
            .map(|n| n.state.is_empty())
            .collect::<Vec<_>>();
        let pressed = port_act(&mut c, LARGER, "activate", now.revision);
        let contrast = port_act(&mut c, CONTRAST, "activate", now.revision);
        let now = snapshot(c.observe(socket::Observe::default()).unwrap());
        let wrote = {
            let mut act = socket::Act::from_revision(
                sid("silences_surface.intent"),
                "set_value".parse().unwrap(),
                now.revision,
            );
            act.param = Some(Value::Text("accessibility".into()));
            c.act(act, |_| {}).unwrap()
        };
        let now = snapshot(c.observe(socket::Observe::default()).unwrap());
        let asked = port_act(&mut c, "silences_surface.propose", "propose", now.revision);
        (band, pressed, contrast, wrote.len(), asked)
    });
    let (band, pressed, contrast, wrote, asked) = seen;
    assert_eq!(band, [true; 6], "the band and its buttons, no state");
    assert_eq!(pressed, ["requested", "rejected"]);
    assert_eq!(contrast, ["requested", "rejected"]);
    assert!(wrote > 0);
    assert_eq!(asked.last(), Some(&"failed"));
    assert_eq!(
        view.panes.silences.generated.host.preferences(),
        Preferences::default()
    );
}

/// The value never leaves through the port: with the band closed, the
/// page projects the same thing, revision and all, with and without the
/// preferences, while the review drawn changed.
#[test]
fn the_page_projects_the_same_with_and_without_the_preferences() {
    let (mut todo, mut page) = review();
    let seen = page.projection().unwrap();
    let review = sid("proposed");
    let drawn = page.generated.host.get(&review).unwrap().verified().clone();
    ask(&mut page, &mut todo, &tester(), "accessibility");
    for button in [LARGER, CONTRAST, DONE] {
        let pressed = act(&mut page, &mut todo, &tester(), button, "activate", None);
        assert!(matches!(pressed, Stage::Completed(_)), "{button}");
    }
    let now = page.projection().unwrap();
    assert_ne!(
        page.generated.host.get(&review).unwrap().verified(),
        &drawn,
        "the review changed"
    );
    // the row's intent is the person's text, the status did not move
    let intent = "silences_surface.intent:text".parse().unwrap();
    assert_eq!(now.value(&intent), text("accessibility").as_ref());
    page.generated.proposing.intent.set_text(RULES);
    let now = page.projection().unwrap();
    assert_eq!(now, seen);
    assert_eq!(now.revision(), seen.revision());
}

/// The font size of the text run `s`, the last drawn: the surface's (the
/// fixed screen, drawn before it, shows the same range).
fn size_of(c: &Compositor, s: &str) -> Option<f32> {
    let nodes = c.layer(LayerId::DEFAULT).map_or(&[][..], |l| l.nodes());
    nodes.iter().rev().find_map(|n| match n {
        SceneNode::Text { key, .. } if key.text == s => Some(f32::from_bits(key.font_size_bits)),
        _ => None,
    })
}

fn settle(view: &mut ShowcaseView) -> Compositor {
    let mut c = Compositor::new();
    view.render(&mut c);
    let mut steps = 0;
    while view.tick(TURN) {
        view.render(&mut c);
        steps += 1;
        assert!(steps < 600, "the page never settles");
    }
    view.render(&mut c);
    c
}

/// `action` on `target` by the regress harness, through the gallery.
fn harness(view: &mut ShowcaseView, target: &str, action: &str, param: Option<Value>) -> Stage {
    let seen = view.semantics().expect("the page projects");
    let who = Actor::Test {
        name: "regress".to_owned(),
    };
    let req = Request::observe(&seen, who, target, action, param).unwrap();
    view.act(req).expect("taken").stage().unwrap().clone()
}

/// The demo of the regress, at the default window: the review by the
/// rules, then "accessibility", larger text, more contrast and done, as
/// the person asks: the range of each cut one size up, the badge filled,
/// the band gone, and the surface and the row projecting what they did.
#[test]
fn the_demo_larger_text_and_more_contrast_over_the_review_by_the_rules() {
    let mut view = ShowcaseView::new(1280.0, 800.0);
    view.jump_to_page("silences");
    harness(
        &mut view,
        "silences_surface.intent",
        "set_value",
        text(RULES),
    );
    harness(&mut view, "silences_surface.propose", "propose", None);
    let c = settle(&mut view);
    // the surface and the row as projected (the fixed screen's worker
    // moves on its own time)
    let generated = |view: &ShowcaseView| {
        let snap = view.semantics().unwrap();
        let ours = |id: &str| id.starts_with("proposed") || id.starts_with("silences_surface");
        snap.nodes()
            .values()
            .filter(|n| ours(n.id.as_str()))
            .cloned()
            .collect::<Vec<_>>()
    };
    let seen = generated(&view);
    let range = "0:03.0 to 0:05.0";
    let body = size_of(&c, range).expect("the range drawn");
    let theme = Theme::hoff_core();
    assert_eq!(body, theme.typography.body().font_size);
    for (target, action, param) in [
        (
            "silences_surface.intent",
            "set_value",
            text("accessibility"),
        ),
        ("silences_surface.propose", "propose", None),
        (LARGER, "activate", None),
        (CONTRAST, "activate", None),
        (DONE, "activate", None),
    ] {
        let done = harness(&mut view, target, action, param);
        assert!(matches!(done, Stage::Completed(_)), "{target}: {done:?}");
    }
    harness(
        &mut view,
        "silences_surface.intent",
        "set_value",
        text(RULES),
    );
    let c = settle(&mut view);
    assert_eq!(
        generated(&view),
        seen,
        "the surface and the row project the same"
    );
    assert_eq!(
        size_of(&c, range),
        Some(theme.typography.title().font_size),
        "one size up"
    );
    assert!(
        view.panes
            .silences
            .generated
            .host
            .preferences()
            .more_contrast
    );
}

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod claude {
    use std::path::PathBuf;
    use std::time::{Duration, Instant};

    use ::claude::ClaudeCli;
    use accord::RequestId;

    use super::*;
    use crate::views::apps::surface::asking::Claude;

    const DENSER: &str = r#"{"ops": [{"op": "density", "step": "denser"}]}"#;

    /// The fake claude in a directory of its own, answering `good`.
    fn fake(page: &mut SilencesPage, good: &str) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let result = serde_json::json!({
            "type": "result", "subtype": "success", "is_error": false,
            "num_turns": 1, "total_cost_usd": 0.05, "result": good
        })
        .to_string();
        std::fs::write(dir.path().join("mode"), "ok").unwrap();
        std::fs::write(dir.path().join("good"), result).unwrap();
        std::fs::write(dir.path().join("bad"), "").unwrap();
        let mut cli = ClaudeCli::new();
        cli.program =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../rust/claude/tests/fakes/claude");
        cli.dir = dir.path().to_owned();
        cli.timeout = Duration::from_secs(20);
        page.generated.proposing.set_claude(Claude::with(cli));
        dir
    }

    fn pump(page: &mut SilencesPage, todo: &mut TodoPage, asked: RequestId) {
        let started = Instant::now();
        while !todo.receipt(asked).unwrap().is_finished() {
            assert!(started.elapsed() < Duration::from_secs(20), "it answered");
            page.wake(TURN, todo);
            std::thread::sleep(Duration::from_millis(25));
        }
    }

    /// The first prompt a person's `intent` sends the Claude over the
    /// review by the rules, with `preferences` on or not.
    fn prompt(intent: &str, preferences: Preferences) -> String {
        let (mut todo, mut page) = review();
        let app = page.snapshot().unwrap();
        page.generated
            .host
            .prefer(preferences, Actor::Human, &app)
            .unwrap();
        let dir = fake(&mut page, DENSER);
        page.generated.proposing.intent.set_text(intent);
        let asked = page.propose(&mut todo).expect("dispatched");
        pump(&mut page, &mut todo, asked);
        std::fs::read_to_string(dir.path().join("prompt.1")).expect("asked")
    }

    /// The value never enters a model's prompt: the prompt of a change
    /// and of a new surface is the same, byte for byte, with the
    /// preferences on and off.
    #[test]
    fn a_model_s_prompt_is_the_same_with_and_without_the_preferences() {
        let on = Preferences {
            larger_text: true,
            more_contrast: true,
            density: Some(plevir::Step::Sparser),
        };
        for intent in ["too much information", "the cuts with a frame each"] {
            let off = prompt(intent, Preferences::default());
            assert!(off.contains("proposed"), "the review is in it: {intent}");
            assert_eq!(prompt(intent, on), off, "{intent}");
        }
    }
}

// -- what a screen reader hears (task-75 phase 2a) --------------------------

const DENSER_BUTTON: &str = "silences_surface.access.denser";
const SPARSER: &str = "silences_surface.access.sparser";

/// The window of a reader: what the bridge is shown this frame, and its
/// tree.
fn reader(view: &mut ShowcaseView) -> (TreeUpdate, Shown) {
    let mut c = Compositor::new();
    view.render(&mut c);
    let snapshot = view.reader_semantics().expect("the page projects");
    let chrome = Chrome {
        title: "plev, design system showcase",
        scale: 2.0,
        size: (2560, 1600),
    };
    let (update, ids) = accessibility::build(&snapshot, &c, &chrome);
    (update, Shown { snapshot, ids })
}

/// How a reader hears each button of the band: its toggled state, the
/// state of a toggle button (none: a plain button).
fn heard(view: &mut ShowcaseView) -> Vec<(&'static str, Option<Toggled>)> {
    let (u, shown) = reader(view);
    [LARGER, CONTRAST, DENSER_BUTTON, SPARSER, DONE]
        .into_iter()
        .map(|id| {
            let at = shown.ids.node(&sid(id));
            let node = u.nodes.iter().find(|(n, _)| Some(*n) == at);
            (id, node.and_then(|(_, n)| n.toggled()))
        })
        .collect()
}

/// A reader presses `id` through its tree.
fn press(view: &mut ShowcaseView, id: &str) -> Stage {
    let (_, shown) = reader(view);
    let req = ActionRequest {
        action: Action::Click,
        target_tree: TreeId::ROOT,
        target_node: shown.ids.node(&sid(id)).expect("in the tree"),
        data: None,
    };
    let Ok(Assist::Request(req)) = accessibility::translate(&req, &shown) else {
        panic!("{id} is pressed");
    };
    assert_eq!(req.actor, Actor::Assistive);
    view.act(req).expect("taken").stage().unwrap().clone()
}

/// The band opened by a person through the row, over the review by the
/// rules.
fn band_open() -> ShowcaseView {
    let mut view = ShowcaseView::new(1280.0, 800.0);
    view.jump_to_page("silences");
    for intent in [RULES, "accessibility"] {
        harness(
            &mut view,
            "silences_surface.intent",
            "set_value",
            text(intent),
        );
        let asked = harness(&mut view, "silences_surface.propose", "propose", None);
        assert!(matches!(asked, Stage::Completed(_)), "{intent}: {asked:?}");
    }
    view
}

/// The debt of task-70 phase 4: a person using a screen reader could not
/// hear which preference was on. The band's four preference buttons are
/// toggle buttons in the reader's tree, and follow what the person turns
/// on and off, the reader's own presses included.
#[test]
fn a_screen_reader_hears_which_preference_is_on() {
    let mut view = band_open();
    let off = Some(Toggled::False);
    assert_eq!(
        heard(&mut view),
        [
            (LARGER, off),
            (CONTRAST, off),
            (DENSER_BUTTON, off),
            (SPARSER, off),
            (DONE, None)
        ]
    );
    assert!(matches!(press(&mut view, LARGER), Stage::Completed(_)));
    assert!(view.panes.silences.generated.host.preferences().larger_text);
    let on = Some(Toggled::True);
    assert_eq!(
        heard(&mut view),
        [
            (LARGER, on),
            (CONTRAST, off),
            (DENSER_BUTTON, off),
            (SPARSER, off),
            (DONE, None)
        ]
    );
    // a person's press, and the reader hears that too
    harness(&mut view, CONTRAST, "activate", None);
    assert_eq!(heard(&mut view)[1], (CONTRAST, on));
    assert!(matches!(press(&mut view, LARGER), Stage::Completed(_)));
    assert_eq!(heard(&mut view)[0], (LARGER, off), "off again");
}

/// The preference a reader hears never leaves: with larger text on and
/// the band open, the agent observes the band's buttons through the real
/// port with no state; the view's projection and the gallery's (what the
/// port serves) carry none; the app's projection (what a model's prompt
/// is built from, `a_model_s_prompt_is_the_same_with_and_without_the_preferences`)
/// has no band at all. The reader's snapshot alone says it.
#[test]
fn the_preference_a_reader_hears_never_goes_out_through_the_port() {
    let dir = Dir(std::env::temp_dir().join(format!("plev-t75b-{}-reader", std::process::id())));
    let before = |view: &mut ShowcaseView| {
        harness(
            view,
            "silences_surface.intent",
            "set_value",
            text("accessibility"),
        );
        harness(view, "silences_surface.propose", "propose", None);
        harness(view, LARGER, "activate", None);
        assert!(view.panes.silences.generated.host.preferences().larger_text);
    };
    let (band, view) = gallery_with(&dir.0, before, |path| {
        let mut c = socket::Client::connect(&path).unwrap();
        c.set_timeout(Some(20_000)).unwrap();
        c.hello("claude").unwrap();
        let now = snapshot(c.observe(socket::Observe::default()).unwrap());
        now.nodes
            .iter()
            .filter(|n| n.parent.as_ref() == Some(&sid(BAND)))
            .map(|n| (n.id.to_string(), n.state.is_empty()))
            .collect::<Vec<_>>()
    });
    assert_eq!(band.len(), 5, "the band's buttons, observed");
    assert!(band.iter().all(|(_, bare)| *bare), "no state: {band:?}");

    let on = |snap: &accord::Snapshot| {
        snap.get(&sid(LARGER))
            .and_then(|n| n.state.get("on").cloned())
    };
    assert_eq!(
        view.reader_semantics().as_ref().and_then(on),
        Some(Value::Bool(true)),
        "the reader's snapshot says it"
    );
    assert_eq!(view.semantics().as_ref().and_then(on), None);
    assert_eq!(view.gallery_semantics().ok().as_ref().and_then(on), None);
    let app = view.panes.silences.snapshot().expect("the app projects");
    assert!(app.get(&sid(BAND)).is_none() && app.get(&sid(LARGER)).is_none());
}
