//! The demo of task-70 phase 2, by the rules, with the review open: "tem
//! informação demais" over the Claude's review of the first round takes
//! the context out (the video and the counts cards, which bind no
//! action), groups each cut's buttons and tightens the spacing, and both
//! cuts come into the default window; "I can't compare these" is refused
//! with why (the regress keeps the images, `examples/regress/state.rs`).
//! What continues keeps its state: a cut applied before stays applied,
//! shown by the surface as before, and a slider in the middle of a drag
//! goes on dragging after the change.

use engine::widgets::prelude::{Rect, WidgetEvent};
use engine::drawing::compositor::Compositor;
use engine::drawing::gpu::image::{ImageHandle, load_image_rgba};
use accord::{Actor, Request, Stage, Value};

use super::super::super::SilencesPage;
use super::super::{TURN, frame, setup, sid, value, wide};
use super::{fixture, show};
use crate::views::ShowcaseView;

const CLAUDE: &str = "../../rust/plevir/tests/fixtures/silences-review-claude.ir.json";

fn wash(ms: u64) -> Result<ImageHandle, String> {
    let v = u8::try_from(40 + (ms / 100) % 200).unwrap_or(120);
    let pixels = std::iter::repeat_n([v, v, v, 255], 32 * 18)
        .flatten()
        .collect();
    load_image_rgba(32, 18, pixels).map_err(|e| e.to_string())
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

fn rect(c: &Compositor, id: &str) -> Rect {
    let [x, y, w, h] = c
        .binding(&sid(id))
        .unwrap_or_else(|| panic!("{id} is bound"))
        .rect;
    Rect::new(x, y, w, h)
}

fn regress() -> Actor {
    Actor::Test {
        name: "regress".to_owned(),
    }
}

/// `action` on `target` by `actor` through the gallery, as the port acts.
fn act(
    view: &mut ShowcaseView,
    actor: &Actor,
    target: &str,
    action: &str,
    param: Option<Value>,
) -> Stage {
    let seen = view.semantics().expect("the page projects");
    let req = Request::observe(&seen, actor.clone(), target, action, param).unwrap();
    let receipt = view.act(req).expect("taken");
    receipt.stage().expect("a stage").clone()
}

/// `intent` asked through the row by `actor`: written, then proposed.
fn ask(view: &mut ShowcaseView, actor: &Actor, intent: &str) -> Stage {
    let text = Some(Value::Text(intent.to_owned()));
    let wrote = act(view, actor, "silences_surface.intent", "set_value", text);
    assert!(matches!(wrote, Stage::Completed(_)), "{wrote:?}");
    act(view, actor, "silences_surface.propose", "propose", None)
}

#[test]
fn too_much_information_brings_the_cuts_of_the_claudes_review_into_the_window() {
    let mut view = ShowcaseView::new(1280.0, 800.0);
    view.jump_to_page("silences");
    view.stage_silences_frames(&mut wash);
    view.stage_silences_ir(&fixture(CLAUDE))
        .expect("the fixture verifies");
    let c = settle(&mut view);
    assert!(rect(&c, "review.cut_item[1]").y > 800.0, "below the window");

    let done = ask(&mut view, &regress(), "tem informação demais");
    let Stage::Completed(outcome) = done else {
        panic!("{done:?}");
    };
    assert_eq!(outcome.result.get("steps"), Some(&Value::Int(4)));
    view.stage_silences_frames(&mut wash);
    let c = settle(&mut view);
    let snap = view.semantics().expect("the page projects");
    for cut in [1, 2] {
        let item = rect(&c, &format!("review.cut_item[{cut}]"));
        assert!(item.y + item.h <= 800.0, "cut {cut} inside: {item:?}");
        let actions = rect(&c, &format!("review.cut_item[{cut}].actions"));
        assert!(actions.w > 0.0);
        // what the cut is stays: its frame, its range, its decision
        for part in ["cut_frame", "cut_range", "cut_decision", "cut_duration"] {
            assert!(
                snap.get(&sid(&format!("review.{part}[{cut}]"))).is_some(),
                "{part}"
            );
        }
    }
    for gone in ["review.video_card", "review.counts_card"] {
        assert!(snap.get(&sid(gone)).is_none(), "{gone} aside");
    }
    assert!(
        snap.get(&sid("silences.counts")).is_some(),
        "the fixed screen's"
    );
    let host = &view.panes.silences.generated.host;
    let by: Vec<&Actor> = host.changes(&sid("review")).iter().map(|c| &c.by).collect();
    assert_eq!(by, [&regress(); 4]);
}

/// A cut applied before the change stays applied: the app's state, shown
/// by the surface through the change as before it.
#[test]
fn a_cut_applied_before_stays_applied_through_the_change() {
    let (mut todo, mut page, _) = setup(false);
    page.generated
        .proposing
        .intent
        .set_text("the list with the video and the counts");
    page.propose(&mut todo).expect("proposed");
    let seen = page.projection().unwrap();
    let apply = Request::observe(
        &seen,
        Actor::Human,
        "proposed.silences.cut[1].apply_cut",
        "activate",
        None,
    )
    .unwrap();
    let applied = page.act(apply, &mut todo, false).expect("a receipt");
    assert!(matches!(applied.stage(), Some(Stage::Completed(_))));
    let shown = |page: &SilencesPage| {
        page.projection()
            .unwrap()
            .value(&"proposed.silences.cut[1].decision:text".parse().unwrap())
            .cloned()
    };
    let decided = Some(Value::Text("apply".to_owned()));
    assert_eq!(shown(&page), decided);

    page.generated
        .proposing
        .intent
        .set_text("too much information");
    let asked = page.propose(&mut todo).expect("dispatched");
    assert!(matches!(
        todo.receipt(asked).unwrap().stage(),
        Some(Stage::Completed(_))
    ));
    assert_eq!(value(&page, "silences.cut[1]:decision"), decided);
    assert_eq!(shown(&page), decided, "the surface shows it still");
    assert_eq!(
        value(&page, "silences.cut[2]:decision"),
        Some(Value::Text("pending".into()))
    );
}

/// A person drags the threshold of the Claude's review; an agent asks for
/// "too much information" meanwhile: the slider stays (the detection
/// binds actions: it is the task), keeps its widget, and the drag goes
/// on over its new place.
#[test]
fn a_slider_in_the_middle_of_a_drag_goes_on_through_the_change() {
    let (mut todo, mut page, theme) = setup(false);
    show(&mut page, CLAUDE);
    let content = wide();
    let c = frame(&mut page, content, &theme);
    let r = rect(&c, "review.threshold");
    let knob = theme.control.slider_knob;
    let along = |r: Rect, t: f32| r.x + knob / 2.0 + t * (r.w - knob);
    let track_y = |r: Rect| r.y + r.h - knob / 2.0;
    let threshold = |page: &SilencesPage| match value(page, "silences.threshold:value") {
        Some(Value::Num(n)) => n,
        other => panic!("{other:?}"),
    };
    assert!((threshold(&page) + 30.0).abs() <= 1.0, "at rest: -30 dB");
    for event in [
        WidgetEvent::MouseMove {
            x: along(r, 0.25),
            y: track_y(r),
        },
        WidgetEvent::MouseDown {
            x: along(r, 0.25),
            y: track_y(r),
        },
    ] {
        page.handle_event(&event, content, &theme, &mut todo, false);
    }
    assert!(
        (threshold(&page) + 45.0).abs() <= 1.0,
        "{}",
        threshold(&page)
    );
    let (x, y) = (along(r, 0.4), track_y(r));
    page.handle_event(
        &WidgetEvent::MouseMove { x, y },
        content,
        &theme,
        &mut todo,
        false,
    );
    assert!(
        (threshold(&page) + 36.0).abs() <= 1.0,
        "{}",
        threshold(&page)
    );

    let agent = Actor::Agent {
        name: "claude".to_owned(),
    };
    let seen = page.projection().unwrap();
    let write = Request::observe(
        &seen,
        agent.clone(),
        "silences_surface.intent",
        "set_value",
        Some(Value::Text("too much information".to_owned())),
    )
    .unwrap();
    page.act(write, &mut todo, false).expect("written");
    let seen = page.projection().unwrap();
    let propose =
        Request::observe(&seen, agent, "silences_surface.propose", "propose", None).unwrap();
    let asked = page.act(propose, &mut todo, false).expect("a receipt");
    assert!(
        matches!(asked.stage(), Some(Stage::Completed(_))),
        "{:?}",
        asked.stage()
    );

    let mut c = frame(&mut page, content, &theme);
    for _ in 0..120 {
        if !page.wake(TURN, &mut todo).is_continuous() {
            break;
        }
        c = frame(&mut page, content, &theme);
    }
    let moved = rect(&c, "review.threshold");
    assert_ne!(moved, r, "the slider moved with the change");
    // the button is still down: the drag goes on over the new place
    let (x, y) = (along(moved, 0.8), track_y(moved));
    page.handle_event(
        &WidgetEvent::MouseMove { x, y },
        content,
        &theme,
        &mut todo,
        false,
    );
    assert!(
        (threshold(&page) + 12.0).abs() <= 1.0,
        "{}",
        threshold(&page)
    );
    page.handle_event(
        &WidgetEvent::MouseUp { x, y },
        content,
        &theme,
        &mut todo,
        false,
    );
}
