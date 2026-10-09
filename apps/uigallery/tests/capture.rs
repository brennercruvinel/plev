//! The Capture page on the window's frame path (task-76 item 10), without
//! a window and without a device: a `ShowcaseView` on the Capture page
//! over the fake capture (`FakeCapture`: frame `n` is one solid color,
//! `FakeCapture::pixel(n)`), drawn by a headless host
//! (`engine::runtime::window::WindowHost`) and read back. A person turns the camera
//! on; the frame drawn before the view's wake shows the empty box, never a
//! captured frame; the first frame after the wake shows the fake's frame
//! 0 inside the box, the next wake frame 1 in the same place (one stream
//! image, replaced in place). The page not seen (the shell's word, as a
//! covered window gives it) stops the fake and the box is empty again in
//! the next frame, not the last frame captured. The view asks the loop
//! for a look a frame period away only while the camera is live. Desktop
//! only, like the shell; skips without a GPU adapter.
#![cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]

use std::sync::{Arc, Mutex};
use std::time::Duration;

use accord::{Actor, Request, SemId, Stage};
use engine::drawing::gpu::RenderConfig;
use engine::drawing::headless::Frame;
use engine::runtime::platform::capture::{DeviceKind, FakeCapture};
use engine::runtime::platform::menu::locked;
use engine::runtime::window::WindowHost;
use uigallery::views::ShowcaseView;

const MAIN: (u32, u32) = (1200, 800);
/// One turn of a 60 Hz display.
const TURN: f32 = 1.0 / 60.0;

fn host() -> Option<WindowHost> {
    let host = WindowHost::headless(MAIN.0, MAIN.1, 1.0, RenderConfig::default());
    if host.is_none() {
        eprintln!("no GPU adapter; skipping");
    }
    host
}

/// The Capture page laid out for the main window over a shared fake, the
/// page active, its knobs at rest.
fn view(fake: &Arc<Mutex<FakeCapture>>) -> ShowcaseView {
    let mut view = ShowcaseView::new(MAIN.0 as f32, MAIN.1 as f32);
    view.jump_to_page("Capture");
    view.resize(MAIN.0 as f32, MAIN.1 as f32, 1.0);
    view.set_capture(Box::new(fake.clone()));
    rest(&mut view);
    view
}

/// Ticks until the knobs rest; what the last tick asked of the loop.
fn rest(view: &mut ShowcaseView) -> Option<Duration> {
    for _ in 0..600 {
        let next = view.wake(TURN).next;
        if next != Some(Duration::ZERO) {
            return next;
        }
    }
    panic!("the view never rests");
}

/// The main window's frame: the whole view, presented and read back.
fn frame(view: &mut ShowcaseView, host: &mut WindowHost) -> Frame {
    host.begin_frame();
    view.render(host.compositor_mut());
    let clear = view.theme.colors.bg.to_linear_array();
    assert!(host.present(clear, "main").is_some(), "the frame drew");
    host.read().expect("the frame reads")
}

/// A point well inside the frame box: clear of its rounded corners and
/// of the words it shows when empty.
fn inside(host: &WindowHost) -> (u32, u32) {
    let [x, y, _, _] = host
        .compositor()
        .binding(&SemId::parse("capture.frame").unwrap())
        .expect("the frame box is bound")
        .rect;
    ((x + 24.0) as u32, (y + 24.0) as u32)
}

fn close(drawn: [u8; 4], want: [u8; 4]) -> bool {
    (0..3).all(|k| (drawn[k] as i32 - want[k] as i32).abs() <= 2)
}

fn turn_camera(view: &mut ShowcaseView) {
    let seen = view.semantics().expect("the page projects");
    let req = Request::observe(&seen, Actor::Human, "capture.camera", "toggle", None).unwrap();
    let receipt = view.act(req).expect("the page takes it");
    assert!(
        matches!(receipt.stage(), Some(Stage::Completed(_))),
        "{:?}",
        receipt.stage()
    );
}

#[test]
fn the_frame_shows_in_the_first_frame_after_the_wake_and_not_before() {
    let Some(mut host) = host() else {
        return;
    };
    let fake = Arc::new(Mutex::new(FakeCapture::new()));
    let mut view = view(&fake);
    let empty = frame(&mut view, &mut host);
    let at = inside(&host);
    let box_color = empty.pixel(at.0, at.1).expect("inside");
    assert_eq!(rest(&mut view), None, "at rest: nothing asked");

    turn_camera(&mut view);
    assert_eq!(locked(&fake).starts(DeviceKind::Camera), 1);
    let before = frame(&mut view, &mut host);
    assert_eq!(
        before.pixel(at.0, at.1),
        Some(box_color),
        "before the wake the box is empty"
    );
    assert_eq!(locked(&fake).frames(), 0, "no frame taken by a draw");

    assert!(view.wake(TURN).now, "the wake takes the frame: draw it");
    let first = frame(&mut view, &mut host);
    let drawn = first.pixel(at.0, at.1).expect("inside");
    assert!(
        close(drawn, FakeCapture::pixel(0)),
        "{drawn:?} is not frame 0 {:?}",
        FakeCapture::pixel(0)
    );
    view.wake(TURN);
    let second = frame(&mut view, &mut host);
    let drawn = second.pixel(at.0, at.1).expect("inside");
    assert!(
        close(drawn, FakeCapture::pixel(1)),
        "{drawn:?} is not frame 1 {:?}",
        FakeCapture::pixel(1)
    );

    // live: a look a frame period away, once the knob rests
    let next = rest(&mut view).expect("live: a look is asked");
    assert!(
        next.abs_diff(Duration::from_secs_f64(1.0 / 15.0)) < Duration::from_millis(1),
        "{next:?}"
    );

    // not seen (a covered window, as the shell says it): the fake is
    // stopped at once and the box is empty in the next frame, not the
    // last frame captured
    assert!(view.set_capture_shown(false));
    assert_eq!(locked(&fake).stops(DeviceKind::Camera), 1);
    let covered = frame(&mut view, &mut host);
    assert_eq!(covered.pixel(at.0, at.1), Some(box_color));
    assert_eq!(rest(&mut view), None, "nothing live, nothing asked");
    let taken = locked(&fake).frames();

    // seen again: started again, and a new frame after the next wake
    view.set_capture_shown(true);
    assert_eq!(locked(&fake).starts(DeviceKind::Camera), 2);
    view.wake(TURN);
    let again = frame(&mut view, &mut host);
    let drawn = again.pixel(at.0, at.1).expect("inside");
    assert!(close(drawn, FakeCapture::pixel(taken)), "{drawn:?}");
}
