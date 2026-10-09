//! Sleep and wake (task-76 item 7): a window the platform says is covered
//! asks for no frame and has no deadline while another animates on; shown
//! again it draws one frame, built from the state as it is then; an app
//! that sleeps draws nothing and wakes every window with a new frame,
//! byte for byte the frame a window that never slept draws in that
//! state, never the frame from before the sleep; and the work beside the
//! frames hears the sleep through `Background`. Through `Windows` over
//! headless hosts (a host without a window draws into a texture), with
//! the platform's events injected, so a machine without a window and
//! without sleeping proves it; skips without a GPU adapter, like its
//! neighbours.
//!
//! The probe this file began as (commit `7d88e4f3`) measured the engine
//! before: `Occluded` was nobody's event, a covered window that animated
//! kept asking for frames and kept its deadline, and its texture kept the
//! frame from before the cover while the state moved on.

use std::time::Duration;

use engine::compositor::Compositor;
use engine::gpu::RenderConfig;
use engine::headless::Frame;
use engine::theme::hoff::PAGE_BG;
use engine::window::{Background, Routed, WindowHost, Windows};
use engine::winit::event::WindowEvent;
use engine::winit::window::WindowId;
use web_time::Instant;

const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
/// Where the scene puts its rect, logical pixels at scale 1.
const RECT: (f32, f32) = (10.0, 10.0);

const A: WindowId = WindowId::dummy();
fn b_id() -> WindowId {
    WindowId::from(2u64)
}

fn clear() -> [f32; 4] {
    PAGE_BG.to_linear_array()
}

fn host() -> Option<WindowHost> {
    let host = WindowHost::headless(200, 100, 1.0, RenderConfig::default());
    if host.is_none() {
        eprintln!("no GPU adapter; skipping");
    }
    host
}

fn two_windows() -> Option<Windows> {
    let (a, b) = (host()?, host()?);
    let mut windows = Windows::new();
    windows.insert(A, a);
    windows.insert(b_id(), b);
    Some(windows)
}

fn scene(c: &mut Compositor, color: [f32; 4]) {
    c.begin_frame();
    c.draw_rect(RECT.0, RECT.1, 40.0, 40.0, color);
}

/// Build `color`'s scene into window `id` and present it; the draws.
fn draw(windows: &mut Windows, id: WindowId, color: [f32; 4]) -> Option<u32> {
    let host = windows.get_mut(id).expect("the window is open");
    draw_into(host, color)
}

fn draw_into(host: &mut WindowHost, color: [f32; 4]) -> Option<u32> {
    host.begin_frame();
    scene(host.compositor_mut(), color);
    host.present(clear(), "sleep_wake")
}

fn read(windows: &Windows, id: WindowId) -> Frame {
    windows
        .get(id)
        .expect("the window is open")
        .read()
        .expect("a frame to read")
}

fn px(frame: &Frame) -> [u8; 4] {
    frame
        .pixel(RECT.0 as u32 + 5, RECT.1 as u32 + 5)
        .expect("inside the frame")
}

/// The frame a host that was never covered and never slept draws for
/// `color`: what a window must show after it shows again.
fn control(color: [f32; 4]) -> Option<Frame> {
    let mut host = host()?;
    assert!(draw_into(&mut host, color).is_some());
    Some(host.read().expect("the control frame reads"))
}

#[test]
fn a_covered_window_asks_for_no_frame_and_has_no_deadline_while_another_animates() {
    let Some(mut windows) = two_windows() else {
        return;
    };
    let soon = Instant::now() + Duration::from_millis(500);
    for id in [A, b_id()] {
        windows.get_mut(id).unwrap().set_animating(true);
        windows.get_mut(id).unwrap().set_deadline(Some(soon));
        assert!(draw(&mut windows, id, RED).is_some());
    }
    assert_eq!(windows.request_pending_redraws(), 2, "both animate");
    assert_eq!(windows.earliest_deadline(), Some(soon));

    // the platform says a is covered: a's event, taken by its host
    assert_eq!(
        windows.route(A, &WindowEvent::Occluded(true)),
        Routed::Handled
    );
    let a = windows.get(A).unwrap();
    assert!(a.occluded());
    assert!(!a.awake());
    assert!(!windows.awake(A));
    assert!(windows.awake(b_id()), "b shows");
    // a asks for nothing and has no deadline; b animates on alone
    assert!(!a.needs_redraw(), "covered: no reason to draw");
    assert!(!a.redraw_asked(), "the pending ask is withdrawn");
    assert_eq!(a.deadline(), None, "covered: no deadline of its own");
    assert!(a.is_animating(), "the animation waits, it is not off");
    assert_eq!(windows.earliest_deadline(), Some(soon), "b's");
    assert_eq!(windows.request_pending_redraws(), 1, "b alone");
    assert!(!windows.get(A).unwrap().redraw_asked());
    assert!(windows.get(b_id()).unwrap().redraw_asked());
    // an animation started, a change, a request while covered: withheld
    windows.get_mut(A).unwrap().set_animating(true);
    windows.get_mut(A).unwrap().compositor_mut().invalidate();
    windows.get_mut(A).unwrap().request_redraw();
    assert!(windows.request_redraw(A), "a is open");
    assert!(!windows.get(A).unwrap().redraw_asked(), "withheld");
    assert_eq!(windows.request_pending_redraws(), 1, "still b alone");
    // b's deadline gone: nobody's
    windows.get_mut(b_id()).unwrap().set_deadline(None);
    assert_eq!(windows.earliest_deadline(), None);
    // the same word twice changes nothing
    assert_eq!(
        windows.route(A, &WindowEvent::Occluded(true)),
        Routed::Handled
    );
    assert!(!windows.get(A).unwrap().redraw_asked());
}

#[test]
fn an_uncovered_window_draws_one_new_frame_with_the_state_it_has_now() {
    let Some(mut windows) = two_windows() else {
        return;
    };
    assert!(draw(&mut windows, A, RED).is_some());
    assert!(
        draw(&mut windows, b_id(), RED).is_some(),
        "b served, at rest"
    );
    let before = read(&windows, A);
    assert_eq!(px(&before), [255, 0, 0, 255]);
    windows.route(A, &WindowEvent::Occluded(true));
    // the state moves on while covered (the scene is blue now), and the
    // frames of a are not asked: its texture keeps the old frame
    assert_eq!(windows.request_pending_redraws(), 0);
    assert!(
        read(&windows, A).rgba == before.rgba,
        "nothing drawn covered"
    );
    assert_eq!(windows.get(A).unwrap().frames(), 1);

    // shown again: invalidated, one frame asked
    assert_eq!(
        windows.route(A, &WindowEvent::Occluded(false)),
        Routed::Handled
    );
    let a = windows.get(A).unwrap();
    assert!(a.awake());
    assert!(a.redraw_asked(), "one frame asked as it shows");
    assert!(a.needs_redraw());
    assert!(a.compositor().needs_render(), "the scene is stale");
    assert_eq!(windows.request_pending_redraws(), 1);
    // that frame is built from the state as it is now: blue, byte for
    // byte the frame of a window that was never covered
    assert!(draw(&mut windows, A, BLUE).is_some());
    let after = read(&windows, A);
    assert_eq!(px(&after), [0, 0, 255, 255]);
    let Some(control) = control(BLUE) else {
        return;
    };
    assert!(after.rgba == control.rgba, "the new frame, byte for byte");
    assert!(after.rgba != before.rgba, "never the old one");
    // and at rest again: nothing more is asked
    assert!(!windows.get(A).unwrap().needs_redraw());
    assert_eq!(windows.request_pending_redraws(), 0);
}

#[test]
fn a_sleeping_app_draws_nothing_and_wakes_every_window_with_a_new_frame() {
    let Some(mut windows) = two_windows() else {
        return;
    };
    let soon = Instant::now() + Duration::from_millis(500);
    windows.get_mut(A).unwrap().set_animating(true);
    windows.get_mut(b_id()).unwrap().set_deadline(Some(soon));
    assert!(draw(&mut windows, A, RED).is_some());
    assert!(draw(&mut windows, b_id(), RED).is_some());
    let before_a = read(&windows, A);
    let before_b = read(&windows, b_id());
    assert!(!windows.asleep());
    assert!(!Background::asleep());

    // the app sleeps: nothing is asked, no deadline, the work hears it
    windows.sleep();
    assert!(windows.asleep());
    assert!(Background::asleep(), "the work beside the frames hears it");
    assert!(!windows.awake(A));
    assert!(!windows.awake(b_id()));
    assert!(windows.get(A).unwrap().sleeping());
    assert!(!windows.get(A).unwrap().occluded(), "asleep is not covered");
    assert_eq!(windows.request_pending_redraws(), 0, "the animation waits");
    assert_eq!(windows.earliest_deadline(), None, "b's deadline waits");
    assert!(!windows.get(b_id()).unwrap().redraw_asked());
    // a change while asleep asks nothing (the state moves on regardless)
    windows.invalidate_all();
    assert_eq!(windows.request_pending_redraws(), 0);
    assert!(!windows.get(A).unwrap().redraw_asked());
    windows.sleep();
    assert!(windows.asleep(), "sleeping twice is sleeping");
    // a window that joins a sleeping app sleeps with it
    let Some(c) = host() else {
        return;
    };
    let c_id = WindowId::from(3u64);
    windows.insert(c_id, c);
    assert!(windows.get(c_id).unwrap().sleeping());
    assert!(!windows.get(c_id).unwrap().redraw_asked());
    assert_eq!(windows.request_pending_redraws(), 0);

    // the app wakes: every window invalidated, one frame each, the work
    // may go on
    windows.wake();
    assert!(!windows.asleep());
    assert!(!Background::asleep());
    for id in [A, b_id(), c_id] {
        let host = windows.get(id).unwrap();
        assert!(host.awake(), "{id:?} awake");
        assert!(host.redraw_asked(), "{id:?} asks for its frame");
        assert!(host.compositor().needs_render(), "{id:?} stale");
    }
    assert_eq!(windows.request_pending_redraws(), 3);
    // the first frame of each is the state as it is now (green), byte
    // for byte a control's, never the frame from before the sleep
    let Some(control) = control(GREEN) else {
        return;
    };
    for id in [A, b_id()] {
        assert!(draw(&mut windows, id, GREEN).is_some());
        let after = read(&windows, id);
        assert_eq!(px(&after), [0, 255, 0, 255]);
        assert!(after.rgba == control.rgba, "{id:?}: new, byte for byte");
    }
    assert!(read(&windows, A).rgba != before_a.rgba);
    assert!(read(&windows, b_id()).rgba != before_b.rgba);
    // a's animation goes on, b rests until its next ask
    assert!(windows.get(A).unwrap().needs_redraw());
    assert!(!windows.get(b_id()).unwrap().needs_redraw());
    windows.wake();
    assert!(!windows.asleep(), "waking twice is awake");
}

#[test]
fn awake_is_neither_covered_nor_asleep() {
    let Some(mut windows) = two_windows() else {
        return;
    };
    assert!(windows.awake(A));
    assert!(!windows.awake(WindowId::from(9u64)), "unknown: not awake");
    // covered and asleep at once: waking the app is not enough
    windows.route(A, &WindowEvent::Occluded(true));
    windows.sleep();
    assert!(!windows.awake(A));
    windows.wake();
    assert!(!windows.awake(A), "still covered");
    assert!(
        !windows.get(A).unwrap().redraw_asked(),
        "no frame asked yet"
    );
    assert!(windows.awake(b_id()), "b woke");
    assert!(windows.get(b_id()).unwrap().redraw_asked());
    // shown: awake, and the frame is asked then
    windows.route(A, &WindowEvent::Occluded(false));
    assert!(windows.awake(A));
    assert!(windows.get(A).unwrap().redraw_asked());
    // covered again, then the app sleeps and wakes: shown last, awake
    windows.route(A, &WindowEvent::Occluded(true));
    windows.sleep();
    windows.route(A, &WindowEvent::Occluded(false));
    assert!(!windows.awake(A), "the app sleeps");
    assert!(!windows.get(A).unwrap().redraw_asked());
    windows.wake();
    assert!(windows.awake(A));
    assert!(windows.get(A).unwrap().redraw_asked());

    // the platform taking the windows (android) is a sleep too
    windows.suspend();
    assert!(windows.asleep());
    assert!(Background::asleep());
    assert!(windows.is_empty());
    assert!(windows.has_suspended());
    assert_eq!(windows.request_pending_redraws(), 0);
    assert_eq!(windows.earliest_deadline(), None);
    assert!(!windows.should_exit());
    // resume needs a live event loop; the wake alone, as it would do
    windows.wake();
    assert!(!Background::asleep());
}

/// The flag is the thread's: a test's app sleeps without another's.
#[test]
fn the_background_flag_is_set_and_read_on_the_same_thread() {
    assert!(!Background::asleep());
    Background::sleeping(true);
    assert!(Background::asleep());
    let other = std::thread::spawn(Background::asleep)
        .join()
        .expect("the other thread answers");
    assert!(!other, "another thread's app is awake");
    Background::sleeping(false);
    assert!(!Background::asleep());
}
