//! Sleep and wake, the probe (task-76 item 7): what a window that
//! animates does today when the platform says it is covered, and when
//! the app is suspended. Through `Windows` over headless hosts (a host
//! without a window draws into a texture), so a machine without a window
//! can measure it; skips without a GPU adapter, like its neighbours.

use engine::compositor::Compositor;
use engine::gpu::RenderConfig;
use engine::headless::Frame;
use engine::theme::hoff::PAGE_BG;
use engine::window::{Routed, WindowHost, Windows};
use engine::winit::event::WindowEvent;
use engine::winit::window::WindowId;

const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
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

/// Build `color`'s scene into window `id` and present it.
fn draw(windows: &mut Windows, id: WindowId, color: [f32; 4]) -> Option<u32> {
    let host = windows.get_mut(id).expect("the window is open");
    host.begin_frame();
    scene(host.compositor_mut(), color);
    host.present(clear(), "sleep_wake_probe")
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

/// The probe, as the engine is before item 7: the platform's `Occluded`
/// is nobody's event, a covered window that animates keeps asking for
/// its frames and keeps its deadline, its texture keeps the frame from
/// before the cover while the state moves on, and `suspend` takes the
/// hosts out of the map (frames stop by absence; `resume` needs a live
/// event loop, so it is not measured here).
#[test]
fn probe_a_covered_window_keeps_asking_today_and_suspend_empties_the_map() {
    let Some(mut windows) = two_windows() else {
        return;
    };
    let now = web_time::Instant::now();
    let soon = now + std::time::Duration::from_millis(500);
    windows.get_mut(A).unwrap().set_animating(true);
    windows.get_mut(A).unwrap().set_deadline(Some(soon));
    assert!(draw(&mut windows, A, RED).is_some());
    assert!(draw(&mut windows, b_id(), BLUE).is_some());
    assert_eq!(px(&read(&windows, A)), [255, 0, 0, 255]);

    // the platform says a is covered: today the event is nobody's
    let covered = WindowEvent::Occluded(true);
    assert_eq!(windows.route(A, &covered), Routed::Handled);
    assert!(
        !windows.get_mut(A).unwrap().handle_event(&covered),
        "today: not the host's event"
    );
    // and nothing pauses: a keeps asking for frames, its deadline stays
    assert!(windows.get(A).unwrap().needs_redraw(), "today: animating on");
    assert_eq!(windows.request_pending_redraws(), 1, "today: a asks");
    assert_eq!(windows.earliest_deadline(), Some(soon), "today: kept");

    // the state moves on while covered and no frame is drawn: the
    // texture keeps the old frame; the first frame drawn is new, since
    // the scene is built from the state (nobody asks for it on uncover)
    assert_eq!(px(&read(&windows, A)), [255, 0, 0, 255], "the old frame");
    assert_eq!(
        windows.route(A, &WindowEvent::Occluded(false)),
        Routed::Handled
    );
    assert!(draw(&mut windows, A, BLUE).is_some());
    assert_eq!(px(&read(&windows, A)), [0, 0, 255, 255], "new once drawn");

    // the app suspended (android): the hosts leave the map, so frames and
    // deadlines stop by absence, and the app is not to exit
    windows.suspend();
    assert!(windows.is_empty());
    assert!(windows.has_suspended());
    assert_eq!(windows.request_pending_redraws(), 0);
    assert_eq!(windows.earliest_deadline(), None);
    assert!(!windows.should_exit());
    assert_eq!(windows.route(A, &covered), Routed::Unknown);
}
