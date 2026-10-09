//! Several windows in one engine (task-76 item 2).
//!
//! The probe first: two GPU contexts with two compositors, each drawing
//! its own scene in the same loop, to see what a second window shares
//! with the first and what it does not. Then `Windows` over headless
//! hosts: an event reaches its owner and nobody else, a frame is drawn
//! for the window that asked and no other, one window closes and the
//! other stays, the last closes the app, and a device lost under one
//! window leaves the other's frame untouched. Through the headless frame
//! path (a host without a window draws into a texture), so a machine
//! without a window can prove it; the tests skip without a GPU adapter,
//! like their neighbours.

use engine::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::gpu::{ImageHandle, RenderConfig, load_image_rgba};
use engine::headless::{Frame, HeadlessError, Renderer, Viewport};
use engine::text::TextStyle;
use engine::theme::hoff::PAGE_BG;
use engine::window::{Routed, WindowHost, Windows};
use engine::winit::dpi::PhysicalPosition;
use engine::winit::event::{DeviceId, WindowEvent};
use engine::winit::window::WindowId;

const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
const IMAGE_RGBA: [u8; 4] = [20, 200, 40, 255];

/// Where each scene puts its things, logical pixels at scale 1.
const RECT: (f32, f32) = (10.0, 10.0);
const IMAGE: (f32, f32) = (100.0, 10.0);
const TEXT: (f32, f32) = (10.0, 60.0);

fn clear() -> [f32; 4] {
    PAGE_BG.to_linear_array()
}

fn bg() -> [u8; 4] {
    let [r, g, b, _] = PAGE_BG.to_array();
    [
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
        255,
    ]
}

fn renderer() -> Option<Renderer> {
    match Renderer::new(Viewport::new(200.0, 100.0, 1.0), RenderConfig::default()) {
        Ok(r) => Some(r),
        Err(HeadlessError::NoAdapter) => {
            eprintln!("no GPU adapter; skipping");
            None
        }
        Err(e) => panic!("headless renderer: {e}"),
    }
}

/// A flat rect, a line of text and, when asked, the shared image.
fn scene(c: &mut Compositor, color: [f32; 4], text: &str, image: Option<ImageHandle>) {
    c.begin_frame();
    c.draw_rect(RECT.0, RECT.1, 40.0, 40.0, color);
    c.push(SceneNode::Text {
        key: TextNodeKey::from_style(text, &TextStyle::new(20.0), None),
        x: TEXT.0,
        y: TEXT.1,
        color: [1.0, 1.0, 1.0, 1.0],
    });
    if let Some(image) = image {
        c.push(SceneNode::Image {
            x: IMAGE.0,
            y: IMAGE.1,
            w: 40.0,
            h: 40.0,
            image,
            corner_radius: 0.0,
        });
    }
}

fn px(frame: &Frame, (x, y): (f32, f32)) -> [u8; 4] {
    frame
        .pixel(x as u32 + 5, y as u32 + 5)
        .expect("inside the frame")
}

/// Pixels in the text's row band that are not the background.
fn text_pixels(frame: &Frame) -> usize {
    let mut n = 0;
    for y in (TEXT.1 as u32)..(TEXT.1 as u32 + 24) {
        for x in (TEXT.0 as u32)..(TEXT.0 as u32 + 120) {
            if frame.pixel(x, y) != Some(bg()) {
                n += 1;
            }
        }
    }
    n
}

/// The probe: two contexts, two compositors, two scenes in one loop.
/// Each shows only what was pushed into it (the glyph atlas, the layer
/// textures, the pipelines are per context); the image store is the one
/// global log, so a handle loaded once draws in either context without
/// a second load.
#[test]
fn two_contexts_draw_two_scenes_in_the_same_loop_and_share_only_the_image_log() {
    let (Some(mut a), Some(mut b)) = (renderer(), renderer()) else {
        return;
    };
    let mut ca = Compositor::new();
    let mut cb = Compositor::new();
    let image = load_image_rgba(8, 8, IMAGE_RGBA.repeat(64)).expect("an 8x8 image packs");

    for round in 0..3 {
        scene(&mut ca, RED, "one", Some(image));
        let draws_a = a.draw(&mut ca, clear());
        scene(&mut cb, BLUE, "two", None);
        let draws_b = b.draw(&mut cb, clear());
        assert!(
            draws_a > 0 && draws_b > 0,
            "round {round}: {draws_a} {draws_b}"
        );
    }

    let fa = a.read().expect("frame a");
    let fb = b.read().expect("frame b");
    assert_eq!(px(&fa, RECT), [255, 0, 0, 255], "a: its own rect");
    assert_eq!(px(&fb, RECT), [0, 0, 255, 255], "b: its own rect");
    assert_eq!(px(&fa, IMAGE), IMAGE_RGBA, "a: the image it pushed");
    assert_eq!(px(&fb, IMAGE), bg(), "b: no image was pushed into it");
    assert!(text_pixels(&fa) > 20, "a: text drawn from its own atlas");
    assert!(text_pixels(&fb) > 20, "b: text drawn from its own atlas");
    assert_ne!(
        fa.rgba, fb.rgba,
        "two scenes, two frames: nothing was drawn into the wrong context"
    );

    // the image loaded before b existed draws in b on its next frame:
    // the store is a log every atlas replays, never drained by a's upload
    scene(&mut cb, BLUE, "two", Some(image));
    b.draw(&mut cb, clear());
    let fb = b.read().expect("frame b");
    assert_eq!(
        px(&fb, IMAGE),
        IMAGE_RGBA,
        "b: the shared image, from the log"
    );
    assert_eq!(px(&fb, RECT), [0, 0, 255, 255]);
}

// -- Windows over headless hosts ------------------------------------------

/// The two ids a test owns: winit's dummy and one more of the same kind.
const A: WindowId = WindowId::dummy();
fn b_id() -> WindowId {
    WindowId::from(2u64)
}

fn host() -> Option<WindowHost> {
    let host = WindowHost::headless(200, 100, 1.0, RenderConfig::default());
    if host.is_none() {
        eprintln!("no GPU adapter; skipping");
    }
    host
}

/// Two headless hosts under two ids.
fn two_windows() -> Option<Windows> {
    let (a, b) = (host()?, host()?);
    let mut windows = Windows::new();
    windows.insert(A, a);
    windows.insert(b_id(), b);
    Some(windows)
}

fn cursor_moved(x: f64, y: f64) -> WindowEvent {
    WindowEvent::CursorMoved {
        device_id: DeviceId::dummy(),
        position: PhysicalPosition::new(x, y),
    }
}

/// Build `color`'s scene into window `id` and present it; the draws.
fn draw(windows: &mut Windows, id: WindowId, color: [f32; 4], text: &str) -> Option<u32> {
    let host = windows.get_mut(id).expect("the window is open");
    host.begin_frame();
    scene(host.compositor_mut(), color, text, None);
    host.present(clear(), "windows_test")
}

fn read(windows: &Windows, id: WindowId) -> Frame {
    windows
        .get(id)
        .expect("the window is open")
        .read()
        .expect("a frame to read")
}

#[test]
fn an_event_reaches_the_window_it_came_from_and_no_other() {
    let Some(mut windows) = two_windows() else {
        return;
    };
    assert_eq!(windows.len(), 2);
    assert_eq!(windows.first(), Some(A));
    assert_eq!(windows.route(A, &cursor_moved(30.0, 40.0)), Routed::Handled);
    // buffered until the frame: the input state sees it on the redraw
    assert_eq!(windows.get(A).unwrap().pending_events(), 1);
    assert_eq!(windows.get(b_id()).unwrap().pending_events(), 0);
    assert_eq!(
        windows.route(A, &WindowEvent::RedrawRequested),
        Routed::Redraw
    );
    assert_eq!(
        windows.get(A).unwrap().input().cursor_position(),
        Some((30.0, 40.0))
    );
    assert_eq!(windows.get(b_id()).unwrap().input().cursor_position(), None);
    assert_eq!(
        windows.route(b_id(), &WindowEvent::RedrawRequested),
        Routed::Redraw
    );
    assert_eq!(windows.get(b_id()).unwrap().input().cursor_position(), None);
    // a resize takes effect on its owner alone
    assert_eq!(
        windows.route(
            b_id(),
            &WindowEvent::Resized(engine::winit::dpi::PhysicalSize::new(300, 150))
        ),
        Routed::Handled
    );
    assert_eq!(windows.get(b_id()).unwrap().physical_size(), (300, 150));
    assert_eq!(windows.get(A).unwrap().physical_size(), (200, 100));
    // an id nobody owns (a window closed already) is nobody's
    assert_eq!(
        windows.route(WindowId::from(9u64), &cursor_moved(1.0, 1.0)),
        Routed::Unknown
    );
}

#[test]
fn a_frame_is_asked_of_the_window_that_changed_and_drawn_for_it_alone() {
    let Some(mut windows) = two_windows() else {
        return;
    };
    // a fresh host asks for its first frame; both are served
    assert!(windows.get(A).unwrap().redraw_asked());
    assert!(windows.get(b_id()).unwrap().redraw_asked());
    draw(&mut windows, A, RED, "one");
    draw(&mut windows, b_id(), BLUE, "two");
    assert!(!windows.get(A).unwrap().needs_redraw(), "a at rest");
    assert!(!windows.get(b_id()).unwrap().needs_redraw(), "b at rest");
    assert_eq!(windows.request_pending_redraws(), 0, "nothing to draw");

    // something changes in a: a alone asks
    windows.get_mut(A).unwrap().compositor_mut().invalidate();
    assert_eq!(windows.request_pending_redraws(), 1);
    assert!(windows.get(A).unwrap().redraw_asked());
    assert!(!windows.get(b_id()).unwrap().redraw_asked());
    let frames_b = windows.get(b_id()).unwrap().frames();
    draw(&mut windows, A, RED, "one");
    assert_eq!(windows.get(A).unwrap().frames(), 2);
    assert_eq!(
        windows.get(b_id()).unwrap().frames(),
        frames_b,
        "b drew nothing"
    );
    assert!(!windows.get(A).unwrap().redraw_asked(), "served");

    // one animates, the other rests: only the animating one keeps asking
    windows.get_mut(b_id()).unwrap().set_animating(true);
    for _ in 0..3 {
        assert_eq!(windows.request_pending_redraws(), 1);
        draw(&mut windows, b_id(), BLUE, "two");
    }
    assert_eq!(
        windows.get(A).unwrap().frames(),
        2,
        "a drew nothing meanwhile"
    );
    assert_eq!(windows.get(b_id()).unwrap().frames(), 4);
    windows.get_mut(b_id()).unwrap().set_animating(false);
    assert_eq!(windows.request_pending_redraws(), 0);

    // an event on a is a frame for a
    windows.route(A, &cursor_moved(1.0, 1.0));
    assert_eq!(windows.request_pending_redraws(), 1);
    assert!(windows.get(A).unwrap().redraw_asked());
}

#[test]
fn closing_one_window_keeps_the_other_and_the_last_closes_the_app() {
    let Some(mut windows) = two_windows() else {
        return;
    };
    assert!(!windows.should_exit(), "two windows open");
    draw(&mut windows, A, RED, "one");
    draw(&mut windows, b_id(), BLUE, "two");

    // the person closes a: the answer is the caller's, then a goes alone
    assert_eq!(
        windows.route(A, &WindowEvent::CloseRequested),
        Routed::CloseRequested
    );
    assert_eq!(windows.len(), 2, "route closes nothing by itself");
    let closed = windows.close(A).expect("a was open");
    assert_eq!(closed.frames(), 1);
    drop(closed);
    assert_eq!(windows.len(), 1);
    assert_eq!(windows.first(), Some(b_id()));
    assert!(!windows.contains(A));
    assert!(!windows.should_exit(), "b is still open");
    assert_eq!(
        windows.route(A, &WindowEvent::Destroyed),
        Routed::Unknown,
        "the platform's Destroyed after the close routes nowhere"
    );

    // b draws on as before
    assert!(draw(&mut windows, b_id(), BLUE, "two").is_some());
    assert_eq!(px(&read(&windows, b_id()), RECT), [0, 0, 255, 255]);

    // the last one closes the app, unless the app said otherwise
    windows.set_exit_when_last_closes(false);
    windows.close(b_id());
    assert!(windows.is_empty());
    assert!(
        !windows.should_exit(),
        "the app keeps running without a window"
    );
    windows.set_exit_when_last_closes(true);
    assert!(windows.should_exit());
    assert!(windows.close(b_id()).is_none(), "closed already");

    // an empty map that never opened a window is not an app to close
    assert!(!Windows::new().should_exit());
}

#[test]
fn a_device_lost_under_one_window_leaves_the_other_untouched() {
    let Some(mut windows) = two_windows() else {
        return;
    };
    draw(&mut windows, A, RED, "one");
    draw(&mut windows, b_id(), BLUE, "two");
    let before_a = read(&windows, A);
    let before_b = read(&windows, b_id());

    // a's device dies; b's is another device on another context
    windows
        .get(A)
        .unwrap()
        .gpu()
        .expect("a headless host has a device")
        .device
        .destroy();
    assert!(windows.get(A).unwrap().device_lost(), "a saw the loss");
    assert!(!windows.get(b_id()).unwrap().device_lost(), "b did not");

    // b's next frame is b's frame; a's next frame recovers and is a's
    assert!(draw(&mut windows, b_id(), BLUE, "two").is_some());
    let after_b = read(&windows, b_id());
    assert!(
        after_b.rgba == before_b.rgba,
        "b: the same frame, byte for byte"
    );
    assert!(!windows.get(b_id()).unwrap().device_lost());

    let draws = draw(&mut windows, A, RED, "one");
    assert!(draws.is_some(), "a recovered and drew");
    assert!(!windows.get(A).unwrap().device_lost(), "a is whole again");
    let after_a = read(&windows, A);
    assert!(
        after_a.rgba == before_a.rgba,
        "a: the same frame on the new device"
    );
    assert_eq!(px(&after_a, RECT), [255, 0, 0, 255]);
    assert_eq!(px(&after_b, RECT), [0, 0, 255, 255]);
}

#[test]
fn the_loop_sleeps_until_the_earliest_deadline_of_any_window() {
    let Some(mut windows) = two_windows() else {
        return;
    };
    assert_eq!(windows.earliest_deadline(), None);
    let now = web_time::Instant::now();
    let soon = now + std::time::Duration::from_millis(100);
    let later = now + std::time::Duration::from_secs(2);
    windows.get_mut(A).unwrap().set_deadline(Some(later));
    assert_eq!(windows.earliest_deadline(), Some(later));
    windows.get_mut(b_id()).unwrap().set_deadline(Some(soon));
    assert_eq!(windows.earliest_deadline(), Some(soon));
    windows.get_mut(b_id()).unwrap().set_deadline(None);
    assert_eq!(windows.earliest_deadline(), Some(later));
}

/// A bare host: a scene and an input with no device and no window, for
/// the shells' tests that never draw. It asks for frames like any other
/// and presents nothing.
#[test]
fn a_bare_host_builds_a_scene_and_presents_nothing() {
    let mut host = WindowHost::bare();
    assert!(!host.has_gpu());
    assert_eq!(host.id(), None);
    assert!(!host.redraw_asked(), "nobody asked yet");
    host.compositor_mut().begin_frame();
    host.compositor_mut().draw_rect(0.0, 0.0, 10.0, 10.0, RED);
    assert!(host.needs_redraw(), "a changed scene wants a frame");
    assert_eq!(host.present(clear(), "bare"), None);
    assert_eq!(host.frames(), 0);
    assert!(host.handle_event(&cursor_moved(3.0, 4.0)));
    host.process_events();
    assert_eq!(host.input().cursor_position(), Some((3.0, 4.0)));
    assert!(!host.handle_event(&WindowEvent::CloseRequested));
}
