//! The generated surface beside the fixed screen (task-69 phase 4b): the
//! row and the app projected apart; the review opened over the app and
//! its frames drawn from the page's cache (the fake worker, no ffmpeg); a
//! cut applied on the generated surface showing on the fixed screen and
//! back, through the one dispatcher; the review becoming the comparison
//! in place with the typed intent and the cut instances going on, and
//! `close` undoing it with the exit transition, by the screen and by a
//! request; the gallery routing the row, the surface and the keys; the
//! Claude answering off the thread (the fake claude); and the page drawn
//! headless with both forms (a png each with `SILENCES_OUT`, the real
//! frames in the ignored test).

use std::path::PathBuf;
use std::time::{Duration, Instant};

use comps::prelude::{Rect, WidgetEvent};
use engine::compositor::Compositor;
use sem::{Actor, Rejection, Request, SemId, Stage, StateRef, Value};
use showcase::model::silences::{Decision, PROBE_STDERR, parse};
use uir::{Capabilities, GenerationPolicy, verify_json};

use super::super::SilencesPage;
use super::super::generated::page::Page;
use super::super::worker::Worker;
use super::{
    TURN, ask_as_test, center, click, frame, images, narrow, setup, sid, stage, text, value, wide,
};
use crate::view::EditKey;
use crate::view::ShowcaseView;
use crate::view::apps::app::TodoPage;

const REVIEW: &str = "../surface/tests/fixtures/showcase-silences-review.ir.json";
const COMPARE: &str = "../surface/tests/fixtures/showcase-silences-compare.ir.json";

fn fixture(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// Verifies the fixture `rel` against the app as it is and shows it: the
/// surface opens, or takes the form in place.
fn show(page: &mut SilencesPage, rel: &str) -> SemId {
    let app = page.snapshot().expect("the app projects");
    let caps = Capabilities::from_snapshot(&app);
    let ir = verify_json(&fixture(rel), &caps, &GenerationPolicy::default())
        .unwrap_or_else(|e| panic!("{rel}: {e:#?}"));
    let (id, _) = Page { app: page }.show(ir).expect("the surface shows");
    id
}

/// The value of `at` in the whole page's projection.
fn seen(page: &SilencesPage, at: &str) -> Option<Value> {
    let snap = page.projection().expect("the page projects");
    snap.value(&StateRef::parse(at).unwrap()).cloned()
}

fn has(page: &SilencesPage, id: &str) -> bool {
    page.projection()
        .expect("the page projects")
        .get(&sid(id))
        .is_some()
}

/// Wakes the page until the fake worker has answered every job.
fn drain(page: &mut SilencesPage, todo: &mut TodoPage) {
    let mut turns = 0;
    loop {
        page.worker.release();
        page.wake(TURN, todo);
        if !page.worker.busy() {
            break;
        }
        turns += 1;
        assert!(turns < 20, "the jobs never end");
    }
}

/// Frames a turn apart for as long as the page asks for one: the last
/// one, and how many.
fn settle(
    page: &mut SilencesPage,
    todo: &mut TodoPage,
    content: Rect,
    theme: &engine::theme::Theme,
) -> (Compositor, usize) {
    let mut c = frame(page, content, theme);
    let mut frames = 0;
    loop {
        let wake = page.wake(TURN, todo);
        if wake.now {
            c = frame(page, content, theme);
            frames += 1;
        }
        if !wake.is_continuous() {
            return (c, frames);
        }
        assert!(frames < 600, "the page never settles");
    }
}

/// The row is the page's: projected with the app for the chrome, apart
/// from it for the registry; the close is disabled while nothing is
/// open, and a request on it is refused as disabled.
#[test]
fn the_row_is_projected_with_the_app_and_kept_out_of_its_registry() {
    let (mut todo, mut page, _) = setup(false);
    let app = page.snapshot().expect("the app projects");
    let whole = page.projection().expect("the page projects");
    for id in [
        "silences_surface",
        "silences_surface.intent",
        "silences_surface.proposer",
        "silences_surface.propose",
        "silences_surface.cancel",
        "silences_surface.status",
        "silences_surface.close",
    ] {
        assert!(app.get(&sid(id)).is_none(), "{id} out of the app");
        assert!(whole.get(&sid(id)).is_some(), "{id} in the page");
    }
    assert!(whole.get(&sid("silences.cut[1]")).is_some());
    let close = whole.get(&sid("silences_surface.close")).unwrap();
    assert!(!close.enabled);
    assert_eq!(
        whole
            .get(&sid("silences_surface.proposer"))
            .unwrap()
            .constraints
            .options,
        Some(vec!["rules".to_owned()]),
        "no claude is looked for in the tests"
    );
    let seen = page.projection().unwrap();
    let req = Request::observe(
        &seen,
        Actor::Test {
            name: "silences".into(),
        },
        "silences_surface.close",
        "close",
        None,
    )
    .unwrap();
    let receipt = page.act(req, &mut todo, false).expect("a receipt");
    assert!(
        matches!(
            receipt.stage(),
            Some(Stage::Rejected(Rejection::Disabled { .. }))
        ),
        "{:?}",
        receipt.stage()
    );
}

/// The acceptance: the review opens over the app beside the fixed
/// screen; the page asks the worker for the frames the surface shows and
/// it lacks, from its wake, and when they arrive the surface draws one
/// per cut and one for the video, through the trait, next to the fixed
/// screen's own.
#[test]
fn the_review_opens_beside_the_fixed_screen_and_draws_its_frames() {
    let (mut todo, mut page, theme) = setup(true);
    let content = wide();
    let c = frame(&mut page, content, &theme);
    assert_eq!(images(&c), 0, "the seed has no frames");
    let id = show(&mut page, REVIEW);
    assert_eq!(id, sid("review"));
    assert!(has(&page, "review.video"));
    assert!(has(&page, "review.cut[1].apply"));
    assert_eq!(seen(&page, "silences_surface.close:enabled"), None);
    assert!(
        page.projection()
            .unwrap()
            .get(&sid("silences_surface.close"))
            .unwrap()
            .enabled
    );

    // beside: the fixed screen in the left half, the surface in the right
    let h = page.halves(content, &theme);
    let surface = h.surface.expect("a surface is open");
    assert!(surface.x >= h.fixed.right(), "{:?} {surface:?}", h.fixed);
    let c = frame(&mut page, content, &theme);
    let video = c.binding(&sid("review.video")).expect("drawn").rect;
    assert!(video[0] >= surface.x - 0.5);
    assert!(
        c.binding(&sid("silences.video")).is_some(),
        "the fixed screen too"
    );
    assert_eq!(images(&c), 0, "nothing decoded yet: the shells");

    // the wake asks for what the surface shows: three frames, one job
    // at a time, and the page polls while they run
    let wake = page.wake(TURN, &mut todo);
    assert!(page.worker.busy(), "the frames are asked for");
    assert_eq!(wake.next, Some(super::super::worker::POLL));
    drain(&mut page, &mut todo);
    let mut at: Vec<u64> = page.frames.keys().copied().collect();
    at.sort_unstable();
    assert_eq!(at, [0, 4017, 9009]);
    let c = frame(&mut page, content, &theme);
    assert_eq!(
        images(&c),
        6,
        "the fixed screen's three and the surface's three"
    );
    // narrow: the surface goes under the fixed screen
    let h = page.halves(narrow(), &theme);
    let under = h.surface.expect("open");
    assert!(under.y >= h.fixed.bottom(), "{:?} {under:?}", h.fixed);
    assert!(page.content_height(narrow(), &theme) >= under.y - narrow().y);
    assert_eq!(page.wake(TURN, &mut todo), comps::prelude::Wake::idle());
}

/// The acceptance: a click on a cut's apply button on the generated
/// surface applies the cut through the app's dispatcher, and the fixed
/// screen shows it; a request on the fixed screen rejects the other cut,
/// and the surface's badge follows.
#[test]
fn applying_on_the_generated_surface_moves_the_fixed_screen_and_back() {
    let (mut todo, mut page, theme) = setup(false);
    let content = wide();
    show(&mut page, REVIEW);
    let c = frame(&mut page, content, &theme);
    click(
        &mut page,
        &mut todo,
        content,
        &theme,
        center(&c, "review.cut[1].apply"),
    );
    assert_eq!(value(&page, "silences.cut[1]:decision"), text("apply"));
    let receipt = todo
        .receipt(sem::RequestId::parse("1").unwrap())
        .expect("kept");
    assert_eq!(receipt.request.actor, Actor::Human);
    assert_eq!(receipt.request.target, sid("silences.cut[1]"));
    assert_eq!(receipt.request.action.as_str(), "apply_cut");
    assert!(matches!(receipt.stage(), Some(Stage::Completed(_))));
    // the fixed screen: the row's button reads dark
    frame(&mut page, content, &theme);
    assert_eq!(
        page.rows[0].apply.variant,
        comps::prelude::ButtonVariant::Secondary
    );
    assert_eq!(page.model.cuts()[0].decision(), Decision::Apply);
    // and the surface shows the badge and the count
    assert_eq!(seen(&page, "review.cut[1].decision:text"), text("apply"));
    assert_eq!(seen(&page, "review.applied:value"), Some(Value::Int(1)));

    // back: a request on the fixed screen, the surface follows
    let done = ask_as_test(&mut page, &mut todo, "silences.cut[2]", "reject_cut", None);
    assert!(matches!(done, Stage::Completed(_)), "{done:?}");
    frame(&mut page, content, &theme);
    assert_eq!(seen(&page, "review.cut[2].decision:text"), text("reject"));
    assert_eq!(seen(&page, "review.rejected:value"), Some(Value::Int(1)));
    // an agent's act on a surface node is the capability the IR bound
    let seen_page = page.projection().unwrap();
    let req = Request::observe(
        &seen_page,
        Actor::Assistive,
        "review.cut[2].apply",
        "activate",
        None,
    )
    .unwrap();
    let receipt = page.act(req, &mut todo, false).expect("routed");
    assert_eq!(receipt.request.target, sid("silences.cut[2]"));
    assert_eq!(receipt.request.action.as_str(), "apply_cut");
    assert!(matches!(receipt.stage(), Some(Stage::Completed(_))));
    assert_eq!(value(&page, "silences.cut[2]:decision"), text("apply"));
}

/// The acceptance of the transformation and of the dismissal: the review
/// becomes the comparison in place, the intent typed in the row and the
/// cut instances going on (a press begun on a cut's button completes
/// after); `close` by a click takes the surface out of the projection at
/// once, fades its picture out, and the fixed screen takes the width
/// back; then `close` by a request is refused, nothing being open.
#[test]
fn the_comparison_keeps_what_continues_and_close_undoes_it() {
    let (mut todo, mut page, theme) = setup(false);
    let content = wide();
    show(&mut page, REVIEW);
    let c = frame(&mut page, content, &theme);
    click(
        &mut page,
        &mut todo,
        content,
        &theme,
        center(&c, "silences_surface.intent"),
    );
    for ch in "compare".chars() {
        assert!(page.handle_text(&ch.to_string(), &mut todo));
    }
    assert_eq!(seen(&page, "silences_surface.intent:text"), text("compare"));
    let video_was = c.binding(&sid("review.video")).expect("drawn").rect;
    let apply = c.binding(&sid("review.cut[1].apply")).expect("drawn").rect;
    let (x, y) = (apply[0] + apply[2] / 2.0, apply[1] + apply[3] / 2.0);
    for event in [
        WidgetEvent::MouseMove { x, y },
        WidgetEvent::MouseDown { x, y },
    ] {
        page.handle_event(&event, content, &theme, &mut todo, false);
    }
    assert_eq!(page.model.cuts()[0].decision(), Decision::Pending);

    let id = show(&mut page, COMPARE);
    assert_eq!(id, sid("review"), "the same surface");
    assert!(
        has(&page, "review.result.video"),
        "the result's view is born"
    );
    assert!(!has(&page, "review.detect"), "the detection row left");
    assert_eq!(
        seen(&page, "silences_surface.intent:text"),
        text("compare"),
        "the intent goes on"
    );
    // the same instance: the release lands on the button where it was
    let c = frame(&mut page, content, &theme);
    assert_eq!(c.binding(&sid("review.cut[1].apply")).unwrap().rect, apply);
    page.handle_event(
        &WidgetEvent::MouseUp { x, y },
        content,
        &theme,
        &mut todo,
        false,
    );
    assert_eq!(page.model.cuts()[0].decision(), Decision::Apply);
    let (c, frames) = settle(&mut page, &mut todo, content, &theme);
    assert!(frames > 1, "the change of form took {frames} frames");
    assert_ne!(
        c.binding(&sid("review.video")).unwrap().rect,
        video_was,
        "the original's view moved to its column"
    );

    // close, by the button: gone from the projection at once, the
    // fixed screen at the whole width, the pictures fading out, then
    // nothing
    click(
        &mut page,
        &mut todo,
        content,
        &theme,
        center(&c, "silences_surface.close"),
    );
    assert!(!has(&page, "review.video"));
    assert!(!has(&page, "review.cut[1].apply"));
    assert!(page.halves(content, &theme).surface.is_none());
    let c = frame(&mut page, content, &theme);
    assert!(
        c.binding(&sid("review.video")).is_none(),
        "bound to nothing"
    );
    assert_eq!(
        c.binding(&sid("silences")).unwrap().rect[2],
        content.w,
        "the fixed screen takes the width back"
    );
    let wake = page.wake(TURN, &mut todo);
    assert!(wake.is_continuous(), "the exit runs: {wake:?}");
    let (_, frames) = settle(&mut page, &mut todo, content, &theme);
    assert!(frames > 1, "the exit took {frames} frames");
    assert_eq!(page.wake(TURN, &mut todo), comps::prelude::Wake::idle());
    let receipt = todo
        .receipt(sem::RequestId::parse("2").unwrap())
        .expect("the close");
    assert_eq!(receipt.request.target, sid("silences_surface.close"));
    assert!(matches!(receipt.stage(), Some(Stage::Completed(_))));
    let seen_page = page.projection().unwrap();
    let req = Request::observe(
        &seen_page,
        Actor::Test {
            name: "silences".into(),
        },
        "silences_surface.close",
        "close",
        None,
    )
    .unwrap();
    let closed = page.act(req, &mut todo, false).expect("a receipt");
    assert!(
        matches!(
            closed.stage(),
            Some(Stage::Rejected(Rejection::Disabled { .. }))
        ),
        "{:?}",
        closed.stage()
    );
}

/// The gallery on the Silences page: the row in its semantics, typing
/// into the intent through the chrome, a proposal by the rules through
/// `act` opening a surface, Escape and a close through `act`.
#[test]
fn the_gallery_routes_the_row_the_surface_and_the_keys() {
    let mut view = ShowcaseView::new(1200.0, 800.0);
    view.jump_to_page("silences");
    let snap = view.semantics().expect("the page projects");
    assert!(snap.get(&sid("silences_surface.intent")).is_some());
    assert!(view.assist_focus(&sid("silences_surface.intent")));
    for ch in "the counts".chars() {
        assert!(view.handle_key(&ch.to_string()), "{ch} typed");
    }
    assert_eq!(
        view.semantics()
            .unwrap()
            .value(&StateRef::parse("silences_surface.intent:text").unwrap()),
        Some(&Value::Text("the counts".into()))
    );
    assert!(view.handle_edit_key(EditKey::Backspace));
    assert!(view.handle_key("s"));
    // Enter proposes: the rules, at once
    assert!(view.handle_enter());
    let snap = view.semantics().unwrap();
    assert!(snap.get(&sid("proposed")).is_some(), "a surface opened");
    let status = snap.get(&sid("silences_surface.status")).unwrap();
    assert!(
        matches!(status.state.get("message"), Some(Value::Text(m)) if m.starts_with("proposed by the rules")),
        "{:?}",
        status.state.get("message")
    );
    let mut c = Compositor::new();
    view.render(&mut c);
    assert!(c.binding(&sid("proposed.card")).is_some());
    assert!(view.close_top_overlay(), "escape blurs the intent");
    // close through act, as an agent would
    let snap = view.semantics().unwrap();
    let req = Request::observe(
        &snap,
        Actor::Assistive,
        "silences_surface.close",
        "close",
        None,
    )
    .unwrap();
    let receipt = view.act(req).expect("taken");
    assert!(matches!(receipt.stage(), Some(Stage::Completed(_))));
    assert!(view.semantics().unwrap().get(&sid("proposed")).is_none());
    assert!(view.tick(TURN), "the exit runs");
}

/// The Claude asked from this page, against the fake claude answering
/// the review: `propose` starts, the page polls, the answer opens the
/// surface and completes the request with the cost.
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
#[test]
fn the_claude_answers_the_review_off_the_thread() {
    use claude_model::ClaudeCli;

    use crate::view::apps::surface_page::asking::Claude;

    let (mut todo, mut page, _) = setup(false);
    let dir = tempfile::tempdir().unwrap();
    let good = serde_json::json!({
        "type": "result", "subtype": "success", "is_error": false,
        "num_turns": 1, "total_cost_usd": 0.05, "result": fixture(REVIEW)
    })
    .to_string();
    std::fs::write(dir.path().join("mode"), "ok").unwrap();
    std::fs::write(dir.path().join("good"), good).unwrap();
    let mut cli = ClaudeCli::new();
    cli.program =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../claude-model/tests/fake-claude");
    cli.dir = dir.path().to_owned();
    cli.timeout = Duration::from_secs(20);
    page.generated.proposing.set_claude(Claude::with(cli));
    page.generated
        .proposing
        .intent
        .set_text("cut the silences of this video, then I want to review the cuts");
    let asked = page.propose(&mut todo).expect("dispatched");
    assert_eq!(stage(&todo, asked), Stage::Executing);
    assert!(page.generated.proposing.waiting());
    let started = Instant::now();
    while !todo.receipt(asked).unwrap().is_finished() {
        assert!(
            started.elapsed() < Duration::from_secs(20),
            "the answer came"
        );
        page.wake(TURN, &mut todo);
        std::thread::sleep(Duration::from_millis(25));
    }
    let Stage::Completed(outcome) = stage(&todo, asked) else {
        panic!("{:?}", stage(&todo, asked));
    };
    assert_eq!(
        outcome.result.get("surface"),
        Some(&Value::Text("review".into()))
    );
    assert!(matches!(
        outcome.result.get("cost_usd"),
        Some(Value::Num(_))
    ));
    assert!(has(&page, "review.cut[1].frame"));
    assert!(
        page.halves(wide(), &engine::theme::Theme::hoff_core())
            .surface
            .is_some()
    );
}

/// The page as the gallery draws it without a window, with the review
/// and then the comparison over the fake frames, every transition
/// settled: the scene and the projection, and with a GPU the pixels
/// (`silences-review.png` and `silences-compare.png` under `SILENCES_OUT`,
/// with their projections).
#[test]
fn the_page_draws_headless_with_the_review_and_the_comparison() {
    use engine::gpu::RenderConfig;
    use engine::headless::{HeadlessError, Renderer, Viewport};

    let viewport = Viewport {
        width: 1200.0,
        height: 800.0,
        scale: 1.0,
    };
    let mut renderer = match Renderer::new(viewport, RenderConfig::default()) {
        Ok(r) => Some(r),
        Err(HeadlessError::NoAdapter) => None,
        Err(e) => panic!("{e}"),
    };
    let mut view = ShowcaseView::new(viewport.width, viewport.height);
    view.jump_to_page("silences");
    view.panes
        .silences
        .set_worker(Worker::fake(parse(PROBE_STDERR), false));
    let mut c = Compositor::new();
    view.render(&mut c);
    for (name, rel, surface_images) in [("review", REVIEW, 3), ("compare", COMPARE, 4)] {
        show(&mut view.panes.silences, rel);
        view.render(&mut c);
        // the frames the surface shows are asked for from the wake and
        // answered by the fake at once; the transitions settle after
        let mut steps = 0;
        loop {
            let moving = view.tick(TURN);
            view.render(&mut c);
            if !moving && !view.panes.silences.worker.busy() {
                break;
            }
            steps += 1;
            assert!(steps < 600, "{name}: the page never settles");
        }
        let snap = view.semantics().expect("the page projects");
        assert!(snap.get(&sid("review.cut[1].frame")).is_some(), "{name}");
        assert!(
            c.binding(&sid("silences.video")).is_some(),
            "{name}: the fixed"
        );
        assert_eq!(
            images(&c),
            3 + surface_images,
            "{name}: the fixed screen's three frames and the surface's"
        );
        let Some(r) = renderer.as_mut() else {
            eprintln!("no GPU adapter: the scene and the projection only");
            continue;
        };
        let clear = view.theme.colors.bg.to_linear_array();
        let frame = r.render(&mut c, clear).expect("the frame draws");
        if let Some(out) = std::env::var_os("SILENCES_OUT") {
            let path = PathBuf::from(out).join(format!("silences-{name}"));
            frame
                .write_png(path.with_extension("png"))
                .expect("the png writes");
            let json = crate::app::semantics_json(&snap, false).expect("the json");
            std::fs::write(path.with_extension("json"), json).expect("the json writes");
            println!("wrote {}.png and .json", path.display());
        }
    }
}

/// The same two forms over the real ffmpeg, ignored (a runner of the
/// gate may not have it): `SILENCES_OUT=<dir> cargo test -p showcase
/// --lib silences::tests::generated::the_real -- --ignored --nocapture`
/// writes `silences-review-real.png` and `silences-compare-real.png`
/// with their projections. Without the program it says so and passes.
#[test]
#[ignore = "runs ffmpeg: cargo test -p showcase --lib silences::tests::generated::the_real -- --ignored --nocapture"]
fn the_real_ffmpeg_draws_the_review_and_the_comparison() {
    use engine::gpu::RenderConfig;
    use engine::headless::{Renderer, Viewport};

    let program = match Worker::find_program() {
        Ok(p) => p,
        Err(why) => {
            eprintln!("skipping: {why}");
            return;
        }
    };
    println!("ffmpeg: {}", program.display());
    let viewport = Viewport {
        width: 1200.0,
        height: 800.0,
        scale: 1.0,
    };
    let mut view = ShowcaseView::new(viewport.width, viewport.height);
    view.jump_to_page("silences");
    view.panes.silences.set_worker(Worker::real());
    let mut c = Compositor::new();
    view.render(&mut c);
    // "cut the silences of this video": the detection first, then the
    // review is shown; the cut at 0:03 previewed and applied, so the
    // comparison shows a result that differs
    let snap = view.semantics().unwrap();
    let detect = Request::observe(
        &snap,
        Actor::Test {
            name: "silences".into(),
        },
        "silences.detect",
        "detect_silence",
        None,
    )
    .unwrap();
    let receipt = view.act(detect).expect("taken");
    let started = Instant::now();
    let id = receipt.id;
    while !view.panes.todo.receipt(id).unwrap().is_finished() {
        assert!(
            started.elapsed() < Duration::from_secs(90),
            "ffmpeg answered"
        );
        std::thread::sleep(Duration::from_millis(5));
        view.tick(TURN);
    }
    println!(
        "detect_silence: {:?}",
        view.panes.todo.receipt(id).unwrap().stage()
    );
    let mut renderer = Renderer::new(viewport, RenderConfig::default()).ok();
    for (name, rel) in [("review", REVIEW), ("compare", COMPARE)] {
        show(&mut view.panes.silences, rel);
        if name == "compare" {
            let first = format!("silences.cut[{}]", view.panes.silences.model.cuts()[0].id());
            let snap = view.semantics().unwrap();
            for action in ["preview_segment", "apply_cut"] {
                let req = Request::observe(
                    &snap,
                    Actor::Test {
                        name: "silences".into(),
                    },
                    first.as_str(),
                    action,
                    None,
                )
                .unwrap();
                view.act(req);
            }
        }
        // the frames the surface shows are decoded from the wake
        let started = Instant::now();
        loop {
            let moving = view.tick(TURN);
            view.render(&mut c);
            if !moving && !view.panes.silences.worker.busy() {
                break;
            }
            assert!(
                started.elapsed() < Duration::from_secs(120),
                "{name}: the frames came"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
        let snap = view.semantics().expect("the page projects");
        println!(
            "{name}: {} images in the frame, frames at {:?} ms",
            images(&c),
            {
                let mut at: Vec<u64> = view.panes.silences.frames.keys().copied().collect();
                at.sort_unstable();
                at
            }
        );
        let Some(r) = renderer.as_mut() else {
            eprintln!("no GPU adapter: the scene and the projection only");
            continue;
        };
        let clear = view.theme.colors.bg.to_linear_array();
        let frame = r.render(&mut c, clear).expect("the frame draws");
        if let Some(out) = std::env::var_os("SILENCES_OUT") {
            let path = PathBuf::from(out).join(format!("silences-{name}-real"));
            frame
                .write_png(path.with_extension("png"))
                .expect("the png writes");
            let json = crate::app::semantics_json(&snap, false).expect("the json");
            std::fs::write(path.with_extension("json"), json).expect("the json writes");
            println!("wrote {}.png and .json", path.display());
        }
    }
}
