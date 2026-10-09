//! The gallery asleep and covered (task-76 item 7), without a window: a
//! `ShowcaseView` over a headless host (`engine::window::WindowHost`, the
//! window's frame path into a texture) under `Windows`. The app sleeps,
//! the state changes meanwhile (a todo toggled through the dispatcher),
//! nothing is asked of the window and its texture keeps the old frame;
//! the wake asks for one frame, and that frame is, byte for byte, the
//! frame a view that never slept draws in the same state, never the one
//! from before the sleep. The same for the main window covered and shown
//! again. Desktop only, like the shell; skips without a GPU adapter, like
//! the headless tests.
#![cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]

use engine::gpu::RenderConfig;
use engine::headless::Frame;
use engine::window::{Background, Routed, WindowHost, Windows};
use engine::winit::event::WindowEvent;
use engine::winit::window::WindowId;
use sem::{Actor, Request, SemId, Value};
use showcase::view::ShowcaseView;

const MAIN: (u32, u32) = (1200, 800);
const A: WindowId = WindowId::dummy();
/// One turn of a 60 Hz display.
const TURN: f32 = 1.0 / 60.0;

fn host() -> Option<WindowHost> {
    let host = WindowHost::headless(MAIN.0, MAIN.1, 1.0, RenderConfig::default());
    if host.is_none() {
        eprintln!("no GPU adapter; skipping");
    }
    host
}

fn sid(text: &str) -> SemId {
    SemId::parse(text).expect("a valid id")
}

/// The Todo page laid out for the main window, settled.
fn view() -> ShowcaseView {
    let mut view = ShowcaseView::new(MAIN.0 as f32, MAIN.1 as f32);
    view.jump_to_page("Todo");
    view.resize(MAIN.0 as f32, MAIN.1 as f32, 1.0);
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

/// The main window's frame: the whole view, presented and read back.
fn frame(view: &mut ShowcaseView, host: &mut WindowHost) -> Frame {
    host.begin_frame();
    view.render(host.compositor_mut());
    let clear = view.theme.colors.bg.to_linear_array();
    assert!(host.present(clear, "main").is_some(), "the frame drew");
    host.read().expect("the frame reads")
}

fn completed(view: &ShowcaseView, item: &SemId) -> Value {
    view.semantics()
        .expect("the todo projects")
        .get(item)
        .expect("the item is projected")
        .state["completed"]
        .clone()
}

/// Item 1 toggled by a person through the page's dispatcher: the state
/// the next frame has to show.
fn toggle(view: &mut ShowcaseView, item: &SemId) {
    let seen = view.semantics().expect("the todo projects");
    let req = Request::observe(&seen, Actor::Human, item, "toggle", None).expect("observed");
    let receipt = view.act(req).expect("the page takes it");
    assert!(
        matches!(receipt.stage(), Some(sem::Stage::Completed(_))),
        "{:?}",
        receipt.stage()
    );
}

/// The pixels inside `rect` of `frame`.
fn pixels(frame: &Frame, [x, y, w, h]: [f32; 4]) -> Vec<[u8; 4]> {
    let mut out = Vec::new();
    for py in (y as u32)..((y + h) as u32) {
        for px in (x as u32)..((x + w) as u32) {
            out.push(frame.pixel(px, py).expect("inside the frame"));
        }
    }
    out
}

/// A view in the state the sleeping one reached, drawn by a host that
/// never slept: the frame the wake has to draw.
fn control(item: &SemId) -> Option<Frame> {
    let mut host = host()?;
    let mut view = view();
    toggle(&mut view, item);
    settle(&mut view);
    Some(frame(&mut view, &mut host))
}

#[test]
fn the_first_frame_after_the_wake_is_the_state_now_byte_for_byte() {
    let Some(main) = host() else {
        return;
    };
    let mut windows = Windows::new();
    windows.insert(A, main);
    let mut view = view();
    let item = sid("todo.item[1]");
    let before = frame(&mut view, windows.get_mut(A).unwrap());
    let rect = windows
        .get(A)
        .unwrap()
        .compositor()
        .binding(&item)
        .expect("the item is bound")
        .rect;
    assert_eq!(completed(&view, &item), Value::Bool(false));
    assert_eq!(windows.request_pending_redraws(), 0, "at rest");

    // the app sleeps; the state changes meanwhile (an agent, a worker):
    // the window is asked for nothing and its texture keeps the old frame
    windows.sleep();
    assert!(Background::asleep());
    toggle(&mut view, &item);
    settle(&mut view);
    assert_eq!(completed(&view, &item), Value::Bool(true));
    windows.invalidate_all();
    assert_eq!(
        windows.request_pending_redraws(),
        0,
        "asleep: nothing asked"
    );
    assert_eq!(windows.earliest_deadline(), None);
    assert!(!windows.get(A).unwrap().redraw_asked());
    let kept = windows.get(A).unwrap().read().expect("reads");
    assert!(kept.rgba == before.rgba, "the texture keeps the old frame");

    // the wake: one frame asked, and it is the state now, byte for byte
    // what a view that never slept draws; never the frame from before
    windows.wake();
    assert!(!Background::asleep());
    assert!(windows.get(A).unwrap().redraw_asked(), "one frame asked");
    assert_eq!(windows.request_pending_redraws(), 1);
    let after = frame(&mut view, windows.get_mut(A).unwrap());
    let Some(control) = control(&item) else {
        return;
    };
    assert!(
        after.rgba == control.rgba,
        "the wake's frame is the control's"
    );
    assert!(after.rgba != before.rgba, "never the old frame");
    assert_ne!(
        pixels(&after, rect),
        pixels(&before, rect),
        "the toggled item shows"
    );
    assert!(!windows.get(A).unwrap().redraw_asked(), "served");
    assert_eq!(windows.request_pending_redraws(), 0, "at rest again");
}

#[test]
fn the_first_frame_after_an_uncover_is_the_state_now_byte_for_byte() {
    let Some(main) = host() else {
        return;
    };
    let mut windows = Windows::new();
    windows.insert(A, main);
    let mut view = view();
    let item = sid("todo.item[1]");
    let before = frame(&mut view, windows.get_mut(A).unwrap());

    // covered: the platform's word, routed to the host
    assert_eq!(
        windows.route(A, &WindowEvent::Occluded(true)),
        Routed::Handled
    );
    assert!(!windows.awake(A));
    assert!(
        !Background::asleep(),
        "covered is not asleep: the work goes on"
    );
    toggle(&mut view, &item);
    settle(&mut view);
    windows.invalidate_all();
    assert_eq!(
        windows.request_pending_redraws(),
        0,
        "covered: nothing asked"
    );
    assert!(
        windows.get(A).unwrap().read().expect("reads").rgba == before.rgba,
        "the texture keeps the old frame"
    );

    // shown again: one frame, the state now
    assert_eq!(
        windows.route(A, &WindowEvent::Occluded(false)),
        Routed::Handled
    );
    assert!(windows.awake(A));
    assert!(windows.get(A).unwrap().redraw_asked());
    let after = frame(&mut view, windows.get_mut(A).unwrap());
    let Some(control) = control(&item) else {
        return;
    };
    assert!(
        after.rgba == control.rgba,
        "the shown frame is the control's"
    );
    assert!(after.rgba != before.rgba, "never the old frame");
}
