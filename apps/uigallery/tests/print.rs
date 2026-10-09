//! The frame on paper (task-76 item 9), without a printer: a
//! `ShowcaseView` on the Todo page, a fake printer with a destination,
//! and a person's `shell.print`. The job runs beside the test's thread
//! (a headless renderer of its own, the png, the fake's queue), the
//! request completes with the job and the document's measures, and the
//! document is, byte for byte, the frame a view in the same state draws
//! at the paper scale through the window's frame path; the main window's
//! own scene and scale are untouched by it. Asleep, a print waits for the
//! wake. Desktop only, like the shell; skips without a GPU adapter, like
//! the headless tests.
#![cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use engine::compositor::{Compositor, LayerId, SceneNode};
use engine::gpu::RenderConfig;
use engine::headless::{Frame, HeadlessError, Renderer, Viewport};
use engine::platform::menu::locked;
use engine::platform::print::{FakePrinter, PAPER_DPI, paper_scale};
use engine::window::Background;
use sem::{Actor, Receipt, ReceiptSink, Request, SemId, Stage, Value};
use showcase::view::ShowcaseView;

const MAIN: (f32, f32) = (1200.0, 800.0);
/// The paper scale of the shell: 300 dpi in points.
const PAPER: f32 = 300.0 / 72.0;
/// One turn of a 60 Hz display.
const TURN: f32 = 1.0 / 60.0;

fn sid(text: &str) -> SemId {
    SemId::parse(text).expect("a valid id")
}

/// The Todo page laid out for the main window at the screen's scale,
/// settled.
fn view() -> ShowcaseView {
    let mut view = ShowcaseView::new(MAIN.0, MAIN.1);
    view.jump_to_page("Todo");
    view.resize(MAIN.0, MAIN.1, 1.0);
    settle(&mut view);
    view
}

/// Ticks until nothing moves (a toggle strikes an item through a tween).
fn settle(view: &mut ShowcaseView) {
    for _ in 0..600 {
        if !view.tick(TURN) {
            return;
        }
    }
    panic!("the view never settles");
}

/// Catches up (`settle_print`) until `done` holds, or `timeout`.
fn wait_for(
    view: &mut ShowcaseView,
    timeout: Duration,
    done: impl Fn(&ShowcaseView) -> bool,
) -> bool {
    let until = Instant::now() + timeout;
    loop {
        view.settle_print();
        if done(view) {
            return true;
        }
        if Instant::now() >= until {
            return false;
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn state(view: &ShowcaseView, at: &str) -> Value {
    view.gallery_semantics()
        .expect("the gallery projects")
        .value(&at.parse().expect("a state ref"))
        .cloned()
        .expect("projected")
}

fn enabled(view: &ShowcaseView, id: &str) -> bool {
    view.gallery_semantics()
        .expect("the gallery projects")
        .get(&sid(id))
        .expect("projected")
        .enabled
}

/// Item 1 toggled by a person through the page's dispatcher: the state
/// the print has to show.
fn toggle(view: &mut ShowcaseView, item: &SemId) {
    let seen = view.semantics().expect("the todo projects");
    let req = Request::observe(&seen, Actor::Human, item, "toggle", None).expect("observed");
    let receipt = view.act(req).expect("the page takes it");
    assert!(
        matches!(receipt.stage(), Some(Stage::Completed(_))),
        "{:?}",
        receipt.stage()
    );
}

/// The scene the main window builds from `view` now, into `c`.
fn scene(view: &mut ShowcaseView, c: &mut Compositor) -> Vec<(LayerId, i32, Vec<SceneNode>)> {
    view.render(c);
    c.layers()
        .iter()
        .map(|l| (l.id, l.z_order, l.nodes().to_vec()))
        .collect()
}

/// Every stage of every receipt of the gallery's dispatcher, kept by the
/// test: how it reads the end of the print's request.
#[derive(Clone, Default)]
struct Stages(Arc<Mutex<Vec<(SemId, Stage)>>>);

impl ReceiptSink for Stages {
    fn on_stage(&mut self, receipt: &Receipt, stage: &Stage) {
        locked(&self.0).push((receipt.request.target.clone(), stage.clone()));
    }
}

impl Stages {
    fn attach(view: &mut ShowcaseView) -> Stages {
        let stages = Stages::default();
        view.add_sink(Box::new(stages.clone()));
        stages
    }

    /// The last stage of a request on `target`, when it ended.
    fn ended(&self, target: &str) -> Option<Stage> {
        locked(&self.0)
            .iter()
            .rev()
            .find(|(t, s)| {
                *t == sid(target) && matches!(s, Stage::Completed(_) | Stage::Failed { .. })
            })
            .map(|(_, s)| s.clone())
    }
}

/// A view in the same state, drawn at the paper scale by a headless
/// renderer through the window's frame path: what the document has to
/// hold. `None` without a GPU adapter.
fn control(item: &SemId) -> Option<Frame> {
    let mut view = view();
    toggle(&mut view, item);
    settle(&mut view);
    view.resize(MAIN.0, MAIN.1, PAPER);
    let mut c = Compositor::new();
    view.render(&mut c);
    let mut renderer = match Renderer::new(
        Viewport::new(MAIN.0, MAIN.1, PAPER),
        RenderConfig::default(),
    ) {
        Ok(r) => r,
        Err(HeadlessError::NoAdapter) => return None,
        Err(e) => panic!("{e}"),
    };
    Some(
        renderer
            .render(&mut c, view.theme.colors.bg.to_linear_array())
            .expect("the control renders"),
    )
}

/// The end of the print, or a skip on a machine without a GPU (the job
/// fails saying so).
fn ended_or_skip(stages: &Stages) -> Option<sem::Outcome> {
    match stages.ended("shell.print").expect("the request ended") {
        Stage::Completed(outcome) => Some(outcome),
        Stage::Failed { error } if error.contains("no GPU adapter") => {
            eprintln!("no GPU adapter; skipping");
            None
        }
        other => panic!("the print ended {other:?}"),
    }
}

#[test]
fn a_person_prints_the_main_windows_frame_as_it_stands_at_the_papers_scale() {
    // the job logs what each piece took (`RUST_LOG=showcase=info`)
    let _ = env_logger::builder().is_test(true).try_init();
    let fake = Arc::new(Mutex::new(FakePrinter::new().destination("Office", true)));
    let mut view = view();
    let stages = Stages::attach(&mut view);
    let mut main = Compositor::new();
    view.set_printer(Box::new(fake.clone()));
    assert!(
        wait_for(&mut view, Duration::from_secs(5), |v| enabled(
            v,
            "shell.print"
        )),
        "the destinations land"
    );
    assert_eq!(state(&view, "shell:print"), Value::from("ready on Office"));

    // the state now: an item toggled; the main window's scene as built
    let item = sid("todo.item[1]");
    toggle(&mut view, &item);
    settle(&mut view);
    let before = scene(&mut view, &mut main);

    // the person asks: the scene is built for paper, the job runs beside
    // the loop, and the request waits
    assert!(view.request_print(), "taken up");
    assert_eq!(
        state(&view, "shell:print"),
        Value::from("printing on Office")
    );
    assert!(!enabled(&view, "shell.print"), "one at a time");
    assert!(stages.ended("shell.print").is_none(), "still running");
    assert!(
        wait_for(&mut view, Duration::from_secs(60), |_| stages
            .ended("shell.print")
            .is_some()),
        "the job answers"
    );
    let Some(outcome) = ended_or_skip(&stages) else {
        return;
    };
    let physical = (
        (MAIN.0 * PAPER).round() as i64,
        (MAIN.1 * PAPER).round() as i64,
    );
    assert_eq!(physical, (5000, 3333));
    assert_eq!(outcome.result["job"], Value::from("Office-1"));
    assert_eq!(outcome.result["destination"], Value::from("Office"));
    assert_eq!(outcome.result["status"], Value::from("queued"));
    assert_eq!(outcome.result["width"], Value::Int(physical.0));
    assert_eq!(outcome.result["height"], Value::Int(physical.1));
    assert_eq!(outcome.result["dpi"], Value::Num(f64::from(PAPER_DPI)));
    assert!(
        matches!(outcome.result["bytes"], Value::Int(n) if n > 100_000),
        "{:?}",
        outcome.result["bytes"]
    );
    assert!(matches!(outcome.result["seconds"], Value::Num(s) if s > 0.0));

    // what the printer was given: the frame at the paper's scale, fit to
    // the page, titled after the page
    let printed = locked(&fake).take_printed();
    assert_eq!(printed.len(), 1);
    let doc = &printed[0].document;
    assert_eq!((doc.width, doc.height), (5000, 3333));
    assert_eq!(doc.dpi, PAPER_DPI);
    assert!((paper_scale(doc.dpi) - PAPER).abs() < 0.001);
    let options = &printed[0].options;
    assert_eq!(options.destination.as_deref(), Some("Office"));
    assert_eq!(options.title, "plev, Todo");
    assert_eq!(options.copies, 1);
    assert!(options.fit_to_page);
    assert_eq!(options.media, None);
    assert_eq!(printed[0].job.to_string(), "Office-1");

    // the document is the frame a view in the same state draws at that
    // scale through the window's frame path, byte for byte
    let decoded = image::load_from_memory(&doc.png).expect("a png").to_rgba8();
    assert_eq!((decoded.width(), decoded.height()), (5000, 3333));
    let Some(control) = control(&item) else {
        return;
    };
    assert_eq!((control.width, control.height), (5000, 3333));
    assert!(
        decoded.as_raw() == &control.rgba,
        "the document is the control's frame"
    );

    // the projection after: the job, its place, the button offered again
    assert_eq!(
        state(&view, "shell:print"),
        Value::from("Office-1 on Office: queued")
    );
    assert_eq!(state(&view, "shell.print:job"), Value::from("Office-1"));
    assert_eq!(state(&view, "shell.print:status"), Value::from("queued"));
    assert_eq!(
        state(&view, "shell.print:destination"),
        Value::from("Office")
    );
    assert!(enabled(&view, "shell.print"));

    // the main window is untouched: its scale, and the scene it builds,
    // node for node
    assert_eq!(view.scale_factor, 1.0);
    let after = scene(&mut view, &mut main);
    assert!(after == before, "the main window's scene moved");
    assert!(!view.settle_print(), "at rest");
}

/// A print asked while the app sleeps starts on the first look after
/// the wake, from the state then; nothing runs meanwhile.
#[test]
fn a_print_asked_while_the_app_sleeps_waits_for_the_wake() {
    let fake = Arc::new(Mutex::new(FakePrinter::new().destination("Office", true)));
    let mut view = view();
    let stages = Stages::attach(&mut view);
    view.set_printer(Box::new(fake.clone()));
    assert!(
        wait_for(&mut view, Duration::from_secs(5), |v| enabled(
            v,
            "shell.print"
        )),
        "the destinations land"
    );

    Background::sleeping(true);
    assert!(view.request_print(), "taken up: held");
    assert_eq!(
        state(&view, "shell:print"),
        Value::from("printing on Office")
    );
    assert!(!enabled(&view, "shell.print"));
    for _ in 0..5 {
        thread::sleep(Duration::from_millis(50));
        assert!(!view.settle_print(), "asleep: nothing looked at");
    }
    assert!(locked(&fake).printed().is_empty(), "nothing started asleep");
    assert!(stages.ended("shell.print").is_none());

    Background::sleeping(false);
    assert!(
        wait_for(&mut view, Duration::from_secs(60), |_| stages
            .ended("shell.print")
            .is_some()),
        "started on the wake, and answered"
    );
    let Some(outcome) = ended_or_skip(&stages) else {
        return;
    };
    assert_eq!(outcome.result["job"], Value::from("Office-1"));
    assert_eq!(locked(&fake).printed().len(), 1);
    assert_eq!(
        state(&view, "shell:print"),
        Value::from("Office-1 on Office: queued")
    );
}
