//! The Silences page as a capability (task-69 phase 4a): what it projects,
//! a click and a test request reaching the same state through the
//! gallery's dispatcher, `detect_silence` started and completed in the
//! wake with the fake worker (the probe's stderr, no ffmpeg), a preview
//! decoding a frame, the stale case, the inert background, the sliders,
//! two viewports, the chrome's routes, and the golden of the app's
//! projection. The agent through the real port is `agent.rs`; the real
//! ffmpeg is `ffmpeg.rs`, ignored; the generated surface beside the fixed
//! screen (phase 4b) is `generated.rs`.

mod agent;
mod composites;
mod ffmpeg;
mod generative;

use engine::widgets::prelude::{Rect, WidgetEvent};
use engine::drawing::compositor::{Compositor, SceneNode};
use engine::styling::theme::Theme;
use accord::{Actor, Change, Rejection, Request, RequestId, Role, SemId, Stage, StateRef, Value};
use crate::domain::silences::{Decision, PROBE_STDERR, parse};

use super::SilencesPage;
use super::fixed_page::semantics::Ids;
use super::fixed_page::worker::Worker;
use crate::views::EditKey;
use crate::views::apps::todo::TodoPage;

/// One turn of a 60 Hz display.
const TURN: f32 = 1.0 / 60.0;

fn sid(text: &str) -> SemId {
    SemId::parse(text).unwrap()
}

fn tester() -> Actor {
    Actor::Test {
        name: "silences".to_owned(),
    }
}

fn wide() -> Rect {
    Rect::new(288.0, 118.0, 900.0, 682.0)
}

fn narrow() -> Rect {
    Rect::new(16.0, 118.0, 358.0, 682.0)
}

/// The page over the fake worker that answers the probe's detection;
/// `hold` keeps every answer until `release`.
fn setup(hold: bool) -> (TodoPage, SilencesPage, Theme) {
    let page = SilencesPage::with(Worker::fake(parse(PROBE_STDERR), hold));
    (TodoPage::new(), page, Theme::hoff_core())
}

fn frame(page: &mut SilencesPage, content: Rect, theme: &Theme) -> Compositor {
    let mut c = Compositor::new();
    page.render(&mut c, content, theme);
    c
}

fn center(c: &Compositor, at: &str) -> (f32, f32) {
    let b = c
        .binding(&sid(at))
        .unwrap_or_else(|| panic!("{at} is bound"));
    let [x, y, w, h] = b.rect;
    (x + w / 2.0, y + h / 2.0)
}

fn click(
    page: &mut SilencesPage,
    todo: &mut TodoPage,
    content: Rect,
    theme: &Theme,
    (x, y): (f32, f32),
) {
    for event in [
        WidgetEvent::MouseMove { x, y },
        WidgetEvent::MouseDown { x, y },
        WidgetEvent::MouseUp { x, y },
    ] {
        page.handle_event(&event, content, theme, todo, false);
    }
}

fn value(page: &SilencesPage, at: &str) -> Option<Value> {
    let snap = page.snapshot().expect("the page projects");
    snap.value(&StateRef::parse(at).unwrap()).cloned()
}

fn text(s: &str) -> Option<Value> {
    Some(Value::Text(s.to_owned()))
}

/// An `Actor::Test` request over the page as it is now, through the
/// gallery's dispatcher; its id.
fn dispatch_as_test(
    page: &mut SilencesPage,
    todo: &mut TodoPage,
    target: &str,
    action: &str,
    param: Option<Value>,
) -> RequestId {
    let seen = page.snapshot().unwrap();
    let req = Request::observe(&seen, tester(), target, action, param).unwrap();
    page.dispatch(req, todo)
}

fn stage(todo: &TodoPage, id: RequestId) -> Stage {
    todo.receipt(id).unwrap().stage().unwrap().clone()
}

fn ask_as_test(
    page: &mut SilencesPage,
    todo: &mut TodoPage,
    target: &str,
    action: &str,
    param: Option<Value>,
) -> Stage {
    let id = dispatch_as_test(page, todo, target, action, param);
    stage(todo, id)
}

fn decisions(page: &SilencesPage) -> Vec<Decision> {
    page.model.cuts().iter().map(|c| c.decision()).collect()
}

/// The image nodes of the frame (the video box and the thumbnails).
fn images(c: &Compositor) -> usize {
    c.layer(engine::drawing::compositor::LayerId::DEFAULT)
        .map(|l| {
            l.nodes()
                .iter()
                .filter(|n| matches!(n, SceneNode::Image { .. }))
                .count()
        })
        .unwrap_or(0)
}

#[test]
fn the_page_projects_every_capability_reading_what_it_changes() {
    assert!(Ids::new().is_ok(), "the constant ids parse");
    let (_, page, _) = setup(false);
    let snap = page.snapshot().unwrap();
    let node = |text: &str| snap.get(&sid(text)).unwrap();
    let actions = |id: &str| -> Vec<(String, Vec<String>, bool)> {
        node(id)
            .actions
            .iter()
            .map(|a| {
                (
                    a.name.to_string(),
                    a.reads.iter().map(ToString::to_string).collect(),
                    a.enabled,
                )
            })
            .collect()
    };

    assert_eq!(node("silences").role, Role::Region);
    // the keys of the cuts, as the todo lists its items: a repeat keys
    // each instance by them
    assert_eq!(
        node("silences").state.get("cuts"),
        Some(&Value::List(vec!["1".into(), "2".into()]))
    );
    let video = node("silences.video");
    assert_eq!(video.role, Role::Media);
    assert_eq!(
        video.state.get("file"),
        Some(&Value::from("silences-test.mp4"))
    );
    assert_eq!(video.state.get("duration"), Some(&Value::Num(12.0)));
    assert_eq!(video.state.get("shown"), Some(&Value::Num(0.0)));
    // the frame refs: the media node and the instant in seconds
    assert_eq!(
        video.state.get("original"),
        Some(&Value::from("silences.video@0"))
    );
    assert_eq!(
        video.state.get("result"),
        Some(&Value::from("silences.video@0"))
    );

    for (id, v) in [
        ("silences.threshold", -30.0),
        ("silences.min_duration", 0.5),
    ] {
        assert_eq!(node(id).role, Role::Slider);
        assert_eq!(node(id).state.get("value"), Some(&Value::Num(v)));
        for (name, reads, _) in actions(id) {
            assert_eq!(reads, [format!("{id}:value")], "{name}");
        }
    }
    assert_eq!(
        actions("silences.detect"),
        [(
            "detect_silence".to_owned(),
            vec![
                "silences:cuts".to_owned(),
                "silences.threshold:value".to_owned(),
                "silences.min_duration:value".to_owned()
            ],
            true
        )]
    );

    let cut = node("silences.cut[1]");
    assert_eq!(cut.role, Role::ListItem);
    assert_eq!(cut.label, "0:03.0 to 0:05.0");
    assert_eq!(cut.state.get("decision"), text("pending").as_ref());
    assert_eq!(cut.state.get("start"), Some(&Value::Num(3.018594)));
    assert_eq!(cut.state.get("range"), text("0:03.0 to 0:05.0").as_ref());
    assert_eq!(
        cut.state.get("frame"),
        text("silences.video@4.017052").as_ref()
    );
    assert_eq!(
        actions("silences.cut[1]"),
        [
            (
                "preview_segment".to_owned(),
                vec!["silences.video:shown".to_owned()],
                true
            ),
            (
                "apply_cut".to_owned(),
                vec!["silences.cut[1]:decision".to_owned()],
                true
            ),
            (
                "reject_cut".to_owned(),
                vec!["silences.cut[1]:decision".to_owned()],
                true
            ),
        ]
    );
    let counts = node("silences.counts");
    assert_eq!(counts.role, Role::Status);
    assert_eq!(counts.label, "2 silences, 0 applied, 0 rejected, 0.0 s cut");
    assert_eq!(counts.state.get("pending"), Some(&Value::Int(2)));
    assert_eq!(
        actions("silences.apply_all"),
        [(
            "apply_all".to_owned(),
            vec!["silences.counts:pending".to_owned()],
            true
        )]
    );
    let status = node("silences.status");
    assert_eq!(status.state.get("busy"), Some(&Value::Bool(false)));
    assert_eq!(
        status.state.get("message"),
        text("2 silences from the recorded run at -30 dB, 0.5 s minimum: detect runs ffmpeg here")
            .as_ref()
    );
    assert_eq!(snap.focused(), None);
}

/// The acceptance: a click on a row's apply button and a test request on
/// the other row reach the same state through the one dispatcher; the
/// decision taken disables its own action and the screen follows.
#[test]
fn apply_by_click_and_reject_by_request_reach_the_same_state() {
    let (mut todo, mut page, theme) = setup(false);
    let content = wide();
    let c = frame(&mut page, content, &theme);
    let [x, y, w, h] = c.binding(&sid("silences.cut[1]")).unwrap().rect;
    // the apply button is the second from the row's right edge
    let (aw, _) = page.rows[0].apply.preferred_size(&theme);
    let (rw, _) = page.rows[0].reject.preferred_size(&theme);
    let apply_x = x + w - rw - theme.spacing.sm - aw / 2.0;
    click(
        &mut page,
        &mut todo,
        content,
        &theme,
        (apply_x, y + h / 2.0),
    );
    assert_eq!(decisions(&page), [Decision::Apply, Decision::Pending]);
    let receipt = todo.receipt(RequestId::parse("1").unwrap()).unwrap();
    assert_eq!(receipt.request.actor, Actor::Human);
    assert_eq!(receipt.request.target, sid("silences.cut[1]"));
    assert_eq!(receipt.request.action.as_str(), "apply_cut");
    assert!(matches!(receipt.stage(), Some(Stage::Completed(_))));

    let stage = ask_as_test(&mut page, &mut todo, "silences.cut[2]", "reject_cut", None);
    assert!(matches!(stage, Stage::Completed(_)), "{stage:?}");
    assert_eq!(decisions(&page), [Decision::Apply, Decision::Reject]);
    assert_eq!(value(&page, "silences.cut[1]:decision"), text("apply"));
    // the result's frame ref: shown at 0, outside every cut, the same
    // instant as the original's; inside the applied cut, its end
    assert_eq!(
        value(&page, "silences.video:result"),
        text("silences.video@0")
    );
    ask_as_test(
        &mut page,
        &mut todo,
        "silences.cut[1]",
        "preview_segment",
        None,
    );
    assert_eq!(
        value(&page, "silences.video:original"),
        text("silences.video@4.017052")
    );
    assert_eq!(
        value(&page, "silences.video:result"),
        text("silences.video@5.01551")
    );
    assert_eq!(value(&page, "silences.cut[2]:decision"), text("reject"));
    assert_eq!(value(&page, "silences.counts:applied"), Some(Value::Int(1)));
    assert_eq!(
        value(&page, "silences.counts:rejected"),
        Some(Value::Int(1))
    );
    assert_eq!(value(&page, "silences.counts:pending"), Some(Value::Int(0)));
    let cut_secs = value(&page, "silences.counts:cut_secs");
    assert!(matches!(cut_secs, Some(Value::Num(s)) if (s - 1.996916).abs() < 1e-6));

    // the decision taken is disabled on its own node
    let again = ask_as_test(&mut page, &mut todo, "silences.cut[1]", "apply_cut", None);
    assert!(
        matches!(again, Stage::Rejected(Rejection::Disabled { .. })),
        "{again:?}"
    );
    // and the counts leave apply all with nothing to do
    let none = ask_as_test(
        &mut page,
        &mut todo,
        "silences.apply_all",
        "apply_all",
        None,
    );
    assert!(
        matches!(none, Stage::Rejected(Rejection::Disabled { .. })),
        "{none:?}"
    );

    // the screen: the taken buttons read dark, the counts moved
    frame(&mut page, content, &theme);
    assert_eq!(
        page.rows[0].apply.variant,
        engine::widgets::prelude::ButtonVariant::Secondary
    );
    assert_eq!(
        page.rows[0].reject.variant,
        engine::widgets::prelude::ButtonVariant::Outline
    );
    assert_eq!(
        page.rows[1].reject.variant,
        engine::widgets::prelude::ButtonVariant::Secondary
    );
    assert_eq!(
        page.counts_text(),
        "2 silences, 1 applied, 1 rejected, 2.0 s cut"
    );
}

#[test]
fn apply_all_applies_the_pending_and_leaves_the_rejected() {
    let (mut todo, mut page, theme) = setup(false);
    ask_as_test(&mut page, &mut todo, "silences.cut[2]", "reject_cut", None);
    let content = wide();
    let c = frame(&mut page, content, &theme);
    click(
        &mut page,
        &mut todo,
        content,
        &theme,
        center(&c, "silences.apply_all"),
    );
    assert_eq!(decisions(&page), [Decision::Apply, Decision::Reject]);
    let Some(Stage::Completed(outcome)) = todo
        .receipt(RequestId::parse("2").unwrap())
        .unwrap()
        .stage()
    else {
        panic!("apply all completes");
    };
    assert_eq!(outcome.result.get("applied"), Some(&Value::Int(1)));
    // the next frame (the click asked for one) reads the button disabled
    frame(&mut page, content, &theme);
    assert!(
        page.apply_all.disabled,
        "nothing pending: the button reads disabled"
    );
}

/// `detect_silence` starts: the request stays open while the page keeps
/// taking requests, a second detect is refused, and the worker's answer
/// ends it in the wake with the cuts replaced (new ids) and the frames
/// asked for.
#[test]
fn detect_silence_starts_and_completes_in_the_wake_with_new_cuts() {
    let (mut todo, mut page, theme) = setup(true);
    ask_as_test(&mut page, &mut todo, "silences.cut[1]", "apply_cut", None);
    let id = dispatch_as_test(
        &mut page,
        &mut todo,
        "silences.detect",
        "detect_silence",
        None,
    );
    assert!(
        matches!(stage(&todo, id), Stage::Executing),
        "{:?}",
        stage(&todo, id)
    );
    assert_eq!(
        value(&page, "silences.status:busy"),
        Some(Value::Bool(true))
    );
    assert_eq!(
        value(&page, "silences.status:message"),
        text("detecting silences at -30 dB, 0.5 s minimum")
    );
    let again = ask_as_test(
        &mut page,
        &mut todo,
        "silences.detect",
        "detect_silence",
        None,
    );
    assert!(
        matches!(again, Stage::Rejected(Rejection::Disabled { .. })),
        "{again:?}"
    );
    // the page goes on: a cut is rejected while ffmpeg would be running
    let meanwhile = ask_as_test(&mut page, &mut todo, "silences.cut[2]", "reject_cut", None);
    assert!(matches!(meanwhile, Stage::Completed(_)));
    // nothing arrived yet: the wake asks to look again, no frame now
    let wake = page.wake(TURN, &mut todo);
    assert!(!wake.now);
    assert_eq!(wake.next, Some(super::fixed_page::worker::POLL));
    assert!(matches!(stage(&todo, id), Stage::Executing));

    page.worker.release();
    let wake = page.wake(TURN, &mut todo);
    assert!(wake.now, "the answer arrived: a frame");
    let Stage::Completed(outcome) = stage(&todo, id) else {
        panic!("the detection completes: {:?}", stage(&todo, id));
    };
    assert_eq!(outcome.result.get("cuts"), Some(&Value::Int(2)));
    assert!(matches!(outcome.result.get("seconds"), Some(Value::Num(_))));
    let ids: Vec<u64> = page.model.cuts().iter().map(|c| c.id()).collect();
    assert_eq!(ids, [3, 4], "a detection mints new ids");
    assert_eq!(decisions(&page), [Decision::Pending, Decision::Pending]);
    assert!(
        value(&page, "silences.cut[1]:decision").is_none(),
        "the old cut is gone from the projection"
    );

    // the frames the screen shows follow, one job at a time: the instant
    // shown and the middle of each cut; the status says so meanwhile
    assert!(page.worker.busy());
    assert_eq!(
        value(&page, "silences.status:message"),
        text("decoding the frame at 0:00.0")
    );
    let mut turns = 0;
    while page.worker.busy() {
        page.worker.release();
        page.wake(TURN, &mut todo);
        turns += 1;
        assert!(turns < 10, "the frames never end");
    }
    let message = value(&page, "silences.status:message");
    assert!(
        matches!(&message, Some(Value::Text(m)) if m.starts_with("2 silences in ") && m.ends_with(" ms at -30 dB, 0.5 s minimum")),
        "{message:?}"
    );
    let mut at: Vec<u64> = page.frames.keys().copied().collect();
    at.sort_unstable();
    assert_eq!(at, [0, 4017, 9009]);
    assert!(page.frames.values().all(Result::is_ok));
    let c = frame(&mut page, wide(), &theme);
    assert_eq!(images(&c), 3, "the video box and two thumbnails");
    assert_eq!(page.wake(TURN, &mut todo), engine::widgets::prelude::Wake::idle());
}

/// A preview: the row's click shows the cut's middle; its frame is
/// decoded off the thread and the request completes when it arrives; a
/// second preview of a decoded instant completes at once.
#[test]
fn preview_segment_shows_the_cut_and_decodes_its_frame() {
    let (mut todo, mut page, theme) = setup(true);
    let content = wide();
    let c = frame(&mut page, content, &theme);
    assert_eq!(images(&c), 0, "the seed has no frames");
    let [x, y, _, h] = c.binding(&sid("silences.cut[2]")).unwrap().rect;
    // the thumbnail's edge: not a button
    click(
        &mut page,
        &mut todo,
        content,
        &theme,
        (x + 10.0, y + h / 2.0),
    );
    let receipt = todo.receipt(RequestId::parse("1").unwrap()).unwrap();
    assert_eq!(receipt.request.action.as_str(), "preview_segment");
    assert!(matches!(receipt.stage(), Some(Stage::Executing)));
    let shown = value(&page, "silences.video:shown");
    assert!(
        matches!(shown, Some(Value::Num(t)) if (t - 9.009342).abs() < 1e-6),
        "{shown:?}"
    );
    assert_eq!(
        page.model.cut_at(page.model.shown()).map(|c| c.id()),
        Some(2)
    );

    page.worker.release();
    assert!(page.wake(TURN, &mut todo).now);
    let Some(Stage::Completed(outcome)) = todo
        .receipt(RequestId::parse("1").unwrap())
        .unwrap()
        .stage()
    else {
        panic!("the preview completes");
    };
    assert!(matches!(outcome.result.get("shown"), Some(Value::Num(t)) if (t - 9.009).abs() < 1e-6));
    let c = frame(&mut page, content, &theme);
    assert_eq!(images(&c), 2, "the video box and the cut's own thumbnail");
    // the other frames follow on their own; drain them
    let mut turns = 0;
    while page.worker.busy() {
        page.worker.release();
        page.wake(TURN, &mut todo);
        turns += 1;
        assert!(turns < 10);
    }
    assert_eq!(images(&frame(&mut page, content, &theme)), 3);

    // a preview of a decoded frame is done at once
    let done = ask_as_test(
        &mut page,
        &mut todo,
        "silences.cut[1]",
        "preview_segment",
        None,
    );
    assert!(matches!(done, Stage::Completed(_)), "{done:?}");
    assert_eq!(
        value(&page, "silences.video:shown"),
        Some(Value::Num(4.017052))
    );
    // a click on a mark of the band previews too (the band is in the
    // fixed screen's rect, under the proposing row)
    let fixed = page.halves(content, &theme).fixed;
    let mark = super::fixed_page::layout::mark_rect(
        super::fixed_page::layout::compute(&page, fixed, &theme).timeline,
        page.model.cuts()[1].segment(),
        12.0,
    );
    click(&mut page, &mut todo, content, &theme, mark.center());
    assert_eq!(
        page.model.cut_at(page.model.shown()).map(|c| c.id()),
        Some(2)
    );
}

/// An actor that decided on a decision another actor changed is refused
/// stale, with the delta; observed again it goes through.
#[test]
fn a_decision_taken_meanwhile_makes_the_request_stale() {
    let (mut todo, mut page, _) = setup(false);
    let seen = page.snapshot().unwrap();
    let stale = Request::observe(&seen, tester(), "silences.cut[1]", "reject_cut", None).unwrap();
    ask_as_test(&mut page, &mut todo, "silences.cut[1]", "apply_cut", None);
    let id = page.dispatch(stale, &mut todo);
    let Stage::Rejected(Rejection::Stale { changes, .. }) = stage(&todo, id) else {
        panic!("the reject must go stale: {:?}", stage(&todo, id));
    };
    let at = StateRef::parse("silences.cut[1]:decision").unwrap();
    assert!(matches!(
        &changes[..],
        [Change::StateChanged { at: a, before: Some(Value::Text(b)), after: Some(Value::Text(c)), .. }]
            if *a == at && b == "pending" && c == "apply"
    ));
    assert_eq!(
        decisions(&page)[0],
        Decision::Apply,
        "the first decision holds"
    );
    let fresh = ask_as_test(&mut page, &mut todo, "silences.cut[1]", "reject_cut", None);
    assert!(matches!(fresh, Stage::Completed(_)));
    assert_eq!(decisions(&page)[0], Decision::Reject);
    // a cut that is not there cannot even be observed
    let seen = page.snapshot().unwrap();
    assert!(Request::observe(&seen, tester(), "silences.cut[9]", "apply_cut", None).is_err());
}

#[test]
fn behind_an_overlay_the_page_is_inert() {
    let (mut todo, mut page, theme) = setup(false);
    let content = wide();
    let c = frame(&mut page, content, &theme);
    let at = center(&c, "silences.apply_all");
    page.set_inert(true);
    assert!(
        page.snapshot()
            .unwrap()
            .get(&sid("silences.cut[1]"))
            .unwrap()
            .inert
    );
    let stage = ask_as_test(&mut page, &mut todo, "silences.cut[1]", "apply_cut", None);
    assert!(
        matches!(stage, Stage::Rejected(Rejection::Inert { .. })),
        "{stage:?}"
    );
    for event in [
        WidgetEvent::MouseDown { x: at.0, y: at.1 },
        WidgetEvent::MouseUp { x: at.0, y: at.1 },
    ] {
        page.handle_event(&event, content, &theme, &mut todo, true);
    }
    assert_eq!(decisions(&page), [Decision::Pending, Decision::Pending]);
    assert!(!page.focus(&sid("silences.threshold")));
    page.set_inert(false);
    assert!(page.focus(&sid("silences.threshold")));
    assert_eq!(
        page.snapshot().unwrap().focused(),
        Some(&sid("silences.threshold"))
    );
}

/// The sliders: `set_value` within the range, a step up and down, an
/// out-of-range value refused, and the status saying the settings moved
/// away from the detection's.
#[test]
fn the_sliders_move_by_request_and_the_status_says_to_detect_again() {
    let (mut todo, mut page, _) = setup(false);
    let set = ask_as_test(
        &mut page,
        &mut todo,
        "silences.threshold",
        "set_value",
        Some(Value::Num(-40.0)),
    );
    assert!(matches!(set, Stage::Completed(_)), "{set:?}");
    assert_eq!(
        value(&page, "silences.threshold:value"),
        Some(Value::Num(-40.0))
    );
    let message = value(&page, "silences.status:message");
    assert!(
        matches!(&message, Some(Value::Text(m)) if m.ends_with("; the sliders moved to -40 dB, 0.5 s minimum: detect again")),
        "{message:?}"
    );
    let up = ask_as_test(
        &mut page,
        &mut todo,
        "silences.min_duration",
        "increment",
        None,
    );
    assert!(matches!(up, Stage::Completed(_)));
    assert!(
        matches!(value(&page, "silences.min_duration:value"), Some(Value::Num(v)) if (v - 0.6).abs() < 1e-6)
    );
    let down = ask_as_test(
        &mut page,
        &mut todo,
        "silences.threshold",
        "decrement",
        None,
    );
    assert!(matches!(down, Stage::Completed(_)));
    assert_eq!(
        value(&page, "silences.threshold:value"),
        Some(Value::Num(-41.0))
    );
    let out = ask_as_test(
        &mut page,
        &mut todo,
        "silences.threshold",
        "set_value",
        Some(Value::Num(12.0)),
    );
    assert!(
        matches!(out, Stage::Rejected(Rejection::InvalidParam { .. })),
        "{out:?}"
    );
    // at the top of the range a step up fails
    ask_as_test(
        &mut page,
        &mut todo,
        "silences.threshold",
        "set_value",
        Some(Value::Num(0.0)),
    );
    let top = ask_as_test(
        &mut page,
        &mut todo,
        "silences.threshold",
        "increment",
        None,
    );
    assert!(matches!(top, Stage::Failed { .. }), "{top:?}");
    // a detection carries the sliders' settings
    let mut page = SilencesPage::with(Worker::fake(parse(PROBE_STDERR), false));
    ask_as_test(
        &mut page,
        &mut todo,
        "silences.min_duration",
        "set_value",
        Some(Value::Num(1.0)),
    );
    ask_as_test(
        &mut page,
        &mut todo,
        "silences.detect",
        "detect_silence",
        None,
    );
    page.wake(TURN, &mut todo);
    assert_eq!(page.model.detected_with().min_duration, 1.0);
    // the frames that follow the detection end first
    let mut turns = 0;
    while page.worker.busy() {
        page.wake(TURN, &mut todo);
        turns += 1;
        assert!(turns < 10);
    }
    let message = value(&page, "silences.status:message");
    assert!(
        matches!(&message, Some(Value::Text(m)) if m.ends_with(" ms at -30 dB, 1.0 s minimum")),
        "{message:?}"
    );
}

/// Tab moves the keyboard focus over the sliders, the arrows nudge the
/// focused one as requests, Escape lets go; a click lets go too.
#[test]
fn tab_and_the_arrows_reach_the_sliders() {
    let (mut todo, mut page, theme) = setup(false);
    assert!(!page.handle_escape(), "nothing focused yet");
    assert!(!page.handle_edit_key(EditKey::Left, &mut todo));
    assert!(page.handle_edit_key(EditKey::Tab, &mut todo));
    assert_eq!(
        page.snapshot().unwrap().focused(),
        Some(&sid("silences.threshold"))
    );
    assert!(page.handle_edit_key(EditKey::Right, &mut todo));
    assert_eq!(
        value(&page, "silences.threshold:value"),
        Some(Value::Num(-29.0))
    );
    assert!(page.handle_edit_key(EditKey::Tab, &mut todo));
    assert_eq!(
        page.snapshot().unwrap().focused(),
        Some(&sid("silences.min_duration"))
    );
    assert!(page.handle_edit_key(EditKey::Left, &mut todo));
    assert!(
        matches!(value(&page, "silences.min_duration:value"), Some(Value::Num(v)) if (v - 0.4).abs() < 1e-6)
    );
    assert!(page.handle_escape());
    assert_eq!(page.snapshot().unwrap().focused(), None);
    assert!(page.handle_edit_key(EditKey::Tab, &mut todo));
    let content = wide();
    let c = frame(&mut page, content, &theme);
    click(
        &mut page,
        &mut todo,
        content,
        &theme,
        center(&c, "silences.counts"),
    );
    assert_eq!(
        page.snapshot().unwrap().focused(),
        None,
        "a click lets the focus go"
    );
    assert!(!page.handle_edit_key(EditKey::Backspace, &mut todo));
}

/// Wide, the video and the detection sit side by side over the cuts;
/// narrow, the three cards stack. Nothing is drawn past the content's
/// width, every capability is bound in both, and the region's height is
/// the page's.
#[test]
fn wide_and_narrow_viewports_lay_the_cards_out_and_bind_the_same_ids() {
    let (_, mut page, theme) = setup(false);
    let mut ids = Vec::new();
    for content in [wide(), narrow()] {
        // the fixed screen sits under the proposing row, at the whole
        // width while no surface is open
        let h = page.halves(content, &theme);
        assert!(h.surface.is_none());
        assert_eq!((h.fixed.x, h.fixed.w), (content.x, content.w));
        assert!(h.fixed.y >= h.row.band.bottom());
        let l = super::fixed_page::layout::compute(&page, h.fixed, &theme);
        let wide = content.w >= theme.size.card_min_w * 2.0 + theme.spacing.xl;
        if wide {
            assert_eq!(l.side_card.y, l.video_card.y, "side by side");
            assert!(l.side_card.x >= l.video_card.right());
            assert_eq!(l.side_card.h, l.video_card.h, "one top row");
        } else {
            assert_eq!(l.side_card.x, l.video_card.x, "stacked");
            assert!(l.side_card.y >= l.video_card.bottom());
        }
        assert!(l.cuts_card.y >= l.side_card.bottom());
        assert_eq!(l.rows.len(), 2);
        for r in [
            l.video_card,
            l.side_card,
            l.cuts_card,
            l.apply_all,
            l.detect,
        ]
        .into_iter()
        .chain(l.rows.iter().copied())
        {
            assert!(
                r.x >= content.x - 0.5 && r.right() <= content.right() + 0.5,
                "{r:?} inside {content:?}"
            );
        }
        assert!(l.apply_all.right() <= l.cuts_card.right());
        assert_eq!(
            page.content_height(content, &theme),
            h.fixed.y - content.y + l.height
        );
        assert_eq!(l.region.h, l.height);
        let c = frame(&mut page, content, &theme);
        let mut bound: Vec<String> = c.bindings().map(|(id, _)| id.to_string()).collect();
        bound.sort();
        ids.push(bound);
    }
    assert_eq!(ids[0], ids[1], "the same capabilities are bound");
    for id in [
        "silences",
        "silences.video",
        "silences.threshold",
        "silences.min_duration",
        "silences.detect",
        "silences.cut[1]",
        "silences.cut[2]",
        "silences.counts",
        "silences.apply_all",
        "silences.status",
        "silences_surface",
        "silences_surface.intent",
        "silences_surface.propose",
        "silences_surface.close",
    ] {
        assert!(ids[0].contains(&id.to_owned()), "{id} in {:?}", ids[0]);
    }
}

/// The app's projection (the fixed screen alone, without the proposing
/// row) is the golden of `crates/uir/fixtures`, the registry a generated
/// surface verifies against and the proposers read: the page as it
/// opens, seeded, no ffmpeg run, settled as the headless staging settles
/// it. `UIR_BLESS=1` writes it again.
#[test]
fn the_app_projection_is_the_uir_golden() {
    let mut view = crate::views::ShowcaseView::new(1200.0, 800.0);
    view.jump_to_page("silences");
    let mut c = Compositor::new();
    view.render(&mut c);
    let mut steps = 0;
    while view.tick(TURN) {
        view.render(&mut c);
        steps += 1;
        assert!(steps < 120, "at rest the page asks for nothing");
    }
    let snap = view.panes.silences.snapshot().expect("the app projects");
    assert!(
        snap.get(&sid("silences_surface.intent")).is_none(),
        "the row is the page's, not the app's"
    );
    let mut text = crate::shell::semantics_json(&snap, false).expect("json prints");
    text.push('\n');
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../uir/fixtures/showcase-silences.snapshot.json");
    if std::env::var_os("UIR_BLESS").is_some() {
        std::fs::write(&path, text).expect("the golden writes");
        return;
    }
    let kept = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(
        kept == text,
        "the projection moved away from the golden; regenerate it with \
         UIR_BLESS=1 cargo test -p showcase --lib silences::tests::the_app_projection, \
         then run cargo test -p uir -p propose -p surface"
    );
}

/// The chrome's routes: the gallery on the Silences page projects it,
/// routes an act and a focus, and draws it; without ffmpeg in the tests
/// a detect ends failed, saying why.
#[test]
fn the_gallery_routes_the_page() {
    let mut view = crate::views::ShowcaseView::new(1200.0, 800.0);
    view.jump_to_page("silences");
    assert_eq!(view.page_title(), "Silences");
    let snap = view.semantics().expect("the page projects");
    assert!(snap.get(&sid("silences.cut[1]")).is_some());
    let req = Request::observe(
        &snap,
        Actor::Assistive,
        "silences.cut[1]",
        "apply_cut",
        None,
    )
    .unwrap();
    let receipt = view.act(req).expect("the page takes it");
    assert!(matches!(receipt.stage(), Some(Stage::Completed(_))));
    assert_eq!(
        view.semantics()
            .unwrap()
            .value(&StateRef::parse("silences.cut[1]:decision").unwrap()),
        Some(&Value::Text("apply".into()))
    );
    assert!(view.assist_focus(&sid("silences.min_duration")));
    assert_eq!(
        view.semantics().unwrap().focused(),
        Some(&sid("silences.min_duration"))
    );
    assert!(view.handle_edit_key(EditKey::Tab));
    assert!(view.close_top_overlay(), "escape lets the focus go");
    let mut c = Compositor::new();
    view.render(&mut c);
    assert!(c.binding(&sid("silences.apply_all")).is_some());

    // no ffmpeg is looked for in the tests: a detect starts and fails
    let snap = view.semantics().unwrap();
    let req = Request::observe(&snap, tester(), "silences.detect", "detect_silence", None).unwrap();
    let receipt = view.act(req).unwrap();
    assert!(matches!(receipt.stage(), Some(Stage::Executing)));
    assert!(view.tick(TURN), "the answer arrives: a frame");
    let receipt = view.panes.todo.receipt(receipt.id).unwrap();
    let Some(Stage::Failed { error }) = receipt.stage() else {
        panic!("{:?}", receipt.stage());
    };
    assert_eq!(error, "ffmpeg is not looked for in the crate's tests");
    assert!(!view.tick(TURN), "at rest the page asks for nothing");
}
